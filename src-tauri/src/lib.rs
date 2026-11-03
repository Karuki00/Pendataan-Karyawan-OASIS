use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::Manager;

use commands::auth::{
    authenticate, cache_security_pin, get_or_initialize_node_mode, get_password_setup_status,
    get_security_auth_accounts, get_security_pin_cache_status, reset_division_account,
    setup_division_passwords,
};

use commands::attendance::{
    get_or_create_periode, initialize_attendance_schema, list_engineering_employees_for_absensi,
    list_housekeeping_employees_for_absensi, list_security_employees_for_absensi,
    list_staff_employees_for_absensi, reset_database, toggle_employee_active_status,
    update_absensi_cell,
};
use commands::employee::{delete_employee, list_employees, save_employee, upsert_employees};
use commands::excel_exporter::export_rekap_absen_excel_1to1;
use commands::leave::{get_rekap_cuti_tahun, update_saldo_cuti_karyawan};
use commands::network::{discover_master, start_master_server};
use commands::system::{export_excel_file, open_folder_dir};

// Shared database state for both Tauri commands and the HTTP server (see commands/network.rs)
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub sessions: Arc<Mutex<HashMap<String, (String, Instant)>>>,
    pub app: Option<tauri::AppHandle>,
}

pub struct Database(pub Arc<Mutex<Connection>>);

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Employee {
    pub nik: String,
    pub timestamp: Option<String>,
    pub join_date: Option<String>,
    pub tanggal_kartap: Option<String>,
    pub nama_lengkap: String,
    pub jenis_kelamin: Option<String>,
    pub tanggal_lahir: Option<String>,
    pub golongan_darah: Option<String>,
    pub nomor_kk: Option<String>,
    pub alamat_ktp: Option<String>,
    pub bpjs_kesehatan: Option<String>,
    pub bpjs_ketenagakerjaan: Option<String>,
    pub nomor_telepon: Option<String>,
    pub jabatan: Option<String>,
    pub divisi: Option<String>,
    pub nama_ibu_kandung: Option<String>,
    pub nama_pasangan: Option<String>,
    pub jumlah_anak: Option<String>,
    pub nomor_telp_keluarga: Option<String>,
    pub foto_ktp: Option<String>,
    pub foto_kk: Option<String>,
    pub foto_bpjs_kesehatan: Option<String>,
    pub foto_bpjs_ketenagakerjaan: Option<String>,
    pub pendidikan_terakhir: Option<String>,
    pub perjanjian_kerja: Option<String>,
    pub status: Option<String>,
    pub tempat_kerja: Option<String>,
    pub regu: Option<String>,
    pub catatan: Option<String>,
    pub is_flagged: Option<bool>,
}

pub mod models {
    pub mod attendance;
    pub mod leave;
    pub mod employee;
}

pub mod commands {
    pub mod attendance;
    pub mod excel_exporter;
    pub mod leave;
    pub mod leave_service;
    pub mod auth;
    pub mod system;
    pub mod employee;
    pub mod network;
}

#[derive(Deserialize)]
pub struct UpdateCellRequest {
    pub nik: String,
    pub periode_id: i64,
    pub tanggal: String,
    pub status: String,
    pub keterangan: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkAttendanceCell {
    pub nik: String,
    pub tanggal: String,
    pub status: String,
    pub keterangan: String,
}

fn initialize_schema(connection: &Connection) -> rusqlite::Result<()> {
    // Enable WAL Mode & Fast Synchronous Mode for high performance desktop SQLite
    connection.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;",
    )?;

