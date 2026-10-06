export interface Employee {
  nik: string;
  timestamp?: string | null;
  join_date?: string | null;
  tanggal_kartap?: string | null;
  nama_lengkap: string;
  jenis_kelamin?: string | null;
  tanggal_lahir?: string | null;
  golongan_darah?: string | null;
  nomor_kk?: string | null;
  alamat_ktp?: string | null;
  bpjs_kesehatan?: string | null;
  bpjs_ketenagakerjaan?: string | null;
  nomor_telepon?: string | null;
  jabatan?: string | null;
  divisi?: string | null;
  nama_ibu_kandung?: string | null;
  nama_pasangan?: string | null;
  jumlah_anak?: string | null;
  nomor_telp_keluarga?: string | null;
  foto_ktp?: string | null;
  foto_kk?: string | null;
  foto_bpjs_kesehatan?: string | null;
  foto_bpjs_ketenagakerjaan?: string | null;
  pendidikan_terakhir?: string | null;
  perjanjian_kerja?: string | null;
  status?: string | null;
  tempat_kerja?: string | null;
}
