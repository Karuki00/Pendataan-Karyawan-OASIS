<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Employee } from "../types";

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

interface MetricCard {
  key: MetricFilterKey;
  label: string;
  value: number;
  detail: string;
  icon: string;
  theme: "emerald" | "amber" | "indigo" | "sky" | "rose" | "teal" | "blue" | "slate";
}

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

const activeEmployees = computed(() => employees.value.filter(isActive));
const retiredEmployees = computed(() => employees.value.filter((employee) => normalized(employee.status) === "PENSIUN"));

const pkwttActive = computed(() => activeEmployees.value.filter((employee) => normalized(employee.perjanjian_kerja || "PKWTT") === "PKWTT"));
const pkwtActive = computed(() => activeEmployees.value.filter((employee) => normalized(employee.perjanjian_kerja) === "PKWT"));

const totalActive = computed(() => activeEmployees.value.length || 1);

// Styled 12 Metrics grouped with custom visual themes & icons
const metrics = computed<MetricCard[]>(() => [
  { key: "ALL_ACTIVE", label: "Total Karyawan Aktif", value: activeEmployees.value.length, detail: "Status Aktif Terdaftar", icon: "👥", theme: "emerald" },
  { key: "PENSIUN", label: "Karyawan Pensiun", value: retiredEmployees.value.length, detail: "Status Pensiun / Non-Aktif", icon: "🏛️", theme: "amber" },
  { key: "PKWTT", label: "Karyawan Tetap (PKWTT)", value: pkwttActive.value.length, detail: `${countGender(pkwttActive.value, true)} L · ${countGender(pkwttActive.value, false)} P`, icon: "📜", theme: "teal" },
  { key: "PKWT", label: "Karyawan Kontrak (PKWT)", value: pkwtActive.value.length, detail: `${countGender(pkwtActive.value, true)} L · ${countGender(pkwtActive.value, false)} P`, icon: "⏳", theme: "sky" },
  { key: "STAFF", label: "Divisi Staff", value: activeEmployees.value.filter((e) => normalized(e.divisi) === "STAFF").length, detail: "Pengelola & Administrasi", icon: "💼", theme: "indigo" },
  { key: "ENGINEERING", label: "Divisi Engineering", value: activeEmployees.value.filter((e) => normalized(e.divisi) === "ENGINEERING").length, detail: "Teknisi & Maintenance", icon: "🛠️", theme: "blue" },
  { key: "SECURITY", label: "Divisi Security", value: activeEmployees.value.filter((e) => normalized(e.divisi) === "SECURITY").length, detail: "Keamanan & Regu Patroli", icon: "🛡️", theme: "rose" },
  { key: "HOUSEKEEPING", label: "Divisi Housekeeping", value: activeEmployees.value.filter((e) => normalized(e.divisi) === "HOUSEKEEPING").length, detail: "Kebersihan & Taman", icon: "🧹", theme: "emerald" },
  { key: "KANTOR", label: "Kantor Pengelola", value: activeEmployees.value.filter((e) => normalized(e.tempat_kerja) === "KANTOR").length, detail: "Karyawan Penempatan Kantor", icon: "🏢", theme: "indigo" },
  { key: "LAPANGAN", label: "Karyawan Lapangan", value: activeEmployees.value.filter((e) => normalized(e.tempat_kerja) === "LAPANGAN").length, detail: "Karyawan Penempatan Lapangan", icon: "🌳", theme: "teal" },
  { key: "LAKI_LAKI", label: "Laki-laki", value: countGender(activeEmployees.value, true), detail: "Karyawan Pria (Aktif)", icon: "👨", theme: "sky" },
  { key: "PEREMPUAN", label: "Perempuan", value: countGender(activeEmployees.value, false), detail: "Karyawan Wanita (Aktif)", icon: "👩", theme: "rose" },
]);

