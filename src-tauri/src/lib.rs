use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Manager, State};
use std::fs;

struct Database(Mutex<Connection>);

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
          tempat_kerja TEXT NOT NULL DEFAULT 'Lapangan'
        );",
    )?;

    for (column, definition) in [
        ("join_date", "TEXT"),
        ("tanggal_kartap", "TEXT"),
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
fn list_employees(database: State<'_, Database>, search: Option<String>) -> Result<Vec<Employee>, String> {
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
                    perjanjian_kerja, status, tempat_kerja
             FROM master_karyawan
             WHERE ?1 = '' OR nama_lengkap LIKE ?1 COLLATE NOCASE
                OR nik LIKE ?1 OR divisi LIKE ?1
             ORDER BY nama_lengkap COLLATE NOCASE",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([pattern], |row| {
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
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
}

fn upsert(connection: &Connection, employee: &Employee) -> rusqlite::Result<()> {
    connection.execute(
        "INSERT INTO master_karyawan (
          nik, timestamp, join_date, tanggal_kartap, nama_lengkap, jenis_kelamin, tanggal_lahir, golongan_darah,
          nomor_kk, alamat_ktp, bpjs_kesehatan, bpjs_ketenagakerjaan, nomor_telepon,
          jabatan, divisi, nama_ibu_kandung, nama_pasangan, jumlah_anak,
          nomor_telp_keluarga, foto_ktp, foto_kk, foto_bpjs_kesehatan,
          foto_bpjs_ketenagakerjaan, pendidikan_terakhir, perjanjian_kerja, status, tempat_kerja
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, COALESCE(?25, 'PKWTT'), COALESCE(?26, 'AKTIF'), COALESCE(?27, 'Lapangan'))
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
          tempat_kerja=excluded.tempat_kerja",
        params![
            employee.nik, employee.timestamp, employee.join_date, employee.tanggal_kartap,
            employee.nama_lengkap, employee.jenis_kelamin, employee.tanggal_lahir, employee.golongan_darah, employee.nomor_kk, employee.alamat_ktp,
            employee.bpjs_kesehatan, employee.bpjs_ketenagakerjaan, employee.nomor_telepon,
            employee.jabatan, employee.divisi, employee.nama_ibu_kandung, employee.nama_pasangan,
            employee.jumlah_anak, employee.nomor_telp_keluarga, employee.foto_ktp, employee.foto_kk,
            employee.foto_bpjs_kesehatan, employee.foto_bpjs_ketenagakerjaan, employee.pendidikan_terakhir,
            employee.perjanjian_kerja, employee.status, employee.tempat_kerja
        ],
    )?;
    Ok(())
}

#[tauri::command]
fn upsert_employees(database: State<'_, Database>, employees: Vec<Employee>) -> Result<usize, String> {
    let mut connection = database.0.lock().map_err(|error| error.to_string())?;
    let transaction = connection.transaction().map_err(|error| error.to_string())?;
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
        .setup(|app: &mut tauri::App| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let connection = Connection::open(data_dir.join("oasis-karyawan.sqlite"))?;
            initialize_schema(&connection)?;
            app.manage(Database(Mutex::new(connection)));
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            list_employees,
            upsert_employees,
            save_employee,
            delete_employee,
            export_excel_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn setup_test_db() -> Connection {
        let connection =
            Connection::open_in_memory().expect("failed to open in-memory test database");
        initialize_schema(&connection).expect("failed to initialize test database schema");
        connection
    }

    fn log_test_result(message: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock is before the Unix epoch")
            .as_secs();
        let line = format!("[{timestamp}] {message}");

        println!("{line}");

        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("test_execution.log")
            .expect("failed to open test_execution.log");
        writeln!(log_file, "{line}").expect("failed to write to test_execution.log");
    }

    #[test]
    fn test_sqlite_full_lifecycle_with_logs() {
        let connection = setup_test_db();
        log_test_result("Test database setup passed.");

        let employee = Employee {
            nik: "31710001".to_string(),
            nama_lengkap: "Budi Santoso".to_string(),
            divisi: Some("STAFF".to_string()),
            status: Some("AKTIF".to_string()),
            ..Default::default()
        };
        upsert(&connection, &employee).expect("employee insert failed");

        let inserted_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM master_karyawan", [], |row| row.get(0))
            .expect("failed to count inserted employees");
        assert_eq!(inserted_count, 1);
        log_test_result("Insert checkpoint passed: Budi Santoso was added.");

        let updated_employee = Employee {
            nama_lengkap: "Budi Santoso, S.H.".to_string(),
            divisi: Some("ENGINEERING".to_string()),
            ..employee
        };
        upsert(&connection, &updated_employee).expect("employee upsert update failed");

        let (updated_name, updated_division, total_count): (String, Option<String>, i64) =
            connection
                .query_row(
                    "SELECT nama_lengkap, divisi, (SELECT COUNT(*) FROM master_karyawan)
                     FROM master_karyawan WHERE nik = ?1",
                    [&updated_employee.nik],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .expect("failed to read updated employee");
        assert_eq!(updated_name, "Budi Santoso, S.H.");
        assert_eq!(updated_division.as_deref(), Some("ENGINEERING"));
        assert_eq!(total_count, 1);
        log_test_result("Upsert checkpoint passed: employee was updated without duplicating the row.");

        connection
            .execute(
                "DELETE FROM master_karyawan WHERE nik = ?1",
                [&updated_employee.nik],
            )
            .expect("employee delete failed");

        let remaining_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM master_karyawan", [], |row| row.get(0))
            .expect("failed to count employees after delete");
        assert_eq!(remaining_count, 0);
        log_test_result("Delete checkpoint passed: the employee table is empty.");
    }
}