<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import DashboardView, { type MetricFilterKey } from "./components/DashboardView.vue";
import ExcelImporter from "./components/ExcelImporter.vue";
import EmployeeTable from "./components/EmployeeTable.vue";
import EmployeeModal from "./components/EmployeeModal.vue";
import AboutModal from "./components/AboutModal.vue";
import tauriConfig from "../src-tauri/tauri.conf.json";
import type { Employee } from "./types";
import logoUrl from "../src/assets/LOGO_OASIS_V2.png";

type Tab = "dashboard" | "master";
const activeTab = ref<Tab>("dashboard");
const refreshKey = ref(0);

const isModalOpen = ref(false);
const isImportModalOpen = ref(false);
const showAboutModal = ref(false);
const employeeToEdit = ref<Employee | null>(null);

const masterSearchQuery = ref("");
const activeMetricKey = ref<MetricFilterKey | null>(null);
const appVersion = tauriConfig.version;

// Floating Reminder State
const showAnomalyToast = ref(false);
const warningEmployeesCount = ref(0);
const anomalyDetails = ref<string[]>([]);
const warningSearchTerms = ref<string[]>([]);

// Fungsi Deteksi Peringatan & Anomali Data Karyawan di Database
async function checkDataAnomalies() {
  try {
    const employees = await invoke<Employee[]>("list_employees", { search: "" });
    
    // Cukup filter karyawan yang memiliki is_flagged === true
    const flaggedEmployees = employees.filter((emp) => emp.is_flagged);
    
    warningEmployeesCount.value = flaggedEmployees.length;

    if (flaggedEmployees.length > 0) {
      anomalyDetails.value = [
        `Terdapat ${flaggedEmployees.length} data karyawan yang perlu diperbaiki/dilengkapi.`
      ];
      showAnomalyToast.value = true;
    } else {
      showAnomalyToast.value = false;
    }
  } catch (err) {
    console.error("Gagal memeriksa flag data:", err);
  }
}

// Handler saat klik tombol "Lihat Data Warning"
function filterWarningData() {
  // Cukup aktifkan filter khusus "flagged" di tabel master
  activeMetricKey.value = "flagged" as any; 
  masterSearchQuery.value = "";
  activeTab.value = "master";
  showAnomalyToast.value = false;
}

function handleSaved() {
  refreshKey.value++;
  void checkDataAnomalies();
}

function openCreate() {
  employeeToEdit.value = null;
  isModalOpen.value = true;
}

function openEdit(employee: Employee) {
  employeeToEdit.value = employee;
  isModalOpen.value = true;
}

function openImport() { isImportModalOpen.value = true; }

function openMaster() {
  masterSearchQuery.value = "";
  activeMetricKey.value = null;
  activeTab.value = "master";
}

function imported() { 
  refreshKey.value++; 
  isImportModalOpen.value = false; 
  openMaster(); 
  void checkDataAnomalies();
}

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

function clearMetricFilter() { activeMetricKey.value = null; }
function resetMasterSearch() { masterSearchQuery.value = ""; }

// Periksa data saat aplikasi dibuka
onMounted(() => {
  void checkDataAnomalies();
});

watch(refreshKey, () => {
  void checkDataAnomalies();
});
</script>

