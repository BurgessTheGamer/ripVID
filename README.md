# Video Downloader

A beautiful desktop application for downloading videos from YouTube and X (Twitter) at the highest quality. Built with Tauri, React, TypeScript, and shadcn/ui components with a purple-tinted theme.

## Features

- 🎥 Download videos from YouTube and X/Twitter
- 🚀 Highest quality video downloads
- 🎨 Modern UI with purple-themed shadcn/ui components
- 🔍 Auto-detect platform from URL
- 📊 Real-time download progress tracking
- 💾 Small executable size (~10MB)
- ⚡ Fast and efficient with Rust backend

## Prerequisites

### Required

1. **Node.js** (v16 or higher)
2. **Rust** (latest stable version)
3. **yt-dlp** - The core download engine

### Installing Prerequisites

#### Windows

1. Install Node.js from [nodejs.org](https://nodejs.org/)
2. Install Rust from [rustup.rs](https://rustup.rs/)
3. Install yt-dlp:
   ```powershell
   # Using Python pip
   pip install yt-dlp
   
   # Or download the executable from:
   # https://github.com/yt-dlp/yt-dlp/releases
   # Place yt-dlp.exe in your PATH
   ```

## Setup

1. Clone or navigate to the project directory:
   ```bash
   cd video-downloader
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

## Development

Run the application in development mode:

```bash
npm run tauri:dev
```

This will:
- Start the Vite development server
- Launch the Tauri application window
- Enable hot module replacement for React

## Building for Windows

1. Build the application:
   ```bash
   npm run tauri:build
   ```

2. The Windows executable will be created in:
   ```
   src-tauri/target/release/video-downloader.exe
   ```

3. The installer will be available in:
   ```
   src-tauri/target/release/bundle/msi/
   ```

## Important Notes

### yt-dlp Requirement

The application requires `yt-dlp` to be installed and accessible in your system PATH. The app uses yt-dlp as the download engine because:
- It's the most reliable and maintained solution
- Supports 1000+ video platforms
- Handles authentication and quality selection
- Receives daily updates for compatibility

### FFmpeg (Optional)

For best results and format conversion capabilities, also install FFmpeg:
- Download from [ffmpeg.org](https://ffmpeg.org/download.html)
- Add to your system PATH

## Usage

1. Launch the application
2. Paste a YouTube or X/Twitter video URL
3. The platform will be auto-detected
4. Click "Download" and choose save location
5. Monitor download progress in real-time

## Troubleshooting

### "yt-dlp not found" error
- Ensure yt-dlp is installed: `yt-dlp --version`
- Make sure it's in your system PATH
- Restart the application after installation

### Build fails with Rust errors
- Update Rust: `rustup update`
- Clean and rebuild: `cargo clean` then `npm run tauri:build`

### Download fails
- Update yt-dlp: `pip install --upgrade yt-dlp`
- Check if the video URL is valid
- Some videos may be region-restricted or private

## Tech Stack

- **Frontend**: React + TypeScript
- **UI Components**: shadcn/ui with Tailwind CSS
- **Desktop Framework**: Tauri (Rust)
- **Download Engine**: yt-dlp
- **Styling**: Purple-themed design system

## License

For personal use only.

## Acknowledgments

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) for the amazing download engine
- [Tauri](https://tauri.app) for the lightweight desktop framework
- [shadcn/ui](https://ui.shadcn.com) for beautiful React components