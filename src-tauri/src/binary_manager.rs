use hex;
use reqwest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinaryInfo {
    pub name: String,
    pub version: String,
    pub last_check: u64,
    pub path: String,
}

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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub binary: String,
    pub progress: f64,
    pub status: String,
}

#[derive(Clone)]
pub struct BinaryManager {
    app_handle: AppHandle,
    data_dir: PathBuf,
}

impl BinaryManager {
    pub fn new(app_handle: AppHandle) -> Self {
        let data_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("binaries");

        Self {
            app_handle,
            data_dir,
        }
    }

    /// Ensure all binaries are present and up-to-date
    /// This is called on app startup
    pub async fn ensure_all_binaries(&self) -> Result<(), String> {
        info!("Ensuring all required binaries are present...");

        // Create data directory
        fs::create_dir_all(&self.data_dir)
            .map_err(|e| format!("Failed to create binaries directory: {}", e))?;

        // Check each binary
        let mut missing = Vec::new();

        if !self.is_binary_present("yt-dlp")? {
            missing.push("yt-dlp");
        }
        if !self.is_binary_present("ffmpeg")? {
            missing.push("ffmpeg");
        }
        if !self.is_binary_present("ffprobe")? {
            missing.push("ffprobe");
        }

        // If any are missing, download them (first run)
        if !missing.is_empty() {
            info!("First run detected. Downloading: {:?}", missing);
            self.emit_progress("setup", 0.0, "Downloading required tools...")?;

            // Download in parallel for speed
            let manager1 = self.clone_for_background();
            let manager2 = self.clone_for_background();
            let manager3 = self.clone_for_background();

            let handles = vec![
                tokio::spawn(async move { manager1.download_ytdlp().await }),
                tokio::spawn(async move { manager2.download_ffmpeg().await }),
                tokio::spawn(async move { manager3.download_ffprobe().await }),
            ];

            let mut errors = Vec::new();
            const BINARY_NAMES: [&str; 3] = ["yt-dlp", "ffmpeg", "ffprobe"];
            for (i, handle) in handles.into_iter().enumerate() {
                let binary_name = BINARY_NAMES[i];
                match handle.await {
                    Ok(Ok(())) => {
                        info!("{} downloaded successfully", binary_name);
                    }
                    Ok(Err(e)) => {
                        error!("{} download failed: {}", binary_name, e);
                        errors.push(format!("{}: {}", binary_name, e));
                    }
                    Err(e) => {
                        error!("{} task panicked: {}", binary_name, e);
                        errors.push(format!("{}: task failed", binary_name));
                    }
                }
            }

            if !errors.is_empty() {
                return Err(format!("Failed to download: {}", errors.join(", ")));
            }

            self.emit_progress("setup", 100.0, "All tools ready!")?;
        }

        // Check for updates in background (non-blocking)
        let manager = self.clone_for_background();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = manager.check_updates_background().await {
                warn!("Background update check failed: {}", e);
            }
        });

        Ok(())
    }

    /// Check for updates in the background (once per day)
    async fn check_updates_background(&self) -> Result<(), String> {
        if !self.should_check_updates()? {
            return Ok(());
        }

        info!("Checking for binary updates...");

        // Update each binary if needed (non-blocking, best effort)
        let _ = self.update_ytdlp_if_needed().await;
        let _ = self.update_ffmpeg_if_needed().await;

        Ok(())
    }

    fn should_check_updates(&self) -> Result<bool, String> {
        let version_file = self.data_dir.join("last-check.json");

        if !version_file.exists() {
            return Ok(true);
        }

        let content = fs::read_to_string(&version_file).ok();
        if let Some(content) = content {
            if let Ok(last_check) = content.parse::<u64>() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                // Check once per day
                return Ok(now - last_check > 86400);
            }
        }

        Ok(true)
    }

    fn save_last_check(&self) -> Result<(), String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let version_file = self.data_dir.join("last-check.json");
        fs::write(version_file, now.to_string()).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Check if a binary is present
    fn is_binary_present(&self, name: &str) -> Result<bool, String> {
        let path = self.get_binary_path(name)?;
        Ok(path.exists())
    }

    /// Get the path for a binary
    pub fn get_binary_path(&self, name: &str) -> Result<PathBuf, String> {
        let filename = if cfg!(windows) {
            format!("{}.exe", name)
        } else {
            name.to_string()
        };

        Ok(self.data_dir.join(filename))
    }

    /// Download yt-dlp
    async fn download_ytdlp(&self) -> Result<(), String> {
        self.emit_progress("yt-dlp", 0.0, "Downloading yt-dlp...")?;

        let client = reqwest::Client::new();

        // Get latest release
        let response = client
            .get("https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest")
            .header("User-Agent", "ripVID")
            .send()
            .await
            .map_err(|e| format!("Failed to fetch yt-dlp release: {}", e))?;

        let release: GitHubRelease = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse release: {}", e))?;

        // Find the right asset
        let asset_name = self.get_ytdlp_asset_name();
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("No asset found for {}", asset_name))?;

        self.emit_progress("yt-dlp", 25.0, "Downloading binary...")?;

        // Download binary
        let response = client
            .get(&asset.browser_download_url)
            .send()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read bytes: {}", e))?;

        self.emit_progress("yt-dlp", 75.0, "Verifying checksum...")?;

        // Verify checksum
        let checksums_url = format!(
            "https://github.com/yt-dlp/yt-dlp/releases/download/{}/SHA2-256SUMS",
            release.tag_name
        );

        let expected_checksum = self
            .fetch_and_parse_checksum(&client, &checksums_url, asset_name)
            .await?;

        let actual_checksum = self.calculate_sha256(&bytes);

        if actual_checksum.to_lowercase() != expected_checksum.to_lowercase() {
            return Err(format!(
                "Checksum mismatch! Expected: {}, Got: {}",
                expected_checksum, actual_checksum
            ));
        }

        // Save binary
        let path = self.get_binary_path("yt-dlp")?;
        fs::write(&path, bytes).map_err(|e| format!("Failed to save: {}", e))?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&path, permissions)
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }

        // Save version info
        self.save_binary_info("yt-dlp", &release.tag_name, &path)?;

        self.emit_progress("yt-dlp", 100.0, "Ready!")?;

        info!("yt-dlp {} installed successfully", release.tag_name);

        Ok(())
    }

    /// Download ffmpeg with fallback sources
    async fn download_ffmpeg(&self) -> Result<(), String> {
        self.emit_progress("ffmpeg", 0.0, "Downloading ffmpeg...")?;

        let client = reqwest::Client::new();

        // Try multiple sources for reliability
        let sources = self.get_ffmpeg_sources();

        for (i, source) in sources.iter().enumerate() {
            info!("Trying ffmpeg source {}/{}: {}", i + 1, sources.len(), source.name);

            match self.download_from_source(&client, "ffmpeg", source).await {
                Ok(()) => {
                    self.emit_progress("ffmpeg", 100.0, "Ready!")?;
                    info!("ffmpeg downloaded successfully from {}", source.name);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to download from {}: {}", source.name, e);
                    if i < sources.len() - 1 {
                        info!("Trying next source...");
                    }
                }
            }
        }

        Err("All ffmpeg sources failed".to_string())
    }

    /// Download ffprobe with fallback sources
    async fn download_ffprobe(&self) -> Result<(), String> {
        self.emit_progress("ffprobe", 0.0, "Downloading ffprobe...")?;

        let client = reqwest::Client::new();

        let sources = self.get_ffprobe_sources();

        for (i, source) in sources.iter().enumerate() {
            info!("Trying ffprobe source {}/{}: {}", i + 1, sources.len(), source.name);

            match self.download_from_source(&client, "ffprobe", source).await {
                Ok(()) => {
                    self.emit_progress("ffprobe", 100.0, "Ready!")?;
                    info!("ffprobe downloaded successfully from {}", source.name);
                    return Ok(());
                }
                Err(e) => {
                    warn!("Failed to download from {}: {}", source.name, e);
                    if i < sources.len() - 1 {
                        info!("Trying next source...");
                    }
                }
            }
        }

        Err("All ffprobe sources failed".to_string())
    }

    async fn download_from_source(
        &self,
        client: &reqwest::Client,
        binary_name: &str,
        source: &DownloadSource,
    ) -> Result<(), String> {
        self.emit_progress(binary_name, 25.0, &format!("Downloading from {}...", source.name))?;

        let response = client
            .get(&source.url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("HTTP {}", response.status()));
        }

        let bytes = response.bytes().await.map_err(|e| e.to_string())?;

        self.emit_progress(binary_name, 75.0, "Saving binary...")?;

        // Handle zip extraction if needed
        let final_bytes = if source.is_zip {
            self.extract_from_zip(&bytes, binary_name)?
        } else {
            bytes.to_vec()
        };

        // Save binary
        let path = self.get_binary_path(binary_name)?;
        fs::write(&path, final_bytes).map_err(|e| format!("Failed to save: {}", e))?;

        // Make executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&path, permissions)
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }

        // Save version info
        self.save_binary_info(binary_name, &source.version, &path)?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn extract_from_zip(&self, bytes: &[u8], binary_name: &str) -> Result<Vec<u8>, String> {
        use std::io::Cursor;
        use zip::ZipArchive;

        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).map_err(|e| format!("Invalid zip: {}", e))?;

        // Look for the binary in the zip
        let target_name = format!("{}.exe", binary_name);

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let file_name = file.name().to_string();

            if file_name.ends_with(&target_name) {
                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer).map_err(|e| e.to_string())?;
                return Ok(buffer);
            }
        }

        Err(format!("{} not found in zip", target_name))
    }

    #[cfg(not(target_os = "windows"))]
    fn extract_from_zip(&self, bytes: &[u8], binary_name: &str) -> Result<Vec<u8>, String> {
        use std::io::Cursor;
        use zip::ZipArchive;

        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).map_err(|e| format!("Invalid zip: {}", e))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let file_name = file.name().to_string();

            if file_name.ends_with(binary_name) || file_name.contains(binary_name) {
                let mut buffer = Vec::new();
                std::io::copy(&mut file, &mut buffer).map_err(|e| e.to_string())?;
                return Ok(buffer);
            }
        }

        Err(format!("{} not found in zip", binary_name))
    }

    fn get_ytdlp_asset_name(&self) -> &str {
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

    fn get_ffmpeg_sources(&self) -> Vec<DownloadSource> {
        #[cfg(target_os = "windows")]
        return vec![
            DownloadSource {
                name: "GyanD/codexffmpeg",
                url: "https://github.com/GyanD/codexffmpeg/releases/download/6.0/ffmpeg-6.0-essentials_build.zip".to_string(),
                version: "6.0".to_string(),
                is_zip: true,
            },
            DownloadSource {
                name: "BtbN/FFmpeg-Builds",
                url: "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip".to_string(),
                version: "latest".to_string(),
                is_zip: true,
            },
        ];

        #[cfg(target_os = "macos")]
        return vec![
            DownloadSource {
                name: "evermeet.cx",
                url: "https://evermeet.cx/ffmpeg/ffmpeg-6.0.zip".to_string(),
                version: "6.0".to_string(),
                is_zip: true,
            },
        ];

        #[cfg(target_os = "linux")]
        return vec![
            DownloadSource {
                name: "johnvansickle.com",
                url: "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz".to_string(),
                version: "latest".to_string(),
                is_zip: false,
            },
        ];
    }

    fn get_ffprobe_sources(&self) -> Vec<DownloadSource> {
        #[cfg(target_os = "windows")]
        return vec![
            DownloadSource {
                name: "GyanD/codexffmpeg",
                url: "https://github.com/GyanD/codexffmpeg/releases/download/6.0/ffmpeg-6.0-essentials_build.zip".to_string(),
                version: "6.0".to_string(),
                is_zip: true,
            },
        ];

        #[cfg(target_os = "macos")]
        return vec![
            DownloadSource {
                name: "evermeet.cx",
                url: "https://evermeet.cx/ffmpeg/ffprobe-6.0.zip".to_string(),
                version: "6.0".to_string(),
                is_zip: true,
            },
        ];

        #[cfg(target_os = "linux")]
        return vec![
            DownloadSource {
                name: "johnvansickle.com",
                url: "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz".to_string(),
                version: "latest".to_string(),
                is_zip: false,
            },
        ];
    }

    async fn update_ytdlp_if_needed(&self) -> Result<(), String> {
        // Similar to download_ytdlp but checks version first
        Ok(())
    }

    async fn update_ffmpeg_if_needed(&self) -> Result<(), String> {
        // Check if update is available
        Ok(())
    }

    fn save_binary_info(&self, name: &str, version: &str, path: &PathBuf) -> Result<(), String> {
        let info = BinaryInfo {
            name: name.to_string(),
            version: version.to_string(),
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            path: path.to_string_lossy().to_string(),
        };

        let info_file = self.data_dir.join(format!("{}-info.json", name));
        let json = serde_json::to_string_pretty(&info).map_err(|e| e.to_string())?;

        fs::write(info_file, json).map_err(|e| e.to_string())?;

        self.save_last_check()?;

        Ok(())
    }

    fn calculate_sha256(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    async fn fetch_and_parse_checksum(
        &self,
        client: &reqwest::Client,
        checksums_url: &str,
        asset_name: &str,
    ) -> Result<String, String> {
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

        for line in checksums_text.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let hash = parts[0];
                let filename = parts[1];

                if filename == asset_name {
                    return Ok(hash.to_string());
                }
            }
        }

        Err(format!("Checksum not found for {}", asset_name))
    }

    fn emit_progress(&self, binary: &str, progress: f64, status: &str) -> Result<(), String> {
        let event = DownloadProgress {
            binary: binary.to_string(),
            progress,
            status: status.to_string(),
        };

        self.app_handle
            .emit("binary-download-progress", event)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn clone_for_background(&self) -> Self {
        self.clone()
    }
}

struct DownloadSource {
    name: &'static str,
    url: String,
    version: String,
    is_zip: bool,
}
