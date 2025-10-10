# ripVID


https://github.com/user-attachments/assets/db3929ad-2251-4d2b-8641-d3c9e34bb72d


A beautiful, modern desktop application for downloading videos from URL at the highest quality. Built with Tauri, React, TypeScript, and a custom purple-themed UI.

## Features

- 🎥 Download videos from YouTube,TikTok. X Facebook, and many more!
- 🚀 Highest quality video downloads (H.264/MP4)
- 🎨 Modern, minimalist UI with purple accents
- 🔍 Auto-detect platform from URL
- 📊 Real-time download progress tracking
- 📦 yt-dlp bundled - no external dependencies
- 💾 Small executable size
- ⚡ Fast and efficient with Rust backend
- 🎵 Audio-only download option (MP3)

## Prerequisites for Development

1. **Node.js** (v16 or higher) - [nodejs.org](https://nodejs.org/)
2. **Rust** (latest stable version) - [rustup.rs](https://rustup.rs/)
3. **Bun** (optional, for faster package management) - [bun.sh](https://bun.sh/)

**Note**: End users don't need any prerequisites - yt-dlp is bundled within the application!

## Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/ripvid.git
   cd ripvid
   ```

2. Install dependencies:
   ```bash
   npm install
   # or with bun
   bun install
   ```

## Development

Run the application in development mode:

```bash
npm run tauri:dev
# or with bun
bun run tauri:dev
```

This will:
- Start the Vite development server
- Launch the Tauri application window
- Enable hot module replacement for React
- Use the bundled yt-dlp for downloads

## Building for Production

### Windows
```bash
npm run tauri:build
# Outputs: src-tauri/target/release/ripVID.exe
# Installer: src-tauri/target/release/bundle/msi/
```

### macOS
```bash
npm run tauri:build
# Outputs: src-tauri/target/release/bundle/dmg/ripVID.dmg
```

### Linux
```bash
npm run tauri:build
# Outputs: src-tauri/target/release/bundle/appimage/ripVID.AppImage
```

## Important Notes

### Bundled yt-dlp

ripVID includes yt-dlp binaries for all platforms, so users don't need to install anything separately. The bundled version provides:
- Support for 1000+ video platforms
- Automatic quality selection
- Regular updates through app updates
- No configuration required

## Usage

1. Launch ripVID
2. Paste a YouTube or X/Twitter video URL
3. The platform will be auto-detected
4. Choose video or audio-only download
5. Click "Download" and select save location
6. Monitor download progress in real-time
7. Access downloaded files directly from the app

## Troubleshooting

### Build fails with Rust errors
- Update Rust: `rustup update`
- Clean and rebuild: `cargo clean` then `npm run tauri:build`

### Download fails
- Check if the video URL is valid
- Ensure you have internet connectivity
- Some videos may be region-restricted or private
- Check the app logs for specific error messages

### Application won't start
- Ensure you have the latest version
- On Windows, check if Windows Defender is blocking the app
- Try running as administrator

## Tech Stack

- **Frontend**: React + TypeScript + Vite
- **Styling**: Tailwind CSS with custom purple theme
- **Desktop Framework**: Tauri 2.0 (Rust)
- **Download Engine**: yt-dlp (bundled)
- **Package Manager**: Bun/npm

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) file for details.

## Legal

**IMPORTANT**: Users are responsible for complying with all applicable laws and third-party terms of service. Please read our [Terms of Service](TERMS.md) before using the application.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## Acknowledgments

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) for the amazing download engine
- [Tauri](https://tauri.app) for the lightweight desktop framework
- All contributors who have helped improve ripVID

## Support

For issues, questions, or suggestions, please open an issue on GitHub.
