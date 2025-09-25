# Quick Start Guide

## Fix for Script Error

**You need to be INSIDE the video-downloader directory to run commands:**

```bash
cd video-downloader
npm run tauri:dev
```

## Prerequisites Checklist

1. ✅ yt-dlp installed (you've done this)
2. ⚠️ Rust needs to be installed: https://rustup.rs/
3. ⚠️ Visual Studio Build Tools (for Windows): Required for Rust compilation

## Commands

```bash
# Navigate to the project
cd video-downloader

# Install dependencies (if needed)
npm install

# Run in development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

## Troubleshooting

- If Rust compilation fails, install Visual Studio Build Tools
- Make sure yt-dlp is in your PATH
- Run commands from INSIDE the video-downloader folder