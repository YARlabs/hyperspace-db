use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, OnceLock, RwLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    ReadWrite,
    ReadOnly,
}

impl UserRole {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => UserRole::Admin,
            "readwrite" | "read_write" => UserRole::ReadWrite,
            _ => UserRole::ReadOnly,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::Admin => "admin",
            UserRole::ReadWrite => "ReadWrite",
            UserRole::ReadOnly => "ReadOnly",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiKeyInfo {
    user_id: String,
    role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GrantEntry {
    owner: String,
    collection: String,
    role: UserRole,
}

const API_KEYS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("api_keys");
const GRANTS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("grants");

struct SecurityRegistry {
    db: Database,
    // key_hash -> (user_id, role)
    api_keys: RwLock<HashMap<String, (String, UserRole)>>,
    // grantee -> Vec<(owner, collection, role)>
    grants: RwLock<HashMap<String, Vec<(String, String, UserRole)>>>,
}

static REGISTRY: OnceLock<Arc<SecurityRegistry>> = OnceLock::new();

impl SecurityRegistry {
    fn load_cache(&self) {
        // Load API Keys
        let read_txn = self.db.begin_read().unwrap();
        if let Ok(table) = read_txn.open_table(API_KEYS_TABLE) {
            let mut api_keys_guard = self.api_keys.write().unwrap();
            if let Ok(mut iter) = table.iter() {
                while let Some(Ok((key, val))) = iter.next() {
                    if let Ok(info) = serde_json::from_str::<ApiKeyInfo>(val.value()) {
                        api_keys_guard.insert(key.value().to_string(), (info.user_id, info.role));
                    }
                }
            }
        }

        // Load Grants
        if let Ok(table) = read_txn.open_table(GRANTS_TABLE) {
            let mut grants_guard = self.grants.write().unwrap();
            if let Ok(mut iter) = table.iter() {
                while let Some(Ok((key, val))) = iter.next() {
                    if let Ok(list) = serde_json::from_str::<Vec<GrantEntry>>(val.value()) {
                        let mapped = list
                            .into_iter()
                            .map(|g| (g.owner, g.collection, g.role))
                            .collect();
                        grants_guard.insert(key.value().to_string(), mapped);
                    }
                }
            }
        }
    }
}

pub fn init(data_dir: &Path) {
    let db_path = data_dir.join("security.db");

    // Open or create the redb v4 database.
    // If the file is from an older redb version (UpgradeRequired error),
    // delete it and start fresh — no production data to migrate.
    let db = match Database::create(&db_path) {
        Ok(db) => db,
        Err(redb::DatabaseError::UpgradeRequired(_)) => {
            eprintln!("[security] Detected legacy redb database format — deleting and recreating (no migration needed).");
            std::fs::remove_file(&db_path).expect("Failed to remove legacy security.db");
            Database::create(&db_path)
                .expect("Failed to create fresh security database after upgrade")
        }
        Err(e) => {
            panic!("Failed to open security database: {e:?}");
        }
    };

    // Ensure tables are initialized
    {
        let write_txn = db.begin_write().unwrap();
        {
            let _ = write_txn.open_table(API_KEYS_TABLE).unwrap();
            let _ = write_txn.open_table(GRANTS_TABLE).unwrap();
        }
        write_txn.commit().unwrap();
    }

    let registry = Arc::new(SecurityRegistry {
        db,
        api_keys: RwLock::new(HashMap::new()),
        grants: RwLock::new(HashMap::new()),
    });

    registry.load_cache();

    // Bootstrap check: if no keys are present, register the env key
    let keys_empty = {
        let guard = registry.api_keys.read().unwrap();
        guard.is_empty()
    };

    if keys_empty {
        if let Ok(env_key) = std::env::var("HYPERSPACE_API_KEY") {
            let mut hasher = sha2::Sha256::new();
            hasher.update(env_key.as_bytes());
            use sha2::Digest;
            let hash = hex::encode(hasher.finalize());

            let info = ApiKeyInfo {
                user_id: "default_admin".to_string(),
                role: UserRole::Admin,
            };
            let val_str = serde_json::to_string(&info).unwrap();

            let write_txn = registry.db.begin_write().unwrap();
            {
                let mut table = write_txn.open_table(API_KEYS_TABLE).unwrap();
                let _ = table.insert(hash.as_str(), val_str.as_str());
            }
            write_txn.commit().unwrap();

            // Refresh cache
            registry
                .api_keys
                .write()
                .unwrap()
                .insert(hash, ("default_admin".to_string(), UserRole::Admin));
            println!(
                "🔒 Bootstrapped security registry with default_admin from HYPERSPACE_API_KEY"
            );
        }
    }

    REGISTRY.set(registry).ok();
}

pub fn validate_key(hash: &str) -> Option<(String, UserRole)> {
    let registry = REGISTRY.get()?;
    let guard = registry.api_keys.read().ok()?;
    guard.get(hash).cloned()
}

pub fn check_access(user_id: &str, owner: &str, collection: &str, required: UserRole) -> bool {
    // 1. Owner always has access
    if user_id == owner {
        return true;
    }

    // 2. Global admin always has access
    if user_id == "default_admin" {
        return true;
    }

    // 3. Check grants
    if let Some(registry) = REGISTRY.get() {
        if let Ok(guard) = registry.grants.read() {
            if let Some(list) = guard.get(user_id) {
                for (o, c, role) in list {
                    if o == owner && c == collection {
                        match (role, required) {
                            (UserRole::Admin, _) => return true,
                            (UserRole::ReadWrite, UserRole::ReadWrite) => return true,
                            (UserRole::ReadWrite, UserRole::ReadOnly) => return true,
                            (UserRole::ReadOnly, UserRole::ReadOnly) => return true,
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    false
}

#[allow(dead_code)]
pub fn add_api_key(key: &str, user_id: &str, role: UserRole) -> Result<(), String> {
    let registry = REGISTRY.get().ok_or("Security registry not initialized")?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(key.as_bytes());
    use sha2::Digest;
    let hash = hex::encode(hasher.finalize());

    let info = ApiKeyInfo {
        user_id: user_id.to_string(),
        role,
    };
    let val_str = serde_json::to_string(&info).map_err(|e| e.to_string())?;

    let write_txn = registry.db.begin_write().map_err(|e| e.to_string())?;
    {
        let mut table = write_txn
            .open_table(API_KEYS_TABLE)
            .map_err(|e| e.to_string())?;
        table
            .insert(hash.as_str(), val_str.as_str())
            .map_err(|e| e.to_string())?;
    }
    write_txn.commit().map_err(|e| e.to_string())?;

    // Update cache
    registry
        .api_keys
        .write()
        .unwrap()
        .insert(hash, (user_id.to_string(), role));
    Ok(())
}

pub fn grant_access(
    owner: &str,
    collection: &str,
    grantee: &str,
    privilege: UserRole,
) -> Result<(), String> {
    let registry = REGISTRY.get().ok_or("Security registry not initialized")?;

    let write_txn = registry.db.begin_write().map_err(|e| e.to_string())?;
    let mut table = write_txn
        .open_table(GRANTS_TABLE)
        .map_err(|e| e.to_string())?;
    let mut list = if let Some(val) = table.get(grantee).map_err(|e| e.to_string())? {
        serde_json::from_str::<Vec<GrantEntry>>(val.value()).unwrap_or_default()
    } else {
        Vec::new()
    };

    // Remove existing grant if any
    list.retain(|g| !(g.owner == owner && g.collection == collection));

    list.push(GrantEntry {
        owner: owner.to_string(),
        collection: collection.to_string(),
        role: privilege,
    });

    let val_str = serde_json::to_string(&list).map_err(|e| e.to_string())?;
    table
        .insert(grantee, val_str.as_str())
        .map_err(|e| e.to_string())?;
    drop(table);
    write_txn.commit().map_err(|e| e.to_string())?;

    // Update cache
    let mapped = list
        .into_iter()
        .map(|g| (g.owner, g.collection, g.role))
        .collect();
    registry
        .grants
        .write()
        .unwrap()
        .insert(grantee.to_string(), mapped);
    Ok(())
}

pub fn revoke_access(owner: &str, collection: &str, grantee: &str) -> Result<(), String> {
    let registry = REGISTRY.get().ok_or("Security registry not initialized")?;

    let write_txn = registry.db.begin_write().map_err(|e| e.to_string())?;
    let mut table = write_txn
        .open_table(GRANTS_TABLE)
        .map_err(|e| e.to_string())?;
    let mut list = if let Some(val) = table.get(grantee).map_err(|e| e.to_string())? {
        serde_json::from_str::<Vec<GrantEntry>>(val.value()).unwrap_or_default()
    } else {
        Vec::new()
    };

    list.retain(|g| !(g.owner == owner && g.collection == collection));

    let val_str = serde_json::to_string(&list).map_err(|e| e.to_string())?;
    if list.is_empty() {
        let _ = table.remove(grantee);
    } else {
        table
            .insert(grantee, val_str.as_str())
            .map_err(|e| e.to_string())?;
    }
    drop(table);
    write_txn.commit().map_err(|e| e.to_string())?;

    // Update cache
    if list.is_empty() {
        registry.grants.write().unwrap().remove(grantee);
    } else {
        let mapped = list
            .into_iter()
            .map(|g| (g.owner, g.collection, g.role))
            .collect();
        registry
            .grants
            .write()
            .unwrap()
            .insert(grantee.to_string(), mapped);
    }
    Ok(())
}

pub fn list_grants(owner: &str, collection: &str) -> Vec<(String, UserRole)> {
    let mut results = Vec::new();
    if let Some(registry) = REGISTRY.get() {
        if let Ok(guard) = registry.grants.read() {
            for (grantee, list) in guard.iter() {
                for (o, c, role) in list {
                    if o == owner && c == collection {
                        results.push((grantee.clone(), *role));
                    }
                }
            }
        }
    }
    results
}

pub fn list_shared_collections(grantee: &str) -> Vec<(String, String, UserRole)> {
    if let Some(registry) = REGISTRY.get() {
        if let Ok(guard) = registry.grants.read() {
            if let Some(list) = guard.get(grantee) {
                return list.clone();
            }
        }
    }
    Vec::new()
}
