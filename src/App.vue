<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import AppHeader from "./components/layouts/AppHeader.vue";
import AppNavigation, { type Tab } from "./components/layouts/AppNavigation.vue";
import AnomalyToast from "./components/layouts/AnomalyToast.vue";

import DashboardView, { type MetricFilterKey } from "./components/DashboardView.vue";
import EmployeeTable from "./components/EmployeeTable.vue";
import AbsensiView from "./views/AbsensiView.vue"; 
import RekapCutiView from "./views/RekapCutiView.vue";

import ExcelImporter from "./components/ExcelImporter.vue";
import EmployeeModal from "./components/EmployeeModal.vue";
import AboutModal from "./components/AboutModal.vue";

import { useUpdater } from "./composables/useUpdater";
import { useAnomalyChecker } from "./composables/useAnomalyChecker";
import type { Employee } from "./types";

// --- Loading Intro State ---
const isInitializing = ref(true);
const loadingProgress = ref(15);
const loadingStatus = ref("Menyiapkan aplikasi...");

const activeTab = ref<Tab>("dashboard");
const refreshKey = ref(0);

const isModalOpen = ref(false);
const isImportModalOpen = ref(false);
const showAboutModal = ref(false);
const employeeToEdit = ref<Employee | null>(null);

const masterSearchQuery = ref("");
const activeMetricKey = ref<MetricFilterKey | null>(null);

const { checkForUpdates } = useUpdater();
const { showAnomalyToast, warningEmployeesCount, anomalyDetails, checkDataAnomalies } = useAnomalyChecker();

// Bootstrap / Initial App Hydration
async function bootstrapApp() {
  try {
    loadingStatus.value = "Memeriksa integritas data & anomali...";
    loadingProgress.value = 30;
    await checkDataAnomalies();

    loadingStatus.value = "Memeriksa pembaruan aplikasi...";
    loadingProgress.value = 60;

    // Automatically check, download, install, and relaunch if an update exists
    const isUpdating = await checkForUpdates(true, (progress, statusText) => {
      loadingStatus.value = statusText;
      // Scale update progress smoothly from 60% to 100% on the splash screen
      loadingProgress.value = 60 + Math.round((progress / 100) * 40);
    });

    // If an update was triggered, relaunch is handled inside useUpdater
    if (isUpdating) return;

    loadingStatus.value = "Menyiapkan antarmuka...";
    loadingProgress.value = 100;

    await new Promise((resolve) => setTimeout(resolve, 300));
  } catch (error) {
    console.error("Gagal melakukan inisialisasi aplikasi:", error);
  } finally {
    isInitializing.value = false;
  }
}

onMounted(() => {
  void bootstrapApp();
});

watch(refreshKey, () => {
  void checkDataAnomalies();
});

function filterWarningData() {
  activeMetricKey.value = "flagged" as any; 
  masterSearchQuery.value = "";
  activeTab.value = "master";
  showAnomalyToast.value = false;
}

function handleSaved() { refreshKey.value++; }
function openCreate() { employeeToEdit.value = null; isModalOpen.value = true; }
function openEdit(employee: Employee) { employeeToEdit.value = employee; isModalOpen.value = true; }
function openMaster() { masterSearchQuery.value = ""; activeMetricKey.value = null; activeTab.value = "master"; }
function imported() { refreshKey.value++; isImportModalOpen.value = false; openMaster(); }

function switchTab(tab: Tab) {
  if (tab === 'master') openMaster();
  else {
    masterSearchQuery.value = "";
    activeTab.value = tab;
  }
}
</script>

