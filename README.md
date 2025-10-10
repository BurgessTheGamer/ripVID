# ripVID

https://github.com/user-attachments/assets/db3929ad-2251-4d2b-8641-d3c9e34bb72d

A beautiful, modern desktop application for downloading videos from popular platforms at the highest quality. Built with Tauri v2, React, TypeScript, and a custom purple-themed UI.

## ✨ Features

### Core Features
- 🎥 **Multi-Platform Support**: Download videos from YouTube, X/Twitter, TikTok, Facebook, Instagram, and 1000+ sites
- 🚀 **Highest Quality Downloads**: H.264/MP4 video with automatic quality selection
- 🎵 **Audio Extraction**: Download audio-only in MP3 format
- 🔄 **Auto-Updates**: Built-in automatic update system keeps the app current
- 📦 **Runtime Binary Management**: Automatically downloads and updates yt-dlp, ffmpeg, and ffprobe on first launch
- 🍪 **Smart Cookie Handling**: Automatic browser cookie extraction for age-restricted and private videos
- 📊 **Real-time Progress**: Live download progress with speed and ETA tracking

### User Experience
- 🎨 **Modern UI**: Minimalist interface with elegant purple accents
- 🔍 **Auto-Detection**: Automatically detects platform from URL
- 💾 **Lightweight**: Small executable with efficient resource usage
- ⚡ **Fast**: Rust backend for optimal performance
- 📂 **Library Management**: View and manage downloaded files directly in the app

### Technical Features
- 🔐 **Secure**: Cryptographically signed updates with minisign
- 🌐 **Cross-Platform**: Windows, macOS, and Linux support
- 🔄 **Daily Updates**: Background checks for yt-dlp updates
- ✅ **SHA-256 Verification**: All downloaded binaries verified for integrity

## 🔧 Prerequisites for Development

