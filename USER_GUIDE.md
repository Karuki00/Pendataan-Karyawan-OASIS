# Panduan Penggunaan OASIS One

Panduan ini menjelaskan cara memasang dan menggunakan OASIS One sebagai node
**Master** atau **Client**.

> **Placeholder gambar**
>
> Gambar pada panduan ini menggunakan lokasi placeholder seperti
> `docs/images/login.png`. Ganti dengan screenshot sebenarnya jika sudah
> tersedia.

## 1. Peran Master dan Client

### Node Master

Node Master adalah komputer HR yang:

- Menyimpan database utama karyawan dan absensi.
- Menjalankan server jaringan lokal pada TCP port `3000`.
- Membagikan data karyawan dan absensi ke node Client.
- Menyinkronkan data karyawan Security, pengguna Security, dan akun Security
  ke Apps Script.
- Sebaiknya tetap menyala selama Client menggunakan data melalui LAN.

### Node Client

Node Client adalah komputer kantor yang terhubung ke jaringan yang sama dengan
Master. Node Client:

- Mencari Master secara otomatis melalui jaringan lokal.
- Membaca data karyawan dan absensi dari Master.
- Menggunakan Apps Script untuk fitur Security dan inspeksi.
- Tidak menjalankan sinkronisasi administrator HR.

Peran node disimpan di database lokal aplikasi. Pembaruan aplikasi normalnya
tidak mengubah Master menjadi Client atau sebaliknya.

## 2. Instalasi Pertama

1. Unduh installer Windows dari halaman release proyek https://github.com/Karuki00/Pendataan-Karyawan-OASIS/releases.
2. Instal OASIS One.
3. Jalankan aplikasi.
4. Pada penggunaan pertama, pilih peran komputer yang benar jika diminta.
5. Gunakan akun Windows yang sama agar database aplikasi tetap ditemukan.

![Placeholder: layar pertama kali dibuka](docs/images/first-launch.png)

### Checklist komputer Master

- Gunakan komputer HR sebagai Master.
- Izinkan aplikasi melalui Windows Firewall.
- Izinkan koneksi masuk TCP `3000`.
- Izinkan UDP/mDNS lokal pada UDP `5353` jika pencarian otomatis tidak
  berhasil.
- Biarkan aplikasi Master tetap berjalan saat Client terhubung.

### Checklist komputer Client

- Hubungkan komputer ke Wi-Fi atau LAN yang sama dengan Master.
- Izinkan aplikasi melalui Windows Firewall jika diminta.
- Jangan mengatur Client sebagai Master baru jika harus menggunakan database
  Master yang sudah ada.

## 3. Login

Masukkan username dan password untuk divisi yang diperlukan.

![Placeholder: layar login](docs/images/login.png)

Periksa badge status pada header:

- `MASTER` berarti komputer ini memiliki database HR lokal dan server.
- `CLIENT` berarti komputer ini menggunakan Master melalui LAN jika tersedia.
- Alamat Master yang tampil menunjukkan koneksi jaringan yang sedang dipakai.

Jika Client tidak menemukan Master:

1. Pastikan kedua komputer berada pada jaringan yang sama.
2. Pastikan aplikasi Master sedang berjalan.
3. Pastikan TCP `3000` diizinkan pada Firewall Master.
4. Pastikan jaringan kantor tidak mengaktifkan isolasi antarperangkat.
5. Jalankan ulang OASIS One setelah memperbaiki jaringan.

## 4. Dashboard

Dashboard menampilkan ringkasan data karyawan, anomali absensi, dan navigasi
cepat ke daftar master karyawan.

![Placeholder: dashboard](docs/images/dashboard.png)

Gunakan metrik dashboard untuk:

- Membuka daftar karyawan yang memiliki peringatan.
- Meninjau anomali absensi.
- Membuka pengelolaan data karyawan.

## 5. Data Master Karyawan

Buka **Master Data** untuk melihat, menambah, mengubah, menonaktifkan, atau
mengimpor data karyawan.

![Placeholder: data master karyawan](docs/images/master-data.png)

### Menambah atau mengubah karyawan

1. Buka **Master Data**.
2. Pilih **Tambah Karyawan** atau edit karyawan yang ada.
3. Masukkan identitas, divisi, jabatan, regu, dan status karyawan.
4. Simpan data.

Pada node Master, perubahan karyawan Security otomatis memulai sinkronisasi:

1. `SecurityEmployees` diperbarui terlebih dahulu.
2. `SecurityUsers` diperbarui berdasarkan status aktif di `SecurityEmployees`.
3. Akun autentikasi Security disinkronkan.

### Mengimpor karyawan dari Excel

1. Buka **Master Data**.
2. Pilih **Import Excel**.
3. Pilih file Excel karyawan.
4. Periksa data yang akan diimpor.
5. Konfirmasi proses import.

![Placeholder: import workbook karyawan](docs/images/import-employees.png)

## 6. Rekapitulasi Absensi

Buka **Rekapitulasi Absensi** untuk memilih divisi, tahun, dan periode
penggajian.

