use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::Write;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuditLogLevel {
    Off,
    Low,
    Medium,
    High,
    Full,
}

impl AuditLogLevel {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => AuditLogLevel::Low,
            "medium" => AuditLogLevel::Medium,
            "high" => AuditLogLevel::High,
            "full" => AuditLogLevel::Full,
            _ => AuditLogLevel::Off,
        }
    }
}

#[derive(Debug)]
pub enum AuditLogDest {
    Stdout,
    File(Mutex<File>),
}

pub struct AuditLogger {
    level: AuditLogLevel,
    dest: AuditLogDest,
}

static AUDIT_LOGGER: OnceLock<AuditLogger> = OnceLock::new();
static GLOBAL_LOGS: OnceLock<Mutex<VecDeque<String>>> = OnceLock::new();
static TENANT_LOGS: OnceLock<Mutex<HashMap<String, VecDeque<String>>>> = OnceLock::new();

#[derive(Serialize)]
struct AuditEntry<'a> {
    timestamp: String,
    actor: &'a str,
    action: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    collection: Option<&'a str>,
    status: &'a str,
    client_ip: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<&'a str>,
}

pub fn init() {
    GLOBAL_LOGS
        .set(Mutex::new(VecDeque::with_capacity(100)))
        .ok();
    TENANT_LOGS.set(Mutex::new(HashMap::new())).ok();

    let level_str = std::env::var("HS_AUDIT_LOG_LEVEL").unwrap_or_else(|_| "off".to_string());
    let level = AuditLogLevel::from_str(&level_str);

    let dest_str = std::env::var("HS_AUDIT_LOG_DEST").unwrap_or_else(|_| "stdout".to_string());
    let dest = if dest_str.to_lowercase() == "file" {
        let path = std::env::var("HS_AUDIT_LOG_PATH").unwrap_or_else(|_| "audit.log".to_string());
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            Ok(file) => AuditLogDest::File(Mutex::new(file)),
            Err(e) => {
                eprintln!(
                    "Failed to open audit log file '{}': {}. Falling back to stdout.",
                    path, e
                );
                AuditLogDest::Stdout
            }
        }
    } else {
        AuditLogDest::Stdout
    };

    let logger = AuditLogger { level, dest };
    if AUDIT_LOGGER.set(logger).is_err() {
        // Already initialized
    }
}

pub fn log_action(
    action_level: AuditLogLevel,
    actor: &str,
    action: &str,
    collection: Option<&str>,
    status: Result<(), &str>,
    client_ip: &str,
) {
    let Some(logger) = AUDIT_LOGGER.get() else {
        return;
    };

    if logger.level == AuditLogLevel::Off || logger.level < action_level {
        return;
    }

    let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let (status_str, error) = match status {
        Ok(_) => ("success", None),
        Err(e) => ("failure", Some(e)),
    };

    let entry = AuditEntry {
        timestamp,
        actor,
        action,
        collection,
        status: status_str,
        client_ip,
        error,
    };

    if let Ok(json_str) = serde_json::to_string(&entry) {
        match &logger.dest {
            AuditLogDest::Stdout => {
                println!("{}", json_str);
            }
            AuditLogDest::File(file_mutex) => {
                if let Ok(mut file) = file_mutex.lock() {
                    let _ = writeln!(file, "{}", json_str);
                }
            }
        }

        if let Some(global_logs_mutex) = GLOBAL_LOGS.get() {
            if let Ok(mut guard) = global_logs_mutex.lock() {
                if guard.len() >= 100 {
                    guard.pop_front();
                }
                guard.push_back(json_str.clone());
            }
        }

        if let Some(tenant_logs_mutex) = TENANT_LOGS.get() {
            if let Ok(mut guard) = tenant_logs_mutex.lock() {
                let deque = guard
                    .entry(actor.to_string())
                    .or_insert_with(|| VecDeque::with_capacity(100));
                if deque.len() >= 100 {
                    deque.pop_front();
                }
                deque.push_back(json_str);
            }
        }
    }
}

pub fn get_tenant_logs(actor: &str, is_admin: bool) -> Vec<String> {
    if is_admin {
        if let Some(global_logs_mutex) = GLOBAL_LOGS.get() {
            if let Ok(guard) = global_logs_mutex.lock() {
                return guard.iter().cloned().collect();
            }
        }
    } else {
        if let Some(tenant_logs_mutex) = TENANT_LOGS.get() {
            if let Ok(guard) = tenant_logs_mutex.lock() {
                if let Some(deque) = guard.get(actor) {
                    return deque.iter().cloned().collect();
                }
            }
        }
    }
    Vec::new()
}
