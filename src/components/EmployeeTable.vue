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

// State untuk Ekspor Excel
const isExporting = ref(false);
const exportProgress = ref(0);
const showExportConfirmModal = ref(false);
const cleanDateFormat = ref(true);

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

const filteredEmployees = computed(() => {
  let result = employees.value;

  if (props.activeMetricKey) {
    result = result.filter((emp) => {
      const status = normalized(emp.status || "AKTIF");
      const isEmpActive = status === "AKTIF";

      switch (props.activeMetricKey) {
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

function triggerExportModal() {
  if (!filteredEmployees.value.length) {
    alert("Tidak ada data untuk diekspor.");
    return;
  }
  showExportConfirmModal.value = true;
}

async function confirmAndExport() {
  showExportConfirmModal.value = false;

  try {
    const defaultFileName = `Data_Karyawan_Oasis_Lengkap_${new Date().toISOString().slice(0, 10)}.xlsx`;
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

      exportData.push({
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
        "Timestamp Data": emp.timestamp || "-",
      });

      if (i % 5 === 0 || i === total - 1) {
        exportProgress.value = 10 + Math.round(((i + 1) / total) * 50);
        await new Promise((r) => setTimeout(r, 1));
      }
    }

    exportProgress.value = 70;
    await new Promise((r) => setTimeout(r, 20));

    const worksheet = XLSX.utils.json_to_sheet(exportData);
    const workbook = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(workbook, worksheet, "Master Data Karyawan");

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
    await new Promise((r) => setTimeout(r, 300));
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
    <div v-if="props.activeMetricKey" class="flex items-center justify-between rounded-lg bg-blue-50 px-4 py-2 border border-blue-200 text-sm text-blue-800">
      <span>Filter Metrik Aktif: <strong>{{ props.activeMetricKey }}</strong></span>
      <button @click="emit('clearMetricFilter')" class="text-xs text-blue-600 underline font-semibold hover:text-blue-800">
        Reset Filter Dashboard
      </button>
    </div>

    <div class="flex flex-col gap-3 sm:flex-row sm:items-center">
      <input v-model="search" class="w-full rounded-lg border border-slate-300 bg-white px-4 py-2.5 outline-none focus:border-blue-500" placeholder="Cari nama, NIK, atau divisi...">
      <select v-model="division" class="rounded-lg border border-slate-300 bg-white px-3 py-2.5 outline-none focus:border-blue-500" aria-label="Filter divisi">
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
        class="inline-flex items-center gap-2 rounded-lg bg-emerald-600 px-4 py-2.5 text-sm font-semibold text-white hover:bg-emerald-700 disabled:opacity-50 transition-colors shadow-sm"
      >
        <svg v-if="isExporting" class="h-4 w-4 animate-spin text-white" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v8H4z"></path>
        </svg>
        <span>{{ isExporting ? `Mengekspor (${exportProgress}%)...` : '📊 Export Excel' }}</span>
      </button>
      
      <span class="whitespace-nowrap text-sm text-slate-500">{{ filteredEmployees.length }} karyawan</span>
    </div>

    <p v-if="errorMessage" class="rounded-lg bg-red-50 p-3 text-sm text-red-700">{{ errorMessage }}</p>

    <div class="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
      <div class="overflow-auto">
        <table class="min-w-[1000px] w-full text-left text-sm">
          <thead class="bg-slate-100 text-xs uppercase text-slate-500">
            <tr>
              <th class="w-16 px-4 py-3">No.</th>
              <th class="px-4 py-3">Nama</th>
              <th class="px-4 py-3">NIK</th>
              <th class="px-4 py-3">Divisi / Jabatan</th>
              <th class="px-4 py-3">Kontak</th>
              <th class="px-4 py-3">Dokumen</th>
              <th class="px-4 py-3">Status</th>
              <th class="px-4 py-3 text-right">Aksi</th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="isLoading"><td colspan="8" class="px-4 py-12 text-center text-slate-500">Memuat data...</td></tr>
            <tr v-else-if="!filteredEmployees.length"><td colspan="8" class="px-4 py-12 text-center text-slate-500">Belum ada data karyawan.</td></tr>
            <tr v-for="(employee, index) in filteredEmployees" v-else :key="employee.nik" class="cursor-pointer border-t border-slate-100 hover:bg-slate-50 transition-colors" @click="openDetail(employee)">
              <td class="px-4 py-3 font-semibold text-slate-400">{{ index + 1 }}</td>
              <td class="px-4 py-3 font-medium text-slate-800">{{ employee.nama_lengkap }}</td>
              <td class="px-4 py-3 font-mono text-xs">{{ employee.nik }}</td>
              <td class="px-4 py-3">{{ employee.divisi || "-" }}<br><span class="text-xs text-slate-500">{{ employee.jabatan || "-" }}</span></td>
              <td class="px-4 py-3">{{ employee.nomor_telepon || "-" }}</td>
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
                  <button class="rounded-md bg-blue-600 px-2.5 py-1.5 text-xs font-semibold text-white hover:bg-blue-700 transition-colors" @click.stop="openDetail(employee)">
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
    </div>
    <EmployeeDetailModal v-if="selectedEmployee" :employee="selectedEmployee" @close="selectedEmployee = null" />
  </section>

  <!-- Modal Konfirmasi Opsi Ekspor Excel -->
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
            <p class="text-xs text-slate-500">Konfigurasi pembersihan format data</p>
          </div>
        </div>

        <div class="rounded-xl bg-slate-50 p-4 border border-slate-200/80 space-y-3">
          <label class="flex items-start gap-3 cursor-pointer">
            <input 
              type="checkbox" 
              v-model="cleanDateFormat" 
              class="mt-1 h-4 w-4 rounded border-slate-300 text-emerald-600 focus:ring-emerald-500"
            />
            <div>
              <span class="text-sm font-semibold text-slate-800">Rapikan Format Tanggal (DD/MM/YYYY)</span>
              <p class="text-xs text-slate-500 mt-0.5">
                Mengubah string tanggal mentah (contoh: <code class="font-mono text-[10px] bg-slate-200 px-1 py-0.5 rounded">Sat Oct 11 1975...</code>) menjadi format standar (<code class="font-mono text-[10px] bg-slate-200 px-1 py-0.5 rounded">11/10/1975</code>).
              </p>
            </div>
          </label>
        </div>

        <div class="text-xs text-slate-500">
          Menyimpan data sebanyak <strong>{{ filteredEmployees.length }} karyawan</strong>.
        </div>

        <div class="flex justify-end gap-2.5 pt-2 border-t border-slate-100">
          <button 
            @click="showExportConfirmModal = false"
            class="rounded-lg border border-slate-200 bg-white px-4 py-2 text-xs font-semibold text-slate-600 hover:bg-slate-50 transition-colors"
          >
            Batal
          </button>
          <button 
            @click="confirmAndExport"
            class="rounded-lg bg-emerald-600 px-4 py-2 text-xs font-semibold text-white hover:bg-emerald-700 transition-colors shadow-sm"
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
</template>