![Placeholder: rekapitulasi absensi](docs/images/attendance-recap.png)

Untuk memasukkan absensi:

1. Pilih karyawan dan tanggal.
2. Pilih status absensi.
3. Tambahkan keterangan jika diperlukan.
4. Simpan atau segarkan tabel.

Data absensi Client biasanya dibaca dari Master melalui LAN. Jika LAN
sementara tidak tersedia, perhatikan status koneksi dan sambungkan kembali ke
Master sebelum menggunakan data untuk laporan.

## 7. Rekapitulasi Cuti

Buka **Rekapitulasi Cuti** untuk melihat penggunaan dan sisa cuti tahunan.

![Placeholder: rekapitulasi cuti](docs/images/leave-recap.png)

Aturan yang berlaku:

- `CUTI`, `IZIN`, dan `SAKIT` dihitung sebagai cuti terpakai.
- `ALPA` hanya dihitung pada kolom potongan gaji (`POT. GAJI`).
- `ALPA` tidak mengurangi sisa cuti yang disetujui.

## 8. Inspeksi Security

Alur inspeksi dimulai dari pemeriksaan data HR. Ikuti urutan ini setiap kali
menyiapkan petugas baru atau memulai periode kerja baru.

### A. Pastikan data HR sudah tersedia di Client

1. Login menggunakan akun Security pada komputer Client.
2. Buka **Rekapitulasi Absensi Security**.
3. Pilih periode yang akan diperiksa.
4. Pastikan daftar petugas dan tabel absensi muncul. Periksa juga indikator
   rekap: harus menunjukkan **Terhubung** dan jumlah baris tersedia.
5. Jika daftar kosong, periode tidak muncul, atau datanya tidak sesuai, jangan
   membuat data pengganti di Client. Hubungi HR agar melakukan sinkronisasi dari
   komputer **Master Node**, lalu muat ulang periode di Client.

> Komputer Master adalah sumber data HR. Client hanya membaca data yang sudah
> disediakan oleh Master atau sumber sinkronisasi resmi.

### B. Siapkan login petugas inspeksi

Setelah HR memastikan data Security sudah tersedia:

1. Kembali ke **Dasbor Inspeksi Hydrant**.
2. Pada panel **Buat / Reset PIN Petugas Security**, pilih nama petugas.
3. Masukkan PIN baru 4-6 digit dan ulangi PIN tersebut.
4. Klik **Buat PIN**.
5. Berikan PIN kepada petugas secara pribadi. Jangan menaruh PIN di grup chat
   atau menempelkannya pada asset.
6. Bila PIN sudah pernah dibuat dan perlu diganti, klik **Reset PIN** dan ikuti
   konfirmasi akun `chief_security`.

Petugas kemudian dapat login menggunakan akun Security yang telah disiapkan
dan memakai PIN tersebut untuk proses inspeksi. Jika nama petugas belum ada di
pilihan, kembali ke langkah A dan minta HR menyinkronkan database Security dari
Master.

![Placeholder: panel PIN petugas Security](docs/images/security-pin-management.png)

### C. Melakukan dan memeriksa inspeksi

1. Petugas login menggunakan akun Security miliknya.
2. Periksa asset berdasarkan QR Code atau daftar asset di lapangan.
3. Isi kondisi setiap komponen, catatan, dan foto jika diperlukan.
4. Setelah inspeksi tersimpan, buka **Dasbor Inspeksi Hydrant**.
5. Klik **Sync Google Sheets** dan tunggu pesan jumlah data berhasil.
6. Periksa baris inspeksi, status komponen, catatan kerusakan, dan foto.

Status komponen dapat berisi `V` (baik), `X` (rusak), atau `R` (perlu
perhatian/perbaikan). Periksa komponen yang berwarna merah dan pastikan foto
atau catatan mendukung laporan tersebut.

![Placeholder: dashboard inspeksi Security](docs/images/security-dashboard.png)

### Menyinkronkan hasil inspeksi

1. Gunakan **Sync Google Sheets** untuk mengambil hasil terbaru.
2. Gunakan **Force Re-sync** hanya jika data lokal tidak sesuai dengan sumber.
   Proses ini menghapus salinan inspeksi lokal lalu mengambil ulang seluruh
   catatan.
3. Jika sinkronisasi gagal, periksa koneksi dan hubungi admin sebelum mencoba
   Force Re-sync berulang kali.

Hasil inspeksi dapat berisi status komponen, catatan umum, keterangan terpisah
untuk Selang, Nozle, Valve, Lampu, Bell, Apar, atau Kunci pilar, serta foto.

### Panduan interaktif di dalam aplikasi

Saat pertama kali login sebagai Security, OASIS One membuka **Panduan langkah
demi langkah**. Tekan **Saya sudah, lanjut** setelah menyelesaikan setiap
langkah. Panduan akan menyorot tombol yang sedang digunakan, termasuk urutan
Rekapitulasi Absensi → verifikasi data HR → pembuatan PIN → sinkronisasi
inspeksi. Jika ditutup, buka kembali dengan tombol **? Bantuan** di header.

