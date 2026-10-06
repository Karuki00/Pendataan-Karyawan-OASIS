<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import * as XLSX from "xlsx";
import type { Employee } from "../types";

const REQUIRED_FIELDS = [
  "Nomor Induk Kependudukan (NIK)",
  "Nama Lengkap",
  "Divisi",
  "Jabatan",
  "Status",
  "Jenis Kelamin",
];
const MINIMUM_SHEET_SCORE = 0.5;
const NUMBER_FIELDS = new Set<keyof Employee>([
  "nik",
  "nomor_kk",
  "bpjs_kesehatan",
  "bpjs_ketenagakerjaan",
  "nomor_telepon",
]);

const emit = defineEmits<{ saved: [] }>();
const fileInput = ref<HTMLInputElement | null>(null);
const preview = ref<Employee[]>([]);
const fileName = ref("");
const isDragging = ref(false);
const isSaving = ref(false);
const errorMessage = ref("");
const successMessage = ref("");
const workbook = ref<XLSX.WorkBook | null>(null);
const sheetNames = ref<string[]>([]);
const selectedSheetName = ref("");
const autoSelectedSheetName = ref("");
const selectedSheetScore = ref(0);
const sheetScores = ref<Record<string, number>>({});

const fields: Array<keyof Employee> = [
  "nama_lengkap", "jenis_kelamin", "tanggal_lahir", "golongan_darah", "nik", "nomor_kk",
  "alamat_ktp", "bpjs_kesehatan", "bpjs_ketenagakerjaan", "nomor_telepon", "jabatan",
  "divisi", "nama_ibu_kandung", "nama_pasangan", "jumlah_anak", "nomor_telp_keluarga",
  "foto_ktp", "foto_kk", "foto_bpjs_kesehatan", "foto_bpjs_ketenagakerjaan", "pendidikan_terakhir",
  "perjanjian_kerja", "status", "join_date", "tanggal_kartap", "tempat_kerja",
];

const headerMap: Record<string, keyof Employee> = {
  timestamp: "timestamp", "nama lengkap": "nama_lengkap", "jenis kelamin": "jenis_kelamin",
  "join date": "join_date", "tanggal join": "join_date", "tanggal kartap": "tanggal_kartap",
  "tanggal lahir": "tanggal_lahir", "golongan darah": "golongan_darah",
  "nomor induk kependudukan (nik)": "nik", nik: "nik", "nomor kartu keluarga (kk)": "nomor_kk",
  kk: "nomor_kk", "alamat sesuai ktp": "alamat_ktp", "nomor bpjs kesehatan": "bpjs_kesehatan",
  "nomor bpjs ketenagakerjaan": "bpjs_ketenagakerjaan", "nomor telepon aktif (hp)": "nomor_telepon",
  jabatan: "jabatan", divisi: "divisi", "nama ibu kandung": "nama_ibu_kandung",
  "nama istri/suami": "nama_pasangan", "jumlah anak keseluruhan": "jumlah_anak",
  "nomor telp keluarga": "nomor_telp_keluarga", "scan/foto ktp": "foto_ktp",
  "scan/foto kk": "foto_kk", "scan/foto kartu bpjs kesehatan": "foto_bpjs_kesehatan",
  "scan/foto kartu bpjs ketenagakerjaan (bpjs-tk)": "foto_bpjs_ketenagakerjaan",
  "pendidikan terakhir": "pendidikan_terakhir",
  "perjanjian kerja": "perjanjian_kerja", "status": "status", "tempat kerja": "tempat_kerja",
};

function normalize(value: unknown): string {
  if (value === undefined || value === null) return "";
  return String(value).trim();
}

function normalizeHeader(header: unknown): string {
  return normalize(header).toLowerCase().replace(/\s+/g, " ");
}

function sanitizeNumberString(value: unknown): string {
  return normalize(value).replace(/\.0+$/, "");
}

function parseStatus(rawStatus: unknown): string {
  if (!rawStatus) return "AKTIF";
  const status = String(rawStatus).trim().toUpperCase();
  if (status.includes("PENSIUN")) return "PENSIUN";
  if (status.includes("RESIGN") || status.includes("KELUAR")) return "RESIGN";
  if (status.includes("AKTIF")) return "AKTIF";
  return status;
}

