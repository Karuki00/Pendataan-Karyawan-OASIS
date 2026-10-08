export interface Employee {
  nik: string;
  nama_lengkap: string;
  jenis_kelamin?: string;
  tanggal_lahir?: string;
  golongan_darah?: string;
  nomor_kk?: string;
  alamat_ktp?: string;
  bpjs_kesehatan?: string;
  bpjs_ketenagakerjaan?: string;
  nomor_telepon?: string;
  jabatan?: string;
  divisi?: string;
  nama_ibu_kandung?: string;
  nama_pasangan?: string;
  jumlah_anak?: string | number;
  nomor_telp_keluarga?: string;
  foto_ktp?: string;
  foto_kk?: string;
  foto_bpjs_kesehatan?: string;
  foto_bpjs_ketenagakerjaan?: string;
  pendidikan_terakhir?: string;
  perjanjian_kerja?: string;
  status?: string;
  join_date?: string;
  tanggal_kartap?: string;
  tempat_kerja?: string;
  is_flagged?: boolean;
}