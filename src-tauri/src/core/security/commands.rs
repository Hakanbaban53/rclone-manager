use crate::core::security::{
    CredentialError, CredentialStore, PasswordValidatorState, SafeEnvironmentManager,
    test_rclone_password,
};
use log::{debug, error, info, warn};
use tauri::{AppHandle, Emitter, State};

/// Store the rclone config password securely
#[tauri::command]
pub async fn store_config_password(
    app: AppHandle,
    password_state: State<'_, PasswordValidatorState>,
    env_manager: State<'_, SafeEnvironmentManager>,
    credential_store: State<'_, CredentialStore>,
    password: String,
) -> Result<(), String> {
    info!("🔑 Storing rclone config password");

    match credential_store.store_config_password(&password) {
        Ok(()) => {
            // Set environment variable for current session using safe manager
            env_manager.set_config_password(password.clone());
            // Reset password validator state on successful storage
            if let Ok(mut validator) = password_state.lock() {
                validator.record_success();
            }

            // Emit event so OAuth can restart if needed
            if let Err(e) = app.emit(
                "rclone_engine",
                serde_json::json!({ "status": "password_stored" }),
            ) {
                error!("Failed to emit password_stored event: {e}");
            }

            info!("✅ Password stored successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to store password: {:?}", e);
            Err(format!("Failed to store password: {:?}", e))
        }
    }
}

/// Retrieve the stored rclone config password
#[tauri::command]
pub async fn get_config_password(
    credential_store: State<'_, CredentialStore>,
) -> Result<String, String> {
    debug!("🔍 Retrieving stored config password");

    match credential_store.get_config_password() {
        Ok(password) => {
            debug!("✅ Password retrieved successfully");
            Ok(password)
        }
        Err(CredentialError::NotFound) => {
            debug!("ℹ️ No password stored");
            Err("No password stored".to_string())
        }
        Err(e) => {
            error!("❌ Failed to retrieve password: {:?}", e);
            Err(format!("Failed to retrieve password: {:?}", e))
        }
    }
}

/// Check if a config password is stored
#[tauri::command]
pub async fn has_stored_password(
    credential_store: State<'_, CredentialStore>,
) -> Result<bool, String> {
    debug!("🔍 Checking if password is stored");

    Ok(credential_store.has_config_password())
}

/// Remove the stored config password
#[tauri::command]
pub async fn remove_config_password(
    env_manager: State<'_, SafeEnvironmentManager>,
    credential_store: State<'_, CredentialStore>,
) -> Result<(), String> {
    info!("🗑️ Removing stored config password");

    match credential_store.remove_config_password() {
        Ok(()) => {
            // Clear environment variable using safe manager
            env_manager.clear_config_password();

            info!("✅ Password removed successfully");
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to remove password: {:?}", e);
            Err(format!("Failed to remove password: {:?}", e))
        }
    }
}

/// Test if a password is valid for rclone
#[tauri::command]
pub async fn validate_rclone_password(
    app: AppHandle,
    password_state: State<'_, PasswordValidatorState>,
    password: String,
) -> Result<(), String> {
    // Check if we're locked out
    if let Ok(validator) = password_state.lock()
        && validator.is_locked_out()
    {
        let remaining = validator
            .remaining_lockout_time()
            .map(|d| d.as_secs())
            .unwrap_or(0);

        error!(
            "🔒 Password validation locked out for {} seconds",
            remaining
        );
        return Err(format!(
            "You are locked out due to too many failed attempts. Please try again in {} seconds.",
            remaining
        ));
    }

    // Test the password
    let result = test_rclone_password(&app, &password).await;

    // Update validator state
    if let Ok(mut validator) = password_state.lock() {
        if result.is_valid {
            validator.record_success();
        } else {
            validator.record_failure();
        }
    }

    if result.is_valid {
        info!("✅ Password validation successful");
    } else {
        error!("❌ Password validation failed: {}", result.message);
        return Err(format!("Password validation failed: {}", result.message));
    }

    Ok(())
}

/// Get the current lockout status
#[tauri::command]
pub async fn get_password_lockout_status(
    password_state: State<'_, PasswordValidatorState>,
) -> Result<serde_json::Value, String> {
    debug!("🔍 Getting password lockout status");

    if let Ok(validator) = password_state.lock() {
        Ok(serde_json::json!({
            "is_locked_out": validator.is_locked_out(),
            "remaining_time": validator.remaining_lockout_time().map(|d| d.as_secs()),
            "failed_attempts": validator.failed_attempts(),
            "max_attempts": validator.max_attempts()
        }))
    } else {
        Err("Failed to get lockout status".to_string())
    }
}

/// Reset the password validator (admin function)
#[tauri::command]
pub async fn reset_password_validator(
    password_state: State<'_, PasswordValidatorState>,
) -> Result<(), String> {
    info!("🔄 Resetting password validator");

    if let Ok(mut validator) = password_state.lock() {
        validator.reset();

        info!("✅ Password validator reset successfully");
        Ok(())
    } else {
        Err("Failed to reset password validator".to_string())
    }
}

