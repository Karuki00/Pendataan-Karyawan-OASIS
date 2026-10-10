<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ask, save } from "@tauri-apps/plugin-dialog";
import * as XLSX from "xlsx";
import EmployeeDetailModal from "./EmployeeDetailModal.vue";
import type { Employee } from "../types";

const props = defineProps<{ 
  refreshKey: number; 
  initialSearch?: string;
  activeMetricKey?: string;
}>();

const emit = defineEmits<{
  edit: [employee: Employee];
  resetSearch: [];
  refresh: [];
  clearMetricFilter: [];
}>();

const employees = ref<Employee[]>([]);
const search = ref(props.initialSearch || "");
const division = ref("ALL");
const isLoading = ref(false);
const errorMessage = ref("");
const selectedEmployee = ref<Employee | null>(null);

// --- PAGINATION STATE ---
const currentPage = ref(1);
const pageSize = ref(10); // Opsi default 10 baris per halaman

// State untuk Ekspor Excel & QOL
const isExporting = ref(false);
const exportProgress = ref(0);
const showExportConfirmModal = ref(false);
const cleanDateFormat = ref(true);

// QOL 1: Preset Selection
type ExportPreset = "FULL" | "CONTACT" | "INSURANCE";
const activePreset = ref<ExportPreset>("FULL");

// QOL 2: Success Toast State
const savedFilePath = ref<string | null>(null);
const showSuccessToast = ref(false);

function normalized(value: string | null | undefined): string {
  return (value || "").trim().toUpperCase();
}

function calculateAge(value: string | null | undefined): number | null {
  if (!value) return null;
  const birthDate = new Date(value);
  if (Number.isNaN(birthDate.getTime())) return null;
  const today = new Date();
  let age = today.getFullYear() - birthDate.getFullYear();
  if (today.getMonth() < birthDate.getMonth()
    || (today.getMonth() === birthDate.getMonth() && today.getDate() < birthDate.getDate())) age--;
  return age;
}

function isMale(employee: Employee): boolean {
  const gender = normalized(employee.jenis_kelamin);
  return gender === "L"
    || gender === "LAKI-LAKI"
    || gender === "LAKI LAKI"
    || gender.includes("LAKI")
    || gender === "PRIA"
    || gender === "MALE";
}

function formatDate(dateStr: string | null | undefined): string {
  if (!dateStr || dateStr === "-") return "-";
  const parsedDate = new Date(dateStr);
  if (Number.isNaN(parsedDate.getTime())) return dateStr;

  const day = String(parsedDate.getDate()).padStart(2, "0");
  const month = String(parsedDate.getMonth() + 1).padStart(2, "0");
  const year = parsedDate.getFullYear();

  return `${day}/${month}/${year}`;
}

// 1. Array Data Terfilter Keseluruhan (Termasuk Filter Flagged)
const filteredEmployees = computed(() => {
  let result = employees.value;

  if (props.activeMetricKey) {
    result = result.filter((emp) => {
      const status = normalized(emp.status || "AKTIF");
      const isEmpActive = status === "AKTIF";

      switch (props.activeMetricKey) {
        case "flagged": return Boolean(emp.is_flagged);
        case "ALL_ACTIVE": return isEmpActive;
        case "PENSIUN": return status === "PENSIUN";
        case "PKWTT": return isEmpActive && normalized(emp.perjanjian_kerja || "PKWTT") === "PKWTT";
        case "PKWT": return isEmpActive && normalized(emp.perjanjian_kerja) === "PKWT";
        case "STAFF": return isEmpActive && normalized(emp.divisi) === "STAFF";
        case "HOUSEKEEPING": return isEmpActive && normalized(emp.divisi) === "HOUSEKEEPING";
        case "ENGINEERING": return isEmpActive && normalized(emp.divisi) === "ENGINEERING";
        case "SECURITY": return isEmpActive && normalized(emp.divisi) === "SECURITY";
        case "KANTOR": return isEmpActive && normalized(emp.tempat_kerja) === "KANTOR";
        case "LAPANGAN": return isEmpActive && normalized(emp.tempat_kerja) === "LAPANGAN";
        case "PEREMPUAN": return isEmpActive && !isMale(emp);
        case "LAKI_LAKI": return isEmpActive && isMale(emp);
        default: return true;
      }
    });
  }

  if (search.value.toUpperCase() === "PENSIUN") {
    result = result.filter((employee) => {
      const age = calculateAge(employee.tanggal_lahir);
      return age !== null && age >= 55;
    });
  }

  return division.value === "ALL"
    ? result
    : result.filter((employee) => (employee.divisi || "").toUpperCase() === division.value);
});

