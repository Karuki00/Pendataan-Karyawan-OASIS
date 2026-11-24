<script setup lang="ts">
import { computed, ref } from "vue";

defineEmits<{ close: [] }>();

const currentStep = ref(0);
const steps = [
  { number: '1', title: 'Selamat datang di OASIS One', text: 'Tutorial ini akan memandu Anda menggunakan aplikasi, satu langkah demi satu langkah.', tip: 'Tekan Berikutnya untuk melanjutkan. Jika berhenti, tutorial dapat dibuka lagi melalui tombol Bantuan.' },
  { number: '2', title: 'Masuk ke aplikasi', text: 'Ketik Username Divisi dan Password, lalu tekan Masuk Sistem. Gunakan akun sesuai divisi Anda.', tip: 'Jika lupa password, hubungi HR. Pada komputer Master tersedia Kode Pemulihan Master.' },
  { number: '3', title: 'Kenali status komputer', text: 'MASTER adalah komputer HR yang menyimpan data utama. CLIENT mengambil data dari Master melalui jaringan kantor.', tip: 'Jika Client tidak terhubung, pastikan Master menyala dan kedua komputer berada di jaringan yang sama.' },
  { number: '4', title: 'Mulai dari Dashboard', text: 'Dashboard menampilkan ringkasan karyawan, peringatan absensi, dan tombol untuk membuka pekerjaan yang sering digunakan.', tip: 'Tekan angka atau kartu peringatan untuk melihat data yang perlu diperiksa.' },
  { number: '5', title: 'Kelola Data Master', text: 'Buka Master Data untuk menambah, mengubah, menonaktifkan, atau mengimpor data karyawan dari Excel.', tip: 'Setelah data karyawan Security berubah di Master, sinkronisasi Security akan dijalankan.' },
  { number: '6', title: 'Catat dan rekap absensi', text: 'Buka Rekapitulasi Absensi, pilih periode dan divisi, lalu pilih status absensi setiap karyawan. Simpan setelah selesai.', tip: 'Client harus terhubung ke Master agar data bersama dapat dibaca dan diekspor.' },
  { number: '7', title: 'Periksa cuti dan buat laporan', text: 'Rekapitulasi Cuti menampilkan cuti terpakai dan sisa cuti. Gunakan Preview & Export Excel untuk membuat laporan.', tip: 'Pastikan tahun, bulan, periode, dan divisi sudah benar sebelum mengekspor.' },
  { number: '8', title: 'Fitur Security dan QR Code', text: 'Pengguna Security dapat membuka inspeksi, absensi Security, dan Cetak QR Code Asset dari menu yang tersedia.', tip: 'Gunakan Sync Google Sheets setelah inspeksi selesai agar data terlihat oleh tim terkait.' },
  { number: '9', title: 'Bantuan selalu tersedia', text: 'Jika lupa cara menggunakan fitur, tekan tombol Bantuan di header aplikasi untuk membuka tutorial ini lagi.', tip: 'Anda dapat maju atau mundur antar langkah kapan saja. Selamat bekerja dengan OASIS One.' },
];
const step = computed(() => steps[currentStep.value]);
const isLastStep = computed(() => currentStep.value === steps.length - 1);
function next() { if (!isLastStep.value) currentStep.value += 1; }
function previous() { if (currentStep.value > 0) currentStep.value -= 1; }
</script>

<template>
  <div class="fixed inset-0 z-[200] flex items-center justify-center bg-slate-950/70 p-4 backdrop-blur-sm" @click.self="$emit('close')">
    <section class="w-full max-w-2xl overflow-hidden rounded-3xl border border-white/70 bg-white shadow-2xl" role="dialog" aria-modal="true" aria-labelledby="tutorial-title">
      <div class="bg-gradient-to-r from-emerald-800 to-teal-700 px-6 py-5 text-white sm:px-8">
        <div class="flex items-start justify-between gap-4">
          <div><p class="text-sm font-bold uppercase tracking-widest text-emerald-100">Panduan OASIS One</p><h2 id="tutorial-title" class="mt-1 text-2xl font-extrabold sm:text-3xl">{{ step.title }}</h2></div>
          <button type="button" @click="$emit('close')" class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-white/15 text-2xl font-bold text-white hover:bg-white/25" aria-label="Tutup tutorial">×</button>
        </div>
        <div class="mt-5 flex gap-1.5"><span v-for="(_, index) in steps" :key="index" class="h-2 flex-1 rounded-full" :class="index <= currentStep ? 'bg-amber-300' : 'bg-white/25'"></span></div>
        <p class="mt-2 text-sm font-semibold text-emerald-100">Langkah {{ currentStep + 1 }} dari {{ steps.length }}</p>
      </div>
      <div class="space-y-6 px-6 py-7 sm:px-10 sm:py-9">
        <div class="flex items-start gap-5"><div class="flex h-16 w-16 shrink-0 items-center justify-center rounded-2xl bg-amber-100 text-3xl font-extrabold text-amber-800 shadow-inner">{{ step.number }}</div><p class="text-lg font-semibold leading-relaxed text-slate-800 sm:text-xl">{{ step.text }}</p></div>
        <div class="rounded-2xl border-2 border-emerald-100 bg-emerald-50 px-5 py-4"><p class="text-base font-bold text-emerald-900">Bantuan untuk langkah ini</p><p class="mt-1 text-base leading-relaxed text-emerald-800">{{ step.tip }}</p></div>
        <div class="flex flex-col-reverse gap-3 border-t border-slate-100 pt-5 sm:flex-row sm:items-center sm:justify-between"><button type="button" @click="$emit('close')" class="rounded-xl px-4 py-3 text-base font-bold text-slate-600 hover:bg-slate-100">Lewati tutorial</button><div class="flex gap-3"><button v-if="currentStep > 0" type="button" @click="previous" class="rounded-xl border-2 border-slate-200 px-5 py-3 text-base font-bold text-slate-700 hover:bg-slate-50">Kembali</button><button type="button" @click="isLastStep ? $emit('close') : next()" class="rounded-xl bg-emerald-800 px-6 py-3 text-base font-bold text-white shadow-lg shadow-emerald-900/20 hover:bg-emerald-900">{{ isLastStep ? 'Selesai' : 'Berikutnya' }}</button></div></div>
      </div>
    </section>
  </div>
</template>
