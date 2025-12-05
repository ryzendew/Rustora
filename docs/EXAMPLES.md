# Examples

Real-world examples of using Rustora for common tasks.

## Installing a Package

**GUI Method:**
1. Open Rustora
2. Go to the **Search** tab
3. Type the package name (like "firefox")
4. Select it from the results
5. Click **Install**
6. Review dependencies and confirm

**CLI Method:**
```bash
rustora install firefox
```

## Setting Up Your Gaming Environment

1. Open Rustora and go to **Tweaks** → **Gaming Meta**
2. Click **Check Status** to see what's already installed
3. Click **Install Gaming Meta** to get everything set up
4. Go to **Proton** sub-tab and install Proton-GE or your preferred compatibility layer
5. Head to **Steam Games** sub-tab to configure compatibility tools per game

## Converting a DEB Package

1. Open Rustora and go to the **FPM** tab
2. Click **Convert DEB to RPM**
3. Pick your `.deb` file
4. Wait for the conversion (you'll see progress in real-time)
5. The install dialog opens automatically when it's done
6. Review the package info and click **Install**

## Speeding Up Package Downloads

1. Go to **Tweaks** → **DNF Config**
2. Set **Parallel Downloads** to 10 (or higher if you want)
3. Enable **Fastest Mirror** if you want
4. Click **Save Configuration**

Now all your future package operations will use these settings automatically.

## Customizing the Look

1. Click the **Settings** button in the top bar
2. Go through the categories (General, Appearance, Fonts, Tabs)
3. Adjust everything to your liking
4. Click **Save Settings**
5. Optionally save it as a theme so you can reuse it later

## Installing Hyprland

1. Navigate to **Tweaks** → **Hyprland**
2. Click **Check Status** to see what's installed
3. Click **Install Hyprland** to install the compositor and all dependencies
4. Optionally click **Install Dotfiles** to get a pre-configured setup

## Managing Proton Builds

1. Navigate to **Tweaks** → **Proton**
2. Wait for builds to load (uses local cache for speed)
3. Select launcher (Steam, Lutris, etc.)
4. Select runner (Proton-GE, Wine-Staging, etc.)
5. Click **Install** on desired build
6. Monitor progress in separate window:
   - Download progress (0-100%)
   - Extraction progress (0-100%)
   - Installation progress
7. View changelog with **Info** button
8. Update "Latest" builds when new versions are available

## System Maintenance

1. Navigate to **Maintenance** tab
2. Click individual task buttons or **Run All**
3. Monitor progress in separate window
4. Each task shows detailed terminal output

## Managing Repositories

1. Navigate to **Repositories** tab
2. Toggle repositories on/off with a click
3. Add new repository with **Add Repository** button
4. View repository details and status