    // 1. Create Base Master Karyawan Table
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS master_karyawan (
         nik TEXT PRIMARY KEY NOT NULL,
         timestamp TEXT,
         join_date TEXT,
         tanggal_kartap TEXT,
         nama_lengkap TEXT NOT NULL,
         jenis_kelamin TEXT,
         tanggal_lahir TEXT,
         golongan_darah TEXT,
         nomor_kk TEXT,
         alamat_ktp TEXT,
         bpjs_kesehatan TEXT,
         bpjs_ketenagakerjaan TEXT,
         nomor_telepon TEXT,
         jabatan TEXT,
         divisi TEXT,
         nama_ibu_kandung TEXT,
         nama_pasangan TEXT,
         jumlah_anak TEXT,
         nomor_telp_keluarga TEXT,
         foto_ktp TEXT,
         foto_kk TEXT,
         foto_bpjs_kesehatan TEXT,
         foto_bpjs_ketenagakerjaan TEXT,
         pendidikan_terakhir TEXT,
         perjanjian_kerja TEXT NOT NULL DEFAULT 'PKWTT',
         status TEXT NOT NULL DEFAULT 'AKTIF',
         tempat_kerja TEXT NOT NULL DEFAULT 'Lapangan',
         is_flagged INTEGER NOT NULL DEFAULT 0,
         regu TEXT,
         catatan TEXT
        );",
    )?;

    let has_regu: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('master_karyawan') WHERE name = 'regu')",
        [],
        |row| row.get(0),
    )?;
    if !has_regu {
        connection.execute("ALTER TABLE master_karyawan ADD COLUMN regu TEXT", [])?;
    }

    let has_catatan: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM pragma_table_info('master_karyawan') WHERE name = 'catatan')",
        [],
        |row| row.get(0),
    )?;
    if !has_catatan {
        connection.execute("ALTER TABLE master_karyawan ADD COLUMN catatan TEXT", [])?;
    }

    // 2. Create Saldo Cuti Table
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS saldo_cuti (
         nik TEXT NOT NULL,
         tahun INTEGER NOT NULL,
         jatah_awal INTEGER NOT NULL DEFAULT 12,
         cuti_bersama INTEGER NOT NULL DEFAULT 6,
         PRIMARY KEY (nik, tahun)
        );",
    )?;

    // 3. Create Libur Nasional Table (Public Holidays)
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS libur_nasional (
         tanggal TEXT PRIMARY KEY NOT NULL,
         keterangan TEXT NOT NULL
        );",
    )?;

    // 4. Create Inspection Tables
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS locations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            type TEXT NOT NULL,
            building_code TEXT
        );

        CREATE TABLE IF NOT EXISTS asset_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT UNIQUE NOT NULL,
            description TEXT
        );

        CREATE TABLE IF NOT EXISTS assets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id INTEGER NOT NULL,
            location_id INTEGER NOT NULL,
            code TEXT UNIQUE NOT NULL,
            name TEXT NOT NULL,
            condition_status TEXT DEFAULT 'good',
            status TEXT DEFAULT 'operational',
            is_active INTEGER NOT NULL DEFAULT 1,
            qr_code_base64 TEXT
        );

        CREATE TABLE IF NOT EXISTS inspections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            google_response_id TEXT UNIQUE,
            asset_id INTEGER NOT NULL,
            inspector_badge_id TEXT NOT NULL,
            inspected_at DATETIME NOT NULL,
            inspection_status TEXT DEFAULT 'NORMAL',
            notes TEXT,
            review_status TEXT DEFAULT 'pending_review',
            photo_url TEXT
        );

        CREATE TABLE IF NOT EXISTS inspection_item_checks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            inspection_id INTEGER NOT NULL,
            component_name TEXT NOT NULL,
            condition_code TEXT NOT NULL,
            remark TEXT
        );

        CREATE TABLE IF NOT EXISTS problems (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            problem_code TEXT UNIQUE NOT NULL,
            inspection_id INTEGER,
            location_id INTEGER NOT NULL,
            category TEXT DEFAULT 'general',
            severity TEXT DEFAULT 'medium',
            description TEXT NOT NULL,
            status TEXT DEFAULT 'open',
            reported_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );",
    )?;

    // 5. Create Division Accounts Table.
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS division_accounts (
            username TEXT PRIMARY KEY NOT NULL,
            password_hash TEXT NOT NULL,
            role_division TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            setting_key TEXT PRIMARY KEY NOT NULL,
            setting_value TEXT NOT NULL
        );

        INSERT OR IGNORE INTO app_settings (setting_key, setting_value)
        VALUES ('password_setup_complete', '0');",
    )?;

    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS security_pin_cache (
            name TEXT PRIMARY KEY NOT NULL,
            pin_hash TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
    )?;

    // 6. Performance Optimization Indexes
    connection.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_karyawan_search
         ON master_karyawan(nama_lengkap COLLATE NOCASE, status, divisi);

         CREATE INDEX IF NOT EXISTS idx_saldo_cuti_nik_tahun
         ON saldo_cuti(nik, tahun);",
    )?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .setup(|app: &mut tauri::App| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;

            let connection = Connection::open(data_dir.join("oasis-karyawan.sqlite"))?;

            initialize_schema(&connection)?;
            initialize_attendance_schema(&connection)?;

            let db_arc = Arc::new(Mutex::new(connection));

            // Register the database for Tauri commands.
            app.manage(Database(db_arc));

            Ok(())
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .invoke_handler(tauri::generate_handler![
            authenticate,
            get_security_auth_accounts,
            get_password_setup_status,
            get_or_initialize_node_mode,
            start_master_server,
            discover_master,
            setup_division_passwords,
            reset_division_account,
            cache_security_pin,
            get_security_pin_cache_status,
            list_employees,
            upsert_employees,
            save_employee,
            delete_employee,
            // Attendance handlers
            get_or_create_periode,
            update_absensi_cell,
            list_staff_employees_for_absensi,
            list_engineering_employees_for_absensi,
            list_housekeeping_employees_for_absensi,
            list_security_employees_for_absensi,
            toggle_employee_active_status,
            export_rekap_absen_excel_1to1,
            reset_database,
            // Leave handlers
            get_rekap_cuti_tahun,
            update_saldo_cuti_karyawan,
            // Excel and system handlers
            export_excel_file,
            open_folder_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}