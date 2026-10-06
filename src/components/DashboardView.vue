<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Employee } from "../types";

const props = defineProps<{ refreshKey: number }>();
const emit = defineEmits<{
  import: [];
  add: [];
}>();

const employees = ref<Employee[]>([]);
const isLoading = ref(false);
const errorMessage = ref("");

function normalized(value: string | null | undefined): string {
  return (value || "").trim().toUpperCase();
}

function isActive(employee: Employee): boolean {
  return normalized(employee.status || "AKTIF") === "AKTIF";
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

function countGender(list: Employee[], male: boolean): number {
  return list.filter((employee) => isMale(employee) === male).length;
}

// 1. Filtered Arrays
const activeEmployees = computed(() => employees.value.filter(isActive));
const retiredEmployees = computed(() => employees.value.filter((employee) => normalized(employee.status) === "PENSIUN"));

// 2. PKWTT & PKWT
const pkwttActive = computed(() => activeEmployees.value.filter((employee) => normalized(employee.perjanjian_kerja || "PKWTT") === "PKWTT"));
const pkwtActive = computed(() => activeEmployees.value.filter((employee) => normalized(employee.perjanjian_kerja) === "PKWT"));

// 3. Exact 12 Metrics matching !DASHBOARD!
const metrics = computed(() => [
  { label: "Total Karyawan Aktif (Saat Ini)", value: activeEmployees.value.length, detail: "Karyawan dengan status AKTIF" },
  { label: "Karyawan Pensiun", value: retiredEmployees.value.length, detail: "Karyawan dengan status PENSIUN" },
  { label: "PKWTT (AKTIF)", value: pkwttActive.value.length, detail: `${countGender(pkwttActive.value, true)} Laki-laki · ${countGender(pkwttActive.value, false)} Perempuan` },
  { label: "PKWT (AKTIF)", value: pkwtActive.value.length, detail: `${countGender(pkwtActive.value, true)} Laki-laki · ${countGender(pkwtActive.value, false)} Perempuan` },
  { label: "Karyawan Staff", value: activeEmployees.value.filter((employee) => normalized(employee.divisi) === "STAFF").length, detail: "Divisi STAFF (Aktif)" },
  { label: "Karyawan Housekeeping", value: activeEmployees.value.filter((employee) => normalized(employee.divisi) === "HOUSEKEEPING").length, detail: "Divisi HOUSEKEEPING (Aktif)" },
  { label: "Karyawan Engineering", value: activeEmployees.value.filter((employee) => normalized(employee.divisi) === "ENGINEERING").length, detail: "Divisi ENGINEERING (Aktif)" },
  { label: "Karyawan Security", value: activeEmployees.value.filter((employee) => normalized(employee.divisi) === "SECURITY").length, detail: "Divisi SECURITY (Aktif)" },
  { label: "Karyawan Kantor Pengelola", value: activeEmployees.value.filter((employee) => normalized(employee.tempat_kerja) === "KANTOR").length, detail: "Tempat Kerja KANTOR (Aktif)" },
  { label: "Karyawan Lapangan", value: activeEmployees.value.filter((employee) => normalized(employee.tempat_kerja) === "LAPANGAN").length, detail: "Tempat Kerja LAPANGAN (Aktif)" },
  { label: "Perempuan", value: countGender(activeEmployees.value, false), detail: "Total Karyawan Perempuan (Aktif)" },
  { label: "Laki-laki", value: countGender(activeEmployees.value, true), detail: "Total Karyawan Laki-laki (Aktif)" },
]);

async function loadEmployees() {
  isLoading.value = true;
  errorMessage.value = "";
  try {
    employees.value = await invoke<Employee[]>("list_employees", { search: "" });
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isLoading.value = false;
  }
}

watch(() => props.refreshKey, () => void loadEmployees());
onMounted(() => void loadEmployees());
</script>

<template>
  <section class="space-y-6">
    <!-- Header -->
    <header class="flex flex-col gap-4 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p class="text-xs font-semibold uppercase tracking-wider text-blue-600">Apartemen Oasis Mitra Sarana</p>
        <h2 class="mt-1 text-2xl font-bold text-slate-900">Dashboard Data Karyawan</h2>
        <p class="mt-1 text-sm text-slate-500">Ringkasan statistik data karyawan yang disesuaikan dengan rekapitulasi master spreadsheet.</p>
      </div>
      <div class="flex flex-wrap gap-2">
        <button class="rounded-lg bg-blue-600 px-4 py-2.5 text-sm font-semibold text-white hover:bg-blue-700 transition-colors" @click="emit('import')">
          📁 Import File Excel
        </button>
        <button class="rounded-lg border border-slate-300 bg-white px-4 py-2.5 text-sm font-semibold text-slate-700 hover:bg-slate-50 transition-colors" @click="emit('add')">
          ➕ Tambah Karyawan
        </button>
      </div>
    </header>

    <p v-if="isLoading" class="rounded-xl bg-white p-5 text-sm text-slate-500 shadow-sm">Memuat ringkasan data...</p>
    <p v-if="errorMessage" class="rounded-lg bg-red-50 p-3 text-sm text-red-700">{{ errorMessage }}</p>

    <section class="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
      <div class="flex items-center justify-between border-b border-slate-200 px-6 py-5">
        <div>
          <h3 class="font-bold text-slate-900 text-lg">Rekapitulasi Data Karyawan</h3>
          <p class="mt-0.5 text-xs text-slate-500">12 metrik utama yang dihitung langsung dari database master karyawan.</p>
        </div>
        <span class="rounded-full bg-blue-50 px-3 py-1 text-xs font-semibold text-blue-700">
          Total Records: {{ employees.length }}
        </span>
      </div>

      <div class="overflow-x-auto">
        <table class="min-w-full text-left text-sm">
          <thead class="bg-slate-50 text-xs uppercase tracking-wider text-slate-500 border-b border-slate-200">
            <tr>
              <th class="w-16 px-6 py-3.5 text-center">No.</th>
              <th class="px-6 py-3.5">Jenis Data</th>
              <th class="px-6 py-3.5 text-right w-48">Keterangan (Jumlah)</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100">
            <tr 
              v-for="(metric, index) in metrics" 
              :key="metric.label" 
              class="hover:bg-slate-50/60 transition-colors"
            >
              <td class="px-6 py-4 text-center font-mono text-xs font-semibold text-slate-400">{{ index + 1 }}</td>
              <td class="px-6 py-4">
                <div class="font-semibold text-slate-800">{{ metric.label }}</div>
                <div class="text-xs text-slate-400 font-normal mt-0.5">{{ metric.detail }}</div>
              </td>
              <td class="px-6 py-4 text-right">
                <span class="text-lg font-bold text-slate-900 font-mono">{{ metric.value }}</span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>
  </section>
</template>