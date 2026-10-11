use crate::models::leave::LeaveSummary;
use crate::commands::leave_service;
use crate::Database;
use tauri::State;

#[tauri::command]
pub fn get_rekap_cuti_tahun(
    database: State<'_, Database>,
    tahun: i32,
) -> Result<Vec<LeaveSummary>, String> {
    let conn = database.0.lock().map_err(|e| e.to_string())?;
    leave_service::fetch_rekap_cuti(&conn, tahun)
}

#[tauri::command]
pub fn update_saldo_cuti_karyawan(
    database: State<'_, Database>,
    nik: String,
    tahun: i32,
    jatah_awal: i32,
    cuti_bersama: i32,
) -> Result<(), String> {
    let conn = database.0.lock().map_err(|e| e.to_string())?;
    leave_service::save_saldo_cuti(&conn, &nik, tahun, jatah_awal, cuti_bersama)
}