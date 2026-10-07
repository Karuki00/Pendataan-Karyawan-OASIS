<script setup lang="ts">
import { reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import DashboardView, { type MetricFilterKey } from "./components/DashboardView.vue";
import ExcelImporter from "./components/ExcelImporter.vue";
import EmployeeTable from "./components/EmployeeTable.vue";
import AboutModal from "./components/AboutModal.vue";
import tauriConfig from "../src-tauri/tauri.conf.json"; // Path relatif sesuai lokasi App.vue
import type { Employee } from "./types";
import logoUrl from "../src/assets/LOGO_OASIS_V1-removebg-preview.png";

type Tab = "dashboard" | "master";
const activeTab = ref<Tab>("dashboard");
const refreshKey = ref(0);
const isModalOpen = ref(false);
const isImportModalOpen = ref(false);
const masterSearchQuery = ref("");
const activeMetricKey = ref<MetricFilterKey | null>(null);
const isSaving = ref(false);
const errorMessage = ref("");
const showAboutModal = ref(false);
const appVersion = tauriConfig.version;

const emptyEmployee = (): Employee => ({
  nik: "", nama_lengkap: "", jenis_kelamin: "", tanggal_lahir: "", golongan_darah: "",
  nomor_kk: "", alamat_ktp: "", bpjs_kesehatan: "", bpjs_ketenagakerjaan: "", nomor_telepon: "",
  jabatan: "", divisi: "", nama_ibu_kandung: "", nama_pasangan: "", jumlah_anak: "",
  nomor_telp_keluarga: "", foto_ktp: "", foto_kk: "", foto_bpjs_kesehatan: "",
  foto_bpjs_ketenagakerjaan: "", pendidikan_terakhir: "", perjanjian_kerja: "PKWTT",
  status: "AKTIF", tempat_kerja: "Lapangan",
});
const form = reactive<Employee>(emptyEmployee());
const formFields: Array<{
  key: keyof Employee;
  label: string;
  type?: string;
  options?: string[];
}> = [
  { key: "nik", label: "NIK *" },
  { key: "nama_lengkap", label: "Nama Lengkap *" },
  { key: "jenis_kelamin", label: "Jenis Kelamin", options: ["Laki-laki", "Perempuan"] },
  { key: "tanggal_lahir", label: "Tanggal Lahir", type: "date" },
  { key: "golongan_darah", label: "Golongan Darah", options: ["A", "B", "AB", "O"] },
  { key: "nomor_kk", label: "Nomor KK" },
  { key: "bpjs_kesehatan", label: "Nomor BPJS Kesehatan" },
  { key: "bpjs_ketenagakerjaan", label: "Nomor BPJS Ketenagakerjaan" },
  { key: "nomor_telepon", label: "Nomor Telepon" },
  { key: "jabatan", label: "Jabatan" },
  { key: "divisi", label: "Divisi", options: ["SECURITY", "ENGINEERING", "HOUSEKEEPING", "STAFF"] },
  { key: "nama_ibu_kandung", label: "Nama Ibu Kandung" },
  { key: "nama_pasangan", label: "Nama Istri/Suami" },
  { key: "jumlah_anak", label: "Jumlah Anak", type: "number" },
  { key: "nomor_telp_keluarga", label: "Nomor Telp Keluarga" },
  { key: "pendidikan_terakhir", label: "Pendidikan Terakhir", options: ["SD", "SMP", "SMA", "SMK", "D3", "S1", "S2", "S3"] },
  { key: "perjanjian_kerja", label: "Perjanjian Kerja", options: ["PKWTT", "PKWT"] },
  { key: "status", label: "Status", options: ["AKTIF", "PENSIUN", "RESIGN"] },
  { key: "tempat_kerja", label: "Tempat Kerja", options: ["Lapangan", "Kantor"] },
];

function openCreate() {
  Object.assign(form, emptyEmployee());
  errorMessage.value = "";
  isModalOpen.value = true;
}
function openEdit(employee: Employee) {
  Object.assign(form, employee);
  errorMessage.value = "";
  isModalOpen.value = true;
}
async function saveEmployee() {
  isSaving.value = true;
  errorMessage.value = "";
  try {
    await invoke("save_employee", { employee: form });
    isModalOpen.value = false;
    refreshKey.value++;
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally { isSaving.value = false; }
}
function openImport() { isImportModalOpen.value = true; }
function openMaster() {
  masterSearchQuery.value = "";
  activeMetricKey.value = null;
  activeTab.value = "master";
}
function imported() { refreshKey.value++; isImportModalOpen.value = false; openMaster(); }
function openMasterWithFilter(filterQuery: string) {
  masterSearchQuery.value = filterQuery;
  activeMetricKey.value = null;
  activeTab.value = "master";
}
function selectMetric(filterKey: MetricFilterKey) {
  activeMetricKey.value = filterKey;
  masterSearchQuery.value = "";
  activeTab.value = "master";
}
function clearMetricFilter() {
  activeMetricKey.value = null;
}
function resetMasterSearch() {
  masterSearchQuery.value = "";
}
</script>

<template>
  <div class="min-h-screen bg-slate-100 text-slate-900">
    <header class="border-b border-slate-200 bg-white">
    <div class="mx-auto flex max-w-7xl items-center justify-between px-6 py-5">
      <div class="flex flex-col items-start gap-1">
        <img 
          :src="logoUrl" 
          alt="Logo Apartemen Oasis" 
          class="h-[13vh] w-auto object-contain"
        />
      <h1 class="text-2xl font-bold text-slate-900">Database</h1>
    </div>

      <div class="flex items-bottom gap-3">
        <!-- About Button Trigger (Versi Otomatis) -->
        <button 
          @click="showAboutModal = true"
          class="flex items-center gap-1.5 rounded-lg border border-slate-200 bg-slate-50 px-3 py-2 text-xs font-semibold text-slate-600 hover:bg-slate-100 hover:text-slate-900 transition-colors"
        >
          <span>ℹ️ Tentang Aplikasi</span>
          <span class="rounded bg-slate-200 px-1.5 py-0.5 text-[10px] text-slate-700 font-mono">
            v{{ appVersion }}
          </span>
        </button>
      </div>
    </div>
  </header>

  <AboutModal v-if="showAboutModal" @close="showAboutModal = false" />
    <main class="mx-auto max-w-7xl space-y-6 px-6 py-8">
      <nav class="flex gap-2 rounded-xl bg-white p-1 shadow-sm">
        <button class="rounded-lg px-4 py-2 font-medium" :class="activeTab === 'dashboard' ? 'bg-blue-600 text-white' : 'text-slate-600 hover:bg-slate-100'" @click="activeTab = 'dashboard'; masterSearchQuery = ''">📊 Dashboard &amp; Decision Center</button>
        <button class="rounded-lg px-4 py-2 font-medium" :class="activeTab === 'master' ? 'bg-blue-600 text-white' : 'text-slate-600 hover:bg-slate-100'" @click="openMaster">📋 Data Karyawan</button>
      </nav>
      <DashboardView v-if="activeTab === 'dashboard'" :refresh-key="refreshKey" @import="openImport" @add="openCreate" @open-master="openMaster" @open-master-with-filter="openMasterWithFilter" @select-metric="selectMetric" />
      <EmployeeTable v-if="activeTab === 'master'" :refresh-key="refreshKey" :initial-search="masterSearchQuery" :active-metric-key="activeMetricKey || undefined" @reset-search="resetMasterSearch" @edit="openEdit" @clear-metric-filter="clearMetricFilter" />
    </main>
    <div v-if="isImportModalOpen" class="fixed inset-0 z-20 flex items-center justify-center bg-slate-900/50 p-4" @click.self="isImportModalOpen = false">
      <div class="max-h-[92vh] w-full max-w-6xl overflow-auto rounded-2xl bg-slate-100 p-6 shadow-xl">
        <div class="mb-4 flex items-center justify-between"><h2 class="text-xl font-bold text-slate-900">Import Data Karyawan</h2><button type="button" class="text-2xl text-slate-400 hover:text-slate-700" aria-label="Tutup import" @click="isImportModalOpen = false">&times;</button></div>
        <ExcelImporter @saved="imported" />
      </div>
    </div>
    <div v-if="isModalOpen" class="fixed inset-0 z-10 flex items-center justify-center bg-slate-900/50 p-4" @click.self="isModalOpen = false">
      <form class="max-h-[90vh] w-full max-w-3xl overflow-auto rounded-2xl bg-white p-6 shadow-xl" @submit.prevent="saveEmployee">
        <div class="flex items-center justify-between"><h2 class="text-xl font-bold">Data Karyawan</h2><button type="button" class="text-2xl text-slate-400" @click="isModalOpen = false">&times;</button></div>
        <div class="mt-5 grid gap-4 sm:grid-cols-2">
          <label v-for="field in formFields" :key="field.key" class="text-sm font-medium text-slate-700">
            {{ field.label }}
            <select v-if="field.options" v-model="form[field.key]" class="mt-1 w-full rounded-lg border border-slate-300 bg-white px-3 py-2 font-normal outline-none focus:border-blue-500">
              <option value="">Pilih {{ field.label.replace(" *", "") }}</option>
              <option v-for="option in field.options" :key="option" :value="option">{{ option }}</option>
            </select>
            <input v-else v-model="form[field.key]" :type="field.type || 'text'" class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 font-normal outline-none focus:border-blue-500">
          </label>
          <label class="text-sm font-medium text-slate-700 sm:col-span-2">Alamat Sesuai KTP<textarea v-model="form.alamat_ktp" rows="2" class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 font-normal outline-none focus:border-blue-500" /></label>
          <label v-for="field in [{ key: 'foto_ktp', label: 'URL Foto KTP' }, { key: 'foto_kk', label: 'URL Foto KK' }, { key: 'foto_bpjs_kesehatan', label: 'URL Foto BPJS Kesehatan' }, { key: 'foto_bpjs_ketenagakerjaan', label: 'URL Foto BPJS Ketenagakerjaan' }]" :key="field.key" class="text-sm font-medium text-slate-700 sm:col-span-2">
            {{ field.label }}<input v-model="form[field.key as keyof Employee]" type="url" class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 font-normal outline-none focus:border-blue-500">
          </label>
        </div>
        <p v-if="errorMessage" class="mt-4 rounded-lg bg-red-50 p-3 text-sm text-red-700">{{ errorMessage }}</p>
        <div class="mt-6 flex justify-end gap-3"><button type="button" class="rounded-lg border border-slate-300 px-4 py-2" @click="isModalOpen = false">Batal</button><button type="submit" class="rounded-lg bg-blue-600 px-4 py-2 font-medium text-white disabled:opacity-50" :disabled="isSaving">{{ isSaving ? "Menyimpan..." : "Simpan Data" }}</button></div>
      </form>
    </div>
  </div>
</template>

<style>
* { box-sizing: border-box; }
body { margin: 0; font-family: Inter, ui-sans-serif, system-ui, sans-serif; }
button, input, textarea { font: inherit; }
</style>