function inferDivision(jabatan: string): string {
  const normalizedJabatan = jabatan.toLowerCase();
  if (normalizedJabatan.includes("security") || normalizedJabatan.includes("satpam")) return "SECURITY";
  if (normalizedJabatan.includes("teknisi") || normalizedJabatan.includes("engineer")) return "ENGINEERING";
  if (normalizedJabatan.includes("cleaning") || normalizedJabatan.includes("kebersihan")) return "HOUSEKEEPING";
  if (normalizedJabatan.includes("admin") || normalizedJabatan.includes("administr")) return "ADMINISTRATION";
  return "";
}

function mapRow(row: Record<string, unknown>): Employee {
  const employee: Partial<Employee> = {};
  Object.entries(row).forEach(([header, value]) => {
    const normalizedHeader = normalizeHeader(header);
    const field = headerMap[normalizedHeader];
    if (field) {
      employee[field] = field === "status"
        ? parseStatus(value)
        : NUMBER_FIELDS.has(field) ? sanitizeNumberString(value) : normalize(value);
    }
  });
  fields.forEach((field) => { if (!(field in employee)) employee[field] = ""; });
  employee.divisi = employee.divisi || inferDivision(normalize(employee.jabatan));
  employee.perjanjian_kerja = employee.perjanjian_kerja || "PKWTT";
  employee.status = parseStatus(employee.status);
  return employee as Employee;
}

function getSheetHeaders(sheet: XLSX.WorkSheet): string[] {
  const rows = XLSX.utils.sheet_to_json<unknown[]>(sheet, { header: 1, defval: "" });
  const firstRow = rows[0];
  return Array.isArray(firstRow) ? firstRow.map(normalizeHeader).filter(Boolean) : [];
}

function getSheetScore(sheet: XLSX.WorkSheet): number {
  const headers = new Set(getSheetHeaders(sheet));
  const matchedFields = REQUIRED_FIELDS.filter((field) => headers.has(normalizeHeader(field))).length;
  return matchedFields / REQUIRED_FIELDS.length;
}

function processSheet(sheetName: string, showAlert = true): boolean {
  if (!workbook.value) return false;
  const sheet = workbook.value.Sheets[sheetName];
  if (!sheet) return false;

  const score = getSheetScore(sheet);
  selectedSheetName.value = sheetName;
  selectedSheetScore.value = score;
  if (score < MINIMUM_SHEET_SCORE) {
    preview.value = [];
    if (showAlert) window.alert("Tidak ditemukan sheet dengan kolom data karyawan yang lengkap.");
    return false;
  }
  const rows = XLSX.utils.sheet_to_json<Record<string, unknown>>(sheet, { defval: "" });
  preview.value = rows.map(mapRow).filter((row) => row.nik && row.nama_lengkap);
  if (!preview.value.length) {
    if (showAlert) errorMessage.value = "Tidak ada baris valid dengan NIK dan Nama Lengkap.";
    return false;
  }
  return true;
}

function onSheetChange() {
  errorMessage.value = "";
  successMessage.value = "";
  processSheet(selectedSheetName.value);
}

async function readFile(file: File) {
  errorMessage.value = "";
  successMessage.value = "";
  try {
    const parsedWorkbook = XLSX.read(await file.arrayBuffer(), { type: "array", cellDates: true });
    if (!parsedWorkbook.SheetNames.length) throw new Error("File Excel tidak memiliki sheet.");
    workbook.value = parsedWorkbook;
    sheetNames.value = parsedWorkbook.SheetNames;
    sheetScores.value = Object.fromEntries(
      sheetNames.value.map((sheetName) => [sheetName, getSheetScore(parsedWorkbook.Sheets[sheetName])]),
    );
    const bestSheet = sheetNames.value.reduce((best, sheetName) =>
      sheetScores.value[sheetName] > sheetScores.value[best] ? sheetName : best,
    );
    autoSelectedSheetName.value = bestSheet;
    selectedSheetName.value = bestSheet;
    fileName.value = file.name;
    if (!processSheet(bestSheet)) return;
  } catch (error) {
    preview.value = [];
    errorMessage.value = error instanceof Error ? error.message : "File tidak dapat dibaca.";
  }
}

function onFileChange(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (file) void readFile(file);
}

function onDrop(event: DragEvent) {
  isDragging.value = false;
  const file = event.dataTransfer?.files[0];
  if (file) void readFile(file);
}

