use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::State;
use crate::Database;

// Hanya wajibkan Role-nya saja, username sekarang bebas (custom)
const REQUIRED_ROLES: [&str; 4] = ["hr", "security", "engineering", "housekeeping"];

#[derive(Debug, Deserialize)]
pub struct DivisionPasswordSetup {
    pub username: String,
    pub password: String,
    pub role_division: String, // ⬅️ Agar Rust tahu username ini untuk divisi apa
}

#[derive(Debug, Serialize)]
pub struct SecurityAuthAccount {
    pub username: String,
    pub role: String,
    pub password_hash: String,
}

fn hash_inspection_pin(pin: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pin.trim().as_bytes());
    format!("{:x}", hasher.finalize())
}

pub(crate) fn hash_password(password: &str) -> String {
    let salt = uuid::Uuid::new_v4().simple().to_string();
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(b":");
    hasher.update(password.as_bytes());
    let digest = hasher.finalize();
    format!("sha256${}${:x}", salt, digest)
}

pub(crate) fn verify_password(password: &str, stored: &str) -> bool {
    let mut parts = stored.split('$');
    if parts.next() != Some("sha256") {
        return false;
    }
    let salt = match parts.next() {
        Some(value) if !value.is_empty() => value,
        _ => return false,
    };
    let expected = match parts.next() {
        Some(value) if !value.is_empty() => value,
        _ => return false,
    };

    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(b":");
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize()) == expected
}

#[tauri::command]
pub fn authenticate(
    database: State<'_, Database>, 
    username: String, 
    password: String
) -> Result<String, String> {
    let connection = database.0.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = connection
        .prepare("SELECT role_division, password_hash FROM division_accounts WHERE username = ?1")
        .map_err(|e| e.to_string())?;
    
    let mut rows = stmt.query(params![username.trim()]).map_err(|e| e.to_string())?;
    
    if let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let role: String = row.get(0).map_err(|e| e.to_string())?;
        let stored_hash: String = row.get(1).map_err(|e| e.to_string())?;
        if verify_password(&password, &stored_hash) {
            Ok(role)
        } else {
            Err("Kredensial tidak valid atau divisi tidak ditemukan.".into())
        }
    } else {
        Err("Kredensial tidak valid atau divisi tidak ditemukan.".into())
    }
}