1. **Node.js** (v18 or higher) - [nodejs.org](https://nodejs.org/)
2. **Rust** (latest stable) - [rustup.rs](https://rustup.rs/)
3. **Bun** (recommended for faster builds) - [bun.sh](https://bun.sh/)

**Note**: End users don't need any prerequisites - all binaries are downloaded automatically on first launch!

## 🚀 Quick Start

### For Users

1. Download the latest release from [GitHub Releases](https://github.com/BurgessTheGamer/ripVID/releases/latest)
2. Install and launch ripVID
3. On first launch, required binaries (yt-dlp, ffmpeg, ffprobe) will download automatically
4. Start downloading videos!

### For Developers

1. Clone the repository:
   ```bash
   git clone https://github.com/BurgessTheGamer/ripVID.git
   cd ripVID/video-downloader
   ```

2. Install dependencies:
   ```bash
   bun install
   # or
   npm install
   ```

3. Run in development mode:
   ```bash
   bun run tauri:dev
   # or
   npm run tauri:dev
   ```

## 🏗️ Building for Production

### Windows (NSIS Installer)
```bash
bun run tauri:build
# Output: src-tauri/target/release/bundle/nsis/ripVID_x.x.x_x64-setup.exe
```

### macOS (DMG)
```bash
bun run tauri:build -- --target universal-apple-darwin
# Output: src-tauri/target/release/bundle/dmg/ripVID.dmg
```

### Linux (AppImage)
```bash
bun run tauri:build
# Output: src-tauri/target/release/bundle/appimage/ripVID_x.x.x_amd64.AppImage
```

## 📚 How It Works

### Runtime Binary Management

ripVID uses a sophisticated binary management system that:

1. **First Launch**: Downloads yt-dlp, ffmpeg, and ffprobe to `~/Videos/ripVID/binaries/`
2. **Daily Updates**: Background task checks for yt-dlp updates every 24 hours
3. **SHA-256 Verification**: All downloads verified against official checksums
4. **Platform Detection**: Automatically selects correct binaries for your OS
5. **Graceful Fallback**: Uses bundled binaries if download fails

### Auto-Update System

- **Update Checking**: App checks for updates every 30 minutes
- **One-Click Updates**: Notification popup with instant update & relaunch
- **Secure Updates**: All releases cryptographically signed with minisign
- **Seamless Experience**: Downloads in background, installs on relaunch

### Smart Cookie Management

- **Automatic Retry**: If download fails, automatically retries with browser cookies
- **Multi-Browser Support**: Extracts cookies from Chrome, Edge, Firefox, Brave, etc.
- **Secure**: Cookies are used only for the current download and not stored
- **Age-Restricted Content**: Enables downloading videos that require login

## 🎯 Usage Guide

1. **Launch ripVID**
2. **Paste Video URL** - Supports YouTube, X/Twitter, TikTok, Facebook, Instagram, and more
3. **Platform Auto-Detection** - App automatically detects the platform
4. **Choose Format**:
   - Video: Highest quality H.264/MP4
   - Audio: MP3 extraction
5. **Select Quality** (for video downloads)
6. **Click Download** and choose save location
7. **Monitor Progress** - Real-time speed, progress, and ETA
8. **Access Files** - View downloads directly in the app's Library tab

## 🛠️ Tech Stack

### Frontend
- **React 18** - UI framework
- **TypeScript** - Type safety
- **Vite** - Build tool and dev server
- **Tailwind CSS** - Styling with custom purple theme
- **Lucide React** - Modern icon library

### Backend
- **Tauri v2** - Rust-based desktop framework
- **Rust** - Core backend logic
- **tokio** - Async runtime
- **reqwest** - HTTP client for downloads
- **tauri-plugin-updater** - Auto-update system

### Tools & Services
- **yt-dlp** - Video download engine (runtime download)
- **ffmpeg** - Video processing (runtime download)
- **ffprobe** - Media information (runtime download)
- **GitHub Actions** - CI/CD pipeline
- **Cloudflare Workers** - Metadata API

## 🔐 Auto-Update Configuration

For developers setting up auto-updates:

1. **Generate Signing Keys**:
   ```bash
   bunx @tauri-apps/cli signer generate -w ~/.tauri/myapp.key
   ```

2. **Add to GitHub Secrets**:
   - `TAURI_PRIVATE_KEY` - Your private key
   - `TAURI_KEY_PASSWORD` - Your key password

3. **Configure `tauri.conf.json`**:
   ```json
   {
     "bundle": {
       "createUpdaterArtifacts": true  // Required for Tauri v2!
     },
     "plugins": {
       "updater": {
         "active": true,
         "pubkey": "YOUR_PUBLIC_KEY",
         "endpoints": [
           "https://github.com/USER/REPO/releases/latest/download/latest.json"
         ]
       }
     }
   }
   ```

## 🐛 Troubleshooting

### Binary Download Issues
- **Check Internet**: Ensure stable connection on first launch
- **Firewall**: Allow ripVID to access github.com
- **Manual Download**: Binaries downloaded to `~/Videos/ripVID/binaries/`

### Download Fails
- **Invalid URL**: Verify the video URL is correct and accessible
- **Age-Restricted**: App will auto-retry with browser cookies
- **Private Content**: Ensure you're logged into the platform in your browser
- **Region Lock**: Some content may be geo-restricted

### Update Issues
- **Manual Update**: Download latest from GitHub Releases
- **Check Signature**: Ensure `latest.json` exists in release assets
- **Firewall**: Allow app to access GitHub for updates

### Build Errors
- **Rust**: Update with `rustup update stable`
- **Clean Build**: Run `cargo clean` in `src-tauri/` directory
- **Dependencies**: Delete `node_modules/` and reinstall

## 📋 Development Roadmap

- [ ] Playlist download support
- [ ] Batch URL processing
- [ ] Custom download presets
- [ ] Scheduled downloads
- [ ] macOS code signing
- [ ] Subtitle download options

## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

For major changes, please open an issue first to discuss what you'd like to change.

## 📄 License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) file for details.

## ⚖️ Legal

**IMPORTANT**: Users are responsible for complying with all applicable laws and third-party terms of service. ripVID is a tool for personal use only. Do not use it to:
- Download copyrighted content without permission
- Violate platform terms of service
- Redistribute downloaded content

Please read our [Terms of Service](TERMS.md) before using the application.

## 🙏 Acknowledgments

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - Powerful download engine
- [Tauri](https://tauri.app) - Lightweight desktop framework
- [FFmpeg](https://ffmpeg.org) - Video processing toolkit
- All contributors who have helped improve ripVID

## 💬 Support

- **Issues**: [GitHub Issues](https://github.com/BurgessTheGamer/ripVID/issues)
- **Discussions**: [GitHub Discussions](https://github.com/BurgessTheGamer/ripVID/discussions)
- **Releases**: [Latest Release](https://github.com/BurgessTheGamer/ripVID/releases/latest)

---

**Current Version**: v2.1.0 | **Status**: ✅ Production Ready | **Auto-Updates**: ✅ Enabled
