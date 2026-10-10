use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use std::sync::Mutex;
use tauri::{Manager, State};
use commands::attendance::{get_or_create_periode, reset_database, list_staff_employees_for_absensi, toggle_employee_active_status, initialize_attendance_schema, update_absensi_cell};
use commands::excel_exporter::{export_rekap_absen_excel_1to1};
use commands::leave::{get_rekap_cuti_tahun, update_saldo_cuti_karyawan};
pub struct Database(pub Mutex<Connection>);

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
    pub is_flagged: Option<bool>,
}

pub mod models {
    pub mod attendance;
}
pub mod commands {
    pub mod attendance;
    pub mod excel_exporter;
    pub mod leave;
}

fn compute_is_flagged(emp: &Employee) -> bool {
    let nik = emp.nik.trim();
    let kk = emp.nomor_kk.as_deref().unwrap_or("").trim();
    let bpjs_kes = emp.bpjs_kesehatan.as_deref().unwrap_or("").trim();
    let bpjs_tk = emp.bpjs_ketenagakerjaan.as_deref().unwrap_or("").trim();

    // Condition 1: NIK bukan 16 digit angka
    let invalid_nik = nik.len() != 16 || !nik.chars().all(|c| c.is_ascii_digit());

    // Condition 2: Nomor KK diisi tetapi bukan 16 digit angka
    let invalid_kk = !kk.is_empty() && (kk.len() != 16 || !kk.chars().all(|c| c.is_ascii_digit()));

    // Condition 3: BPJS belum lengkap
    let missing_bpjs = bpjs_kes.is_empty() || bpjs_tk.is_empty();

    invalid_nik || invalid_kk || missing_bpjs
}

fn initialize_schema(connection: &Connection) -> rusqlite::Result<()> {
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
          is_flagged INTEGER NOT NULL DEFAULT 0
        );",
    )?;

    for (column, definition) in [
        ("join_date", "TEXT"),
        ("tanggal_kartap", "TEXT"),
        ("is_flagged", "INTEGER NOT NULL DEFAULT 0"),
    ] {
        let exists: bool = connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM pragma_table_info('master_karyawan') WHERE name = ?1)",
            [column],
            |row| row.get(0),
        )?;
        if !exists {
            connection.execute(
                &format!("ALTER TABLE master_karyawan ADD COLUMN {column} {definition}"),
                [],
            )?;
        }
    }
    Ok(())
}

