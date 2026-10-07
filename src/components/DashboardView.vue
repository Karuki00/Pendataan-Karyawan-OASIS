<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Employee } from "../types";

// Key/filter yang dikirim saat metrik diklik
export type MetricFilterKey =
  | "ALL_ACTIVE"
  | "PENSIUN"
  | "PKWTT"
  | "PKWT"
  | "STAFF"
  | "HOUSEKEEPING"
  | "ENGINEERING"
  | "SECURITY"
  | "KANTOR"
  | "LAPANGAN"
  | "PEREMPUAN"
  | "LAKI_LAKI";

const props = defineProps<{ refreshKey: number }>();
const emit = defineEmits<{
  import: [];
  add: [];
  selectMetric: [filterKey: MetricFilterKey];
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

// 3. Exact 12 Metrics matching !DASHBOARD! dengan key filter
const metrics = computed(() => [
  { key: "ALL_ACTIVE" as MetricFilterKey, label: "Total Karyawan Aktif (Saat Ini)", value: activeEmployees.value.length, detail: "Karyawan dengan status AKTIF" },
  { key: "PENSIUN" as MetricFilterKey, label: "Karyawan Pensiun", value: retiredEmployees.value.length, detail: "Karyawan dengan status PENSIUN" },
  { key: "PKWTT" as MetricFilterKey, label: "PKWTT (AKTIF)", value: pkwttActive.value.length, detail: `${countGender(pkwttActive.value, true)} Laki-laki · ${countGender(pkwttActive.value, false)} Perempuan` },
  { key: "PKWT" as MetricFilterKey, label: "PKWT (AKTIF)", value: pkwtActive.value.length, detail: `${countGender(pkwtActive.value, true)} Laki-laki · ${countGender(pkwtActive.value, false)} Perempuan` },
  { key: "STAFF" as MetricFilterKey, label: "Karyawan Staff", value: activeEmployees.value.filter((employee) => normalized(employee.divisi) === "STAFF").length, detail: "Divisi STAFF (Aktif)" },
  { key: "HOUSEKEEPING" as MetricFilterKey, label: "Karyawan Housekeeping", value: activeEmployees.value.filter((employee) => normalized(employee.divisi) === "HOUSEKEEPING").length, detail: "Divisi HOUSEKEEPING (Aktif)" },
  { key: "ENGINEERING" as MetricFilterKey, label: "Karyawan Engineering", value: activeEmployees.value.filter((employee) => normalized(employee.divisi) === "ENGINEERING").length, detail: "Divisi ENGINEERING (Aktif)" },
  { key: "SECURITY" as MetricFilterKey, label: "Karyawan Security", value: activeEmployees.value.filter((employee) => normalized(employee.divisi) === "SECURITY").length, detail: "Divisi SECURITY (Aktif)" },
  { key: "KANTOR" as MetricFilterKey, label: "Karyawan Kantor Pengelola", value: activeEmployees.value.filter((employee) => normalized(employee.tempat_kerja) === "KANTOR").length, detail: "Tempat Kerja KANTOR (Aktif)" },
  { key: "LAPANGAN" as MetricFilterKey, label: "Karyawan Lapangan", value: activeEmployees.value.filter((employee) => normalized(employee.tempat_kerja) === "LAPANGAN").length, detail: "Tempat Kerja LAPANGAN (Aktif)" },
  { key: "PEREMPUAN" as MetricFilterKey, label: "Perempuan", value: countGender(activeEmployees.value, false), detail: "Total Karyawan Perempuan (Aktif)" },
  { key: "LAKI_LAKI" as MetricFilterKey, label: "Laki-laki", value: countGender(activeEmployees.value, true), detail: "Total Karyawan Laki-laki (Aktif)" },
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
  <div class="space-y-6">
    <!-- Header Action Buttons -->
    <div class="flex flex-wrap items-center justify-between gap-3">
      <h2 class="text-xl font-bold text-slate-800">Dashboard Ringkasan</h2>
      <div class="flex gap-2">
        <button @click="emit('import')" class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-semibold text-white hover:bg-emerald-700 transition-colors">
          Import Excel
        </button>
        <button @click="emit('add')" class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-700 transition-colors">
          + Tambah Karyawan
        </button>
      </div>
    </div>

    <p v-if="errorMessage" class="rounded-lg bg-red-50 p-3 text-sm text-red-700">{{ errorMessage }}</p>

    <!-- 12 Interactive Metric Cards Grid -->
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      <div
        v-for="item in metrics"
        :key="item.key"
        @click="emit('selectMetric', item.key)"
        class="group cursor-pointer rounded-2xl border border-slate-200 bg-white p-5 shadow-sm hover:border-blue-500 hover:shadow-md transition-all active:scale-[0.98]"
      >
        <div class="flex items-center justify-between">
          <span class="text-xs font-semibold text-slate-500 group-hover:text-blue-600 transition-colors">{{ item.label }}</span>
          <span class="rounded-full bg-slate-100 px-2 py-0.5 text-[10px] font-bold text-slate-600 group-hover:bg-blue-50 group-hover:text-blue-600">Klik Filter</span>
        </div>
        <div class="mt-2 text-3xl font-extrabold text-slate-800">{{ item.value }}</div>
        <div class="mt-1 text-xs text-slate-400">{{ item.detail }}</div>
      </div>
    </div>
  </div>
</template>