/// Set the config password environment variable (for current session)
#[tauri::command]
pub async fn set_config_password_env(
    app: AppHandle,
    env_manager: State<'_, SafeEnvironmentManager>,
    password: String,
) -> Result<(), String> {
    debug!("🔧 Setting config password environment variable");

    env_manager.set_config_password(password);
    // Emit event so other parts (engine) can react and clear any
    // startup password error state.
    if let Err(e) = app.emit(
        "rclone_engine",
        serde_json::json!({ "status": "password_stored" }),
    ) {
        error!("Failed to emit password_stored event: {e}");
    }

    debug!("✅ Environment variable set");
    Ok(())
}

/// Clear the config password environment variable
#[tauri::command]
pub async fn clear_config_password_env(
    env_manager: State<'_, SafeEnvironmentManager>,
) -> Result<(), String> {
    debug!("🧹 Clearing config password environment variable");

    env_manager.clear_config_password();

    debug!("✅ Environment variable cleared");
    Ok(())
}

/// Check if config password environment variable is set
#[tauri::command]
pub async fn has_config_password_env(
    env_manager: State<'_, SafeEnvironmentManager>,
) -> Result<bool, String> {
    debug!("🔍 Checking config password environment variable");
    Ok(env_manager.has_config_password())
}

/// Check if the rclone configuration is encrypted
/// Get cached encryption status if available (faster, no I/O)
#[tauri::command]
pub fn get_cached_encryption_status() -> Option<bool> {
    use crate::utils::rclone::process_common::get_cached_encryption_status;

    debug!("⚡ Getting cached encryption status");
    get_cached_encryption_status()
}

/// Clear cached encryption status (e.g., when config changes)
#[tauri::command]
pub fn clear_encryption_cache() {
    use crate::utils::rclone::process_common::clear_encryption_cache;

    debug!("🗑️ Clearing encryption cache");
    clear_encryption_cache();
}

/// Check if config is encrypted (with caching for performance)
#[tauri::command]
pub async fn is_config_encrypted_cached(app: AppHandle) -> Result<bool, String> {
    use crate::utils::rclone::process_common::get_cached_encryption_status;

    debug!("🚀 Checking config encryption with cache");

    // First try to get cached status
    if let Some(cached_status) = get_cached_encryption_status() {
        debug!("Using cached encryption status: {}", cached_status);
        return Ok(cached_status);
    }

    // If not cached, fall back to full check
    debug!("No cached status found, performing full encryption check");
    is_config_encrypted(app).await
}

