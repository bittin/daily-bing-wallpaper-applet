# 🌄 Bing Wallpaper Applet for COSMIC

A COSMIC desktop applet that fetches, previews, and automatically sets the daily Bing wallpaper.

## ✨ Features

- 🖼️ Displays Bing “Wallpaper of the Day” with preview
- 📅 Shows formatted date (e.g. *September 28, 2024*)
- 📥 Downloads wallpapers to `~/Pictures/BingWallpaper`
- 🔁 Avoids re-downloading existing images
- ⚡ One-click “Set wallpaper”
- ⏰ Automatically updates wallpaper daily at **00:05**
- 🎨 Native COSMIC-style UI with full-width hover interactions

---

## 📸 Preview

- Full image preview inside popup
- Caption and copyright info
- Action menu with icons
- Clean COSMIC-native layout

---

## 📂 Storage

Downloaded wallpapers are saved to:

```
~/Pictures/BingWallpaper
```

Filenames are based on Bing’s ID and date to avoid duplicates.

---

## ⚙️ How It Works

1. Fetches Bing API:
   https://www.bing.com/HPImageArchive.aspx

2. Extracts:
   - Image URL (UHD)
   - Title
   - Date
   - Metadata

3. Downloads image locally

4. Updates COSMIC background config:
   ~/.config/cosmic/com.system76.CosmicBackground/v1/all

---

## ⏰ Automatic Wallpaper Update

The applet checks the time every minute and:

- At **00:05**, it:
  - Fetches the new wallpaper
  - Downloads it (if needed)
  - Sets it automatically

Runs **once per day**.

---

## 🧱 Tech Stack

- 🦀 Rust
- 🧊 COSMIC (libcosmic)
- 🎨 iced (via COSMIC)
- 🌐 reqwest
- 📅 chrono

---

## 🚀 Installation

### 1. Clone

```
git clone https://github.com/vinesnts/bing-wallpaper-applet.git
cd bing-wallpaper-applet
```

### 2. Build

```
cargo build --release
```

### 3. Run

```
cargo run
```

---

## 🧩 Requirements

- COSMIC Desktop Environment
- Linux (tested on COSMIC session)
- Internet connection

---

## 🎛️ Controls

Inside the applet popup:

- **Set wallpaper** → applies current image
- **Refresh** → fetches latest wallpaper manually

---

## 🧠 Notes

- Uses Bing UHD images when available
- Falls back automatically if UHD is unavailable
- Designed to feel native to COSMIC UI/UX

---

## 🛠️ Future Improvements

- 🗂️ Wallpaper history browser
- 🌍 Localization (date + text)
- ⚙️ Configurable update time
- 🖱️ Clickable image (set wallpaper)
- 🌙 Light/dark theme-aware filtering

---

## 📄 License

MPL-2.0

---

## 🙌 Credits

- Microsoft Bing for daily wallpapers
- System76 COSMIC team
