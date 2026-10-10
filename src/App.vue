<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import AppHeader from "./components/layouts/AppHeader.vue";
import AppNavigation, { type Tab } from "./components/layouts/AppNavigation.vue";
import AnomalyToast from "./components/layouts/AnomalyToast.vue";

import DashboardView, { type MetricFilterKey } from "./components/DashboardView.vue";
import EmployeeTable from "./components/EmployeeTable.vue";
import AbsensiView from "./views/AbsensiView.vue"; 
import RekapCutiView from "./views/RekapCutiView.vue"; // <-- Imported view

import ExcelImporter from "./components/ExcelImporter.vue";
import EmployeeModal from "./components/EmployeeModal.vue";
import AboutModal from "./components/AboutModal.vue";

import { useUpdater } from "./composables/useUpdater";
import { useAnomalyChecker } from "./composables/useAnomalyChecker";
import type { Employee } from "./types";

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

onMounted(() => {
  void checkDataAnomalies();
  void checkForUpdates();
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
  <div class="min-h-screen text-slate-900 bg-gradient-to-br from-[#fef3e2] via-[#fce6bd] to-[#f7d6a0] antialiased">
    
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
  </div>
</template>