#[tauri::command]
pub async fn is_config_encrypted(app: AppHandle) -> Result<bool, String> {
    use crate::core::check_binaries::read_rclone_path;
    use std::process::Stdio;

    debug!("🔍 Checking if rclone config is encrypted");

    let rclone_path = read_rclone_path(&app);

    // Simple approach: try listremotes with --ask-password=false
    // If encrypted: "unable to decrypt configuration and not allowed to ask for password"
    // If not encrypted: succeeds and lists remotes
    let output = tokio::process::Command::new(rclone_path)
        .args(["listremotes", "--ask-password=false"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_remove("RCLONE_CONFIG_PASS")
        .output()
        .await
        .map_err(|e| format!("Failed to execute rclone listremotes: {e}"))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    debug!("🔍 rclone listremotes stderr: {}", stderr.trim());

    // Check for encryption error message
    if stderr.contains("unable to decrypt configuration and not allowed to ask for password")
        || stderr.contains("Failed to load config file") && stderr.contains("unable to decrypt")
    {
        debug!("🔒 Configuration is encrypted");
        Ok(true)
    } else if output.status.success() {
        debug!("🔓 Configuration is NOT encrypted");
        Ok(false)
    } else {
        // Other error - assume not encrypted if we can't determine
        warn!("⚠️ Unexpected error checking encryption: {}", stderr);
        Ok(false)
    }
}

/// Encrypt the rclone configuration with a password
#[tauri::command]
pub async fn encrypt_config(
    app: AppHandle,
    password_state: State<'_, PasswordValidatorState>,
    env_manager: State<'_, SafeEnvironmentManager>,
    credential_store: State<'_, CredentialStore>,
    password: String,
) -> Result<(), String> {
    use crate::core::check_binaries::read_rclone_path;
    use std::process::Stdio;

    info!("🔐 Encrypting rclone configuration (using password-command)");

    let rclone_path = read_rclone_path(&app);

    // Create a cross-platform command that outputs the password
    let password_command = if cfg!(windows) {
        // Windows: Use PowerShell Write-Host with -NoNewline to avoid extra newlines
        format!(
            "powershell -Command \"Write-Host {} -NoNewline\"",
            password.replace("'", "''")
        )
    } else {
        // Unix/Linux: Use printf to avoid newlines and quote issues (no quotes)
        format!("echo \"{}\"", password)
    };
    // Use --password-command to avoid stdin password prompt issues
    let child = tokio::process::Command::new(rclone_path)
        .args([
            "config",
            "encryption",
            "set",
            "--password-command",
            &password_command,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start rclone config encryption set: {e}"))?;

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("Failed to wait for rclone config encryption set: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    debug!("📤 rclone config encryption set stdout: {}", stdout);
    debug!("📤 rclone config encryption set stderr: {}", stderr);

    if output.status.success()
        || stdout.contains("Password set")
        || stdout.contains("Your configuration is encrypted")
    {
        // Store the password securely after successful encryption
        if let Err(e) = credential_store.store_config_password(&password) {
            warn!("⚠️ Failed to store password after encryption: {:?}", e);
        }

        // Set environment variable for current session using safe manager
        env_manager.set_config_password(password.clone());

        // Reset password validator state on successful encryption
        if let Ok(mut validator) = password_state.lock() {
            validator.record_success();
        }

        info!("✅ Configuration encrypted successfully");
        Ok(())
    } else {
        error!(
            "❌ Failed to encrypt configuration - stdout: {}, stderr: {}",
            stdout, stderr
        );
        Err(format!(
            "Failed to encrypt configuration: {}",
            if !stderr.trim().is_empty() {
                stderr
            } else {
                stdout
            }
        ))
    }
}

/// Unencrypt (decrypt) the rclone configuration
#[tauri::command]
pub async fn unencrypt_config(
    app: AppHandle,
    env_manager: State<'_, SafeEnvironmentManager>,
    password: String,
) -> Result<(), String> {
    use crate::core::check_binaries::read_rclone_path;
    use std::process::Stdio;
    info!("🔓 Unencrypting rclone configuration (using password-command)");

    let rclone_path = read_rclone_path(&app);

    // Create a cross-platform password command
    let password_command = if cfg!(windows) {
        format!(
            "powershell -Command \"Write-Host {} -NoNewline\"",
            password.replace("'", "''")
        )
    } else {
        format!("echo \"{}\"", password)
    };

    let child = tokio::process::Command::new(rclone_path)
        .args([
            "config",
            "encryption",
            "remove",
            "--password-command",
            &password_command,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start rclone config encryption remove: {e}"))?;

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("Failed to wait for rclone config encryption remove: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    debug!("📤 rclone unencrypt stdout: {}", stdout);
    debug!("📤 rclone unencrypt stderr: {}", stderr);

    if output.status.success()
        || stdout.contains("Your configuration is not encrypted")
        || stderr.contains("config file is NOT encrypted")
    {
        // Remove stored password since config is no longer encrypted
        let credential_store = CredentialStore::new();
        if let Err(e) = credential_store.remove_config_password() {
            warn!(
                "⚠️ Failed to remove stored password after unencryption: {:?}",
                e
            );
        }

        // Clear environment variable using safe manager
        env_manager.clear_config_password();

        info!("✅ Configuration unencrypted successfully");
        Ok(())
    } else {
        error!(
            "❌ Failed to unencrypt configuration - stdout: {}, stderr: {}",
            stdout, stderr
        );
        Err(format!(
            "Failed to unencrypt configuration: {}",
            if !stderr.trim().is_empty() {
                stderr
            } else {
                stdout
            }
        ))
    }
}

/// Change the rclone configuration password (using simple two-step approach)
#[tauri::command]
pub async fn change_config_password(
    app: AppHandle,
    password_state: State<'_, PasswordValidatorState>,
    env_manager: State<'_, SafeEnvironmentManager>,
    credential_store: State<'_, CredentialStore>,
    current_password: String,
    new_password: String,
) -> Result<(), String> {
    info!("🔄 Changing rclone configuration password (two-step approach)");

    // Step 1: Remove encryption with current password
    debug!("🔓 Step 1: Removing current encryption");
    unencrypt_config(app.clone(), env_manager.clone(), current_password)
        .await
        .map_err(|e| format!("Failed to remove current encryption: {}", e))?;

    // Step 2: Encrypt with new password
    debug!("🔒 Step 2: Encrypting with new password");
    encrypt_config(
        app.clone(),
        password_state.clone(),
        env_manager.clone(),
        credential_store.clone(),
        new_password.clone(),
    )
    .await
    .map_err(|e| format!("Failed to encrypt with new password: {}", e))?;

    // Update stored password with new password
    if let Err(e) = credential_store.store_config_password(&new_password) {
        warn!("⚠️ Failed to store new password: {:?}", e);
    }

    // Update environment variable for current session using safe manager
    env_manager.set_config_password(new_password.clone());

    // Reset password validator state on successful password change
    if let Ok(mut validator) = password_state.lock() {
        validator.record_success();
    }

    info!("✅ Configuration password changed successfully");
    Ok(())
}
