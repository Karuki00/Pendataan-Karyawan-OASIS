<script setup lang="ts">
import { computed, ref } from "vue";
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

type RowStatus = "READY" | "WARNING" | "CRITICAL";

interface EvaluatedRow {
  data: Employee;
  status: RowStatus;
  reasons: string[];
}

const emit = defineEmits<{ saved: [] }>();
const fileInput = ref<HTMLInputElement | null>(null);

const evaluatedRows = ref<EvaluatedRow[]>([]);

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
  "perjanjian_kerja", "status", "join_date", "tanggal_kartap", "tempat_kerja", "is_flagged"
];

const headerMap: Record<string, keyof Employee> = {
  "nama lengkap": "nama_lengkap", "jenis kelamin": "jenis_kelamin",
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
  "perjanjian kerja": "perjanjian_kerja", status: "status", "tempat kerja": "tempat_kerja",
};

const importableRows = computed(() => 
  evaluatedRows.value.filter((r) => r.status === "READY" || r.status === "WARNING")
);
const warningRows = computed(() => 
  evaluatedRows.value.filter((r) => r.status === "WARNING")
);
const criticalRows = computed(() => 
  evaluatedRows.value.filter((r) => r.status === "CRITICAL")
);

function normalize(value: unknown): string {
  if (value === undefined || value === null) return "";
  return String(value).trim();
}

function normalizeHeader(header: unknown): string {
  return normalize(header).toLowerCase().replace(/\s+/g, " ");
}

function sanitizeNumberString(value: unknown): string {
  if (value === undefined || value === null) return "";
  let str = String(value).trim().replace(/\.0+$/, "");
  return str.replace(/\D/g, "");
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
  if (normalizedJabatan.includes("admin") || normalizedJabatan.includes("administr")) return "STAFF";
  return "";
}

function mapRow(row: Record<string, unknown>): Employee {
  const employee: Record<string, unknown> = {};

  Object.entries(row).forEach(([header, value]) => {
    const normalizedHeader = normalizeHeader(header);
    const field = headerMap[normalizedHeader];
    if (field) {
      employee[field] = field === "status"
        ? parseStatus(value)
        : NUMBER_FIELDS.has(field) ? sanitizeNumberString(value) : normalize(value);
    }
  });

  fields.forEach((field) => {
    if (!(field in employee)) {
      employee[field] = field === "is_flagged" ? false : "";
    }
  });

  employee.divisi = employee.divisi || inferDivision(normalize(employee.jabatan));
  employee.perjanjian_kerja = employee.perjanjian_kerja || "PKWTT";
  employee.status = parseStatus(employee.status);

  return employee as unknown as Employee;
}

