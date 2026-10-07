<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ask } from "@tauri-apps/plugin-dialog";
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

const filteredEmployees = computed(() => {
  let result = employees.value;

  // 1. Jika ada filter metrik aktif dari dashboard
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

  // 2. Pencarian kata kunci manual
  if (search.value.toUpperCase() === "PENSIUN") {
    result = result.filter((employee) => {
      const age = calculateAge(employee.tanggal_lahir);
      return age !== null && age >= 55;
    });
  }

  // 3. Filter dropdown divisi
  return division.value === "ALL"
    ? result
    : result.filter((employee) => (employee.divisi || "").toUpperCase() === division.value);
});

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
</template>