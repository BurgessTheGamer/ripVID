#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::collections::HashMap;
use std::fs;
use std::process::Command;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

mod binary_manager;
mod download;
mod errors;
mod logging;
mod validation;
mod ytdlp_updater;

use binary_manager::BinaryManager;
use download::{
    cancel_download, download_content_with_smart_retry, BrowserConfig, DownloadHandle, DownloadType,
};
use validation::validate_path;
use ytdlp_updater::YtdlpUpdater;

/// Application state shared across all commands
struct AppState {
    ytdlp_updater: Arc<Mutex<YtdlpUpdater>>,
    active_downloads: Arc<Mutex<HashMap<String, DownloadHandle>>>,
    binary_manager: Arc<BinaryManager>,
}

/// Detect the platform from a URL
#[tauri::command]
async fn detect_platform(url: String) -> Result<String, String> {
    info!("Detecting platform for URL: {}", url);

    if url.contains("youtube.com") || url.contains("youtu.be") {
        Ok("youtube".to_string())
    } else if url.contains("x.com") || url.contains("twitter.com") {
        Ok("x".to_string())
    } else if url.contains("facebook.com") || url.contains("fb.watch") {
        Ok("facebook".to_string())
    } else if url.contains("instagram.com") {
        Ok("instagram".to_string())
    } else if url.contains("tiktok.com") {
        Ok("tiktok".to_string())
    } else {
        warn!("Unsupported platform: {}", url);
        Err("Unsupported platform".to_string())
    }
}

/// Get video information using yt-dlp
#[tauri::command]
async fn get_video_info(url: String, app: tauri::AppHandle) -> Result<String, String> {
    info!("Fetching video info for: {}", url);

    let output = app
        .shell()
        .sidecar("yt-dlp")
        .map_err(|e| {
            error!("Failed to create sidecar: {}", e);
            e.to_string()
        })?
        .args(&["--no-playlist", "--dump-json", &url])
        .output()
        .await
        .map_err(|e| {
            error!("Failed to execute yt-dlp: {}", e);
            e.to_string()
        })?;

    if output.status.success() {
        let json_output = String::from_utf8_lossy(&output.stdout).to_string();
        info!("Successfully fetched video info");
        Ok(json_output)
    } else {
        let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
        error!("Failed to fetch video info: {}", error_msg);
        Err(error_msg)
    }
}

