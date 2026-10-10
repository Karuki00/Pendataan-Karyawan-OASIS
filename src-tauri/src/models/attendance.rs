use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodeAbsensi {
    pub id: Option<i64>,
    pub nama_periode: String,
    pub tanggal_mulai: String,   // Format YYYY-MM-DD (17th prev month)
    pub tanggal_selesai: String, // Format YYYY-MM-DD (16th current month)
    pub total_hari: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordAbsensi {
    pub id: Option<i64>,
    pub nik: String,
    pub periode_id: i64,
    pub tanggal: String,
    pub status: String, // ON, OFF, SAKIT, SAKIT_SD, IZIN, ALPA, CUTI
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RekapKaryawanRow {
    pub nik: String,
    pub nama_lengkap: String,
    pub jabatan: Option<String>,
    pub attendances: Vec<RecordAbsensi>,
    pub total_kerja: i32,
    pub total_off: i32,
    pub total_sakit: i32,
    pub total_izin: i32,
    pub total_alpa: i32,
    pub total_cuti: i32,
}