// --- COMPUTED COMPUTATION UNTUK PAGINATION ---
const totalPages = computed(() => {
  return Math.ceil(filteredEmployees.value.length / pageSize.value) || 1;
});

const paginatedEmployees = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value;
  const end = start + pageSize.value;
  return filteredEmployees.value.slice(start, end);
});

// Reset ke Halaman 1 jika data terfilter berubah
watch([filteredEmployees, pageSize], () => {
  currentPage.value = 1;
});

function goToPage(page: number) {
  if (page >= 1 && page <= totalPages.value) {
    currentPage.value = page;
  }
}

function triggerExportModal() {
  if (!filteredEmployees.value.length) {
    alert("Tidak ada data untuk diekspor.");
    return;
  }
  showExportConfirmModal.value = true;
}

async function openExportFolder() {
  if (!savedFilePath.value) return;

  try {
    const normalizedPath = savedFilePath.value.replace(/\\/g, "/");
    const parentFolder = normalizedPath.substring(0, normalizedPath.lastIndexOf("/"));

    if (parentFolder) {
      await invoke("open_folder_dir", { path: parentFolder });
    }
  } catch (error) {
    console.error("Gagal membuka folder:", error);
    errorMessage.value =
      "Gagal membuka folder: " + (error instanceof Error ? error.message : String(error));
  }
}

