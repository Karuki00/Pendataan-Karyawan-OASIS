use crate::Database;
use chrono::{Datelike, NaiveDate};
use rusqlite::{Connection, OptionalExtension};
use rust_xlsxwriter::*;
use tauri::State;

/// Converts 0-indexed column number to Excel column name (0 -> "A", 4 -> "E", 27 -> "AB")
pub fn col_number_to_name(col: u16) -> String {
    let mut col = col;
    let mut name = String::new();
    loop {
        let remainder = (col % 26) as u8;
        name.insert(0, (b'A' + remainder) as char);
        if col < 26 {
            break;
        }
        col = (col / 26) - 1;
    }
    name
}

struct EmployeeRef {
    nik: String,
    nama_lengkap: String,
    jabatan: String,
    join_date: Option<String>,
}

fn fetch_active_staff(conn: &Connection) -> Result<Vec<EmployeeRef>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT nik, nama_lengkap, jabatan, join_date 
             FROM master_karyawan 
             WHERE status = 'AKTIF' AND (divisi = 'STAFF' OR tempat_kerja = 'Pengelola')
             ORDER BY nama_lengkap COLLATE NOCASE",
        )
        .map_err(|e| e.to_string())?;

    let employees = stmt
        .query_map([], |row| {
            Ok(EmployeeRef {
                nik: row.get(0)?,
                nama_lengkap: row.get(1)?,
                jabatan: row.get(2)?,
                join_date: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(employees)
}

/// Helper to write Sheet 1: Rekap Absen
fn build_sheet_rekap_absen(
    workbook: &mut Workbook,
    conn: &Connection,
    employees: &[EmployeeRef],
    year: i32,
    month: u32,
) -> Result<(), String> {
    let worksheet = workbook
        .add_worksheet()
        .set_name("Rekap Absen")
        .map_err(|e| e.to_string())?;

    // Formats
    let title_format = Format::new().set_bold().set_font_size(14).set_font_name("Calibri");
    let subtitle_format = Format::new().set_bold().set_font_size(12).set_font_name("Calibri");
    let header_format = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color(Color::RGB(0xE2EFDA))
        .set_border(FormatBorder::Thin);

    let cell_center = Format::new().set_align(FormatAlign::Center).set_border(FormatBorder::Thin);
    let cell_left = Format::new().set_align(FormatAlign::Left).set_border(FormatBorder::Thin);
    let cell_bold_center = Format::new().set_bold().set_align(FormatAlign::Center).set_border(FormatBorder::Thin);

    // Header Titles
    worksheet.write_with_format(0, 2, "REKAPITULASI ABSENSI STAFF PENGELOLA", &title_format).ok();
    worksheet.write_with_format(1, 2, "APARTEMEN MITRA OASIS SARANA", &subtitle_format).ok();

    // Table Headers
    worksheet.write_with_format(3, 1, "NO", &header_format).ok();
    worksheet.write_with_format(3, 2, "NAMA ANGGOTA", &header_format).ok();
    worksheet.write_with_format(3, 3, "JABATAN", &header_format).ok();

    let start_date = NaiveDate::from_ymd_opt(year, month, 1).ok_or("Tanggal tidak valid")?;
    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };
    let end_date = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .unwrap()
        .pred_opt()
        .unwrap();

    let start_str = start_date.format("%Y-%m-%d").to_string();
    let end_str = end_date.format("%Y-%m-%d").to_string();

    let periode_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM periode_absensi WHERE tanggal_mulai = ?1 AND tanggal_selesai = ?2",
            rusqlite::params![start_str, end_str],
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    let days_in_month = (end_date - start_date).num_days() as u16 + 1;

    // Date Columns
    let mut current = start_date;
    for day_idx in 0..days_in_month {
        let col = 4 + day_idx;
        worksheet.write_with_format(3, col, current.day() as u16, &header_format).ok();

        let day_name = match current.weekday() {
            chrono::Weekday::Mon => "SEN",
            chrono::Weekday::Tue => "SEL",
            chrono::Weekday::Wed => "RAB",
            chrono::Weekday::Thu => "KAM",
            chrono::Weekday::Fri => "JUM",
            chrono::Weekday::Sat => "SAB",
            chrono::Weekday::Sun => "MIN",
        };
        worksheet.write_with_format(4, col, day_name, &header_format).ok();
        current = current + chrono::Duration::days(1);
    }

    // Summary Columns
    let summary_col_start = 4 + days_in_month;
    worksheet.write_with_format(3, summary_col_start, "Jmh Hari Kerja", &header_format).ok();
    worksheet.write_with_format(3, summary_col_start + 1, "OFF/LIBUR", &header_format).ok();
    worksheet.write_with_format(3, summary_col_start + 2, "ALPA", &header_format).ok();
    worksheet.write_with_format(3, summary_col_start + 3, "IZIN", &header_format).ok();
    worksheet.write_with_format(3, summary_col_start + 4, "SAKIT", &header_format).ok();
    worksheet.write_with_format(3, summary_col_start + 5, "CUTI", &header_format).ok();
    worksheet.write_with_format(3, summary_col_start + 6, "TOTAL HARI", &header_format).ok();

    // Rows
    let mut row_idx = 5u32;
    for (no, emp) in employees.iter().enumerate() {
        worksheet.write_with_format(row_idx, 1, (no + 1) as u32, &cell_center).ok();
        worksheet.write_with_format(row_idx, 2, &emp.nama_lengkap, &cell_left).ok();
        worksheet.write_with_format(row_idx, 3, &emp.jabatan, &cell_left).ok();

        let mut date_curr = start_date;
        for day_idx in 0..days_in_month {
            let col = 4 + day_idx;
            let date_str = date_curr.format("%Y-%m-%d").to_string();

            let (status, keterangan): (String, String) = if let Some(p_id) = periode_id {
                conn.query_row(
                    "SELECT status, COALESCE(keterangan, '') FROM absensi WHERE nik = ?1 AND periode_id = ?2 AND tanggal = ?3",
                    rusqlite::params![emp.nik, p_id, date_str],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap_or_else(|_| ("".to_string(), "".to_string()))
            } else {
                ("".to_string(), "".to_string())
            };

            worksheet.write_with_format(row_idx, col, &status, &cell_center).ok();

            if !keterangan.trim().is_empty() {
                let note = Note::new(&keterangan);
                worksheet.insert_note(row_idx, col, &note).ok();
            }

            date_curr = date_curr + chrono::Duration::days(1);
        }

        let first_col_name = col_number_to_name(4);
        let last_col_name = col_number_to_name(3 + days_in_month);
        let excel_row = row_idx + 1;
        let range_str = format!("{}{}:{}{}", first_col_name, excel_row, last_col_name, excel_row);

        worksheet.write_formula_with_format(row_idx, summary_col_start, format!("=COUNTIF({}, \"ON\")", range_str).as_str(), &cell_bold_center).ok();
        worksheet.write_formula_with_format(row_idx, summary_col_start + 1, format!("=COUNTIF({}, \"OFF\")", range_str).as_str(), &cell_bold_center).ok();
        worksheet.write_formula_with_format(row_idx, summary_col_start + 2, format!("=COUNTIF({}, \"ALPA\")", range_str).as_str(), &cell_bold_center).ok();
        worksheet.write_formula_with_format(row_idx, summary_col_start + 3, format!("=COUNTIF({}, \"IZIN\")", range_str).as_str(), &cell_bold_center).ok();
        worksheet.write_formula_with_format(row_idx, summary_col_start + 4, format!("=COUNTIF({}, \"SAKIT*\")", range_str).as_str(), &cell_bold_center).ok();
        worksheet.write_formula_with_format(row_idx, summary_col_start + 5, format!("=COUNTIF({}, \"CUTI\")", range_str).as_str(), &cell_bold_center).ok();
        worksheet.write_formula_with_format(row_idx, summary_col_start + 6, format!("=COUNTA({})", range_str).as_str(), &cell_bold_center).ok();

        row_idx += 1;
    }

    Ok(())
}

/// Helper to write Sheet 2: Rekap Cuti
fn build_sheet_rekap_cuti(
    workbook: &mut Workbook,
    conn: &Connection,
    employees: &[EmployeeRef],
    year: i32,
) -> Result<(), String> {
    let worksheet = workbook
        .add_worksheet()
        .set_name("Rekap Cuti")
        .map_err(|e| e.to_string())?;

    let title_format = Format::new().set_bold().set_font_size(14).set_font_name("Calibri");
    let subtitle_format = Format::new().set_bold().set_font_size(12).set_font_name("Calibri");
    let header_format = Format::new()
        .set_bold()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter)
        .set_background_color(Color::RGB(0xE2EFDA))
        .set_border(FormatBorder::Thin);

    let cell_center = Format::new().set_align(FormatAlign::Center).set_border(FormatBorder::Thin);
    let cell_left = Format::new().set_align(FormatAlign::Left).set_border(FormatBorder::Thin);
    let cell_bold_center = Format::new().set_bold().set_align(FormatAlign::Center).set_border(FormatBorder::Thin);
    let cell_purple = Format::new().set_bold().set_align(FormatAlign::Center).set_border(FormatBorder::Thin).set_background_color(Color::RGB(0xF3E8FF));
    let cell_amber = Format::new().set_bold().set_align(FormatAlign::Center).set_border(FormatBorder::Thin).set_background_color(Color::RGB(0xFEF3C7));

    worksheet.write_with_format(0, 2, "REKAPITULASI CUTI STAFF PENGELOLA", &title_format).ok();
    worksheet.write_with_format(1, 2, format!("Periode Tahun: {}", year).as_str(), &subtitle_format).ok();

    worksheet.write_with_format(3, 1, "NO", &header_format).ok();
    worksheet.write_with_format(3, 2, "NAMA", &header_format).ok();
    worksheet.write_with_format(3, 3, "JABATAN", &header_format).ok();
    worksheet.write_with_format(3, 4, "JOIN DATE", &header_format).ok();
    worksheet.write_with_format(3, 5, format!("JATAH CUTI {}", year).as_str(), &header_format).ok();

    let month_names = ["JAN", "FEB", "MAR", "APR", "MEI", "JUN", "JUL", "AGS", "SEP", "OKT", "NOV", "DES"];
    for (m_idx, m_name) in month_names.iter().enumerate() {
        let col_a = 6 + (m_idx as u16 * 2);
        let col_b = col_a + 1;

        worksheet.write_with_format(3, col_a, *m_name, &header_format).ok();
        worksheet.write_with_format(4, col_a, "A", &header_format).ok();
        worksheet.write_with_format(4, col_b, "B", &header_format).ok();
    }

    let summary_col_cuti = 6 + 24;
    worksheet.write_with_format(3, summary_col_cuti, "CUTI TERPAKAI", &header_format).ok();
    worksheet.write_with_format(3, summary_col_cuti + 1, "POT. GAJI", &header_format).ok();
    worksheet.write_with_format(3, summary_col_cuti + 2, format!("SISA CUTI {}", year).as_str(), &header_format).ok();

    let mut row_idx = 5u32;
    for (no, emp) in employees.iter().enumerate() {
        worksheet.write_with_format(row_idx, 1, (no + 1) as u32, &cell_center).ok();
        worksheet.write_with_format(row_idx, 2, &emp.nama_lengkap, &cell_left).ok();
        worksheet.write_with_format(row_idx, 3, &emp.jabatan, &cell_left).ok();
        worksheet.write_with_format(row_idx, 4, emp.join_date.as_deref().unwrap_or("-"), &cell_center).ok();

        let (jatah_awal, cuti_bersama): (i32, i32) = conn
            .query_row(
                "SELECT jatah_awal, cuti_bersama FROM saldo_cuti WHERE nik = ?1 AND tahun = ?2",
                rusqlite::params![emp.nik, year],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((12, 6));

        worksheet.write_with_format(row_idx, 5, jatah_awal, &cell_bold_center).ok();

        for m in 1..=12 {
            let m_str = format!("{:02}", m);
            let ym = format!("{}-{}", year, m_str);

            let count_a: i32 = conn
                .query_row("SELECT COUNT(*) FROM absensi WHERE nik = ?1 AND status = 'CUTI' AND strftime('%Y-%m', tanggal) = ?2", rusqlite::params![emp.nik, ym], |r| r.get(0))
                .unwrap_or(0);

            let count_b: i32 = conn
                .query_row("SELECT COUNT(*) FROM absensi WHERE nik = ?1 AND status IN ('ALPA', 'POTONG_GAJI') AND strftime('%Y-%m', tanggal) = ?2", rusqlite::params![emp.nik, ym], |r| r.get(0))
                .unwrap_or(0);

            let col_a = 6 + ((m as u16 - 1) * 2);
            let col_b = col_a + 1;

            if count_a > 0 { worksheet.write_with_format(row_idx, col_a, count_a, &cell_purple).ok(); } 
            else { worksheet.write_with_format(row_idx, col_a, "-", &cell_center).ok(); }

            if count_b > 0 { worksheet.write_with_format(row_idx, col_b, count_b, &cell_amber).ok(); } 
            else { worksheet.write_with_format(row_idx, col_b, "-", &cell_center).ok(); }
        }

        let e_row = row_idx + 1;
        let formula_a = format!("=SUMIF($G$5:$AD$5, \"A\", G{}:AD{})", e_row, e_row);
        let formula_b = format!("=SUMIF($G$5:$AD$5, \"B\", G{}:AD{})", e_row, e_row);
        let formula_sisa = format!("=F{} - AE{} - {}", e_row, e_row, cuti_bersama);

        worksheet.write_formula_with_format(row_idx, summary_col_cuti, formula_a.as_str(), &cell_purple).ok();
        worksheet.write_formula_with_format(row_idx, summary_col_cuti + 1, formula_b.as_str(), &cell_amber).ok();
        worksheet.write_formula_with_format(row_idx, summary_col_cuti + 2, formula_sisa.as_str(), &cell_bold_center).ok();

        row_idx += 1;
    }

    Ok(())
}

/// Unified Export Command generating 2 Sheets inside one Excel Workbook
#[tauri::command]
pub fn export_rekap_absen_excel_1to1(
    database: State<'_, Database>,
    file_path: String,
    year: i32,
    month: u32,
) -> Result<(), String> {
    let conn = database.0.lock().map_err(|e| e.to_string())?;
    let employees = fetch_active_staff(&conn)?;

    let mut workbook = Workbook::new();

    // Sheet 1: Rekap Absen
    build_sheet_rekap_absen(&mut workbook, &conn, &employees, year, month)?;

    // Sheet 2: Rekap Cuti
    build_sheet_rekap_cuti(&mut workbook, &conn, &employees, year)?;

    workbook.save(file_path).map_err(|e| e.to_string())?;
    Ok(())
}