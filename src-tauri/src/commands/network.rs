use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use mdns_sd::{IfKind, IfPredicate, ServiceDaemon, ServiceEvent, ServiceInfo};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::Emitter;

use crate::commands::auth::{hash_password, verify_password};
use crate::{AppState, Database, Employee, NetworkAttendanceCell, UpdateCellRequest};

const SERVICE_TYPE: &str = "_oasis._tcp.local.";
const SERVER_PORT: u16 = 3000;

// --- Request structs ---
#[derive(Debug, Deserialize)]
struct NetworkLoginRequest { username: String, password: String }

#[derive(Debug, Deserialize)]
struct NetworkChangePasswordRequest { session_token: String, new_password: String }

#[derive(Debug, Deserialize)]
struct NetworkChangeCredentialsRequest {
    session_token: String,
    current_password: String,
    new_username: String,
    new_password: String,
}

#[derive(Debug, Deserialize)]
struct NetworkResetDivisionRequest {
    session_token: String,
    actor_password: String,
    target_role: String,
    new_username: String,
    new_password: String,
}

#[derive(Debug, Deserialize)]
struct NetworkSessionRequest { session_token: String }

#[derive(Debug, Deserialize)]
struct NetworkRecoveryRequest { recovery_code: String, new_password: String }

// --- Helpers ---
fn auth_error(message: &str, status: StatusCode) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "ok": false, "message": message })))
}

fn session_user(state: &AppState, token: &str) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let mut sessions = state.sessions.lock().map_err(|_| auth_error("Sesi sedang sibuk.", StatusCode::INTERNAL_SERVER_ERROR))?;
    let (username, created) = sessions.get(token).cloned().ok_or_else(|| auth_error("Sesi login sudah tidak berlaku. Silakan login lagi.", StatusCode::UNAUTHORIZED))?;
    if created.elapsed() > Duration::from_secs(8 * 60 * 60) {
        sessions.remove(token);
        return Err(auth_error("Sesi login sudah berakhir. Silakan login lagi.", StatusCode::UNAUTHORIZED));
    }
    Ok(username)
}

fn is_unique_violation(error: &rusqlite::Error) -> bool {
    matches!(error, rusqlite::Error::SqliteFailure(f, _) if f.code == rusqlite::ErrorCode::ConstraintViolation)
}

// Same rule as auth.rs: usernames are unique ignoring case.
fn username_taken(conn: &rusqlite::Connection, name: &str, except_role: &str) -> bool {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM division_accounts WHERE LOWER(username) = LOWER(?1) AND role_division != ?2)",
        rusqlite::params![name, except_role],
        |row| row.get::<_, i64>(0),
    )
    .map(|value| value == 1)
    .unwrap_or(true) // if the check itself fails, refuse rather than risk a duplicate
}

// Sessions store the username, so a rename must follow it or the next request fails.
fn rename_sessions(state: &AppState, old: &str, new: &str) {
    if let Ok(mut sessions) = state.sessions.lock() {
        for (name, _) in sessions.values_mut() {
            if name == old {
                *name = new.to_string();
            }
        }
    }
}

fn drop_sessions_of(state: &AppState, username: &str) {
    if let Ok(mut sessions) = state.sessions.lock() {
        sessions.retain(|_, (name, _)| name != username);
    }
}

fn notify_credentials_changed(state: &AppState) {
    if let Some(app) = &state.app {
        let _ = app.emit("credentials-changed", ());
    }
}

// --- Handlers ---
async fn api_health_check(State(_state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "online",
        "app": "OASIS One Master Node"
    }))
}

async fn api_auth_setup_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let needs_setup = state.db.lock().ok().and_then(|conn| conn.query_row(
        "SELECT setting_value FROM app_settings WHERE setting_key = 'password_setup_complete'",
        [], |row| row.get::<_, String>(0),
    ).ok()).map(|value| value != "1").unwrap_or(true);
    Json(serde_json::json!({ "ok": true, "needs_setup": needs_setup }))
}