async function confirmAndExport() {
  showExportConfirmModal.value = false;

  try {
    const defaultFileName = `Data_Karyawan_${activePreset.value}_${new Date().toISOString().slice(0, 10)}.xlsx`;
    const filePath = await save({
      defaultPath: defaultFileName,
      filters: [{ name: "Excel Spreadsheet", extensions: ["xlsx"] }],
    });

    if (!filePath) return;

    isExporting.value = true;
    exportProgress.value = 10;
    await new Promise((r) => setTimeout(r, 50));

    const list = filteredEmployees.value;
    const total = list.length;
    const exportData: Record<string, any>[] = [];

    for (let i = 0; i < total; i++) {
      const emp = list[i] as Record<string, any>;

      const birthDate = cleanDateFormat.value ? formatDate(emp.tanggal_lahir) : emp.tanggal_lahir || "-";
      const joinDate = cleanDateFormat.value ? formatDate(emp.join_date) : emp.join_date || "-";
      const kartapDate = cleanDateFormat.value ? formatDate(emp.tanggal_kartap) : emp.tanggal_kartap || "-";

      let rowData: Record<string, any> = {};

      if (activePreset.value === "CONTACT") {
        rowData = {
          "No.": i + 1,
          "NIK": emp.nik || "-",
          "Nama Lengkap": emp.nama_lengkap || "-",
          "Divisi": emp.divisi || "-",
          "Nomor Telepon": emp.nomor_telepon || "-",
          "Nomor Telp Keluarga": emp.nomor_telp_keluarga || "-",
        };
      } else if (activePreset.value === "INSURANCE") {
        rowData = {
          "No.": i + 1,
          "NIK": emp.nik || "-",
          "Nama Lengkap": emp.nama_lengkap || "-",
          "Nomor KK": emp.nomor_kk || "-",
          "BPJS Kesehatan": emp.bpjs_kesehatan || "-",
          "BPJS Ketenagakerjaan": emp.bpjs_ketenagakerjaan || "-",
          "Perjanjian Kerja": emp.perjanjian_kerja || "PKWTT",
          "Status": emp.status || "AKTIF",
        };
      } else {
        rowData = {
          "No.": i + 1,
          "NIK": emp.nik || "-",
          "Nama Lengkap": emp.nama_lengkap || "-",
          "Jenis Kelamin": emp.jenis_kelamin || "-",
          "Tanggal Lahir": birthDate,
          "Golongan Darah": emp.golongan_darah || "-",
          "Nomor KK": emp.nomor_kk || "-",
          "Alamat KTP": emp.alamat_ktp || "-",
          "BPJS Kesehatan": emp.bpjs_kesehatan || "-",
          "BPJS Ketenagakerjaan": emp.bpjs_ketenagakerjaan || "-",
          "Nomor Telepon": emp.nomor_telepon || "-",
          "Divisi": emp.divisi || "-",
          "Jabatan": emp.jabatan || "-",
          "Perjanjian Kerja": emp.perjanjian_kerja || "PKWTT",
          "Tempat Kerja": emp.tempat_kerja || "Lapangan",
          "Status": emp.status || "AKTIF",
          "Join Date": joinDate,
          "Tanggal Kartap": kartapDate,
          "Pendidikan Terakhir": emp.pendidikan_terakhir || "-",
          "Nama Ibu Kandung": emp.nama_ibu_kandung || "-",
          "Nama Pasangan": emp.nama_pasangan || "-",
          "Jumlah Anak": emp.jumlah_anak || "0",
          "Nomor Telp Keluarga": emp.nomor_telp_keluarga || "-",
          "Foto KTP": emp.foto_ktp || "-",
          "Foto KK": emp.foto_kk || "-",
          "Foto BPJS Kesehatan": emp.foto_bpjs_kesehatan || "-",
          "Foto BPJS Ketenagakerjaan": emp.foto_bpjs_ketenagakerjaan || "-",
        };
      }

      exportData.push(rowData);

      if (i % 5 === 0 || i === total - 1) {
        exportProgress.value = 10 + Math.round(((i + 1) / total) * 50);
        await new Promise((r) => setTimeout(r, 1));
      }
    }

    exportProgress.value = 70;
    await new Promise((r) => setTimeout(r, 20));

    const worksheet = XLSX.utils.json_to_sheet(exportData);
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, worksheet, "Data Karyawan");

    if (exportData.length > 0) {
      const colWidths = Object.keys(exportData[0]).map((key) => {
        const maxLen = exportData.reduce((max, row) => {
          const val = String(row[key] || "");
          return Math.max(max, val.length);
        }, key.length);
        return { wch: Math.min(Math.max(maxLen + 3, 12), 40) };
      });
      worksheet["!cols"] = colWidths;
    }

    exportProgress.value = 85;
    await new Promise((r) => setTimeout(r, 20));
    const excelBuffer = XLSX.write(workbook, { bookType: "xlsx", type: "array" });

    await invoke("export_excel_file", {
      path: filePath,
      bytes: Array.from(new Uint8Array(excelBuffer)),
    });

    exportProgress.value = 100;
    await new Promise((r) => setTimeout(r, 200));

    savedFilePath.value = filePath;
    showSuccessToast.value = true;
    setTimeout(() => {
      showSuccessToast.value = false;
    }, 8000);
  } catch (error) {
    console.error("Export error:", error);
    errorMessage.value =
      "Gagal mengekspor data: " + (error instanceof Error ? error.message : String(error));
  } finally {
    isExporting.value = false;
    exportProgress.value = 0;
  }
}

