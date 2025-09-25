# 🎬 ripVID - ULTRA MINIMAL VIDEO RIPPER

## ✨ **NEW FEATURES:**

### **Ultra-Minimal Design**
- **NO cards, NO title, NO buttons**
- Just a single floating input bar
- Clean dark purple gradient background

### **Animated Purple Border**
- Like Google AI's rainbow effect, but PURPLE
- Spins around input when downloading
- Smooth conic gradient animation

### **Press Enter to Download**
- No download button needed
- Enter: Download
- Escape: Clear input
- Tab: Toggle archive panel

### **Auto-Save Folder System**
- Saves to `~/Videos/ripVID/` automatically
- No file picker dialog
- Creates folder if it doesn't exist
- Files named: `platform_timestamp.mp4`

### **Archive Slide-Out Panel**
- Slides from right side
- Shows all downloaded videos
- Click any item to open its folder
- Persists across sessions (localStorage)
- Clean minimal design

### **Status Information**
- Appears below input when active
- Shows: percentage, speed, ETA
- Success/error states
- Auto-clears after 3 seconds

## 🎯 **HOW TO USE:**

1. **Paste URL** - Just paste a YouTube or X link
2. **Press Enter** - That's it!
3. **Tab** - Toggle archive panel
4. **Click archive items** - Opens folder location

## 🚀 **RUN THE APP:**

```bash
cd video-downloader
bun run tauri:dev
```

Or double-click `RUN_APP.bat`

## 📁 **WHERE ARE MY VIDEOS?**

All videos automatically save to:
```
C:\Users\[YourName]\Videos\ripVID\
```

## 🎨 **DESIGN FEATURES:**

- **Minimal**: Just an input bar floating in space
- **Dark theme**: Deep purple/black gradient
- **Animated border**: Purple gradient spins when downloading
- **Clean status**: Info appears only when needed
- **Hidden archive**: Slides out from the right
- **Auto-focus**: Input always ready for next URL

## ⚡ **KEYBOARD SHORTCUTS:**

- `Enter` - Download video
- `Escape` - Clear input
- `Tab` - Toggle archive

## 📊 **TECHNICAL:**

- App name: **ripVID**
- Version: **2.0.0**
- Size: ~10MB exe
- Backend: Rust + Tauri
- Frontend: React (minimal)
- Downloads: yt-dlp

---

**ripVID** - Rip videos with style. No clutter. Just works.