<template>
  <div class="min-h-screen text-slate-900 bg-gradient-to-br from-[#fef3e2] via-[#fce6bd] to-[#f7d6a0] antialiased relative">
    
    <!-- Intro Loading Screen Overlay -->
    <Transition name="splash-fade">
      <div 
        v-if="isInitializing" 
        class="fixed inset-0 z-[99999] flex flex-col items-center justify-center bg-gradient-to-br from-[#1e293b] via-[#0f172a] to-[#020617] p-6 select-none"
      >
        <div class="flex flex-col items-center gap-6 max-w-sm text-center">
          <!-- Animated Branding Badge -->
          <div class="relative flex items-center justify-center w-20 h-20 rounded-3xl bg-amber-500/10 border border-amber-500/20 shadow-2xl shadow-amber-900/30">
            <span class="text-4xl animate-bounce">🏢</span>
            <div class="absolute inset-0 rounded-3xl border border-amber-400/20 animate-ping"></div>
          </div>

          <!-- Application Title & Subtitle -->
          <div>
            <h1 class="text-xl font-extrabold tracking-wide text-white">Pendataan Karyawan OASIS</h1>
            <p class="text-xs text-amber-200/70 mt-1 font-medium">Apartemen Mitra Oasis Sarana</p>
          </div>

          <!-- Progress Bar & Status Text -->
          <div class="w-full space-y-2.5">
            <div class="h-2 w-full bg-slate-800/80 rounded-full overflow-hidden border border-slate-700/60 p-0.5">
              <div 
                class="h-full bg-gradient-to-r from-amber-500 via-emerald-400 to-teal-400 rounded-full transition-all duration-300 ease-out"
                :style="{ width: `${loadingProgress}%` }"
              ></div>
            </div>
            <p class="text-[11px] font-mono text-amber-300/90 tracking-wide">
              {{ loadingStatus }}
            </p>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Main Application Canvas -->
    <template v-if="!isInitializing">
      <AnomalyToast 
        :show="showAnomalyToast" 
        :count="warningEmployeesCount" 
        :details="anomalyDetails"
        @close="showAnomalyToast = false"
        @view-warning="filterWarningData"
      />

      <AppHeader @open-about="showAboutModal = true" />
      <AboutModal v-if="showAboutModal" @close="showAboutModal = false" />

      <main class="mx-auto max-w-7xl space-y-6 px-6 py-8">
        <!-- 4-Tab Navigation Bar -->
        <AppNavigation :active-tab="activeTab" @select-tab="switchTab" />

        <!-- Page 1: Dashboard -->
        <DashboardView 
          v-if="activeTab === 'dashboard'" 
          :refresh-key="refreshKey" 
          @import="isImportModalOpen = true" 
          @add="openCreate" 
          @open-master="openMaster" 
          @open-master-with-filter="(q: string) => { masterSearchQuery = q; activeTab = 'master'; }" 
          @select-metric="(k: MetricFilterKey) => { activeMetricKey = k; activeTab = 'master'; }" 
        />

        <!-- Page 2: Master Karyawan Table -->
        <EmployeeTable 
          v-if="activeTab === 'master'" 
          :refresh-key="refreshKey" 
          :initial-search="masterSearchQuery" 
          :active-metric-key="activeMetricKey || undefined" 
          @reset-search="masterSearchQuery = ''" 
          @edit="openEdit" 
          @clear-metric-filter="activeMetricKey = null" 
        />

        <!-- Page 3: Rekapitulasi Absensi -->
        <AbsensiView v-if="activeTab === 'absensi'" />

        <!-- Page 4: Rekap Cuti Staff -->
        <RekapCutiView v-if="activeTab === 'cuti'" />
      </main>

      <EmployeeModal 
        :is-open="isModalOpen" 
        :employee-to-edit="employeeToEdit"
        @close="isModalOpen = false"
        @saved="handleSaved"
      />

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
    </template>
  </div>
</template>

<style scoped>
/* Smooth Splash Screen Fade-out Transition */
.splash-fade-leave-active {
  transition: opacity 0.4s ease-in-out, transform 0.4s ease-in-out;
}
.splash-fade-leave-to {
  opacity: 0;
  transform: scale(0.97);
}
</style>