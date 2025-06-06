# Changelog
# All notable changes to this project will be documented in this file.

## [beta-0.1.1] - 2025-04-06
### Added
- MacOS support added
- Single instance support added
- MacOS mount plugin installer support implemented
- Remote root path selection added (That will be active after remote added)
- Remote Operations added: Sync and Copy  feature added (Syncs or copies remote with local folder, remote with remote or local with remote (if you want to copy local to local its working too. Idk why you would do that but it works))
- Bandwidth limit feature added (Limits the bandwidth for remote operations)
- Support for custom rclone config file location added
- Restrict visibility of the some tokens in the UI (like client secret, access token, etc.). It can be configured in the settings. (default is enabled)

### Fixed
- In the tray icon, the "Show App" option now correctly opens the app window. (Fixed)
- Rclone Configuration file is now correctly exported and imported.
- Fixed the issue where the application would not close when it could not find the rclone binary file.

### Changed
- Updated the cargo dependencies to the latest versions.
- Updated the npm dependencies to the latest versions.

## [beta-0.1.0] - 2024-12-05
### Added
- Added a new feature to manage remotes with a user-friendly interface.
- GTK-themed Angular frontend
- Tauri backend
- Basic remote management (add/edit/delete)
- Exporting and importing configurations
- Mounting and unmounting remotes
- File browser for mounted remotes
- OAuth support for OAuth2 providers
- VFS options
- Tray icon support
- Light/dark mode
- Cross-platform (Linux and Windows-ready, macOS coming soon)