/// Download video with specified quality
/// Uses smart retry: tries without cookies first, auto-retries with cookies if needed
#[tauri::command]
async fn download_video(
    url: String,
    output_path: String,
    quality: String,
    _use_browser_cookies: Option<bool>, // Deprecated but kept for API compatibility
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!("Video download requested: url={}, quality={}", url, quality);

    // Use smart retry - no manual cookie configuration needed
    download_content_with_smart_retry(
        url,
        output_path,
        DownloadType::Video { quality },
        window,
        app,
        state.ytdlp_updater.clone(),
        state.active_downloads.clone(),
        state.binary_manager.clone(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// Download audio (MP3)
/// Uses smart retry: tries without cookies first, auto-retries with cookies if needed
#[tauri::command]
async fn download_audio(
    url: String,
    output_path: String,
    _use_browser_cookies: Option<bool>, // Deprecated but kept for API compatibility
    window: tauri::WebviewWindow,
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    info!("Audio download requested: url={}", url);

    // Use smart retry - no manual cookie configuration needed
    download_content_with_smart_retry(
        url,
        output_path,
        DownloadType::Audio,
        window,
        app,
        state.ytdlp_updater.clone(),
        state.active_downloads.clone(),
        state.binary_manager.clone(),
    )
    .await
    .map_err(|e| e.to_string())
}

/// Cancel an active download
#[tauri::command]
async fn cancel_download_command(
    download_id: String,
    window: tauri::WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    info!("Cancel requested for download: {}", download_id);

    cancel_download(download_id, state.active_downloads.clone(), window)
        .await
        .map_err(|e| e.to_string())
}

/// Create a directory
#[tauri::command]
fn create_directory(path: String) -> Result<(), String> {
    info!("Creating directory: {}", path);
    fs::create_dir_all(&path).map_err(|e| {
        error!("Failed to create directory {}: {}", path, e);
        e.to_string()
    })
}

/// Open file location in the system file manager
/// Gracefully handles missing files by opening parent directory instead
#[tauri::command]
fn open_file_location(path: String) -> Result<(), String> {
    info!("Opening file location: {}", path);

    // Basic security: ensure path is within user's home directory
    // But be more lenient to handle edge cases
    let path_buf = std::path::PathBuf::from(&path);

    // Check if path is absolute (basic security)
    if !path_buf.is_absolute() {
        warn!("Rejected relative path: {}", path);
        return Err("Invalid path: must be absolute".to_string());
    }

    // Ensure path is within safe directories
    if let Some(home) = dirs::home_dir() {
        if !path.starts_with(home.to_string_lossy().as_ref()) {
            warn!("Path outside home directory: {}", path);
            return Err("Access denied: path outside allowed directories".to_string());
        }
    }

    // Try to open the exact file if it exists
    if path_buf.exists() && path_buf.is_file() {
        info!("File exists, opening with file manager");

        #[cfg(target_os = "windows")]
        {
            let normalized_path = path.replace("/", "\\");
            let result = Command::new("explorer")
                .args(&["/select,", &normalized_path])
                .spawn();

            return match result {
                Ok(_) => {
                    info!("Successfully opened Windows Explorer");
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to open with Explorer: {}", e);
                    Err(format!("Failed to open file manager: {}", e))
                }
            };
        }

        #[cfg(not(target_os = "windows"))]
        {
            // For macOS/Linux, continue to platform-specific code below
        }
    } else {
        // File doesn't exist - try to open parent directory instead
        warn!(
            "File not found: {}. Attempting to open parent directory.",
            path
        );

        if let Some(parent) = path_buf.parent() {
            if parent.exists() {
                info!("Opening parent directory: {:?}", parent);
                let parent_str = parent.to_string_lossy().to_string();
                return open_folder_fallback(parent_str);
            }
        }

        // Neither file nor parent exists
        return Err("File not found. It may have been moved or deleted.".to_string());
    }

    // macOS/Linux file opening (file is confirmed to exist at this point)
    #[cfg(target_os = "macos")]
    {
        let path_str = path.clone();
        let result = Command::new("open").args(&["-R", &path_str]).spawn();

        match result {
            Ok(_) => {
                info!("Successfully opened Finder");
                return Ok(());
            }
            Err(e) => {
                warn!("Failed to open with Finder: {}", e);
                return open_folder_fallback(path_str);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        let path_str = path.clone();
        if let Some(parent) = path_buf.parent() {
            let parent_str = parent.to_string_lossy().to_string();

            // Try xdg-open first
            if Command::new("xdg-open").arg(&parent_str).spawn().is_ok() {
                info!("Successfully opened file manager with xdg-open");
                return Ok(());
            }

            // Try nautilus (GNOME)
            if Command::new("nautilus")
                .arg("--select")
                .arg(&path_str)
                .spawn()
                .is_ok()
            {
                info!("Successfully opened Nautilus");
                return Ok(());
            }

            // Try dolphin (KDE)
            if Command::new("dolphin")
                .arg("--select")
                .arg(&path_str)
                .spawn()
                .is_ok()
            {
                info!("Successfully opened Dolphin");
                return Ok(());
            }

            return open_folder_fallback(parent_str);
        }
        return Err("Could not open file location on Linux".to_string());
    }

    #[allow(unreachable_code)]
    Err("Could not open file location".to_string())
}

/// Helper function to open just the folder
/// Assumes path has already been validated by caller
fn open_folder_fallback(path: String) -> Result<(), String> {
    let path_buf = std::path::PathBuf::from(&path);

    let folder_path = if path_buf.is_file() {
        path_buf
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone())
    } else {
        path.clone()
    };

    info!("Opening folder: {}", folder_path);

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&folder_path.replace("/", "\\"))
            .spawn()
            .map_err(|e| {
                error!("Failed to open folder: {}", e);
                format!("Failed to open folder: {}", e)
            })?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| {
                error!("Failed to open folder: {}", e);
                format!("Failed to open folder: {}", e)
            })?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&folder_path)
            .spawn()
            .map_err(|e| {
                error!("Failed to open folder: {}", e);
                format!("Failed to open folder: {}", e)
            })?;
    }

    Ok(())
}