async function loadEmployees() {
  isLoading.value = true;
  try {
    employees.value = await invoke<Employee[]>("list_employees", { search: search.value.toUpperCase() === "PENSIUN" ? "" : search.value });
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally { isLoading.value = false; }
}

async function handleDelete(employee: Employee) {
  const confirmed = await ask(
    `Apakah Anda yakin ingin menghapus data "${employee.nama_lengkap}" (NIK: ${employee.nik})?\n\nTindakan ini akan menghapus data dari SQLite secara permanen.`,
    {
      title: "⚠️ Konfirmasi Hapus Data Karyawan",
      kind: "warning",
      okLabel: "Ya, Hapus Permanen",
      cancelLabel: "Batal",
    }
  );

  if (!confirmed) return;

  try {
    await invoke("delete_employee", { nik: employee.nik });
    await loadEmployees();
    emit("refresh");
  } catch (error) {
    errorMessage.value = "Gagal menghapus data karyawan: " + (error instanceof Error ? error.message : String(error));
  }
}

function photoLinks(employee: Employee): Array<[string, string]> {
  return [["KTP", employee.foto_ktp], ["KK", employee.foto_kk], ["BPJS K", employee.foto_bpjs_kesehatan], ["BPJS TK", employee.foto_bpjs_ketenagakerjaan]]
    .filter((entry): entry is [string, string] => Boolean(entry[1]));
}

function openDetail(employee: Employee) {
  selectedEmployee.value = employee;
}

async function openLink(url: string) {
  await openUrl(url);
}

function statusBadgeClass(status?: string | null): string {
  const norm = (status || "AKTIF").trim().toUpperCase();
  if (norm === "PENSIUN") return "bg-slate-100 text-slate-700 border border-slate-300";
  if (norm === "RESIGN") return "bg-rose-100 text-rose-700 border border-rose-200";
  return "bg-emerald-100 text-emerald-700 border border-emerald-200";
}

watch(search, (value) => {
  if (!value && props.initialSearch) emit("resetSearch");
  void loadEmployees();
});
watch(() => props.initialSearch, (value) => {
  search.value = value || "";
});
watch(() => props.refreshKey, () => void loadEmployees());
onMounted(() => void loadEmployees());
</script>

<template>
  <section class="space-y-4">
    <!-- Indicator Filter Aktif dari Dashboard -->
    <div v-if="props.activeMetricKey" class="flex items-center justify-between rounded-lg bg-amber-50 px-4 py-2 border border-amber-200 text-sm text-amber-900">
      <span class="flex items-center gap-2 font-semibold">
        <template v-if="props.activeMetricKey === 'flagged'">
          ⚠️ Filter Aktif: Menampilkan Data Anomali
        </template>
        <template v-else>
          🔍 Filter Metrik Aktif: <strong>{{ props.activeMetricKey }}</strong>
        </template>
      </span>
      <button @click="emit('clearMetricFilter')" class="text-xs text-amber-700 underline font-semibold hover:text-amber-950 cursor-pointer">
        Reset Filter
      </button>
    </div>

    <!-- Toolbar Filter & Pencarian -->
    <div class="flex flex-col gap-3 sm:flex-row sm:items-center">
      <input v-model="search" class="w-full rounded-lg border border-slate-300 bg-white px-4 py-2.5 outline-none focus:border-blue-500 text-sm shadow-xs" placeholder="Cari nama, NIK, atau divisi...">
      <select v-model="division" class="rounded-lg border border-slate-300 bg-white px-3 py-2.5 outline-none focus:border-blue-500 text-sm shadow-xs" aria-label="Filter divisi">
        <option value="ALL">Semua Divisi</option>
        <option value="SECURITY">SECURITY</option>
        <option value="ENGINEERING">ENGINEERING</option>
        <option value="HOUSEKEEPING">HOUSEKEEPING</option>
        <option value="STAFF">STAFF</option>
      </select>
      
      <!-- Tombol Trigger Ekspor Excel -->
      <button 
        @click="triggerExportModal" 
        :disabled="isExporting"
        class="inline-flex items-center gap-2 rounded-lg bg-emerald-600 px-4 py-2.5 text-sm font-semibold text-white hover:bg-emerald-700 disabled:opacity-50 transition-colors shadow-sm cursor-pointer whitespace-nowrap"
      >
        <svg v-if="isExporting" class="h-4 w-4 animate-spin text-white" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
        </svg>
        <span>{{ isExporting ? `Mengekspor (${exportProgress}%)...` : '📊 Export Excel' }}</span>
      </button>
      
      <span class="whitespace-nowrap text-sm text-slate-500 font-medium">{{ filteredEmployees.length }} karyawan</span>
    </div>

    <p v-if="errorMessage" class="rounded-lg bg-rose-50 p-3 text-sm text-rose-700 border border-rose-200">{{ errorMessage }}</p>

    <!-- Kontainer Tabel Karyawan -->
    <div class="overflow-hidden rounded-2xl border border-slate-200/80 bg-white shadow-sm">
      <div class="overflow-auto">
        <table class="min-w-[1000px] w-full text-left text-sm">
          <thead class="bg-slate-50 text-xs uppercase text-slate-500 border-b border-slate-100">
            <tr>
              <th class="w-16 px-4 py-3.5">No.</th>
              <th class="px-4 py-3.5">Nama</th>
              <th class="px-4 py-3.5">NIK</th>
              <th class="px-4 py-3.5">Divisi / Jabatan</th>
              <th class="px-4 py-3.5">Kontak</th>
              <th class="px-4 py-3.5">Dokumen</th>
              <th class="px-4 py-3.5">Status</th>
              <th class="px-4 py-3.5 text-right">Aksi</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100">
            <tr v-if="isLoading">
              <td colspan="8" class="px-4 py-12 text-center text-slate-500">Memuat data...</td>
            </tr>
            <tr v-else-if="!filteredEmployees.length">
              <td colspan="8" class="px-4 py-12 text-center text-slate-500">Belum ada data karyawan.</td>
            </tr>
            <tr 
              v-for="(employee, index) in paginatedEmployees" 
              v-else 
              :key="employee.nik" 
              class="cursor-pointer hover:bg-slate-50 transition-colors" 
              :class="{ 'bg-amber-50/40': employee.is_flagged }"
              @click="openDetail(employee)"
            >
              <td class="px-4 py-3 font-semibold text-slate-400">
                {{ (currentPage - 1) * pageSize + index + 1 }}
              </td>
              <td class="px-4 py-3 font-medium text-slate-800">
                <div class="flex items-center gap-2">
                  <span>{{ employee.nama_lengkap }}</span>
                  <span v-if="employee.is_flagged" title="Data belum lengkap / format tidak sesuai" class="rounded bg-amber-100 text-amber-800 px-1.5 py-0.5 text-[10px] font-bold border border-amber-300">
                    ⚠️ Warning
                  </span>
                </div>
              </td>
              <td class="px-4 py-3 font-mono text-xs text-slate-600">{{ employee.nik }}</td>
              <td class="px-4 py-3">
                <span class="font-semibold text-slate-800">{{ employee.divisi || "-" }}</span>
                <br>
                <span class="text-xs text-slate-500">{{ employee.jabatan || "-" }}</span>
              </td>
              <td class="px-4 py-3 text-slate-600 font-mono text-xs">{{ employee.nomor_telepon || "-" }}</td>
              <td class="px-4 py-3">
                <div class="flex flex-wrap gap-1">
                  <button v-for="[label, url] in photoLinks(employee)" :key="label" class="text-xs text-blue-600 underline hover:text-blue-800" @click.stop="openLink(url)">{{ label }}</button>
                  <span v-if="!photoLinks(employee).length" class="text-slate-400">-</span>
                </div>
              </td>
              <td class="px-4 py-3">
                <span class="rounded-full px-2.5 py-1 text-xs font-semibold" :class="statusBadgeClass(employee.status)">
                  {{ employee.status || "AKTIF" }}
                </span>
              </td>
              <td class="px-4 py-3">
                <div class="flex justify-end gap-1.5">
                  <button class="rounded-md bg-blue-600 px-2.5 py-1.5 text-xs font-semibold text-white hover:bg-blue-700 transition-colors shadow-xs" @click.stop="openDetail(employee)">
                    Detail
                  </button>
                  <button class="rounded-md bg-amber-100 px-2.5 py-1.5 text-xs font-semibold text-amber-800 hover:bg-amber-200 transition-colors" @click.stop="emit('edit', employee)">
                    Edit
                  </button>
                  <button class="rounded-md bg-rose-50 px-2.5 py-1.5 text-xs font-semibold text-rose-700 hover:bg-rose-100 transition-colors border border-rose-200" @click.stop="handleDelete(employee)">
                    Hapus
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- NAVBAR TOOLBAR PAGINATION BAWAH -->
      <div v-if="filteredEmployees.length > 0" class="flex flex-wrap items-center justify-between gap-4 border-t border-slate-100 px-6 py-3.5 bg-slate-50/60">
        <!-- Selector Jumlah Data Per Halaman -->
        <div class="flex items-center gap-2 text-xs text-slate-500 font-medium">
          <span>Tampilkan</span>
          <select 
            v-model="pageSize" 
            class="rounded-lg border border-slate-300 bg-white px-2.5 py-1 text-xs font-semibold text-slate-700 outline-none focus:border-emerald-500 shadow-xs"
          >
            <option :value="10">10</option>
            <option :value="25">25</option>
            <option :value="50">50</option>
            <option :value="100">100</option>
          </select>
          <span>data per halaman</span>
        </div>

        <!-- Indikator Rincian Data Aktif & Navigasi -->
        <div class="flex items-center gap-4">
          <span class="text-xs text-slate-500 font-medium">
            Menampilkan 
            <strong class="text-slate-800">
              {{ Math.min((currentPage - 1) * pageSize + 1, filteredEmployees.length) }}
            </strong> 
            - 
            <strong class="text-slate-800">
              {{ Math.min(currentPage * pageSize, filteredEmployees.length) }}
            </strong> 
            dari 
            <strong class="text-slate-800">{{ filteredEmployees.length }}</strong> karyawan
          </span>

          <!-- Tombol-tombol Navigasi Halaman -->
          <div class="flex items-center gap-1">
            <button 
              @click="goToPage(1)" 
              :disabled="currentPage === 1"
              title="Halaman Pertama"
              class="rounded-lg border border-slate-200 bg-white px-2 py-1 text-xs font-semibold text-slate-600 hover:bg-slate-100 disabled:opacity-40 disabled:cursor-not-allowed transition-colors shadow-xs"
            >
              «
            </button>
            <button 
              @click="goToPage(currentPage - 1)" 
              :disabled="currentPage === 1"
              class="rounded-lg border border-slate-200 bg-white px-3 py-1 text-xs font-semibold text-slate-600 hover:bg-slate-100 disabled:opacity-40 disabled:cursor-not-allowed transition-colors shadow-xs"
            >
              Prev
            </button>
            
            <span class="px-2 text-xs font-bold text-slate-700 font-mono">
              {{ currentPage }} / {{ totalPages }}
            </span>

            <button 
              @click="goToPage(currentPage + 1)" 
              :disabled="currentPage === totalPages"
              class="rounded-lg border border-slate-200 bg-white px-3 py-1 text-xs font-semibold text-slate-600 hover:bg-slate-100 disabled:opacity-40 disabled:cursor-not-allowed transition-colors shadow-xs"
            >
              Next
            </button>
            <button 
              @click="goToPage(totalPages)" 
              :disabled="currentPage === totalPages"
              title="Halaman Terakhir"
              class="rounded-lg border border-slate-200 bg-white px-2 py-1 text-xs font-semibold text-slate-600 hover:bg-slate-100 disabled:opacity-40 disabled:cursor-not-allowed transition-colors shadow-xs"
            >
              »
            </button>
          </div>
        </div>
      </div>
    </div>

    <EmployeeDetailModal v-if="selectedEmployee" :employee="selectedEmployee" @close="selectedEmployee = null" />
  </section>

  <!-- Modal Konfirmasi Opsi Ekspor Excel + Preset Selection -->
  <Teleport to="body">
    <div 
      v-if="showExportConfirmModal" 
      class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/50 backdrop-blur-xs p-4"
      @click.self="showExportConfirmModal = false"
    >
      <div class="w-full max-w-md rounded-2xl bg-white p-6 shadow-2xl border border-slate-100 space-y-5">
        <div class="flex items-center gap-3 border-b border-slate-100 pb-3">
          <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-emerald-50 text-xl">
            ⚙️
          </div>
          <div>
            <h3 class="font-bold text-slate-800 text-base">Opsi Ekspor Excel</h3>
            <p class="text-xs text-slate-500">Pilih preset kolom dan format data</p>
          </div>
        </div>

        <!-- QOL 1: Preset Selection Pills -->
        <div class="space-y-2">
          <label class="text-xs font-semibold text-slate-700 uppercase tracking-wider">Pilih Preset Kolom</label>
          <div class="grid grid-cols-1 gap-2">
            <button
              type="button"
              @click="activePreset = 'FULL'"
              class="flex items-center justify-between rounded-xl border p-3 text-left transition-all"
              :class="activePreset === 'FULL' ? 'border-emerald-600 bg-emerald-50/50 text-emerald-900 font-semibold shadow-xs' : 'border-slate-200 hover:bg-slate-50 text-slate-700'"
            >
              <div class="flex items-center gap-2.5">
                <span class="text-base">📋</span>
                <div>
                  <div class="text-xs">Semua Kolom</div>
                  <div class="text-[10px] text-slate-500 font-normal font-sans">Seluruh 28 kolom data master karyawan</div>
                </div>
              </div>
              <span class="text-[10px] font-mono rounded-md bg-white px-2 py-0.5 border border-slate-200">28 Kolom</span>
            </button>

            <button
              type="button"
              @click="activePreset = 'CONTACT'"
              class="flex items-center justify-between rounded-xl border p-3 text-left transition-all"
              :class="activePreset === 'CONTACT' ? 'border-emerald-600 bg-emerald-50/50 text-emerald-900 font-semibold shadow-xs' : 'border-slate-200 hover:bg-slate-50 text-slate-700'"
            >
              <div class="flex items-center gap-2.5">
                <span class="text-base">📞</span>
                <div>
                  <div class="text-xs">Ringkasan Kontak</div>
                  <div class="text-[10px] text-slate-500 font-normal font-sans">NIK, Nama, Divisi, No. HP, No. HP Keluarga</div>
                </div>
              </div>
              <span class="text-[10px] font-mono rounded-md bg-white px-2 py-0.5 border border-slate-200">6 Kolom</span>
            </button>

            <button
              type="button"
              @click="activePreset = 'INSURANCE'"
              class="flex items-center justify-between rounded-xl border p-3 text-left transition-all"
              :class="activePreset === 'INSURANCE' ? 'border-emerald-600 bg-emerald-50/50 text-emerald-900 font-semibold shadow-xs' : 'border-slate-200 hover:bg-slate-50 text-slate-700'"
            >
              <div class="flex items-center gap-2.5">
                <span class="text-base">🛡️</span>
                <div>
                  <div class="text-xs">Data BPJS & Legal</div>
                  <div class="text-[10px] text-slate-500 font-normal font-sans">NIK, Nama, KK, BPJS Kesehatan, BPJS TK, PKWT</div>
                </div>
              </div>
              <span class="text-[10px] font-mono rounded-md bg-white px-2 py-0.5 border border-slate-200">8 Kolom</span>
            </button>
          </div>
        </div>

        <!-- Opsi Format Tanggal -->
        <div class="rounded-xl bg-slate-50 p-3.5 border border-slate-200/80">
          <label class="flex items-start gap-3 cursor-pointer">
            <input 
              type="checkbox" 
              v-model="cleanDateFormat" 
              class="mt-0.5 h-4 w-4 rounded border-slate-300 text-emerald-600 focus:ring-emerald-500"
            />
            <div>
              <span class="text-xs font-semibold text-slate-800">Rapikan Format Tanggal (DD/MM/YYYY)</span>
              <p class="text-[11px] text-slate-500 mt-0.5">
                Konversi string mentah menjadi <code class="font-mono text-[10px] bg-slate-200 px-1 py-0.5 rounded">11/10/1975</code>
              </p>
            </div>
          </label>
        </div>

        <div class="text-xs text-slate-500">
          Menyimpan data sebanyak <strong>{{ filteredEmployees.length }} karyawan</strong>.
        </div>

        <!-- Action Buttons -->
        <div class="flex justify-end gap-2.5 pt-2 border-t border-slate-100">
          <button 
            @click="showExportConfirmModal = false"
            class="rounded-lg border border-slate-200 bg-white px-4 py-2 text-xs font-semibold text-slate-600 hover:bg-slate-50 transition-colors cursor-pointer"
          >
            Batal
          </button>
          <button 
            @click="confirmAndExport"
            class="rounded-lg bg-emerald-600 px-4 py-2 text-xs font-semibold text-white hover:bg-emerald-700 transition-colors shadow-sm cursor-pointer"
          >
            Mulai Export
          </button>
        </div>
      </div>
    </div>
  </Teleport>

  <!-- Modal Progress Ekspor Excel -->
  <Teleport to="body">
    <div 
      v-if="isExporting" 
      class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/40 backdrop-blur-xs p-4"
    >
      <div class="w-full max-w-sm rounded-2xl bg-white p-6 shadow-2xl border border-slate-100 text-center space-y-4">
        <div class="mx-auto flex h-12 w-12 items-center justify-center rounded-full bg-emerald-50 text-2xl">
          📊
        </div>
        
        <div>
          <h3 class="font-bold text-slate-800 text-base">Memproses Export Excel</h3>
          <p class="text-xs text-slate-500 mt-1">
            Menulis data {{ filteredEmployees.length }} karyawan...
          </p>
        </div>

        <div class="w-full bg-slate-100 rounded-full h-3 overflow-hidden border border-slate-200">
          <div 
            class="bg-emerald-600 h-full transition-all duration-150 ease-out rounded-full"
            :style="{ width: `${exportProgress}%` }"
          ></div>
        </div>

        <div class="text-xs font-mono font-semibold text-slate-600">
          {{ exportProgress }}%
        </div>
      </div>
    </div>
  </Teleport>

  <!-- QOL 2: Export Summary Toast / Notification -->
  <Teleport to="body">
    <Transition name="toast">
      <div 
        v-if="showSuccessToast" 
        class="fixed bottom-6 right-6 z-[9999] flex items-center gap-3 rounded-2xl bg-slate-900 p-4 text-white shadow-2xl border border-slate-800 max-w-md"
      >
        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-emerald-500/20 text-emerald-400 text-xl">
          ✅
        </div>

        <div class="flex-1 min-w-0">
          <h4 class="text-xs font-semibold text-slate-200">Berhasil Diekspor!</h4>
          <p class="text-[11px] text-slate-400 truncate mt-0.5 font-mono" :title="savedFilePath || ''">
            {{ savedFilePath }}
          </p>
        </div>

        <div class="flex items-center gap-2 shrink-0">
          <button 
            @click="openExportFolder" 
            class="rounded-lg bg-emerald-600 px-3 py-1.5 text-xs font-semibold text-white hover:bg-emerald-500 transition-colors shadow-sm cursor-pointer flex items-center gap-1.5"
          >
            <span>📁 Buka Folder</span>
          </button>
          <button 
            @click="showSuccessToast = false" 
            class="text-slate-400 hover:text-white p-1 text-xs cursor-pointer"
          >
            ✕
          </button>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(1rem) scale(0.95);
}
</style>