async fn api_auth_login(
    State(state): State<AppState>, Json(request): Json<NetworkLoginRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let username = request.username.trim();
    let conn = state.db.lock().map_err(|_| auth_error("Database sedang sibuk.", StatusCode::INTERNAL_SERVER_ERROR))?;
    let result = conn.query_row(
        "SELECT role_division, password_hash FROM division_accounts WHERE username = ?1",
        [username], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );
    match result {
        Ok((role, stored_hash)) if verify_password(&request.password, &stored_hash) => {
            let token = uuid::Uuid::new_v4().simple().to_string();
            state.sessions.lock().map_err(|_| auth_error("Sesi sedang sibuk.", StatusCode::INTERNAL_SERVER_ERROR))?
                .insert(token.clone(), (username.to_string(), Instant::now()));
            Ok(Json(serde_json::json!({ "ok": true, "role": role, "session_token": token })))
        }
        _ => Err(auth_error("Username atau password tidak valid.", StatusCode::UNAUTHORIZED)),
    }
}

// Legacy route (password only). Delete it once nothing calls changeMasterPassword.
async fn api_auth_change_password(
    State(state): State<AppState>, Json(request): Json<NetworkChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if request.new_password.chars().count() < 8 { return Err(auth_error("Password baru minimal 8 karakter.", StatusCode::BAD_REQUEST)); }
    let username = session_user(&state, &request.session_token)?;
    let conn = state.db.lock().map_err(|_| auth_error("Database sedang sibuk.", StatusCode::INTERNAL_SERVER_ERROR))?;
    conn.execute("UPDATE division_accounts SET password_hash = ?1 WHERE username = ?2",
        rusqlite::params![hash_password(&request.new_password), username])
        .map_err(|_| auth_error("Password gagal disimpan.", StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// Any logged-in account changes its OWN username and password.
async fn api_auth_change_credentials(
    State(state): State<AppState>, Json(request): Json<NetworkChangeCredentialsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let new_password = request.new_password.trim().to_string();
    if new_password.chars().count() < 8 { return Err(auth_error("Password baru minimal 8 karakter.", StatusCode::BAD_REQUEST)); }
    let new_user = request.new_username.trim().to_string();
    if new_user.chars().count() < 3 { return Err(auth_error("Username baru minimal 3 karakter.", StatusCode::BAD_REQUEST)); }

    let username = session_user(&state, &request.session_token)?;
    let conn = state.db.lock().map_err(|_| auth_error("Database sedang sibuk.", StatusCode::INTERNAL_SERVER_ERROR))?;

    let (role, stored_hash): (String, String) = conn.query_row(
        "SELECT role_division, password_hash FROM division_accounts WHERE username = ?1", [&username],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|_| auth_error("Akun tidak ditemukan.", StatusCode::UNAUTHORIZED))?;

    if !verify_password(&request.current_password, &stored_hash) {
        return Err(auth_error("Password lama salah.", StatusCode::UNAUTHORIZED));
    }

    if username_taken(&conn, &new_user, &role) {
        return Err(auth_error("Username tersebut sudah digunakan oleh divisi lain.", StatusCode::CONFLICT));
    }

    conn.execute(
        "UPDATE division_accounts SET username = ?1, password_hash = ?2 WHERE username = ?3",
        rusqlite::params![new_user, hash_password(&new_password), username],
    ).map_err(|e| if is_unique_violation(&e) {
        auth_error("Username tersebut sudah digunakan oleh divisi lain.", StatusCode::CONFLICT)
    } else {
        auth_error("Kredensial gagal disimpan.", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    rename_sessions(&state, &username, &new_user);
    notify_credentials_changed(&state);
    Ok(Json(serde_json::json!({ "ok": true, "username": new_user })))
}

// HR superadmin: reset ANOTHER division's username and password.
async fn api_auth_reset_division(
    State(state): State<AppState>, Json(request): Json<NetworkResetDivisionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let new_password = request.new_password.trim().to_string();
    if new_password.chars().count() < 8 { return Err(auth_error("Password baru minimal 8 karakter.", StatusCode::BAD_REQUEST)); }
    let new_user = request.new_username.trim().to_string();
    if new_user.chars().count() < 3 { return Err(auth_error("Username baru minimal 3 karakter.", StatusCode::BAD_REQUEST)); }

    let admin_username = session_user(&state, &request.session_token)?;
    let conn = state.db.lock().map_err(|_| auth_error("Database sedang sibuk.", StatusCode::INTERNAL_SERVER_ERROR))?;

    let (admin_role, admin_hash): (String, String) = conn.query_row(
        "SELECT role_division, password_hash FROM division_accounts WHERE username = ?1", [&admin_username],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ).map_err(|_| auth_error("Akun admin tidak ditemukan.", StatusCode::UNAUTHORIZED))?;

    if admin_role != "hr" { return Err(auth_error("Hanya sesi HR admin yang dapat mengelola kredensial.", StatusCode::FORBIDDEN)); }
    if !verify_password(&request.actor_password, &admin_hash) {
        return Err(auth_error("Password HR salah.", StatusCode::UNAUTHORIZED));
    }

    let target_role = request.target_role.trim().to_string();
    let target_old: String = conn.query_row(
        "SELECT username FROM division_accounts WHERE role_division = ?1", [&target_role], |row| row.get(0),
    ).map_err(|_| auth_error("Divisi tidak ditemukan.", StatusCode::NOT_FOUND))?;

    if username_taken(&conn, &new_user, &target_role) {
        return Err(auth_error("Username tersebut sudah digunakan oleh divisi lain.", StatusCode::CONFLICT));
    }

    conn.execute(
        "UPDATE division_accounts SET username = ?1, password_hash = ?2 WHERE role_division = ?3",
        rusqlite::params![new_user, hash_password(&new_password), target_role],
    ).map_err(|e| if is_unique_violation(&e) {
        auth_error("Username tersebut sudah digunakan oleh divisi lain.", StatusCode::CONFLICT)
    } else {
        auth_error("Kredensial gagal diperbarui.", StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    if target_old == admin_username {
        rename_sessions(&state, &target_old, &new_user);
    } else {
        // The division is logged out; it must use the credentials HR just set.
        drop_sessions_of(&state, &target_old);
    }

    notify_credentials_changed(&state);
    Ok(Json(serde_json::json!({ "ok": true })))
}

// HR only: lets the HR screen show each division's current username.
async fn api_auth_list_accounts(
    State(state): State<AppState>, Json(request): Json<NetworkSessionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let admin_username = session_user(&state, &request.session_token)?;
    let conn = state.db.lock().map_err(|_| auth_error("Database sedang sibuk.", StatusCode::INTERNAL_SERVER_ERROR))?;

    let admin_role: String = conn.query_row(
        "SELECT role_division FROM division_accounts WHERE username = ?1", [&admin_username], |row| row.get(0),
    ).map_err(|_| auth_error("Akun admin tidak ditemukan.", StatusCode::UNAUTHORIZED))?;
    if admin_role != "hr" { return Err(auth_error("Hanya HR yang dapat melihat daftar akun.", StatusCode::FORBIDDEN)); }

    let mut stmt = conn
        .prepare("SELECT username, role_division FROM division_accounts ORDER BY role_division")
        .map_err(|_| auth_error("Gagal membaca akun.", StatusCode::INTERNAL_SERVER_ERROR))?;
    let accounts: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "username": row.get::<_, String>(0)?,
                "role": row.get::<_, String>(1)?,
            }))
        })
        .map_err(|_| auth_error("Gagal membaca akun.", StatusCode::INTERNAL_SERVER_ERROR))?
        .filter_map(Result::ok)
        .collect();

    Ok(Json(serde_json::json!({ "ok": true, "accounts": accounts })))
}

async fn api_auth_recover_master(
    State(state): State<AppState>, Json(request): Json<NetworkRecoveryRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    if request.new_password.chars().count() < 8 { return Err(auth_error("Password baru minimal 8 karakter.", StatusCode::BAD_REQUEST)); }
    let conn = state.db.lock().map_err(|_| auth_error("Database sedang sibuk.", StatusCode::INTERNAL_SERVER_ERROR))?;
    let stored_hash: String = conn.query_row(
        "SELECT setting_value FROM app_settings WHERE setting_key = 'master_recovery_hash'", [], |row| row.get(0),
    ).map_err(|_| auth_error("Kode pemulihan belum tersedia.", StatusCode::NOT_FOUND))?;
    if !verify_password(&request.recovery_code.trim().to_uppercase(), &stored_hash) {
        return Err(auth_error("Kode pemulihan salah.", StatusCode::UNAUTHORIZED));
    }
    conn.execute("UPDATE division_accounts SET password_hash = ?1 WHERE role_division = 'hr'",
        rusqlite::params![hash_password(&request.new_password)])
        .map_err(|_| auth_error("Password gagal disimpan.", StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn api_get_employees(State(state): State<AppState>) -> Json<Vec<Employee>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT nik, nama_lengkap, divisi, jabatan, status FROM master_karyawan").unwrap();

    let employee_iter = stmt.query_map([], |row| {
        Ok(Employee {
            nik: row.get(0)?,
            nama_lengkap: row.get(1)?,
            divisi: row.get(2)?,
            jabatan: row.get(3)?,
            status: row.get(4)?,
            ..Default::default()
        })
    }).unwrap();

    let employees: Vec<Employee> = employee_iter.filter_map(Result::ok).collect();
    Json(employees)
}

async fn api_get_division_employees(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<Employee>> {
    let division = params.get("divisi").cloned().unwrap_or_default();
    let pattern = format!("%{}%", division.to_lowercase()); // bound parameter, not string-built SQL
    let conn = state.db.lock().unwrap();

    let query = if division.eq_ignore_ascii_case("staff") {
        "SELECT nik, timestamp, join_date, tanggal_kartap, nama_lengkap, jenis_kelamin,
                tanggal_lahir, golongan_darah, nomor_kk, alamat_ktp, bpjs_kesehatan,
                bpjs_ketenagakerjaan, nomor_telepon, jabatan, divisi, nama_ibu_kandung,
                nama_pasangan, jumlah_anak, nomor_telp_keluarga, foto_ktp, foto_kk,
                foto_bpjs_kesehatan, foto_bpjs_ketenagakerjaan, pendidikan_terakhir,
                perjanjian_kerja, status, tempat_kerja, is_flagged, regu, catatan
         FROM master_karyawan
         WHERE (LOWER(divisi) LIKE ?1 OR tempat_kerja = 'Pengelola') AND status = 'AKTIF'
         ORDER BY nama_lengkap COLLATE NOCASE"
    } else {
        "SELECT nik, timestamp, join_date, tanggal_kartap, nama_lengkap, jenis_kelamin,
                tanggal_lahir, golongan_darah, nomor_kk, alamat_ktp, bpjs_kesehatan,
                bpjs_ketenagakerjaan, nomor_telepon, jabatan, divisi, nama_ibu_kandung,
                nama_pasangan, jumlah_anak, nomor_telp_keluarga, foto_ktp, foto_kk,
                foto_bpjs_kesehatan, foto_bpjs_ketenagakerjaan, pendidikan_terakhir,
                perjanjian_kerja, status, tempat_kerja, is_flagged, regu, catatan
         FROM master_karyawan
         WHERE LOWER(divisi) LIKE ?1 AND status = 'AKTIF'
         ORDER BY nama_lengkap COLLATE NOCASE"
    };

    let mut stmt = match conn.prepare(query) {
        Ok(s) => s,
        Err(_) => return Json(vec![]),
    };

    let employee_iter = stmt.query_map([pattern], |row| {
        let flagged_int: i32 = row.get(27)?;
        Ok(Employee {
            nik: row.get(0)?,
            timestamp: row.get(1)?,
            join_date: row.get(2)?,
            tanggal_kartap: row.get(3)?,
            nama_lengkap: row.get(4)?,
            jenis_kelamin: row.get(5)?,
            tanggal_lahir: row.get(6)?,
            golongan_darah: row.get(7)?,
            nomor_kk: row.get(8)?,
            alamat_ktp: row.get(9)?,
            bpjs_kesehatan: row.get(10)?,
            bpjs_ketenagakerjaan: row.get(11)?,
            nomor_telepon: row.get(12)?,
            jabatan: row.get(13)?,
            divisi: row.get(14)?,
            nama_ibu_kandung: row.get(15)?,
            nama_pasangan: row.get(16)?,
            jumlah_anak: row.get(17)?,
            nomor_telp_keluarga: row.get(18)?,
            foto_ktp: row.get(19)?,
            foto_kk: row.get(20)?,
            foto_bpjs_kesehatan: row.get(21)?,
            foto_bpjs_ketenagakerjaan: row.get(22)?,
            pendidikan_terakhir: row.get(23)?,
            perjanjian_kerja: row.get(24)?,
            status: row.get(25)?,
            tempat_kerja: row.get(26)?,
            is_flagged: Some(flagged_int != 0),
            regu: row.get(28)?,
            catatan: row.get(29)?,
        })
    });

    let employees = match employee_iter {
        Ok(iter) => iter.filter_map(Result::ok).collect(),
        Err(_) => vec![],
    };

    Json(employees)
}

async fn api_get_or_create_periode(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<crate::models::attendance::PeriodeAbsensi>, StatusCode> {
    let year = params.get("year").and_then(|v| v.parse::<i32>().ok()).unwrap_or(2026);
    let month = params.get("month").and_then(|v| v.parse::<u32>().ok()).unwrap_or(10);

    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let prev_month = if month == 12 { 11 } else if month == 1 { 12 } else { month - 1 };
    let prev_year = if month == 1 { year - 1 } else { year };

    let start_date = chrono::NaiveDate::from_ymd_opt(prev_year, prev_month, 17)
        .ok_or(StatusCode::BAD_REQUEST)?;
    let end_date = chrono::NaiveDate::from_ymd_opt(year, month, 16)
        .ok_or(StatusCode::BAD_REQUEST)?;

    let start_str = start_date.format("%Y-%m-%d").to_string();
    let end_str = end_date.format("%Y-%m-%d").to_string();
    let total_days = (end_date - start_date).num_days() as i32 + 1;
    let nama_periode = format!(
        "Periode {} s/d {}",
        start_date.format("%d %b %Y"),
        end_date.format("%d %b %Y")
    );

    let mut stmt = conn
        .prepare("SELECT id, nama_periode, tanggal_mulai, tanggal_selesai, total_hari FROM periode_absensi WHERE tanggal_mulai = ?1 AND tanggal_selesai = ?2")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let existing = stmt.query_row([&start_str, &end_str], |row| {
        Ok(crate::models::attendance::PeriodeAbsensi {
            id: Some(row.get(0)?),
            nama_periode: row.get(1)?,
            tanggal_mulai: row.get(2)?,
            tanggal_selesai: row.get(3)?,
            total_hari: row.get(4)?,
        })
    });

    if let Ok(p) = existing {
        return Ok(Json(p));
    }

    conn.execute(
        "INSERT INTO periode_absensi (nama_periode, tanggal_mulai, tanggal_selesai, total_hari) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![nama_periode, start_str, end_str, total_days],
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let id = conn.last_insert_rowid();
    Ok(Json(crate::models::attendance::PeriodeAbsensi {
        id: Some(id),
        nama_periode,
        tanggal_mulai: start_str,
        tanggal_selesai: end_str,
        total_hari: total_days,
    }))
}

async fn api_update_absensi_cell(
    State(state): State<AppState>,
    Json(payload): Json<UpdateCellRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    conn.execute(
        "INSERT INTO absensi (nik, periode_id, tanggal, status, keterangan)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(nik, tanggal) DO UPDATE SET
            status = excluded.status,
            keterangan = excluded.keterangan",
        rusqlite::params![
            payload.nik,
            payload.periode_id,
            payload.tanggal,
            payload.status,
            payload.keterangan.unwrap_or_default()
        ],
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({ "success": true })))
}

async fn api_get_attendance_cells(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<NetworkAttendanceCell>>, StatusCode> {
    let periode_id = params
        .get("periode_id")
        .and_then(|value| value.parse::<i64>().ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let conn = state.db.lock().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut statement = conn
        .prepare(
            "SELECT nik, tanggal, status, COALESCE(keterangan, '')
             FROM absensi WHERE periode_id = ?1 ORDER BY tanggal, nik",
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let rows = statement
        .query_map([periode_id], |row| {
            Ok(NetworkAttendanceCell {
                nik: row.get(0)?,
                tanggal: row.get(1)?,
                status: row.get(2)?,
                keterangan: row.get(3)?,
            })
        })
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    rows.collect::<Result<Vec<_>, _>>()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// --- Discovery (mDNS) ---

// Keep the daemon alive for the whole app lifetime.
static MDNS_DAEMON: OnceLock<ServiceDaemon> = OnceLock::new();

fn advertise_master(port: u16) -> Result<(), String> {
    let daemon = ServiceDaemon::new().map_err(|e| e.to_string())?;

    // Empty IP + enable_addr_auto(): advertise every local interface address
    // and update automatically when the PC joins a different Wi-Fi.
    let mut info = ServiceInfo::new(
        SERVICE_TYPE,
        "oasis-master",
        "oasis-master.local.",
        "",
        port,
        HashMap::<String, String>::new(),
    )
    .map_err(|e| e.to_string())?
    .enable_addr_auto();

    info.set_interfaces(vec![IfKind::Predicate(IfPredicate::new(|interface| {
        matches!(
            interface.ip(),
            IpAddr::V4(address) if !address.is_loopback() && !address.is_link_local()
        )
    }))]);

    daemon.register(info).map_err(|e| e.to_string())?;
    println!("[master] mDNS advertising {SERVICE_TYPE} on port {port}");
    let _ = MDNS_DAEMON.set(daemon);
    Ok(())
}

/// Returns every master address found on the current network,
/// e.g. ["http://192.168.1.20:3000"]. Real LAN addresses come first and
/// loopback last, so the frontend health-checks the useful ones first.
#[tauri::command]
pub async fn discover_master() -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let daemon = ServiceDaemon::new().map_err(|e| e.to_string())?;
        let receiver = daemon.browse(SERVICE_TYPE).map_err(|e| e.to_string())?;

        let mut deadline = Instant::now() + Duration::from_secs(4);
        let mut found: Vec<String> = Vec::new();
        let mut grace_applied = false;

        while let Some(remaining) = deadline.checked_duration_since(Instant::now()) {
            match receiver.recv_timeout(remaining) {
                Ok(ServiceEvent::ServiceResolved(info)) => {
                    for ip in info.get_addresses_v4() {
                        if ip.is_link_local() {
                            continue;
                        }
                        found.push(format!("http://{}:{}", ip, info.get_port()));
                    }
                    // After the first answer, wait a little longer so the master's
                    // other interfaces can be collected too.
                    if !found.is_empty() && !grace_applied {
                        grace_applied = true;
                        deadline = deadline.min(Instant::now() + Duration::from_millis(800));
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }

        let _ = daemon.shutdown();
        found.sort();
        found.dedup();
        found.sort_by_key(|url| url.contains("//127.")); // stable: real addresses first
        println!("[discovery] mDNS scan finished, found: {:?}", found);
        Ok::<Vec<String>, String>(found)
    })
    .await
    .map_err(|e| e.to_string())?
}

// --- Server startup ---
static MASTER_SERVER_STARTED: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn start_master_server(
    app: tauri::AppHandle,
    database: tauri::State<'_, Database>,
) -> Result<(), String> {
    if MASTER_SERVER_STARTED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Ok(());
    }

    let server_state = AppState {
        db: Arc::clone(&database.0),
        sessions: Arc::new(Mutex::new(HashMap::new())),
        app: Some(app),
    };

    let (tx, rx) = mpsc::channel::<Result<(), String>>();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(error) => {
                MASTER_SERVER_STARTED.store(false, Ordering::SeqCst);
                let _ = tx.send(Err(format!("Cannot create Tokio runtime: {error}")));
                return;
            }
        };

        rt.block_on(async move {
            let app_router = Router::new()
                .route("/api/health", get(api_health_check))
                .route("/api/auth/setup-status", get(api_auth_setup_status))
                .route("/api/auth/login", post(api_auth_login))
                .route("/api/auth/change-password", post(api_auth_change_password))
                .route("/api/auth/change-credentials", post(api_auth_change_credentials))
                .route("/api/auth/reset-division", post(api_auth_reset_division))
                .route("/api/auth/list-accounts", post(api_auth_list_accounts))
                .route("/api/auth/recover-master", post(api_auth_recover_master))
                .route("/api/employees", get(api_get_employees))
                .route("/api/employees/division", get(api_get_division_employees))
                .route("/api/attendance/periode", get(api_get_or_create_periode))
                .route("/api/attendance/cells", get(api_get_attendance_cells))
                .route("/api/attendance/cell", post(api_update_absensi_cell))
                .with_state(server_state);

            let addr = SocketAddr::from(([0, 0, 0, 0], SERVER_PORT));

            match tokio::net::TcpListener::bind(addr).await {
                Ok(listener) => {
                    println!("OASIS Master API server running on port {SERVER_PORT}");

                    if let Err(error) = advertise_master(SERVER_PORT) {
                        eprintln!("mDNS advertise failed: {error}");
                    }

                    let _ = tx.send(Ok(()));

                    if let Err(error) = axum::serve(listener, app_router).await {
                        eprintln!("OASIS network server stopped: {error}");
                        MASTER_SERVER_STARTED.store(false, Ordering::SeqCst);
                    }
                }
                Err(error) => {
                    MASTER_SERVER_STARTED.store(false, Ordering::SeqCst);
                    let _ = tx.send(Err(format!(
                        "Cannot start OASIS server on port {SERVER_PORT}: {error}"
                    )));
                }
            }
        });
    });

    rx.recv()
        .map_err(|error| format!("Server startup failed: {error}"))?
}