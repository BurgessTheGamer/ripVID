#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::process::Command;
use std::fs;
use tauri::Emitter;
use serde::{Deserialize, Serialize};
use serde_json;
use regex::Regex;

#[derive(Debug, Serialize, Deserialize)]
struct DownloadProgress {
    percent: f32,
    speed: String,
    eta: String,
}

#[tauri::command]
async fn detect_platform(url: String) -> Result<String, String> {
    if url.contains("youtube.com") || url.contains("youtu.be") {
        Ok("youtube".to_string())
    } else if url.contains("x.com") || url.contains("twitter.com") {
        Ok("x".to_string())
    } else {
        Err("Unsupported platform".to_string())
    }
}

#[tauri::command]
async fn get_video_info(url: String) -> Result<String, String> {
    let output = Command::new("yt-dlp")
        .args(&["--no-playlist", "--dump-json", &url])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
async fn download_video(
    url: String,
    output_path: String,
    quality: String,
    window: tauri::WebviewWindow,
) -> Result<String, String> {
    let mut args = vec![
        url.clone(),
        "--no-playlist".to_string(),
        "-f".to_string(),
        "bestvideo[ext=mp4][vcodec^=avc]+bestaudio[ext=m4a]/best[ext=mp4]/best".to_string(),  // Force H.264 codec, NOT AV1
        "--merge-output-format".to_string(),
        "mp4".to_string(),
        "-o".to_string(),
        output_path.clone(),
        "--progress".to_string(),
        "--newline".to_string(),
    ];

    println!("Running yt-dlp with args: {:?}", args);

    let mut child = Command::new("yt-dlp")
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn yt-dlp: {}", e))?;

    // Emit download started event immediately
    window.emit("download-started", output_path.clone()).ok();

    // Take ownership of stdout and stderr
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let window_clone = window.clone();
    let window_clone2 = window.clone();

    // Spawn thread to handle stdout (progress)
    if let Some(stdout) = stdout {
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[stdout] {}", line);

                    if line.contains("[download]") && line.contains("%") {
                        if let Some(percent_match) = Regex::new(r"(\d+(?:\.\d+)?)%").unwrap().captures(&line) {
                            let percent: f32 = percent_match[1].parse().unwrap_or(0.0);

                            let speed = if let Some(speed_match) = Regex::new(r"at\s+(\S+)").unwrap().captures(&line) {
                                speed_match[1].to_string()
                            } else {
                                "---".to_string()
                            };

                            let eta = if let Some(eta_match) = Regex::new(r"ETA\s+(\S+)").unwrap().captures(&line) {
                                eta_match[1].to_string()
                            } else {
                                "--:--".to_string()
                            };

                            let progress = DownloadProgress {
                                percent,
                                speed,
                                eta,
                            };
                            window_clone.emit("download-progress", &progress).ok();
                        }
                    }
                }
            }
        });
    }

    // Spawn thread to handle stderr (status messages)
    if let Some(stderr) = stderr {
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stderr);

            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[stderr] {}", line);

                    // Emit status messages for important events
                    if line.contains("Sleeping") || line.contains("rate limit") {
                        window_clone2.emit("download-status", line.clone()).ok();
                    }
                }
            }
        });
    }

    // Spawn thread to wait for process completion
    std::thread::spawn(move || {
        let status = child.wait();
        match status {
            Ok(status) if status.success() => {
                window.emit("download-complete", serde_json::json!({
                    "success": true,
                    "path": output_path
                })).ok();
            }
            Ok(status) => {
                let exit_code = status.code().unwrap_or(-1);
                window.emit("download-complete", serde_json::json!({
                    "success": false,
                    "error": format!("Exit code: {}", exit_code)
                })).ok();
            }
            Err(e) => {
                window.emit("download-complete", serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                })).ok();
            }
        }
    });

    // Return immediately - download is running in background
    Ok("Download started successfully".to_string())
}

#[tauri::command]
fn create_directory(path: String) -> Result<(), String> {
    fs::create_dir_all(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn open_file_location(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .args(&["/select,", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(&["-R", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try to open the parent directory
        if let Some(parent) = std::path::Path::new(&path).parent() {
            Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
async fn download_audio(
    url: String,
    output_path: String,
    window: tauri::WebviewWindow,
) -> Result<String, String> {
    let args = vec![
        url.clone(),
        "--no-playlist".to_string(),
        "-x".to_string(),
        "--audio-format".to_string(),
        "mp3".to_string(),
        "--audio-quality".to_string(),
        "0".to_string(),
        "--embed-thumbnail".to_string(),
        "--add-metadata".to_string(),
        "-o".to_string(),
        output_path.clone(),
        "--progress".to_string(),
        "--newline".to_string(),
    ];

    println!("Running yt-dlp with args: {:?}", args);

    let mut child = Command::new("yt-dlp")
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn yt-dlp: {}", e))?;

    // Emit download started event immediately
    window.emit("download-started", output_path.clone()).ok();

    // Take ownership of stdout and stderr
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let window_clone = window.clone();
    let window_clone2 = window.clone();

    // Spawn thread to handle stdout (progress)
    if let Some(stdout) = stdout {
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[stdout] {}", line);

                    if line.contains("[download]") && line.contains("%") {
                        if let Some(percent_match) = Regex::new(r"(\d+(?:\.\d+)?)%").unwrap().captures(&line) {
                            let percent: f32 = percent_match[1].parse().unwrap_or(0.0);

                            let speed = if let Some(speed_match) = Regex::new(r"at\s+(\S+)").unwrap().captures(&line) {
                                speed_match[1].to_string()
                            } else {
                                "---".to_string()
                            };

                            let eta = if let Some(eta_match) = Regex::new(r"ETA\s+(\S+)").unwrap().captures(&line) {
                                eta_match[1].to_string()
                            } else {
                                "--:--".to_string()
                            };

                            let progress = DownloadProgress {
                                percent,
                                speed,
                                eta,
                            };
                            window_clone.emit("download-progress", &progress).ok();
                        }
                    }
                }
            }
        });
    }

    // Spawn thread to handle stderr (status messages)
    if let Some(stderr) = stderr {
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(stderr);

            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("[stderr] {}", line);

                    // Emit status messages for important events
                    if line.contains("Sleeping") || line.contains("rate limit") {
                        window_clone2.emit("download-status", line.clone()).ok();
                    }
                }
            }
        });
    }

    // Spawn thread to wait for process completion
    std::thread::spawn(move || {
        let status = child.wait();
        match status {
            Ok(status) if status.success() => {
                window.emit("download-complete", serde_json::json!({
                    "success": true,
                    "path": output_path
                })).ok();
            }
            Ok(status) => {
                let exit_code = status.code().unwrap_or(-1);
                window.emit("download-complete", serde_json::json!({
                    "success": false,
                    "error": format!("Exit code: {}", exit_code)
                })).ok();
            }
            Err(e) => {
                window.emit("download-complete", serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                })).ok();
            }
        }
    });

    // Return immediately - download is running in background
    Ok("Audio download started successfully".to_string())
}

#[tauri::command]
fn recycle_file(path: String) -> Result<(), String> {
    trash::delete(&path).map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            detect_platform,
            get_video_info,
            download_video,
            download_audio,
            create_directory,
            open_file_location,
            recycle_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}