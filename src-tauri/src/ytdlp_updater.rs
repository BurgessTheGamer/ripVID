use hex;
use reqwest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct YtdlpVersion {
    version: String,
    last_check: u64,
    path: String,
}

#[derive(Clone)]
pub struct YtdlpUpdater {
    app_handle: AppHandle,
    data_dir: PathBuf,
}

impl YtdlpUpdater {
    pub fn new(app_handle: AppHandle) -> Self {
        let data_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."));

        Self {
            app_handle,
            data_dir,
        }
    }

    pub async fn ensure_updated(&self) -> Result<PathBuf, String> {
        // Check if we need to update (once per day)
        if !self.should_check_update()? {
            // Return the current yt-dlp path
            return self.get_ytdlp_path();
        }

        // Check for updates in the background
        let updater = self.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = updater.check_and_update().await {
                println!("Failed to update yt-dlp: {}", e);
            }
        });

        // Return current path immediately (don't block)
        self.get_ytdlp_path()
    }

    pub fn clone_for_background(&self) -> Self {
        self.clone()
    }

    fn should_check_update(&self) -> Result<bool, String> {
        let version_file = self.data_dir.join("ytdlp-version.json");

        if !version_file.exists() {
            return Ok(true);
        }

        let content = fs::read_to_string(&version_file).map_err(|e| e.to_string())?;

        let version_info: YtdlpVersion =
            serde_json::from_str(&content).map_err(|e| e.to_string())?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check once per day
        Ok(now - version_info.last_check > 86400)
    }

    async fn check_and_update(&self) -> Result<(), String> {
        tracing::info!("Checking for yt-dlp updates...");

        // Ensure data directory exists
        fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        // Get latest release info
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest")
            .header("User-Agent", "ripVID")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let release: GitHubRelease = response.json().await.map_err(|e| e.to_string())?;

        // Check if we need to update
        let version_file = self.data_dir.join("ytdlp-version.json");
        let current_version = if version_file.exists() {
            let content = fs::read_to_string(&version_file).map_err(|e| e.to_string())?;
            let info: YtdlpVersion = serde_json::from_str(&content).map_err(|e| e.to_string())?;
            info.version
        } else {
            String::new()
        };

        if current_version == release.tag_name {
            println!("yt-dlp is already up to date ({})", release.tag_name);

            // Update last check time
            self.save_version_info(&release.tag_name)?;
            return Ok(());
        }

        // Find the right asset for the platform
        let asset_name = self.get_platform_asset_name();
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("No asset found for {}", asset_name))?;

        println!("Downloading yt-dlp {} ...", release.tag_name);

        // Download the new version
        let response = client
            .get(&asset.browser_download_url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let bytes = response.bytes().await.map_err(|e| e.to_string())?;

        // SECURITY: Download and verify SHA256 checksum
        tracing::info!("Verifying yt-dlp checksum for security...");
        let checksums_url = format!(
            "https://github.com/yt-dlp/yt-dlp/releases/download/{}/SHA2-256SUMS",
            release.tag_name
        );

        let expected_checksum = self
            .fetch_and_parse_checksum(&client, &checksums_url, asset_name)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch checksum: {}", e);
                format!("Checksum verification failed: {}", e)
            })?;

        // Calculate actual checksum of downloaded file
        let actual_checksum = self.calculate_sha256(&bytes);

        // Verify checksums match
        if actual_checksum.to_lowercase() != expected_checksum.to_lowercase() {
            tracing::error!(
                "SECURITY ALERT: Checksum mismatch! Expected: {}, Got: {}",
                expected_checksum,
                actual_checksum
            );
            return Err(
                "Security error: Downloaded file checksum does not match official release. Update aborted.".to_string()
            );
        }

        tracing::info!("Checksum verified successfully: {}", actual_checksum);

        // Backup existing version before replacing (rollback capability)
        let ytdlp_path = self.data_dir.join("yt-dlp.exe");
        let backup_path = self.data_dir.join("yt-dlp.exe.backup");

        if ytdlp_path.exists() {
            fs::copy(&ytdlp_path, &backup_path)
                .map_err(|e| format!("Failed to create backup: {}", e))?;
            tracing::debug!("Created backup of existing yt-dlp binary");
        }

        // Save the verified binary
        if let Err(e) = fs::write(&ytdlp_path, bytes) {
            // Rollback on failure
            tracing::error!("Failed to write new yt-dlp binary: {}", e);
            if backup_path.exists() {
                tracing::warn!("Rolling back to previous version");
                if let Err(rollback_err) = fs::copy(&backup_path, &ytdlp_path) {
                    tracing::error!("Rollback failed: {}", rollback_err);
                }
            }
            return Err(format!("Failed to save updated binary: {}", e));
        }

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o755);
            if let Err(e) = fs::set_permissions(&ytdlp_path, permissions) {
                tracing::error!("Failed to set executable permissions: {}", e);
                // Rollback
                if backup_path.exists() {
                    tracing::warn!("Rolling back due to permission error");
                    fs::copy(&backup_path, &ytdlp_path).ok();
                }
                return Err(format!("Failed to set permissions: {}", e));
            }
        }

        // Remove backup after successful update
        if backup_path.exists() {
            fs::remove_file(&backup_path).ok();
        }

        // Save version info
        self.save_version_info(&release.tag_name)?;

        tracing::info!("Successfully updated yt-dlp to {}", release.tag_name);
        Ok(())
    }

    fn save_version_info(&self, version: &str) -> Result<(), String> {
        // Ensure data directory exists
        fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let ytdlp_path = self.data_dir.join("yt-dlp.exe");
        let version_info = YtdlpVersion {
            version: version.to_string(),
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            path: ytdlp_path.to_string_lossy().to_string(),
        };

        let version_file = self.data_dir.join("ytdlp-version.json");
        let json = serde_json::to_string_pretty(&version_info).map_err(|e| e.to_string())?;

        fs::write(version_file, json).map_err(|e| e.to_string())?;

        Ok(())
    }

    fn get_platform_asset_name(&self) -> &str {
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return "yt-dlp.exe";

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return "yt-dlp_macos";

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return "yt-dlp_macos";

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return "yt-dlp";

        #[cfg(not(any(
            all(target_os = "windows", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "linux", target_arch = "x86_64")
        )))]
        return "yt-dlp";
    }

    pub fn get_ytdlp_path(&self) -> Result<PathBuf, String> {
        let updated_path = self.data_dir.join("yt-dlp.exe");

        // Use updated version if it exists
        if updated_path.exists() {
            return Ok(updated_path);
        }

        // Fall back to bundled sidecar
        Ok(PathBuf::from("yt-dlp"))
    }

    /// Calculate SHA-256 checksum of binary data
    ///
    /// # Security
    /// Used to verify integrity of downloaded yt-dlp binaries
    fn calculate_sha256(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Fetch and parse checksum file from GitHub release
    ///
    /// # Security
    /// Downloads the official SHA2-256SUMS file and extracts the checksum
    /// for the specific platform binary
    async fn fetch_and_parse_checksum(
        &self,
        client: &reqwest::Client,
        checksums_url: &str,
        asset_name: &str,
    ) -> Result<String, String> {
        tracing::debug!("Fetching checksums from: {}", checksums_url);

        let response = client
            .get(checksums_url)
            .header("User-Agent", "ripVID")
            .send()
            .await
            .map_err(|e| format!("Failed to download checksum file: {}", e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to download checksum file: HTTP {}",
                response.status()
            ));
        }

        let checksums_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read checksum file: {}", e))?;

        // Parse the checksums file (format: "hash  filename")
        // Example: "a1b2c3d4...  yt-dlp.exe"
        for line in checksums_text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let hash = parts[0];
                let filename = parts[1];

                // Match the filename to our asset
                if filename == asset_name {
                    tracing::debug!("Found checksum for {}: {}", asset_name, hash);
                    return Ok(hash.to_string());
                }
            }
        }

        Err(format!(
            "Checksum not found for {} in checksum file",
            asset_name
        ))
    }
}