async function savePreview() {
  isSaving.value = true;
  errorMessage.value = "";
  successMessage.value = "";
  try {
    const count = await invoke<number>("upsert_employees", { employees: preview.value });
    successMessage.value = `${count} data berhasil disimpan atau diperbarui.`;
    preview.value = [];
    emit("saved");
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isSaving.value = false;
  }
}
</script>

<template>
  <section class="space-y-6">
    <div
      class="rounded-2xl border-2 border-dashed p-10 text-center transition"
      :class="isDragging ? 'border-blue-500 bg-blue-50' : 'border-slate-300 bg-white'"
      @dragover.prevent="isDragging = true"
      @dragleave.prevent="isDragging = false"
      @drop.prevent="onDrop"
    >
      <p class="text-lg font-semibold text-slate-800">Impor respons Google Forms</p>
      <p class="mt-2 text-sm text-slate-500">Seret file .xlsx ke sini atau pilih file dari komputer.</p>
      <button class="mt-5 rounded-lg bg-blue-600 px-4 py-2 font-medium text-white hover:bg-blue-700" @click="fileInput?.click()">
        Pilih File Excel
      </button>
      <input ref="fileInput" class="hidden" type="file" accept=".xlsx,.xls" @change="onFileChange">
      <p v-if="fileName" class="mt-3 text-sm text-slate-600">{{ fileName }}</p>
    </div>
    <div v-if="sheetNames.length" class="rounded-lg border border-blue-200 bg-blue-50 p-4 text-sm text-blue-900">
      <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <p class="font-medium">
            Sheet terpilih secara otomatis: "{{ autoSelectedSheetName }}"
            (Skor Kelengkapan: {{ Math.round((sheetScores[autoSelectedSheetName] ?? 0) * 100) }}%)
          </p>
          <p v-if="selectedSheetName !== autoSelectedSheetName" class="mt-1 text-blue-700">
            Override manual: "{{ selectedSheetName }}" (Skor: {{ Math.round(selectedSheetScore * 100) }}%)
          </p>
        </div>
        <label class="flex items-center gap-2 font-medium">
          <span class="whitespace-nowrap">Pilih sheet:</span>
          <select v-model="selectedSheetName" class="rounded-md border border-blue-300 bg-white px-2 py-1.5" @change="onSheetChange">
            <option v-for="sheetName in sheetNames" :key="sheetName" :value="sheetName">
              {{ sheetName }} ({{ Math.round((sheetScores[sheetName] ?? 0) * 100) }}%)
            </option>
          </select>
        </label>
      </div>
    </div>
    <p v-if="errorMessage" class="rounded-lg bg-red-50 p-3 text-sm text-red-700">{{ errorMessage }}</p>
    <p v-if="successMessage" class="rounded-lg bg-emerald-50 p-3 text-sm text-emerald-700">{{ successMessage }}</p>
    <div v-if="preview.length" class="overflow-hidden rounded-2xl border border-slate-200 bg-white">
      <div class="flex items-center justify-between border-b border-slate-200 p-4">
        <div><h2 class="font-semibold text-slate-800">Preview Data</h2><p class="text-sm text-slate-500">{{ preview.length }} baris siap diimpor</p></div>
        <button class="rounded-lg bg-emerald-600 px-4 py-2 font-medium text-white hover:bg-emerald-700 disabled:opacity-50" :disabled="isSaving" @click="savePreview">
          {{ isSaving ? "Menyimpan..." : "Simpan ke Database Master" }}
        </button>
      </div>
      <div class="max-h-[28rem] overflow-auto">
        <table class="min-w-full text-left text-sm">
          <thead class="sticky top-0 bg-slate-100 text-xs uppercase text-slate-500"><tr><th class="px-4 py-3">NIK</th><th class="px-4 py-3">Nama</th><th class="px-4 py-3">Divisi</th><th class="px-4 py-3">Jabatan</th><th class="px-4 py-3">Status</th></tr></thead>
          <tbody><tr v-for="row in preview" :key="row.nik" class="border-t border-slate-100"><td class="whitespace-nowrap px-4 py-3">{{ row.nik }}</td><td class="px-4 py-3 font-medium">{{ row.nama_lengkap }}</td><td class="px-4 py-3">{{ row.divisi || "-" }}</td><td class="px-4 py-3">{{ row.jabatan || "-" }}</td><td class="px-4 py-3">{{ row.status || "AKTIF" }}</td></tr></tbody>
        </table>
      </div>
    </div>
  </section>
</template>
