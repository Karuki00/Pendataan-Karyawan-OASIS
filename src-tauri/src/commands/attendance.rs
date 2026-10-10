use crate::models::attendance::PeriodeAbsensi;
use crate::Database;
use chrono::{NaiveDate};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct AbsensiCellData {
    pub status: String,
    pub keterangan: String,
}

pub fn initialize_attendance_schema(conn: &Connection) -> Result<()> {
    // 1. Create tables if they do not exist
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS periode_absensi (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nama_periode TEXT NOT NULL,
            tanggal_mulai TEXT NOT NULL,
            tanggal_selesai TEXT NOT NULL,
            total_hari INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS absensi (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nik TEXT NOT NULL,
            periode_id INTEGER NOT NULL,
            tanggal TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'ON',
            FOREIGN KEY(nik) REFERENCES master_karyawan(nik) ON DELETE CASCADE,
            FOREIGN KEY(periode_id) REFERENCES periode_absensi(id) ON DELETE CASCADE,
            UNIQUE(nik, tanggal)
        );

        CREATE TABLE IF NOT EXISTS saldo_cuti (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            nik TEXT NOT NULL,
            tahun INTEGER NOT NULL,
            jatah_awal INTEGER NOT NULL DEFAULT 12,
            cuti_bersama INTEGER NOT NULL DEFAULT 6,
            cuti_terpakai INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(nik) REFERENCES master_karyawan(nik) ON DELETE CASCADE,
            UNIQUE(nik, tahun)
        );",
    )?;

    // 2. Safe Column Migration
    let mut stmt = conn.prepare("PRAGMA table_info(absensi)")?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;

    let mut has_keterangan = false;
    for col in columns {
        if col? == "keterangan" {
            has_keterangan = true;
            break;
        }
    }

    if !has_keterangan {
        conn.execute(
            "ALTER TABLE absensi ADD COLUMN keterangan TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_or_create_periode(
    database: State<'_, Database>,
    year: i32,
    month: u32,
) -> Result<PeriodeAbsensi, String> {
    let conn = database.0.lock().map_err(|e| e.to_string())?;

    let start_1 = NaiveDate::from_ymd_opt(year, month, 1).ok_or("Tanggal tidak valid")?;

    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let end_date = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .ok_or("Tanggal tidak valid")?
        .pred_opt()
        .ok_or("Tanggal tidak valid")?;

    let start_str = start_1.format("%Y-%m-%d").to_string();
    let end_str = end_date.format("%Y-%m-%d").to_string();
    let total_days = (end_date - start_1).num_days() as i32 + 1;
    let nama_periode = format!(
        "Periode {} s/d {}",
        start_1.format("%d %b %Y"),
        end_date.format("%d %b %Y")
    );

    let mut stmt = conn
        .prepare("SELECT id, nama_periode, tanggal_mulai, tanggal_selesai, total_hari FROM periode_absensi WHERE tanggal_mulai = ?1 AND tanggal_selesai = ?2")
        .map_err(|e| e.to_string())?;

    let existing = stmt.query_row([&start_str, &end_str], |row| {
        Ok(PeriodeAbsensi {
            id: Some(row.get(0)?),
            nama_periode: row.get(1)?,
            tanggal_mulai: row.get(2)?,
            tanggal_selesai: row.get(3)?,
            total_hari: row.get(4)?,
        })
    });

    if let Ok(p) = existing {
        return Ok(p);
    }

    conn.execute(
        "INSERT INTO periode_absensi (nama_periode, tanggal_mulai, tanggal_selesai, total_hari) VALUES (?1, ?2, ?3, ?4)",
        params![nama_periode, start_str, end_str, total_days],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    Ok(PeriodeAbsensi {
        id: Some(id),
        nama_periode,
        tanggal_mulai: start_str,
        tanggal_selesai: end_str,
        total_hari: total_days,
    })
}

#[tauri::command]
pub fn update_absensi_cell(
    db: State<'_, Database>,
    nik: String,
    periode_id: i64,
    tanggal: String,
    status: String,
    keterangan: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO absensi (nik, periode_id, tanggal, status, keterangan)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(nik, tanggal) DO UPDATE SET 
            status = excluded.status,
            keterangan = excluded.keterangan",
        params![nik, periode_id, tanggal, status, keterangan.unwrap_or_default()],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_absensi_cell(
    database: State<'_, Database>,
    nik: String,
    tanggal: String,
) -> Result<Option<AbsensiCellData>, String> {
    let conn = database.0.lock().map_err(|e| e.to_string())?;

    let cell = conn
        .query_row(
            "SELECT status, COALESCE(keterangan, '') FROM absensi WHERE nik = ?1 AND tanggal = ?2",
            params![nik, tanggal],
            |r| {
                Ok(AbsensiCellData {
                    status: r.get(0)?,
                    keterangan: r.get(1)?,
                })
            },
        )
        .ok();

    Ok(cell)
}

#[tauri::command]
pub fn reset_database(database: State<'_, Database>) -> Result<(), String> {
    let mut conn = database.0.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    tx.execute_batch(
        "PRAGMA foreign_keys = OFF;
        DELETE FROM absensi;
        DELETE FROM saldo_cuti;
        DELETE FROM periode_absensi;
        DELETE FROM master_karyawan;
        DELETE FROM sqlite_sequence WHERE name IN ('absensi', 'saldo_cuti', 'periode_absensi');
        PRAGMA foreign_keys = ON;",
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_staff_employees_for_absensi(
    database: State<'_, Database>,
) -> Result<Vec<crate::Employee>, String> {
    let conn = database.0.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT nik, timestamp, join_date, tanggal_kartap, nama_lengkap, jenis_kelamin, 
                    tanggal_lahir, golongan_darah, nomor_kk, alamat_ktp, bpjs_kesehatan, 
                    bpjs_ketenagakerjaan, nomor_telepon, jabatan, divisi, nama_ibu_kandung, 
                    nama_pasangan, jumlah_anak, nomor_telp_keluarga, foto_ktp, foto_kk, 
                    foto_bpjs_kesehatan, foto_bpjs_ketenagakerjaan, pendidikan_terakhir, 
                    perjanjian_kerja, status, tempat_kerja, is_flagged 
             FROM master_karyawan 
             WHERE status = 'AKTIF' AND (divisi = 'STAFF' OR tempat_kerja = 'Pengelola')
             ORDER BY nama_lengkap COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            let flagged_int: i32 = row.get(27)?;
            Ok(crate::Employee {
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
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_employee_active_status(
    database: State<'_, Database>,
    nik: String,
    new_status: String,
) -> Result<(), String> {
    let conn = database.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE master_karyawan SET status = ?1 WHERE nik = ?2",
        [new_status, nik],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}