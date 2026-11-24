# OASIS One v1.2.7

## Perubahan

- Memperbarui panduan Security menjadi alur lengkap dari pemeriksaan
  Rekapitulasi Absensi, konfirmasi sinkronisasi HR dari Master Node, pembuatan
  PIN petugas, login inspeksi, sampai sinkronisasi hasil inspeksi.
- Menambahkan langkah interaktif pada tutorial Security untuk menyorot tab
  absensi, status rekap, panel PIN, dan kontrol sinkronisasi inspeksi.
- Menjelaskan tindakan yang benar saat data Security belum tersedia: hubungi HR
  untuk sinkronisasi dari Master dan jangan membuat data pengganti di Client.
- Memperluas `USER_GUIDE.md` dengan tutorial operasional Security dan panduan
  penggunaan Force Re-sync.

## Verifikasi

- `npm.cmd run build` berhasil.
- `cargo check` berhasil.

# OASIS One v1.2.6

## Perubahan

- Menambahkan tutorial interaktif berbahasa Indonesia untuk pengguna baru.
- Tutorial menampilkan langkah demi langkah dengan sorotan pada tombol dan kontrol yang sedang dijelaskan.
- Menambahkan panduan khusus untuk HR:
  - Master Data Karyawan.
  - Import Excel, pemilihan sheet, skor kelengkapan kolom, pemeriksaan data, dan proses impor.
  - Rekapitulasi Absensi.
  - Rekapitulasi Cuti.
  - Export laporan Excel.
- Menambahkan panduan khusus untuk Security:
  - Sinkronisasi inspeksi Google Sheets.
  - Pemeriksaan kerusakan dan foto inspeksi.
  - Force Re-sync.
  - Sinkronisasi absensi Security.
  - Cetak QR Code Asset.
- Menambahkan panduan khusus untuk Engineering dan Housekeeping pada fitur absensi.
- Menambahkan tombol `? Bantuan` di header untuk membuka kembali panduan sesuai halaman aktif.
- Menonaktifkan sinkronisasi spreadsheet/database eksternal saat menjalankan `tauri dev`.
- Mengaktifkan sinkronisasi eksternal hanya pada production release build.
- Release production dikonfigurasi sebagai node `CLIENT` secara default.

## Verifikasi

- `npm.cmd run build` berhasil.
- `cargo check` berhasil.