<template>
  <div class="min-h-screen text-slate-900 bg-gradient-to-br from-[#fef3e2] via-[#fce6bd] to-[#f7d6a0] antialiased">
    
    <!-- Floating Reminder Toast Notification (Akan selalu muncul saat app dibuka jika ada warning) -->
    <Teleport to="body">
      <Transition name="toast">
        <div 
          v-if="showAnomalyToast" 
          class="fixed top-6 right-6 z-[9999] flex items-start gap-3.5 rounded-2xl bg-slate-900/95 p-4.5 text-white shadow-2xl border border-amber-500/40 backdrop-blur-md max-w-md"
        >
          <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-amber-500/20 text-amber-400 text-xl border border-amber-500/30">
            ⚠️
          </div>
          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between gap-2">
              <h4 class="text-xs font-bold text-amber-300 uppercase tracking-wider">
                Pengingat Data Karyawan (Warning)
              </h4>
              <button 
                @click="showAnomalyToast = false" 
                title="Tutup Notifikasi"
                class="text-slate-400 hover:text-white transition-colors p-0.5 text-sm cursor-pointer"
              >
                ✕
              </button>
            </div>
            
            <p class="text-[11px] text-slate-300 mt-1">
              Ditemukan <strong class="text-amber-300 font-bold">{{ warningEmployeesCount }} data karyawan</strong> yang memerlukan perhatian:
            </p>

            <ul class="mt-1.5 space-y-1 text-xs text-amber-100/90 list-disc list-inside bg-amber-950/40 p-2 rounded-lg border border-amber-500/20">
              <li v-for="(detail, index) in anomalyDetails" :key="index" class="truncate">
                {{ detail }}
              </li>
            </ul>

            <div class="mt-3 flex items-center gap-2 pt-2 border-t border-slate-800">
              <button 
                @click="filterWarningData"
                class="rounded-lg bg-amber-500 px-3 py-1.5 text-[11px] font-bold text-slate-950 hover:bg-amber-400 transition-colors shadow-xs cursor-pointer flex items-center gap-1.5"
              >
                <span>🔍 Lihat Data Warning</span>
              </button>
              <button 
                @click="showAnomalyToast = false"
                class="rounded-lg bg-slate-800 px-3 py-1.5 text-[11px] font-semibold text-slate-300 hover:bg-slate-700 transition-colors cursor-pointer"
              >
                Nanti Saja
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Header Utama -->
    <header class="sticky top-0 z-30 border-b border-emerald-900/10 bg-gradient-to-r from-[#0d361f] via-[#115231] to-[#18603b] shadow-lg backdrop-blur-md">
      <div class="mx-auto flex max-w-7xl items-center justify-between px-6 py-3.5">
        <div class="flex items-center gap-4">
          <div class="flex items-center justify-center rounded-2xl bg-white/10 p-2 border border-white/10 shadow-inner">
            <img :src="logoUrl" alt="Logo Apartemen Oasis" class="h-12 w-auto object-contain" />
          </div>
          <div>
            <h1 class="text-xl font-bold text-white flex items-center gap-2">
              Database Karyawan
              <span class="rounded-full bg-emerald-400/20 px-2 py-0.5 text-[10px] text-emerald-200 border border-emerald-400/30">
                Oasis Portal
              </span>
            </h1>
            <p class="text-xs text-emerald-100/70">Sistem Pendataan &amp; Pusat Keputusan Manajemen</p>
          </div>
        </div>

        <button @click="showAboutModal = true" class="flex items-center gap-2 rounded-xl border border-white/20 bg-white/10 px-3.5 py-2 text-xs font-semibold text-white hover:bg-white/20 cursor-pointer">
          <span>ℹ️ Tentang Aplikasi</span>
          <span class="rounded-lg bg-emerald-950/40 px-2 py-0.5 text-[10px] font-mono">v{{ appVersion }}</span>
        </button>
      </div>
    </header>

    <AboutModal v-if="showAboutModal" @close="showAboutModal = false" />

    <!-- Content Wrapper -->
    <main class="mx-auto max-w-7xl space-y-6 px-6 py-8">
      <nav class="flex items-center gap-1.5 rounded-2xl bg-white/80 p-1.5 shadow-md border border-amber-900/5">
        <button 
          class="flex items-center gap-2 rounded-xl px-5 py-2.5 text-sm font-semibold transition-all cursor-pointer"
          :class="activeTab === 'dashboard' ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-md' : 'text-slate-600 hover:bg-slate-100'" 
          @click="activeTab = 'dashboard'; masterSearchQuery = ''"
        >
          <span>📊 Dashboard &amp; Decision Center</span>
        </button>
        <button 
          class="flex items-center gap-2 rounded-xl px-5 py-2.5 text-sm font-semibold transition-all cursor-pointer"
          :class="activeTab === 'master' ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-md' : 'text-slate-600 hover:bg-slate-100'" 
          @click="openMaster"
        >
          <span>📋 Master Data Karyawan</span>
        </button>
      </nav>

      <DashboardView 
        v-if="activeTab === 'dashboard'" 
        :refresh-key="refreshKey" 
        @import="openImport" 
        @add="openCreate" 
        @open-master="openMaster" 
        @open-master-with-filter="openMasterWithFilter" 
        @select-metric="selectMetric" 
      />

      <EmployeeTable 
        v-if="activeTab === 'master'" 
        :refresh-key="refreshKey" 
        :initial-search="masterSearchQuery" 
        :active-metric-key="activeMetricKey || undefined" 
        @reset-search="resetMasterSearch" 
        @edit="openEdit" 
        @clear-metric-filter="clearMetricFilter" 
      />
    </main>

    <EmployeeModal 
      :is-open="isModalOpen" 
      :employee-to-edit="employeeToEdit"
      @close="isModalOpen = false"
      @saved="handleSaved"
    />

    <!-- Modal Import Excel -->
    <Teleport to="body">
      <div v-if="isImportModalOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/60 p-4" @click.self="isImportModalOpen = false">
        <div class="max-h-[92vh] w-full max-w-6xl overflow-auto rounded-3xl bg-slate-50 p-6 shadow-2xl space-y-4">
          <div class="flex items-center justify-between border-b pb-4">
            <h2 class="text-lg font-bold text-slate-900">Import Data Karyawan</h2>
            <button type="button" class="text-xl text-slate-400 hover:text-slate-700 cursor-pointer" @click="isImportModalOpen = false">&times;</button>
          </div>
          <ExcelImporter @saved="imported" />
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.35s cubic-bezier(0.16, 1, 0.3, 1);
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(-1rem) scale(0.95);
}
</style>