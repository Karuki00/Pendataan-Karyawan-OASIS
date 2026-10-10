use crate::Database;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaveSummary {
    pub nik: String,
    pub nama_lengkap: String,
    pub jabatan: String,
    pub join_date: Option<String>,
    pub jatah_awal: i32,
    pub cuti_bersama: i32,
    pub monthly_a: Vec<i32>, // Jan-Dec Column A (Quota Leave)
    pub monthly_b: Vec<i32>, // Jan-Dec Column B (Potongan Gaji)
    pub total_cuti_terpakai: i32,
    pub total_potongan_gaji: i32,
    pub sisa_cuti: i32,
}

#[tauri::command]
pub fn get_rekap_cuti_tahun(
    database: State<'_, Database>,
    tahun: i32,
) -> Result<Vec<LeaveSummary>, String> {
    let conn = database.0.lock().map_err(|e| e.to_string())?;

    // 1. Fetch Active Staff Employees
    let mut stmt = conn
        .prepare(
            "SELECT nik, nama_lengkap, jabatan, join_date 
             FROM master_karyawan 
             WHERE status = 'AKTIF' AND (divisi = 'STAFF' OR tempat_kerja = 'Pengelola')
             ORDER BY nama_lengkap COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let employees: Vec<(String, String, String, Option<String>)> = stmt
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut result: Vec<LeaveSummary> = Vec::new();

    for (nik, nama, jabatan, join_date) in employees {
        // 2. Fetch or Init Saldo Cuti for Year
        let (jatah_awal, cuti_bersama): (i32, i32) = conn
            .query_row(
                "SELECT jatah_awal, cuti_bersama FROM saldo_cuti WHERE nik = ?1 AND tahun = ?2",
                params![nik, tahun],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((12, 6)); // Default 12 days quota, 6 days Cuti Bersama

        let mut monthly_a = vec![0; 12];
        let mut monthly_b = vec![0; 12];

        // 3. Aggregate Monthly Absences for CUTI and Potongan Gaji
        for month in 1..=12 {
            let month_str = format!("{:02}", month);
            let year_month = format!("{}-{}", tahun, month_str);

            // Count CUTI status (Column A)
            let cuti_count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM absensi 
                     WHERE nik = ?1 AND status = 'CUTI' AND strftime('%Y-%m', tanggal) = ?2",
                    params![nik, year_month],
                    |r| r.get(0),
                )
                .unwrap_or(0);

            // Count Potongan Gaji statuses (Column B) e.g. ALPA, excess IZIN
            let pot_gaji_count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM absensi 
                     WHERE nik = ?1 AND status IN ('ALPA', 'POTONG_GAJI') AND strftime('%Y-%m', tanggal) = ?2",
                    params![nik, year_month],
                    |r| r.get(0),
                )
                .unwrap_or(0);

            monthly_a[(month - 1) as usize] = cuti_count;
            monthly_b[(month - 1) as usize] = pot_gaji_count;
        }

        let total_cuti_terpakai: i32 = monthly_a.iter().sum();
        let total_potongan_gaji: i32 = monthly_b.iter().sum();
        let sisa_cuti = jatah_awal - total_cuti_terpakai - cuti_bersama;

        result.push(LeaveSummary {
            nik,
            nama_lengkap: nama,
            jabatan,
            join_date,
            jatah_awal,
            cuti_bersama,
            monthly_a,
            monthly_b,
            total_cuti_terpakai,
            total_potongan_gaji,
            sisa_cuti,
        });
    }

    Ok(result)
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

    conn.execute(
        "INSERT INTO saldo_cuti (nik, tahun, jatah_awal, cuti_bersama)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(nik, tahun) DO UPDATE SET 
            jatah_awal = excluded.jatah_awal,
            cuti_bersama = excluded.cuti_bersama",
        params![nik, tahun, jatah_awal, cuti_bersama],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}