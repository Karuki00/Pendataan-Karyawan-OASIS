use serde::{Deserialize, Serialize};

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

pub fn compute_is_flagged(emp: &Employee) -> bool {
    let nik = emp.nik.trim();
    let kk = emp.nomor_kk.as_deref().unwrap_or("").trim();
    let bpjs_kes = emp.bpjs_kesehatan.as_deref().unwrap_or("").trim();
    let bpjs_tk = emp.bpjs_ketenagakerjaan.as_deref().unwrap_or("").trim();

    let invalid_nik = nik.len() != 16 || !nik.chars().all(|c| c.is_ascii_digit());
    let invalid_kk = !kk.is_empty() && (kk.len() != 16 || !kk.chars().all(|c| c.is_ascii_digit()));
    let missing_bpjs = bpjs_kes.is_empty() || bpjs_tk.is_empty();

    invalid_nik || invalid_kk || missing_bpjs
}
