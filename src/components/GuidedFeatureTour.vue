<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';

export type GuidedFeature = 'hr-master' | 'hr-absensi' | 'hr-cuti' | 'security-inspection' | 'security-absensi' | 'security-qr' | 'engineering-absensi' | 'housekeeping-absensi';

const props = defineProps<{ feature: GuidedFeature }>();
const emit = defineEmits<{ close: []; finished: [feature: GuidedFeature] }>();

interface TourStep {
  title: string;
  text: string;
  help: string;
  target: string;
}

const tours: Record<GuidedFeature, TourStep[]> = {
  'hr-master': [
    { title: 'Buka Master Data Karyawan', text: 'Klik menu Master Data Karyawan untuk melihat dan mengelola seluruh data karyawan.', help: 'Tombol yang disorot adalah pintu masuk data karyawan.', target: 'hr-master-tab' },
    { title: 'Pilih Import Excel', text: 'Di halaman ini, klik Import Excel jika Anda memiliki data karyawan dalam berkas Excel.', help: 'Gunakan import untuk memasukkan banyak karyawan sekaligus. Data lama dengan NIK yang sama akan diperbarui.', target: 'employee-import-button' },
    { title: 'Pilih berkas Excel', text: 'Klik Pilih Berkas Excel, lalu cari berkas .xlsx atau .xls di komputer Anda.', help: 'Setelah dipilih, OASIS membaca nama sheet dan kolom secara otomatis.', target: 'excel-file-picker' },
    { title: 'Pilih sheet yang benar', text: 'Periksa Sheet Terpilih Otomatis. Jika perlu, gunakan pilihan Ganti Sheet untuk memilih sheet lain.', help: 'Persentase Skor Kelengkapan Kolom menunjukkan seberapa cocok isi sheet dengan kolom karyawan yang diperlukan.', target: 'excel-sheet-selector' },
    { title: 'Periksa hasil pemeriksaan', text: 'Baca Pratinjau Impor. Baris Siap dapat disimpan, baris Peringatan masih dapat disimpan tetapi perlu diperiksa, dan baris Ditolak harus diperbaiki.', help: 'NIK dan Nama Lengkap wajib diisi. Periksa juga NIK, KK, dan nomor BPJS yang belum lengkap.', target: 'excel-preview' },
    { title: 'Simpan ke database', text: 'Jika data sudah benar, klik Proses Impor. Setelah selesai, daftar karyawan akan diperbarui.', help: 'Pada komputer Master, perubahan karyawan Security juga disinkronkan ke data Security.', target: 'excel-import-confirm' },
  ],
  'hr-absensi': [
    { title: 'Buka Rekapitulasi Absensi', text: 'Klik menu Rekapitulasi Absensi untuk mencatat absensi dan menyiapkan laporan.', help: 'Halaman ini digunakan untuk memilih periode, divisi, dan memeriksa tabel harian.', target: 'hr-attendance-tab' },
    { title: 'Pilih divisi dan periode', text: 'Pilih divisi pada pilihan Pilih Divisi, lalu tentukan bulan pada Periode.', help: 'Pastikan bulan dan divisi benar sebelum mengubah status absensi.', target: 'attendance-division' },
    { title: 'Isi tabel absensi', text: 'Pada tabel, pilih status setiap karyawan dan tambahkan keterangan bila diperlukan.', help: 'Simpan perubahan pada sel absensi. Gunakan halaman dan tombol Next jika karyawan lebih banyak.', target: 'attendance-table' },
    { title: 'Buat laporan Excel', text: 'Jika data sudah lengkap, klik Preview & Export Excel untuk memeriksa dan menyimpan laporan.', help: 'Periksa kembali periode, divisi, karyawan, cuti, dan potongan gaji sebelum menyimpan.', target: 'attendance-export' },
  ],
  'hr-cuti': [
    { title: 'Buka Rekapitulasi Cuti', text: 'Klik Rekap Cuti untuk melihat penggunaan dan sisa cuti karyawan.', help: 'Tabel menampilkan jatah awal, pemakaian per bulan, potongan gaji, dan sisa cuti.', target: 'hr-leave-tab' },
    { title: 'Pilih divisi', text: 'Pilih divisi yang ingin diperiksa dari pilihan Divisi.', help: 'Gunakan Staff & Pengelola, Security, Housekeeping, atau Engineering sesuai kebutuhan.', target: 'leave-division' },
    { title: 'Pilih tahun', text: 'Pilih tahun laporan pada pengatur tahun.', help: 'Rekap cuti dihitung berdasarkan tahun yang dipilih.', target: 'leave-year' },
    { title: 'Periksa dan ubah jatah', text: 'Baca kolom Cuti Terpakai, Pot. Gaji, dan Sisa Cuti. Klik Kuota jika jatah awal perlu diperbaiki.', help: 'CUTI, IZIN, dan SAKIT mengurangi cuti terpakai. ALPA masuk ke potongan gaji, bukan sisa cuti.', target: 'leave-quota' },
  ],
  'security-inspection': [
    { title: 'Mulai dari Rekapitulasi Absensi', text: 'Sebelum melakukan inspeksi, buka Rekapitulasi Absensi Security dan pilih periode yang akan digunakan.', help: 'Tutorial menyorot tombolnya. Jika tombol belum terlihat, tutup panduan sementara, buka halaman tersebut, lalu tekan Bantuan lagi.', target: 'security-attendance-tab' },
    { title: 'Pastikan data HR tersedia', text: 'Periksa apakah daftar petugas dan data rekap absensi sudah muncul. Jika kosong atau tidak sesuai, hentikan proses dan hubungi HR untuk menyinkronkan data dari komputer Master Node.', help: 'Jangan membuat data atau akun pengganti di Client. Data dari HR adalah sumber utama.', target: 'attendance-table' },
    { title: 'Periksa status rekap', text: 'Pastikan indikator rekap menunjukkan Terhubung dan jumlah baris tersedia untuk periode yang dipilih.', help: 'Jika Koneksi gagal, minta HR memastikan sinkronisasi dari Master selesai, lalu muat ulang periode.', target: 'security-attendance-status' },
    { title: 'Buka dashboard inspeksi', text: 'Setelah data Security tersedia, kembali ke Dasbor Inspeksi Hydrant untuk melanjutkan persiapan inspeksi.', help: 'Akun dan daftar petugas harus berasal dari data yang sudah disiapkan HR.', target: 'security-inspection-tab' },
    { title: 'Buat PIN petugas', text: 'Chief Security memilih nama petugas, memasukkan PIN 4-6 digit dua kali, lalu menekan Buat PIN. Sampaikan PIN secara pribadi kepada petugas.', help: 'Jika PIN sudah ada, gunakan Reset PIN hanya setelah konfirmasi akun chief_security.', target: 'security-pin-management' },
    { title: 'Masuk dan lakukan inspeksi', text: 'Petugas menggunakan akun Security dan PIN yang diberikan untuk login inspeksi, lalu mengisi pemeriksaan asset sesuai kondisi lapangan.', help: 'Jangan membagikan PIN di grup atau menuliskannya pada label asset.', target: 'security-pin-management' },
    { title: 'Sinkronkan hasil inspeksi', text: 'Klik Sync Google Sheets setelah inspeksi selesai untuk mengambil hasil terbaru dan periksa jumlah data yang berhasil.', help: 'Tinjau status komponen V, X, atau R, catatan, kerusakan, dan foto lapangan.', target: 'security-sync' },
    { title: 'Gunakan Force Re-sync bila perlu', text: 'Jika data lokal tidak sesuai dengan sumber inspeksi, gunakan Force Re-sync untuk mengambil ulang seluruh catatan.', help: 'Gunakan hanya setelah memastikan sumber benar karena data inspeksi lokal akan dibangun ulang.', target: 'security-force-sync' },
  ],
  'security-absensi': [
    { title: 'Buka Absensi Security', text: 'Pilih Rekapitulasi Absensi Security untuk melihat dan mengisi absensi petugas.', help: 'Pilih tanggal dan status absensi seperti biasa.', target: 'security-attendance-sync' },
    { title: 'Sinkronkan rekap', text: 'Setelah data periode selesai, klik Sync Rekap Security.', help: 'Pastikan status koneksi menunjukkan terhubung sebelum mengirim data.', target: 'security-attendance-sync' },
    { title: 'Periksa hasil kirim', text: 'Periksa pesan berhasil dan jumlah baris yang terkirim.', help: 'Jika gagal, periksa koneksi dan coba lagi setelah jaringan tersedia.', target: 'security-attendance-sync' },
  ],
  'security-qr': [
    { title: 'Buka Cetak QR Code Asset', text: 'Pilih kategori atau cari kode asset jika ingin mencetak QR tertentu.', help: 'Gunakan pencarian untuk menemukan asset dengan cepat.', target: 'qr-print-current' },
    { title: 'Pilih jumlah kartu per halaman', text: 'Pilih 6, 12, atau 15 kartu per halaman sesuai ukuran kertas yang diperlukan.', help: 'Jumlah lebih sedikit membuat QR lebih besar dan mudah dipindai.', target: 'qr-print-current' },
    { title: 'Cetak QR Code', text: 'Klik Cetak Hal Ini untuk halaman yang sedang tampil atau Cetak Semua untuk seluruh hasil.', help: 'Pada dialog cetak, pilih printer atau Save as PDF.', target: 'qr-print-all' },
  ],
  'engineering-absensi': [
    { title: 'Buka Absensi Engineering', text: 'Halaman ini digunakan untuk mencatat absensi karyawan Engineering berdasarkan periode kerja.', help: 'Periksa judul halaman agar Anda berada di Rekapitulasi Absensi Engineering.', target: 'engineering-attendance-tab' },
    { title: 'Pilih periode', text: 'Pilih bulan pada kotak Periode sebelum mengisi tabel.', help: 'Gunakan bulan yang sesuai dengan periode laporan agar data tersimpan di bulan yang benar.', target: 'attendance-month' },
    { title: 'Isi tabel absensi', text: 'Pilih status setiap karyawan Engineering pada tanggal yang sesuai dan isi keterangan bila diperlukan.', help: 'Gunakan tombol halaman jika daftar karyawan lebih dari satu halaman.', target: 'attendance-table' },
    { title: 'Buat laporan', text: 'Klik Preview & Export Excel untuk memeriksa dan menyimpan rekap absensi Engineering.', help: 'Periksa kembali periode dan nama karyawan sebelum menyimpan laporan.', target: 'attendance-export' },
  ],
  'housekeeping-absensi': [
    { title: 'Buka Absensi Housekeeping', text: 'Halaman ini digunakan untuk mencatat absensi karyawan Housekeeping berdasarkan periode kerja.', help: 'Periksa judul halaman agar Anda berada di Rekapitulasi Absensi Housekeeping.', target: 'housekeeping-attendance-tab' },
    { title: 'Pilih periode', text: 'Pilih bulan pada kotak Periode sebelum mengisi tabel.', help: 'Gunakan bulan yang sesuai dengan periode laporan agar data tersimpan di bulan yang benar.', target: 'attendance-month' },
    { title: 'Isi tabel absensi', text: 'Pilih status setiap karyawan Housekeeping pada tanggal yang sesuai dan isi keterangan bila diperlukan.', help: 'Gunakan tombol halaman jika daftar karyawan lebih dari satu halaman.', target: 'attendance-table' },
    { title: 'Buat laporan', text: 'Klik Preview & Export Excel untuk memeriksa dan menyimpan rekap absensi Housekeeping.', help: 'Periksa kembali periode dan nama karyawan sebelum menyimpan laporan.', target: 'attendance-export' },
  ],
};