function evaluateRow(employee: Employee): { status: RowStatus; reasons: string[]; isFlagged: boolean } {
  const cleanNama = (employee.nama_lengkap || "").trim();
  const cleanNik = (employee.nik || "").trim();
  const cleanKk = (employee.nomor_kk || "").trim();
  const reasons: string[] = [];

  // Syarat Mutlak HRD: NIK & Nama tidak boleh kosong
  if (!cleanNik || !cleanNama) {
    const criticalReasons: string[] = [];
    if (!cleanNik) criticalReasons.push("NIK Wajib Diisi (Kosong)");
    if (!cleanNama) criticalReasons.push("Nama Lengkap Kosong");
    return { status: "CRITICAL", reasons: criticalReasons, isFlagged: true };
  }

  // Peringatan NIK (Harus 16 Digit)
  if (cleanNik.length !== 16) {
    reasons.push(`NIK (${cleanNik}) berjumlah ${cleanNik.length} digit (harus 16 digit)`);
  }

  // Peringatan KK (HANYA jika diisi dan jumlah digit bukan 16)
  if (cleanKk && cleanKk.length !== 16) {
    reasons.push(`Nomor KK (${cleanKk}) berjumlah ${cleanKk.length} digit (harus 16 digit)`);
  }

  // Peringatan BPJS Kesehatan / Ketenagakerjaan
  if (!employee.bpjs_kesehatan || !employee.bpjs_ketenagakerjaan) {
    reasons.push("Nomor BPJS belum lengkap");
  }

  const isFlagged = reasons.length > 0;
  if (isFlagged) {
    return { status: "WARNING", reasons, isFlagged: true };
  }

  return { status: "READY", reasons: ["Data lengkap & valid"], isFlagged: false };
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
    evaluatedRows.value = [];
    if (showAlert) window.alert("Tidak ditemukan sheet dengan kolom data karyawan yang lengkap.");
    return false;
  }

  const rows = XLSX.utils.sheet_to_json<Record<string, unknown>>(sheet, { defval: "", raw: false });
  
  evaluatedRows.value = rows.map((r) => {
    const mapped = mapRow(r);
    const evaluation = evaluateRow(mapped);
    mapped.is_flagged = evaluation.isFlagged;
    return {
      data: mapped,
      status: evaluation.status,
      reasons: evaluation.reasons,
    };
  });

  if (!evaluatedRows.value.length) {
    if (showAlert) errorMessage.value = "Tidak ada baris data dalam sheet ini.";
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
    evaluatedRows.value = [];
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
  if (!importableRows.value.length) {
    errorMessage.value = "Tidak ada data valid dengan NIK yang dapat disimpan.";
    return;
  }

  isSaving.value = true;
  errorMessage.value = "";
  successMessage.value = "";
  try {
    const employeesToSave = importableRows.value.map((r) => r.data);
    const count = await invoke<number>("upsert_employees", { employees: employeesToSave });
    
    successMessage.value = `${count} data karyawan berhasil disimpan/diperbarui ke SQLite!`;
    evaluatedRows.value = [];
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
      class="rounded-3xl border-2 border-dashed p-8 text-center transition-all cursor-pointer"
      :class="isDragging ? 'border-emerald-500 bg-emerald-50/50 scale-[1.01]' : 'border-slate-300 bg-slate-50/50 hover:bg-slate-100/80'"
      @dragover.prevent="isDragging = true"
      @dragleave.prevent="isDragging = false"
      @drop.prevent="onDrop"
      @click="fileInput?.click()"
    >
      <div class="mx-auto flex h-12 w-12 items-center justify-center rounded-2xl bg-emerald-100 text-emerald-800 text-2xl font-bold mb-3 shadow-xs">
        📥
      </div>
      <p class="text-base font-bold text-slate-800">Impor Berkas Excel Data Karyawan</p>
      <p class="mt-1 text-xs text-slate-500">Seret berkas .xlsx ke sini atau klik tombol di bawah ini</p>
      
      <button 
        type="button"
        class="mt-4 inline-flex items-center gap-2 rounded-xl bg-slate-900 px-4 py-2.5 text-xs font-semibold text-white hover:bg-slate-800 transition-colors shadow-sm cursor-pointer" 
        @click.stop="fileInput?.click()"
      >
        <span>📄 Pilih Berkas Excel</span>
      </button>
      <input ref="fileInput" class="hidden" type="file" accept=".xlsx,.xls" @change="onFileChange">
      <p v-if="fileName" class="mt-3 text-xs font-mono font-semibold text-emerald-700">📁 Berkas terpilih: {{ fileName }}</p>
    </div>

    <div v-if="sheetNames.length" class="rounded-2xl border border-emerald-200 bg-emerald-50/60 p-4 text-xs text-emerald-950 space-y-2">
      <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <p class="font-bold flex items-center gap-1.5 text-sm text-emerald-900">
            <span>🔍</span> Sheet Terpilih Otomatis: "{{ autoSelectedSheetName }}"
          </p>
          <p class="text-[11px] text-emerald-700 mt-0.5">
            Skor Kelengkapan Kolom: <strong class="font-mono">{{ Math.round((sheetScores[autoSelectedSheetName] ?? 0) * 100) }}%</strong>
          </p>
        </div>
        
        <label class="flex items-center gap-2 font-semibold">
          <span class="whitespace-nowrap">Ganti Sheet:</span>
          <select v-model="selectedSheetName" class="rounded-xl border border-emerald-300 bg-white px-3 py-1.5 text-xs font-semibold outline-none focus:ring-2 focus:ring-emerald-500/20" @change="onSheetChange">
            <option v-for="sheetName in sheetNames" :key="sheetName" :value="sheetName">
              {{ sheetName }} ({{ Math.round((sheetScores[sheetName] ?? 0) * 100) }}%)
            </option>
          </select>
        </label>
      </div>
    </div>

    <p v-if="errorMessage" class="rounded-xl bg-rose-50 p-4 text-xs font-semibold text-rose-700 border border-rose-200">⚠️ {{ errorMessage }}</p>
    <p v-if="successMessage" class="rounded-xl bg-emerald-50 p-4 text-xs font-semibold text-emerald-700 border border-emerald-200">✅ {{ successMessage }}</p>

    <div v-if="evaluatedRows.length" class="space-y-4 overflow-hidden rounded-2xl border border-slate-200/80 bg-white p-5 shadow-sm">
      <div class="flex flex-wrap items-center justify-between gap-3 border-b border-slate-100 pb-4">
        <div>
          <h3 class="font-bold text-slate-900 text-sm flex items-center gap-2">
            Pratinjau Impor Data Karyawan
          </h3>
          <p class="text-xs text-slate-500 mt-0.5">Data bertanda Peringatan tetap diizinkan masuk ke database (is_flagged = true)</p>
        </div>

        <div class="flex items-center gap-3">
          <div class="flex items-center gap-1.5 text-xs font-semibold">
            <span class="rounded-full bg-emerald-100 px-2.5 py-1 text-emerald-800 border border-emerald-200">
              ✓ Siap: {{ evaluatedRows.filter(r => r.status === 'READY').length }}
            </span>
            <span v-if="warningRows.length > 0" class="rounded-full bg-amber-100 px-2.5 py-1 text-amber-800 border border-amber-200">
              ⚠️ Peringatan: {{ warningRows.length }}
            </span>
            <span v-if="criticalRows.length > 0" class="rounded-full bg-rose-100 px-2.5 py-1 text-rose-800 border border-rose-200">
              ✕ Ditolak (Tanpa NIK): {{ criticalRows.length }}
            </span>
          </div>

          <button 
            class="inline-flex items-center gap-2 rounded-xl bg-emerald-600 px-5 py-2.5 text-xs font-semibold text-white shadow-md shadow-emerald-600/20 hover:bg-emerald-700 disabled:opacity-50 transition-all cursor-pointer" 
            :disabled="isSaving || !importableRows.length" 
            @click="savePreview"
          >
            <svg v-if="isSaving" class="h-4 w-4 animate-spin text-white" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
            </svg>
            <span>{{ isSaving ? "Menyimpan..." : `Proses Impor ${importableRows.length} Data` }}</span>
          </button>
        </div>
      </div>

      <div class="max-h-[28rem] overflow-auto rounded-xl border border-slate-100">
        <table class="min-w-full text-left text-xs">
          <thead class="sticky top-0 bg-slate-50 text-slate-500 font-semibold uppercase tracking-wider border-b border-slate-100 z-10">
            <tr>
              <th class="px-3.5 py-3">Status Impor</th>
              <th class="px-3.5 py-3">NIK</th>
              <th class="px-3.5 py-3">Nama Lengkap</th>
              <th class="px-3.5 py-3">Divisi / Jabatan</th>
              <th class="px-3.5 py-3">Status Kerja</th>
              <th class="px-3.5 py-3">Hasil Evaluasi</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100">
            <tr 
              v-for="(row, index) in evaluatedRows" 
              :key="index" 
              class="transition-colors"
              :class="{
                'hover:bg-slate-50': row.status === 'READY',
                'bg-amber-50/30 hover:bg-amber-50/60': row.status === 'WARNING',
                'bg-rose-50/50 hover:bg-rose-50/80': row.status === 'CRITICAL'
              }"
            >
              <td class="px-3.5 py-2.5 font-bold whitespace-nowrap">
                <span 
                  class="inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-[10px]"
                  :class="{
                    'bg-emerald-100 text-emerald-800': row.status === 'READY',
                    'bg-amber-100 text-amber-800': row.status === 'WARNING',
                    'bg-rose-100 text-rose-800': row.status === 'CRITICAL'
                  }"
                >
                  <template v-if="row.status === 'READY'">✓ Siap</template>
                  <template v-else-if="row.status === 'WARNING'">⚠️ Masuk (Ada Catatan)</template>
                  <template v-else>✕ Ditolak (Tanpa NIK)</template>
                </span>
              </td>
              <td class="whitespace-nowrap px-3.5 py-2.5 font-mono text-slate-700 font-semibold">
                {{ row.data.nik || "KOSONG" }}
              </td>
              <td class="px-3.5 py-2.5 font-medium text-slate-900">{{ row.data.nama_lengkap || "-" }}</td>
              <td class="px-3.5 py-2.5 text-slate-600">
                <span class="font-semibold text-slate-800">{{ row.data.divisi || "-" }}</span> 
                <span class="text-slate-400">/</span> {{ row.data.jabatan || "-" }}
              </td>
              <td class="px-3.5 py-2.5 text-slate-600 font-semibold">{{ row.data.status || "AKTIF" }}</td>
              <td class="px-3.5 py-2.5">
                <span 
                  class="font-medium text-xs"
                  :class="{
                    'text-slate-400': row.status === 'READY',
                    'text-amber-800 font-semibold': row.status === 'WARNING',
                    'text-rose-700 font-bold': row.status === 'CRITICAL'
                  }"
                >
                  {{ row.reasons.join(", ") }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </section>
</template>