@echo off
echo Building Video Downloader .exe...
echo.
cd /d "%~dp0"
bun run tauri:build
echo.
echo Build complete! Find your .exe in:
echo src-tauri\target\release\video-downloader.exe
echo.
echo Installer available in:
echo src-tauri\target\release\bundle\nsis\
pause