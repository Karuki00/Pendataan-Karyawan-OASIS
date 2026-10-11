use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaveSummary {
    pub nik: String,
    pub nama_lengkap: String,
    pub jabatan: String,
    pub join_date: Option<String>,
    pub jatah_awal: i32,
    pub cuti_bersama: i32,
    pub monthly_a: Vec<i32>, // Jan-Dec Quota Leave (CUTI, SAKIT (TANPA SD), ALPA)
    pub monthly_b: Vec<i32>, // Jan-Dec Potongan Gaji (e.g. POTONG_GAJI, excess IZIN)
    pub total_cuti_terpakai: i32,
    pub total_potongan_gaji: i32,
    pub sisa_cuti: i32,
}