function getThemeClasses(theme: MetricCard["theme"]) {
  switch (theme) {
    case "emerald":
      return {
        card: "hover:border-emerald-500/50 hover:shadow-emerald-500/10",
        iconBg: "bg-emerald-500/10 text-emerald-600",
        bar: "bg-emerald-500",
        badge: "bg-emerald-50 text-emerald-700 border-emerald-200",
      };
    case "amber":
      return {
        card: "hover:border-amber-500/50 hover:shadow-amber-500/10",
        iconBg: "bg-amber-500/10 text-amber-600",
        bar: "bg-amber-500",
        badge: "bg-amber-50 text-amber-700 border-amber-200",
      };
    case "teal":
      return {
        card: "hover:border-teal-500/50 hover:shadow-teal-500/10",
        iconBg: "bg-teal-500/10 text-teal-600",
        bar: "bg-teal-500",
        badge: "bg-teal-50 text-teal-700 border-teal-200",
      };
    case "sky":
      return {
        card: "hover:border-sky-500/50 hover:shadow-sky-500/10",
        iconBg: "bg-sky-500/10 text-sky-600",
        bar: "bg-sky-500",
        badge: "bg-sky-50 text-sky-700 border-sky-200",
      };
    case "indigo":
      return {
        card: "hover:border-indigo-500/50 hover:shadow-indigo-500/10",
        iconBg: "bg-indigo-500/10 text-indigo-600",
        bar: "bg-indigo-500",
        badge: "bg-indigo-50 text-indigo-700 border-indigo-200",
      };
    case "blue":
      return {
        card: "hover:border-blue-500/50 hover:shadow-blue-500/10",
        iconBg: "bg-blue-500/10 text-blue-600",
        bar: "bg-blue-500",
        badge: "bg-blue-50 text-blue-700 border-blue-200",
      };
    case "rose":
      return {
        card: "hover:border-rose-500/50 hover:shadow-rose-500/10",
        iconBg: "bg-rose-500/10 text-rose-600",
        bar: "bg-rose-500",
        badge: "bg-rose-50 text-rose-700 border-rose-200",
      };
    default:
      return {
        card: "hover:border-slate-400/50 hover:shadow-slate-500/10",
        iconBg: "bg-slate-500/10 text-slate-600",
        bar: "bg-slate-500",
        badge: "bg-slate-50 text-slate-700 border-slate-200",
      };
  }
}

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
    <!-- Header Action Toolbar -->
    <div class="flex flex-wrap items-center justify-between gap-4 rounded-2xl bg-white/70 p-4 shadow-sm border border-slate-200/80 backdrop-blur-md">
      <div>
        <h2 class="text-lg font-bold tracking-tight text-slate-900">Dashboard &amp; Decision Center</h2>
        <p class="text-xs text-slate-500">Klik kartu metrik untuk memfilter tabel master data secara instan</p>
      </div>
      <div class="flex items-center gap-2.5">
        <button 
          @click="emit('import')" 
          class="inline-flex items-center gap-2 rounded-xl border border-emerald-600/30 bg-emerald-50 px-4 py-2.5 text-xs font-semibold text-emerald-700 hover:bg-emerald-600 hover:text-white transition-all shadow-xs cursor-pointer active:scale-95"
        >
          <span>📥 Import Excel</span>
        </button>
        <button 
          @click="emit('add')" 
          class="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-emerald-600 to-teal-600 px-4 py-2.5 text-xs font-semibold text-white shadow-md shadow-emerald-600/20 hover:from-emerald-700 hover:to-teal-700 transition-all cursor-pointer active:scale-95"
        >
          <span>➕ Tambah Karyawan</span>
        </button>
      </div>
    </div>

    <p v-if="errorMessage" class="rounded-xl bg-rose-50 p-4 text-xs font-semibold text-rose-700 border border-rose-200">
      ⚠️ {{ errorMessage }}
    </p>

    <!-- 12 Dynamic High-Tech Metric Cards Grid -->
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
      <div
        v-for="item in metrics"
        :key="item.key"
        @click="emit('selectMetric', item.key)"
        class="group relative cursor-pointer overflow-hidden rounded-2xl border border-slate-200/80 bg-white/90 p-5 shadow-sm transition-all duration-200 hover:-translate-y-1 hover:shadow-lg active:scale-98 backdrop-blur-md"
        :class="getThemeClasses(item.theme).card"
      >
        <!-- Card Header: Title & Icon -->
        <div class="flex items-start justify-between gap-3">
          <div class="space-y-1">
            <span class="text-xs font-semibold tracking-wide text-slate-500 group-hover:text-slate-900 transition-colors line-clamp-1">
              {{ item.label }}
            </span>
            <div class="text-3xl font-black text-slate-900 tracking-tight">
              {{ item.value }}
            </div>
          </div>
          
          <div 
            class="flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl text-xl shadow-xs transition-transform group-hover:scale-110"
            :class="getThemeClasses(item.theme).iconBg"
          >
            {{ item.icon }}
          </div>
        </div>

        <!-- Card Footer: Details & Ratio Progress Bar -->
        <div class="mt-4 space-y-2 border-t border-slate-100 pt-3">
          <div class="flex items-center justify-between text-[11px]">
            <span class="text-slate-500 font-medium truncate max-w-[150px]">{{ item.detail }}</span>
            <span 
              class="rounded-full border px-2 py-0.5 text-[9px] font-bold uppercase transition-colors"
              :class="getThemeClasses(item.theme).badge"
            >
              Filter ➔
            </span>
          </div>

          <!-- Mini Percentage Progress Bar -->
          <div class="h-1.5 w-full rounded-full bg-slate-100 overflow-hidden">
            <div 
              class="h-full rounded-full transition-all duration-500 ease-out"
              :class="getThemeClasses(item.theme).bar"
              :style="{ width: `${Math.min(Math.round((item.value / totalActive) * 100), 100)}%` }"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>