#[tauri::command]
pub fn get_security_auth_accounts(
    database: State<'_, Database>,
) -> Result<Vec<SecurityAuthAccount>, String> {
    let connection = database.0.lock().map_err(|e| e.to_string())?;
    let mut statement = connection
        .prepare(
            "SELECT username, role_division, password_hash
             FROM division_accounts WHERE role_division = 'security'",
        )
        .map_err(|e| e.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok(SecurityAuthAccount {
                username: row.get(0)?,
                role: row.get(1)?,
                password_hash: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_password_setup_status(database: State<'_, Database>) -> Result<bool, String> {
    let connection = database.0.lock().map_err(|e| e.to_string())?;
    let complete: String = connection
        .query_row(
            "SELECT setting_value FROM app_settings WHERE setting_key = 'password_setup_complete'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(complete != "1")
}

/// Stores the computer role once so updater builds can be shared by HR and clients.
/// The build-time flag is only used when this database has no role yet.
#[tauri::command]
pub fn get_or_initialize_node_mode(
    database: State<'_, Database>,
    preferred_master: bool,
) -> Result<bool, String> {
    let connection = database.0.lock().map_err(|e| e.to_string())?;
    let existing: Option<String> = connection
        .query_row(
            "SELECT setting_value FROM app_settings WHERE setting_key = 'node_mode'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if let Some(mode) = existing {
        return Ok(mode == "master");
    }

    let mode = if preferred_master { "master" } else { "client" };
    connection
        .execute(
            "INSERT INTO app_settings (setting_key, setting_value) VALUES ('node_mode', ?1)",
            [mode],
        )
        .map_err(|e| e.to_string())?;
    Ok(preferred_master)
}

#[tauri::command]
pub fn setup_division_passwords(
    database: State<'_, Database>,
    accounts: Vec<DivisionPasswordSetup>,
) -> Result<String, String> {
    if accounts.len() != REQUIRED_ROLES.len() {
        return Err("Semua kredensial divisi (4 divisi) harus dibuat.".into());
    }

    let mut connection = database.0.lock().map_err(|e| e.to_string())?;
    let setup_complete: String = connection
        .query_row(
            "SELECT setting_value FROM app_settings WHERE setting_key = 'password_setup_complete'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if setup_complete == "1" {
        return Err("Password sudah pernah dibuat. Gunakan menu ganti password.".into());
    }

    let mut submitted_usernames = std::collections::HashSet::new();
    let mut submitted_roles = std::collections::HashSet::new();

    for account in &accounts {
        let username = account.username.trim();
        let password = account.password.trim();
        let role = account.role_division.trim();

        if username.len() < 3 {
            return Err(format!("Username {} minimal 3 karakter.", username));
        }
        if password.chars().count() < 8 {
            return Err(format!("Password {} minimal 8 karakter.", username));
        }
        if !REQUIRED_ROLES.contains(&role) {
            return Err(format!("Role divisi '{}' tidak valid.", role));
        }
        
        if !submitted_usernames.insert(username.to_lowercase()) {
            return Err("Username divisi tidak boleh duplikat.".into());
        }
        submitted_roles.insert(role.to_string());
    }

    if submitted_roles.len() != REQUIRED_ROLES.len() {
        return Err("Kredensial untuk semua divisi (HR, Security, Engineering, Housekeeping) harus diisi lengkap.".into());
    }

    let transaction = connection.transaction().map_err(|e| e.to_string())?;
    let recovery_code = uuid::Uuid::new_v4().simple().to_string()[..12].to_uppercase();
    transaction.execute("DELETE FROM division_accounts", [])
        .map_err(|e| e.to_string())?;

    for account in accounts {
        transaction.execute(
            "INSERT INTO division_accounts (username, password_hash, role_division) VALUES (?1, ?2, ?3)",
            params![account.username.trim(), hash_password(account.password.trim()), account.role_division.trim()],
        ).map_err(|e| e.to_string())?;
    }

    transaction.execute(
        "UPDATE app_settings SET setting_value = '1' WHERE setting_key = 'password_setup_complete'",
        [],
    ).map_err(|e| e.to_string())?;
    
    transaction.execute(
        "INSERT OR REPLACE INTO app_settings (setting_key, setting_value) VALUES ('master_recovery_hash', ?1)",
        params![hash_password(&recovery_code)],
    ).map_err(|e| e.to_string())?;
    
    transaction.commit().map_err(|e| e.to_string())?;
    Ok(recovery_code)
}

/// Allows HR Admin to update custom usernames and reset passwords for any department from any workstation
#[tauri::command]
pub fn reset_division_account(
    database: State<'_, Database>,
    actor_username: String, // Username yang sedang login
    actor_password: String, // Password yang sedang login (untuk verifikasi)
    target_role: String,    // Role divisi yang ingin diubah ('hr', 'security', etc.)
    new_username: String,   // Username baru
    new_password: String,   // Password baru
) -> Result<(), String> {
    let connection = database.0.lock().map_err(|e| e.to_string())?;

    // 1. Ambil role dan password hash dari user yang melakukan aksi (actor)
    let (actor_role, stored_hash): (String, String) = connection
        .query_row(
            "SELECT role_division, password_hash FROM division_accounts WHERE username = ?1",
            params![actor_username.trim()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "Akun tidak ditemukan.".to_string())?;

    // 2. Verifikasi password actor
    if !verify_password(&actor_password, &stored_hash) {
        return Err("Password saat ini salah.".into());
    }

    // 3. Tentukan apakah ini aksi self-service atau HR admin reset
    // Jika actor BUKAN 'hr', mereka hanya boleh mengubah akun mereka sendiri (target_role harus sesuai role mereka)
    let target = target_role.trim();
    if actor_role != "hr" && actor_role != target {
        return Err("Akses ditolak. Anda hanya dapat mengubah akun Anda sendiri.".into());
    }

    // 4. Validasi input baru
    let new_user = new_username.trim();
    let new_pass = new_password.trim();
    if new_user.len() < 3 {
        return Err("Username baru minimal 3 karakter.".into());
    }
    if new_pass.chars().count() < 8 {
        return Err("Password baru minimal 8 karakter.".into());
    }

    // 5. Cek apakah username baru sudah dipakai oleh divisi lain
    let existing_role: Option<String> = connection
        .query_row(
            "SELECT role_division FROM division_accounts WHERE LOWER(username) = LOWER(?1) AND role_division != ?2",
            params![new_user, target],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    if existing_role.is_some() {
        return Err("Username tersebut sudah digunakan oleh divisi lain.".into());
    }

    // 6. Update username dan password hash di database
    connection.execute(
        "UPDATE division_accounts SET username = ?1, password_hash = ?2 WHERE role_division = ?3",
        params![new_user, hash_password(new_pass), target],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn cache_security_pin(
    database: State<'_, Database>,
    name: String,
    pin: String,
) -> Result<(), String> {
    if name.trim().is_empty() || !pin.trim().chars().all(|character| character.is_ascii_digit()) {
        return Err("Nama dan PIN tidak valid.".into());
    }
    let connection = database.0.lock().map_err(|e| e.to_string())?;
    connection.execute(
        "INSERT INTO security_pin_cache (name, pin_hash, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(name) DO UPDATE SET pin_hash = excluded.pin_hash, updated_at = excluded.updated_at",
        params![name.trim(), hash_inspection_pin(&pin), chrono::Utc::now().to_rfc3339()],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_security_pin_cache_status(
    database: State<'_, Database>,
    name: String,
) -> Result<bool, String> {
    let connection = database.0.lock().map_err(|e| e.to_string())?;
    let exists: i64 = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM security_pin_cache WHERE name = ?1)",
        params![name.trim()],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    Ok(exists == 1)
}