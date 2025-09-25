@echo off
echo Starting Video Downloader Desktop App...
echo.
echo This is a LOCAL DESKTOP APPLICATION - NOT a browser app!
echo Using Bun for faster performance instead of Node.js
echo.
cd /d "%~dp0"
bun run tauri:dev
pause