const currentStep = ref(0);
const targetRect = ref<DOMRect | null>(null);
let resizeHandler: (() => void) | null = null;

const steps = computed(() => tours[props.feature]);
const step = computed(() => steps.value[currentStep.value]);
const isLast = computed(() => currentStep.value === steps.value.length - 1);

function refreshTarget() {
  const element = document.querySelector(`[data-tour="${step.value.target}"]`);
  targetRect.value = element?.getBoundingClientRect() ?? null;
  if (element) element.scrollIntoView({ behavior: 'smooth', block: 'center', inline: 'nearest' });
}

function next() {
  if (isLast.value) {
    emit('finished', props.feature);
    emit('close');
    return;
  }
  currentStep.value += 1;
  void nextTick(refreshTarget);
}

function previous() {
  if (currentStep.value > 0) {
    currentStep.value -= 1;
    void nextTick(refreshTarget);
  }
}

watch(() => props.feature, () => {
  currentStep.value = 0;
  void nextTick(refreshTarget);
});

onMounted(() => {
  void nextTick(refreshTarget);
  resizeHandler = () => refreshTarget();
  window.addEventListener('resize', resizeHandler);
  window.addEventListener('scroll', resizeHandler, true);
});

onBeforeUnmount(() => {
  if (resizeHandler) {
    window.removeEventListener('resize', resizeHandler);
    window.removeEventListener('scroll', resizeHandler, true);
  }
});
</script>