## 9. Absensi Security

Buka **Rekapitulasi Absensi Security** untuk memasukkan absensi Security.

![Placeholder: absensi Security](docs/images/security-attendance.png)

Untuk setiap karyawan dan tanggal:

1. Pilih status absensi.
2. Masukkan keterangan jika diperlukan.
3. Jalankan sinkronisasi Security setelah periode siap.
4. Pastikan Apps Script menampilkan sinkronisasi berhasil.

## 10. Mencetak QR Code Asset

Buka **Cetak QR Code Asset** dari menu Security.

![Placeholder: generator QR Code](docs/images/qr-generator.png)

Untuk mencetak:

1. Cari atau filter asset yang diperlukan.
2. Pilih jumlah kartu per halaman.
3. Pilih **Cetak Hal Ini** atau **Cetak Semua**.
4. Pada dialog cetak, pilih printer atau **Save as PDF**.

Mode cetak menyembunyikan header aplikasi, navigasi, filter, dan kontrol.
Hanya kartu QR Code yang dicetak.

![Placeholder: pratinjau cetak QR Code](docs/images/qr-print-preview.png)

## 11. Export Excel

Export absensi dan cuti tersedia dari halaman rekapitulasi.

![Placeholder: dialog export Excel](docs/images/excel-export.png)

Alur yang disarankan:

1. Pilih tahun, bulan, periode, dan divisi yang benar.
2. Buka **Preview & Export Excel**.
3. Periksa pratinjau.
4. Pilih lokasi penyimpanan.
5. Buka workbook dan pastikan karyawan, absensi, cuti, dan potongan gaji
   sudah benar.

Untuk export dari Client, pastikan Master dapat dijangkau agar data absensi
bersama ikut disertakan.

## 12. Sinkronisasi Database Security di Master

Hanya HR pada node Master yang boleh menggunakan **Sync Database Security**.

![Placeholder: tombol sinkronisasi Security pada Master](docs/images/master-security-sync.png)

Sinkronisasi otomatis berjalan saat Master dimulai dan setelah data karyawan
berubah. Tombol tersebut dapat digunakan untuk mencoba ulang jika Apps Script
sempat tidak tersedia.

Jika sinkronisasi gagal:

- Pastikan badge header menunjukkan `MASTER`.
- Pastikan URL admin Apps Script tersedia pada build.
- Pastikan secret sinkronisasi masih valid.
- Periksa pesan error pada header.
- Coba lagi setelah Apps Script tersedia.

## 13. Memperbarui Aplikasi

Updater memasang release baru yang ditandatangani tanpa mengganti database
aplikasi yang sudah ada.

Sebelum memperbarui:

- Pastikan database sudah dicadangkan.
- Tutup jendela OASIS lain pada komputer yang sama.
- Jangan menghapus folder data aplikasi.

Setelah memperbarui:

1. Jalankan OASIS One.
2. Pastikan header masih menunjukkan peran `MASTER` atau `CLIENT` yang benar.
3. Pastikan Master dapat membuka daftar karyawan.
4. Pastikan sinkronisasi Security tersedia pada Master.
5. Uji koneksi satu Client jika kantor menggunakan LAN.

## 14. Pemecahan Masalah

### Client tidak dapat terhubung ke Master

- Pastikan kedua komputer berada pada jaringan yang sama.
- Pastikan aplikasi Master terbuka.
- Izinkan TCP `3000` masuk pada Windows Firewall Master.
- Uji menggunakan alamat IP Master yang ditampilkan aplikasi.
- Periksa apakah Wi-Fi kantor memblokir koneksi antarperangkat.

### Tombol sinkronisasi Master tidak terlihat

- Pastikan badge header menunjukkan `MASTER`.
- Jalankan ulang aplikasi setelah update.
- Untuk instalasi lama, peran database lebih penting daripada flag build.

### Data Security tidak diperbarui

- Jalankan **Sync Database Security** pada Master.
- Pastikan `SecurityEmployees` diperbarui terlebih dahulu.
- Pastikan nilai `Active` karyawan adalah `TRUE` atau `FALSE`.
- Pastikan `SecurityUsers.Active` mengikuti data karyawan tersebut.

### Export Excel tidak memiliki data

- Pastikan periode dan divisi yang dipilih benar.
- Pastikan Master online ketika export dari Client.
- Segarkan tabel absensi sebelum membuka pratinjau export.
- Sambungkan kembali ke Master lalu coba lagi.

## 15. Praktik Operasional yang Disarankan

- Gunakan satu komputer HR sebagai Master.
- Jangan memasang database baru di atas folder data Master yang sudah ada.
- Biarkan Master aktif selama jam operasional.
- Cadangkan database sebelum update besar.
- Uji release pada instalasi development terlebih dahulu.
- Anggap URL Apps Script sebagai endpoint publik; jangan menaruh secret
  pribadi di dokumentasi.
