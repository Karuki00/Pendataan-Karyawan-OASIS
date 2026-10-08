<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Employee } from "../types";

defineProps<{ employee: Employee }>();
const emit = defineEmits<{ close: [] }>();

function value(val: string | null | undefined): string {
  return val?.trim() || "-";
}

async function openDocument(url: string | null | undefined) {
  if (url) await openUrl(url);
}
</script>

<template>
  <div class="fixed inset-0 z-20 flex items-center justify-center bg-slate-900/50 p-4" @click.self="emit('close')">
    <aside class="max-h-[92vh] w-full max-w-4xl overflow-y-auto rounded-2xl bg-white shadow-2xl" role="dialog" aria-modal="true" aria-label="Detail data karyawan">
      <div class="sticky top-0 z-10 flex items-start justify-between border-b border-slate-200 bg-white px-6 py-5">
        <div>
          <p class="text-xs font-semibold uppercase tracking-wider text-blue-600">Detail Karyawan</p>
          <h2 class="mt-1 text-2xl font-bold text-slate-900">{{ employee.nama_lengkap }}</h2>
          <p class="mt-1 text-sm text-slate-500">{{ value(employee.divisi) }} · {{ value(employee.jabatan) }}</p>
        </div>
        <button type="button" class="rounded-lg p-2 text-2xl leading-none text-slate-400 hover:bg-slate-100 hover:text-slate-700" aria-label="Tutup detail" @click="emit('close')">&times;</button>
      </div>

      <div class="grid gap-4 p-6 md:grid-cols-2">
        <section class="rounded-xl border border-slate-200 bg-slate-50 p-5">
          <h3 class="mb-4 font-semibold text-slate-900">Data Pribadi</h3>
          <dl class="grid gap-3 sm:grid-cols-2">
            <div><dt class="text-xs text-slate-500">Nama Lengkap</dt><dd class="font-medium">{{ value(employee.nama_lengkap) }}</dd></div>
            <div><dt class="text-xs text-slate-500">NIK</dt><dd class="font-medium">{{ value(employee.nik) }}</dd></div>
            <div><dt class="text-xs text-slate-500">No. KK</dt><dd>{{ value(employee.nomor_kk) }}</dd></div>
            <div><dt class="text-xs text-slate-500">Jenis Kelamin</dt><dd>{{ value(employee.jenis_kelamin) }}</dd></div>
            <div><dt class="text-xs text-slate-500">Tanggal Lahir</dt><dd>{{ value(employee.tanggal_lahir) }}</dd></div>
            <div><dt class="text-xs text-slate-500">Gol. Darah</dt><dd>{{ value(employee.golongan_darah) }}</dd></div>
            <div><dt class="text-xs text-slate-500">Pendidikan Terakhir</dt><dd>{{ value(employee.pendidikan_terakhir) }}</dd></div>
            <div><dt class="text-xs text-slate-500">No. Telepon</dt><dd>{{ value(employee.nomor_telepon) }}</dd></div>
            <div class="sm:col-span-2"><dt class="text-xs text-slate-500">Alamat KTP</dt><dd>{{ value(employee.alamat_ktp) }}</dd></div>
          </dl>
        </section>

        <section class="rounded-xl border border-slate-200 bg-slate-50 p-5">
          <h3 class="mb-4 font-semibold text-slate-900">Status Pekerjaan</h3>
          <dl class="grid gap-3 sm:grid-cols-2">
            <div><dt class="text-xs text-slate-500">Divisi</dt><dd>{{ value(employee.divisi) }}</dd></div>
            <div><dt class="text-xs text-slate-500">Jabatan</dt><dd>{{ value(employee.jabatan) }}</dd></div>
            <div><dt class="text-xs text-slate-500">Perjanjian Kerja</dt><dd>{{ value(employee.perjanjian_kerja || "PKWTT") }}</dd></div>
            <div><dt class="text-xs text-slate-500">Status</dt><dd>{{ value(employee.status || "AKTIF") }}</dd></div>
            <div><dt class="text-xs text-slate-500">Join Date</dt><dd>{{ value(employee.join_date) }}</dd></div>
            <div><dt class="text-xs text-slate-500">Tanggal Kartap</dt><dd>{{ value(employee.tanggal_kartap) }}</dd></div>
            <div class="sm:col-span-2"><dt class="text-xs text-slate-500">Tempat Kerja</dt><dd>{{ value(employee.tempat_kerja) }}</dd></div>
          </dl>
        </section>

        <section class="rounded-xl border border-slate-200 bg-slate-50 p-5">
          <h3 class="mb-4 font-semibold text-slate-900">Informasi Keluarga</h3>
          <dl class="grid gap-3 sm:grid-cols-2">
            <div><dt class="text-xs text-slate-500">Nama Ibu Kandung</dt><dd>{{ value(employee.nama_ibu_kandung) }}</dd></div>
            <div><dt class="text-xs text-slate-500">Pasangan (Istri/Suami)</dt><dd>{{ value(employee.nama_pasangan) }}</dd></div>
            <div><dt class="text-xs text-slate-500">Jumlah Anak</dt><dd>{{ value(String(employee.jumlah_anak ?? "-")) }}</dd></div>
            <div><dt class="text-xs text-slate-500">No. Telp Keluarga</dt><dd>{{ value(employee.nomor_telp_keluarga) }}</dd></div>
          </dl>
        </section>

        <section class="rounded-xl border border-slate-200 bg-slate-50 p-5">
          <h3 class="mb-4 font-semibold text-slate-900">Dokumen Lampiran</h3>
          <div class="grid gap-2 sm:grid-cols-2">
            <button v-for="[label, url] in [['KTP', employee.foto_ktp], ['KK', employee.foto_kk], ['BPJS Kesehatan', employee.foto_bpjs_kesehatan], ['BPJS TK', employee.foto_bpjs_ketenagakerjaan]]" :key="label" type="button" class="rounded-lg border border-slate-300 bg-white px-3 py-2 text-left text-sm font-medium text-blue-700 transition hover:border-blue-400 hover:bg-blue-50 disabled:cursor-not-allowed disabled:text-slate-400" :disabled="!url" @click="openDocument(url)">
              <span class="mr-2">↗</span>{{ label }}<span class="block text-xs font-normal text-slate-500">{{ url ? "Buka Google Drive" : "Belum tersedia" }}</span>
            </button>
          </div>
        </section>
      </div>

      <div class="flex justify-end border-t border-slate-200 px-6 py-4">
        <button type="button" class="rounded-lg border border-slate-300 px-4 py-2 font-medium text-slate-700 hover:bg-slate-50" @click="emit('close')">Tutup</button>
      </div>
    </aside>
  </div>
</template>