<template>
  <div class="fixed inset-0 z-[180] pointer-events-none" aria-live="polite">
    <div v-if="targetRect" class="absolute rounded-xl border-4 border-amber-400 ring-4 ring-amber-300/40 transition-all duration-300" :style="{ left: `${targetRect.left - 8}px`, top: `${targetRect.top - 8}px`, width: `${targetRect.width + 16}px`, height: `${targetRect.height + 16}px`, boxShadow: '0 0 0 9999px rgba(15, 23, 42, 0.58)' }"></div>
    <div class="pointer-events-auto absolute bottom-5 left-1/2 w-[min(92vw,560px)] -translate-x-1/2 rounded-3xl border border-amber-200 bg-white p-6 shadow-2xl sm:bottom-8 sm:p-7">
      <div class="flex items-start justify-between gap-4">
        <div><p class="text-xs font-extrabold uppercase tracking-widest text-emerald-700">Panduan langkah demi langkah</p><h2 class="mt-1 text-2xl font-extrabold text-slate-900">{{ step.title }}</h2></div>
        <button type="button" @click="$emit('close')" class="text-2xl font-bold text-slate-400 hover:text-slate-800" aria-label="Tutup panduan">×</button>
      </div>
      <p class="mt-4 text-base leading-relaxed text-slate-700">{{ step.text }}</p>
      <div class="mt-4 rounded-2xl bg-amber-50 p-4 text-sm leading-relaxed text-amber-950"><strong>Bantuan:</strong> {{ step.help }}</div>
      <p v-if="!targetRect" class="mt-3 text-xs font-semibold text-rose-700">Kontrol ini belum terlihat. Selesaikan langkah sebelumnya, lalu tekan Berikutnya.</p>
      <div class="mt-5 flex items-center justify-between gap-3 border-t border-slate-100 pt-4"><span class="text-sm font-bold text-slate-500">{{ currentStep + 1 }} / {{ steps.length }}</span><div class="flex gap-2"><button v-if="currentStep > 0" type="button" @click="previous" class="rounded-xl border-2 border-slate-200 px-4 py-2.5 text-sm font-bold text-slate-700">Kembali</button><button type="button" @click="next" class="rounded-xl bg-emerald-800 px-5 py-2.5 text-sm font-bold text-white hover:bg-emerald-900">{{ isLast ? 'Selesai' : 'Saya sudah, lanjut' }}</button></div></div>
    </div>
  </div>
</template>
