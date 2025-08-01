# Changelog
# All notable changes to this project will be documented in this file.

## [Unreleased]
### Added
- Native console support for the native terminal. You can now open the remote configuration in the native terminal by clicking the "Remote Terminal" button in the top left add button. It will use the preferred terminal app from the settings. Also, you can set the preferred terminal app in the settings.

- Implemented the `bisync` and `move` operations for remotes.
  - Bisync: This operation synchronizes two remotes in both directions, ensuring that changes made in either remote are reflected in the other.
  - Move: This operation moves files from one remote to another, effectively transferring data without leaving duplicates.
- Added other configs for operations. (e.g. mountType, createEmptySrcDirs etc.)

### Changed
- Rclone configuration file path is now set with api call instead of directly accessing the state.

### Need Fix
- After engine restart, need the apply the startup settings again. (e.g. config file path, bw limit, etc.)
- Remote updates not working properly. When you update a some settings to default, it does not update the remote. I know whats the problem.

## [beta-0.1.2] - 2025-07-15
### Added
- General tab added.
- Remote Clone feature added. Under the remote detail ellipsis button (Clones a remote with settings to new remote.).
- Rclone pid watcher feature added with instant stop Rclone process functionality. Also listens for changes in the rclone process state and updates the UI accordingly. You can find it in `About RClone Manager > About Rclone`  (I see the core/pid rcd command and I want to make something for it. IDK why but I did it.)
- Detecting the metered connection and showing a warning banner (Linux needed Network Manager. Its `nmcli` command is used to check for metered connections). Not supported on macOS because it does not support metered network detection (For now, it is only show the warning banner.).
- Watcher for mounted remotes added. It will automatically unmount the remote if it is not mounted anymore. It will also update the UI accordingly (5 seconds interval). You can also force check the mounted remotes by this Shortcut: Ctrl + Shift + M.
- Linting and formatting scripts added for the frontend and backend. It uses ESLint, Prettier, Clippy, and Rustfmt.
- Rclone update check feature added. It will check for the latest version of Rclone. Under the `About RClone Manager > About Rclone` section, you can find the update status and the update button.
- Rclone binary location selection feature added. You can select the Rclone binary location in the settings, onboard and the repair sheet. It will be used for the Rclone operations. If you don't select it, it will use the default location.

### Changed
- UI design has been improved.
- Mount path selection not forced to select a path from the file browser anymore. You can also type the path manually but it will be validated. Also added support for AllowNonEmpty option in the mount step. This allows you to mount a remote to a non-empty folder if its true.
- Onboarding process has been improved.
- Frontend and backend services have been refactored to use a more modular approach.


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