#[tauri::command]
fn open_folder_dir(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let win_path = path.replace("/", "\\");
        Command::new("explorer.exe")
            .arg(&win_path)
            .spawn()
            .map_err(|e| format!("Gagal membuka Explorer: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn list_employees(
    database: State<'_, Database>,
    search: Option<String>,
) -> Result<Vec<Employee>, String> {
    let connection = database.0.lock().map_err(|error| error.to_string())?;
    let pattern = format!("%{}%", search.unwrap_or_default().trim());
    let mut statement = connection
        .prepare(
            "SELECT nik, timestamp, join_date, tanggal_kartap, nama_lengkap, jenis_kelamin, tanggal_lahir,
                    golongan_darah, nomor_kk, alamat_ktp, bpjs_kesehatan,
                    bpjs_ketenagakerjaan, nomor_telepon, jabatan, divisi,
                    nama_ibu_kandung, nama_pasangan, jumlah_anak,
                    nomor_telp_keluarga, foto_ktp, foto_kk, foto_bpjs_kesehatan,
                    foto_bpjs_ketenagakerjaan, pendidikan_terakhir,
                    perjanjian_kerja, status, tempat_kerja, is_flagged
             FROM master_karyawan
             WHERE ?1 = '' OR nama_lengkap LIKE ?1 COLLATE NOCASE
                OR nik LIKE ?1 OR divisi LIKE ?1
             ORDER BY nama_lengkap COLLATE NOCASE",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([pattern], |row| {
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
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn upsert(connection: &Connection, employee: &Employee) -> rusqlite::Result<()> {
    let is_flagged_val = if compute_is_flagged(employee) { 1 } else { 0 };

    connection.execute(
        "INSERT INTO master_karyawan (
          nik, timestamp, join_date, tanggal_kartap, nama_lengkap, jenis_kelamin, tanggal_lahir, golongan_darah,
          nomor_kk, alamat_ktp, bpjs_kesehatan, bpjs_ketenagakerjaan, nomor_telepon,
          jabatan, divisi, nama_ibu_kandung, nama_pasangan, jumlah_anak,
          nomor_telp_keluarga, foto_ktp, foto_kk, foto_bpjs_kesehatan,
          foto_bpjs_ketenagakerjaan, pendidikan_terakhir, perjanjian_kerja, status, tempat_kerja, is_flagged
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, COALESCE(?25, 'PKWTT'), COALESCE(?26, 'AKTIF'), COALESCE(?27, 'Lapangan'), ?28)
        ON CONFLICT(nik) DO UPDATE SET
          timestamp=excluded.timestamp, join_date=excluded.join_date, tanggal_kartap=excluded.tanggal_kartap,
          nama_lengkap=excluded.nama_lengkap, jenis_kelamin=excluded.jenis_kelamin,
          tanggal_lahir=excluded.tanggal_lahir, golongan_darah=excluded.golongan_darah, nomor_kk=excluded.nomor_kk,
          alamat_ktp=excluded.alamat_ktp, bpjs_kesehatan=excluded.bpjs_kesehatan,
          bpjs_ketenagakerjaan=excluded.bpjs_ketenagakerjaan, nomor_telepon=excluded.nomor_telepon,
          jabatan=excluded.jabatan, divisi=excluded.divisi, nama_ibu_kandung=excluded.nama_ibu_kandung,
          nama_pasangan=excluded.nama_pasangan, jumlah_anak=excluded.jumlah_anak,
          nomor_telp_keluarga=excluded.nomor_telp_keluarga, foto_ktp=excluded.foto_ktp, foto_kk=excluded.foto_kk,
          foto_bpjs_kesehatan=excluded.foto_bpjs_kesehatan,
          foto_bpjs_ketenagakerjaan=excluded.foto_bpjs_ketenagakerjaan,
          pendidikan_terakhir=excluded.pendidikan_terakhir,
          perjanjian_kerja=excluded.perjanjian_kerja, status=excluded.status,
          tempat_kerja=excluded.tempat_kerja, is_flagged=excluded.is_flagged",
        params![
            employee.nik, employee.timestamp, employee.join_date, employee.tanggal_kartap,
            employee.nama_lengkap, employee.jenis_kelamin, employee.tanggal_lahir, employee.golongan_darah, employee.nomor_kk, employee.alamat_ktp,
            employee.bpjs_kesehatan, employee.bpjs_ketenagakerjaan, employee.nomor_telepon,
            employee.jabatan, employee.divisi, employee.nama_ibu_kandung, employee.nama_pasangan,
            employee.jumlah_anak, employee.nomor_telp_keluarga, employee.foto_ktp, employee.foto_kk,
            employee.foto_bpjs_kesehatan, employee.foto_bpjs_ketenagakerjaan, employee.pendidikan_terakhir,
            employee.perjanjian_kerja, employee.status, employee.tempat_kerja, is_flagged_val
        ],
    )?;
    Ok(())
}

#[tauri::command]
fn upsert_employees(
    database: State<'_, Database>,
    employees: Vec<Employee>,
) -> Result<usize, String> {
    let mut connection = database.0.lock().map_err(|error| error.to_string())?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;
    for employee in &employees {
        if employee.nik.trim().is_empty() || employee.nama_lengkap.trim().is_empty() {
            return Err("Setiap baris harus memiliki NIK dan Nama Lengkap.".into());
        }
        upsert(&transaction, employee).map_err(|error| error.to_string())?;
    }
    transaction.commit().map_err(|error| error.to_string())?;
    Ok(employees.len())
}

#[tauri::command]
fn save_employee(database: State<'_, Database>, employee: Employee) -> Result<(), String> {
    if employee.nik.trim().is_empty() || employee.nama_lengkap.trim().is_empty() {
        return Err("NIK dan Nama Lengkap wajib diisi.".into());
    }
    let connection = database.0.lock().map_err(|error| error.to_string())?;
    upsert(&connection, &employee).map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_employee(database: State<'_, Database>, nik: String) -> Result<(), String> {
    let connection = database.0.lock().map_err(|error| error.to_string())?;
    connection
        .execute("DELETE FROM master_karyawan WHERE nik = ?1", [nik])
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
async fn export_excel_file(path: String, bytes: Vec<u8>) -> Result<(), String> {
    fs::write(&path, bytes).map_err(|e| format!("Gagal menyimpan file ke disk: {}", e))
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
            initialize_attendance_schema(&connection)?;// <-- Added Attendance Schema
            
            app.manage(Database(Mutex::new(connection)));
            Ok(())
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_employees,
            upsert_employees,
            save_employee,
            delete_employee,
            export_excel_file,
            open_folder_dir,
            // --- Attendance Handlers ---
            get_or_create_periode,
            update_absensi_cell,
            list_staff_employees_for_absensi,
            toggle_employee_active_status,
            export_rekap_absen_excel_1to1,
            reset_database,

            get_rekap_cuti_tahun,
            update_saldo_cuti_karyawan
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}