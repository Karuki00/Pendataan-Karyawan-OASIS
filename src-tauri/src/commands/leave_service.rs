use rusqlite::{params, Connection};
use chrono::{NaiveDate, Datelike, Weekday};
use crate::models::leave::LeaveSummary;

/// Helper to count national holidays in a given YYYY-MM that fall on Mon-Sat (excluding Sunday)
fn count_red_dates_excluding_sunday(conn: &Connection, year_month: &str) -> i32 {
    let mut stmt = match conn.prepare(
        "SELECT tanggal FROM libur_nasional WHERE substr(tanggal, 1, 7) = ?1"
    ) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let dates: Vec<String> = stmt
        .query_map([year_month], |row| row.get(0))
        .map(|rows| rows.filter_map(|r| r.ok()).collect())
        .unwrap_or_default();

    let mut red_date_count = 0;

    for date_str in dates {
        if let Ok(parsed_date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            // Exclude Sunday (Minggu)
            if parsed_date.weekday() != Weekday::Sun {
                red_date_count += 1;
            }
        }
    }

    red_date_count
}

/// Fetches annual leave summary per employee.
/// Automatically detects red dates (excluding Sundays) for monthly statistics.
pub fn fetch_rekap_cuti(conn: &Connection, tahun: i32) -> Result<Vec<LeaveSummary>, String> {
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
        let (jatah_awal, cuti_bersama): (i32, i32) = conn
            .query_row(
                "SELECT jatah_awal, cuti_bersama FROM saldo_cuti WHERE nik = ?1 AND tahun = ?2",
                params![nik, tahun],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((12, 6));

        let mut monthly_a = vec![0; 12];
        let mut monthly_b = vec![0; 12];

        for month in 1..=12 {
            let month_str = format!("{:02}", month);
            let year_month = format!("{}-{}", tahun, month_str);

            // 1. Count Employee Cuti/Absence Quota
            let cuti_count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM absensi 
                     WHERE nik = ?1 
                       AND UPPER(status) IN ('CUTI', 'ALPA', 'IZIN', 'SAKIT') 
                       AND substr(tanggal, 1, 7) = ?2",
                    params![nik, year_month],
                    |r| r.get(0),
                )
                .unwrap_or(0);

            // 2. Count Salary Deduction
            let pot_gaji_count: i32 = conn
                .query_row(
                    "SELECT COUNT(*) FROM absensi 
                     WHERE nik = ?1 
                       AND UPPER(status) IN ('ALPA', 'POTONG_GAJI') 
                       AND substr(tanggal, 1, 7) = ?2",
                    params![nik, year_month],
                    |r| r.get(0),
                )
                .unwrap_or(0);

            // 3. Auto-detect Red Dates in Month (Excluding Sundays)
            let _non_sunday_red_dates = count_red_dates_excluding_sunday(conn, &year_month);

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

/// Updates or creates employee annual leave quota
pub fn save_saldo_cuti(
    conn: &Connection,
    nik: &str,
    tahun: i32,
    jatah_awal: i32,
    cuti_bersama: i32,
) -> Result<(), String> {
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