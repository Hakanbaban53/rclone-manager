use once_cell::sync::Lazy;
use tokio::sync::RwLock;

use crate::utils::types::{LogCache, LogEntry};

pub static LOG_CACHE: Lazy<LogCache> = Lazy::new(|| LogCache::new(1000));

impl LogCache {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(Vec::with_capacity(max_entries)),
            max_entries,
        }
    }

    pub async fn add_entry(&self, entry: LogEntry) {
        let mut entries = self.entries.write().await;
        entries.push(entry);

        // Maintain max size
        let len = entries.len();
        if len > self.max_entries {
            entries.drain(0..(len - self.max_entries));
        }
    }

    pub async fn get_logs_for_remote(&self, remote_name: Option<&str>) -> Vec<LogEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter_map(|e| {
                if let Some(name) = &e.remote_name {
                    if remote_name.is_none() || name == remote_name.unwrap() {
                        Some(LogEntry {
                            timestamp: e.timestamp,
                            remote_name: Some(name.clone()),
                            level: e.level.clone(),
                            message: e.message.clone(),
                            context: e.context.clone(),
                            operation: e.operation.clone(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    pub async fn clear_for_remote(&self, remote_name: &str) {
        let mut entries = self.entries.write().await;
        entries.retain(|e| e.remote_name.as_deref() != Some(remote_name));
    }
}

#[tauri::command]
pub async fn get_remote_logs(remote_name: Option<String>) -> Result<Vec<LogEntry>, String> {
    let logs = LOG_CACHE.get_logs_for_remote(remote_name.as_deref()).await;
    Ok(logs)
}

#[tauri::command]
pub async fn clear_remote_logs(remote_name: Option<String>) -> Result<(), String> {
    if let Some(name) = remote_name {
        LOG_CACHE.clear_for_remote(&name).await;
    }
    Ok(())
}