/// Move a file to the recycle bin
#[tauri::command]
fn recycle_file(path: String) -> Result<(), String> {
    info!("Moving file to recycle bin: {}", path);
    trash::delete(&path).map_err(|e| {
        error!("Failed to recycle file {}: {}", path, e);
        e.to_string()
    })
}

/// Check if a file exists at the given path
#[tauri::command]
fn file_exists(path: String) -> Result<bool, String> {
    let path_buf = std::path::PathBuf::from(&path);
    Ok(path_buf.exists() && path_buf.is_file())
}

/// Scan downloads folders and return list of actual files
#[tauri::command]
async fn scan_downloads_folder() -> Result<Vec<serde_json::Value>, String> {
    use serde_json::json;

    let home = dirs::home_dir().ok_or("Could not determine home directory")?;
    let ripvid_base = home.join("Videos").join("ripVID");

    let mut files = Vec::new();

    // Scan MP4 folder
    let mp4_dir = ripvid_base.join("MP4");
    if mp4_dir.exists() {
        if let Ok(entries) = fs::read_dir(&mp4_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        let filename = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        files.push(json!({
                            "path": path.to_string_lossy().to_string(),
                            "filename": filename,
                            "format": "mp4",
                            "size": metadata.len(),
                            "modified": metadata.modified()
                                .ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs())
                        }));
                    }
                }
            }
        }
    }

    // Scan MP3 folder
    let mp3_dir = ripvid_base.join("MP3");
    if mp3_dir.exists() {
        if let Ok(entries) = fs::read_dir(&mp3_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        let filename = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        files.push(json!({
                            "path": path.to_string_lossy().to_string(),
                            "filename": filename,
                            "format": "mp3",
                            "size": metadata.len(),
                            "modified": metadata.modified()
                                .ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs())
                        }));
                    }
                }
            }
        }
    }

    info!("Scanned downloads folder, found {} files", files.len());
    Ok(files)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Initialize logging
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));

            if let Err(e) = logging::init_logging(app_data_dir.clone()) {
                eprintln!("Failed to initialize logging: {}", e);
            }

            info!("ripVID application starting...");
            info!("App data directory: {:?}", app_data_dir);

            // Initialize binary manager for runtime binary downloads
            info!("Initializing binary manager...");
            let binary_manager = Arc::new(BinaryManager::new(app.handle().clone()));

            // Ensure all binaries are downloaded/updated (blocks window until ready)
            info!("Ensuring all binaries are ready...");
            let manager_clone = binary_manager.clone();
            tauri::async_runtime::block_on(async move {
                match manager_clone.ensure_all_binaries().await {
                    Ok(()) => info!("All binaries ready"),
                    Err(e) => {
                        error!("Failed to ensure binaries: {}", e);
                        return Err(e);
                    }
                }
                Ok::<(), String>(())
            })?;

            // Initialize yt-dlp updater (legacy - will be replaced by binary manager)
            let updater = YtdlpUpdater::new(app.handle().clone());

            // Check for updates on startup (non-blocking)
            let updater_clone = updater.clone_for_background();
            tauri::async_runtime::spawn(async move {
                match updater_clone.ensure_updated().await {
                    Ok(path) => info!("yt-dlp ready at: {:?}", path),
                    Err(e) => warn!("Failed to update yt-dlp: {}", e),
                }
            });

            // Initialize app state
            app.manage(AppState {
                ytdlp_updater: Arc::new(Mutex::new(updater)),
                active_downloads: Arc::new(Mutex::new(HashMap::new())),
                binary_manager: binary_manager.clone(),
            });

            info!("Application setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            detect_platform,
            get_video_info,
            download_video,
            download_audio,
            cancel_download_command,
            create_directory,
            open_file_location,
            recycle_file,
            file_exists,
            scan_downloads_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
