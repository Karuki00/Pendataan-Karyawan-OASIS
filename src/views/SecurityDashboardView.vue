<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { getDb, fetchDashboardStats, fetchReportedProblems, seedAllHydrantAssets } from '../services/db.ts';
import { syncGoogleSheetToSqlite } from '../services/syncService.ts';
import { fetchSecurityRosterFromAppsScript, getSecurityPinStatus, loginSecurityFromAppsScript, syncSecurityUsers, updateSecurityPin } from '../services/securityUserSync.ts';
import type { Employee } from '../types';

const props = withDefaults(defineProps<{ canManagePins?: boolean }>(), {
  canManagePins: false,
});
import type { StatMetric, FacilityProblem } from '../types/inspection.ts';

interface SpreadsheetInspectionRow {
  id: number;
  response_id: string;
  inspected_at: string;
  token: string;
  inspector_badge_id: string;
  asset_code: string;
  selang: string;
  nozle: string;
  valve: string;
  lampu: string;
  bell: string;
  jack_inter: string;
  apar: string;
  pressure_gauge: string;
  kunci_pilar: string;
  notes: string;
  photo_url: string;
}

// Reactive States
const isSyncing = ref(false);
const syncMessage = ref('');
const loading = ref(false);
const securityEmployees = ref<Employee[]>([]);
const selectedSecurityName = ref('');
const newPin = ref('');
const confirmPin = ref('');
const isSavingPin = ref(false);
const pinMessage = ref('');
const isResetLoginOpen = ref(false);
const resetUsername = ref('');
const resetPassword = ref('');
const resetLoginError = ref('');
const isAuthorizingReset = ref(false);
const isSyncingSecurityUsers = ref(false);
const securityUserSyncMessage = ref('');
const selectedPinExists = ref(false);
const selectedPinLoading = ref(false);

async function loadSecurityEmployees() {
  let employees: Employee[] = [];
  try {
    employees = await invoke<Employee[]>('list_security_employees_for_absensi');
  } catch {
    employees = await fetchSecurityRosterFromAppsScript();
  }
  if (!employees.length) employees = await fetchSecurityRosterFromAppsScript();
  securityEmployees.value = employees.filter((employee) => {
    const job = String(employee.jabatan || '').toUpperCase();
    return !job.includes('CHIEF SECURITY') && !job.includes('CHIEF SATPAM');
  });
}

async function syncSecurityEmployees() {
  securityUserSyncMessage.value = '';
  isSyncingSecurityUsers.value = true;
  try {
    const employees = await invoke<Employee[]>('list_employees', { search: '' });
    await syncSecurityUsers(employees);
    securityUserSyncMessage.value = 'Data Security berhasil disinkronkan.';
    await loadSecurityEmployees();
  } catch (error) {
    securityUserSyncMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isSyncingSecurityUsers.value = false;
  }
}

async function checkSelectedPin() {
  selectedPinExists.value = false;
  if (!selectedSecurityName.value) return;
  selectedPinLoading.value = true;
  try {
    const status = await getSecurityPinStatus(selectedSecurityName.value);
    selectedPinExists.value = status.hasPin;
  } catch (error) {
    pinMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    selectedPinLoading.value = false;
  }
}

async function resetSecurityPin() {
  pinMessage.value = '';
  if (!selectedSecurityName.value || !/^\d{4,6}$/.test(newPin.value)) {
    pinMessage.value = 'Pilih security dan masukkan PIN baru 4-6 digit.';
    return;
  }
  if (newPin.value !== confirmPin.value) {
    pinMessage.value = 'Konfirmasi PIN tidak sama.';
    return;
  }
  if (selectedPinExists.value && !window.confirm(`Reset PIN untuk ${selectedSecurityName.value}? PIN lama tidak dapat digunakan lagi.`)) {
    return;
  }
  isSavingPin.value = true;
  try {
    const selectedEmployee = securityEmployees.value.find((employee) => employee.nama_lengkap === selectedSecurityName.value);
    await updateSecurityPin(selectedSecurityName.value, newPin.value, selectedEmployee?.regu || selectedEmployee?.jabatan || '');
    pinMessage.value = selectedPinExists.value
      ? 'PIN berhasil di-reset. Berikan PIN baru kepada petugas secara pribadi.'
      : 'PIN baru berhasil dibuat. Berikan PIN kepada petugas secara pribadi.';
    newPin.value = '';
    confirmPin.value = '';
  } catch (error) {
    pinMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isSavingPin.value = false;
  }
}

function openResetPinLogin() {
  pinMessage.value = '';
  resetLoginError.value = '';
  resetUsername.value = '';
  resetPassword.value = '';
  isResetLoginOpen.value = true;
}

function cancelResetPinLogin() {
  isResetLoginOpen.value = false;
  resetUsername.value = '';
  resetPassword.value = '';
  resetLoginError.value = '';
}

async function authorizeAndResetPin() {
  resetLoginError.value = '';
  if (!resetUsername.value || !resetPassword.value) {
    resetLoginError.value = 'Username dan password wajib diisi.';
    return;
  }

  isAuthorizingReset.value = true;
  try {
    let role: string;
    try {
      role = await invoke<string>('authenticate', {
        username: resetUsername.value,
        password: resetPassword.value,
      });
    } catch {
      role = await loginSecurityFromAppsScript(resetUsername.value, resetPassword.value);
    }
    const isChiefSecurity = resetUsername.value === 'chief_security' && role === 'security';
    if (!isChiefSecurity) {
      throw new Error('Reset/buat PIN petugas hanya dapat dikonfirmasi oleh akun chief_security.');
    }
    cancelResetPinLogin();
    await resetSecurityPin();
  } catch (error) {
    resetLoginError.value = error instanceof Error ? error.message : String(error);
  } finally {
    isAuthorizingReset.value = false;
  }
}

// Pagination States
const currentPage = ref(1);
const itemsPerPage = 10;
const totalPages = ref(1);

// Selection state for detail preview card
const selectedInspection = ref<SpreadsheetInspectionRow | null>(null);

// Reactive Database Data
const stats = ref<StatMetric[]>([]);
const problems = ref<FacilityProblem[]>([]);
const spreadsheetRows = ref<SpreadsheetInspectionRow[]>([]);

// Fetch real data from SQLite database with Pagination
async function loadData() {
  loading.value = true;
  try {
    const db = await getDb();
    stats.value = await fetchDashboardStats();
    problems.value = await fetchReportedProblems();

    const countResult = await db.select<{total: number}[]>(`SELECT COUNT(*) as total FROM inspections`);
    const totalRecords = countResult[0]?.total || 0;
    totalPages.value = Math.max(1, Math.ceil(totalRecords / itemsPerPage));

    if (currentPage.value > totalPages.value) {
      currentPage.value = totalPages.value;
    }

    const offset = (currentPage.value - 1) * itemsPerPage;

    const rows = await db.select<SpreadsheetInspectionRow[]>(`
      SELECT 
        i.id, i.google_response_id as response_id, i.inspected_at, i.inspector_badge_id,
        a.code as asset_code, i.notes, i.photo_url,
        MAX(CASE WHEN c.component_name IN ('Selang') THEN c.condition_code END) as selang,
        MAX(CASE WHEN c.component_name IN ('Nozle', 'Nozzle') THEN c.condition_code END) as nozle,
        MAX(CASE WHEN c.component_name IN ('Valve') THEN c.condition_code END) as valve,
        MAX(CASE WHEN c.component_name IN ('Lampu') THEN c.condition_code END) as lampu,
        MAX(CASE WHEN c.component_name IN ('Bell') THEN c.condition_code END) as bell,
        MAX(CASE WHEN c.component_name IN ('J.Inter', 'Jack Inter') THEN c.condition_code END) as jack_inter,
        MAX(CASE WHEN c.component_name IN ('Apar', 'APAR') THEN c.condition_code END) as apar,
        MAX(CASE WHEN c.component_name IN ('Pressure Gauge') THEN c.condition_code END) as pressure_gauge,
        MAX(CASE WHEN c.component_name IN ('Kunci pilar', 'Kunci Box') THEN c.condition_code END) as kunci_pilar
      FROM inspections i
      JOIN assets a ON i.asset_id = a.id
      LEFT JOIN inspection_item_checks c ON i.id = c.inspection_id
      GROUP BY i.id
      ORDER BY i.inspected_at DESC
      LIMIT ${itemsPerPage} OFFSET ${offset}
    `);

    spreadsheetRows.value = rows;
    
    if (rows.length > 0 && (!selectedInspection.value || !rows.some(r => r.id === selectedInspection.value?.id))) {
      selectedInspection.value = rows[0];
    } else if (rows.length === 0) {
      selectedInspection.value = null;
    }
  } catch (err) {
    console.error('Error fetching SQLite data:', err);
  } finally {
    loading.value = false;
  }
}

function prevPage() {
  if (currentPage.value > 1) {
    currentPage.value--;
    loadData();
  }
}

function nextPage() {
  if (currentPage.value < totalPages.value) {
    currentPage.value++;
    loadData();
  }
}

async function handleSync() {
  isSyncing.value = true;
  syncMessage.value = 'Syncing...';
  
  try {
    const result = await syncGoogleSheetToSqlite();
    syncMessage.value = `Synced: ${result.synced} records.`;
    currentPage.value = 1;
    await loadData();
    setTimeout(() => syncMessage.value = '', 5000);
  } catch (err) {
    syncMessage.value = 'Sync failed.';
    console.error('Sync Error:', err);
  } finally {
    isSyncing.value = false;
  }
}

async function handleForceResync() {
  isSyncing.value = true;
  syncMessage.value = 'Re-pulling...';
  
  try {
    const db = await getDb();
    await db.execute('DELETE FROM inspection_item_checks;');
    await db.execute('DELETE FROM problems;');
    await db.execute('DELETE FROM inspections;');

    const result = await syncGoogleSheetToSqlite();
    syncMessage.value = `Reset Complete: ${result.synced} pulled!`;
    currentPage.value = 1;
    await loadData();
    setTimeout(() => syncMessage.value = '', 5000);
  } catch (err) {
    syncMessage.value = 'Reset sync failed.';
    console.error('Force Resync Error:', err);
  } finally {
    isSyncing.value = false;
  }
}

function getCellClass(val: string | null): string {
  switch (val) {
    case 'V': return 'bg-emerald-100 text-emerald-800 font-bold';
    case 'X': return 'bg-red-100 text-red-800 font-bold';
    case 'R': return 'bg-amber-100 text-amber-800 font-bold';
    default: return 'text-slate-300';
  }
}

function formatDate(dateStr: string): string {
  if (!dateStr) return '-';
  return new Date(dateStr).toLocaleString('id-ID', {
    day: '2-digit', month: '2-digit', year: 'numeric',
    hour: '2-digit', minute: '2-digit'
  });
}

onMounted(async () => {
  await loadSecurityEmployees();
  await seedAllHydrantAssets();
  await loadData();
});
</script>

<template>
  <div class="space-y-6">
    
    <!-- Header Action Toolbar -->
    <div class="flex flex-wrap items-center justify-between gap-4 rounded-2xl bg-white/70 p-4 shadow-sm border border-slate-200/80 backdrop-blur-md">
      <div>
        <h2 class="text-lg font-bold tracking-tight text-slate-900">Dashboard Inspeksi Fasilitas</h2>
        <p class="text-xs text-slate-500">Live monitoring & sinkronisasi data lapangan via Google Sheets</p>
      </div>

      <div class="flex items-center gap-2.5">
        <button
          v-if="props.canManagePins"
          @click="syncSecurityEmployees"
          :disabled="isSyncingSecurityUsers"
          class="inline-flex items-center gap-2 rounded-xl border border-blue-600/30 bg-blue-50 px-4 py-2.5 text-xs font-semibold text-blue-700 hover:bg-blue-600 hover:text-white transition-all shadow-sm cursor-pointer active:scale-95 disabled:opacity-50"
        >
          {{ isSyncingSecurityUsers ? 'Menyinkronkan Security...' : 'Sync Data Security' }}
        </button>
        <span v-if="syncMessage" class="mr-2 text-xs text-emerald-700 font-semibold bg-emerald-50 border border-emerald-200 px-3 py-1.5 rounded-lg shadow-sm">
          {{ syncMessage }}
        </span>

        <button 
          @click="handleSync" 
          :disabled="isSyncing"
          class="inline-flex items-center gap-2 rounded-xl border border-emerald-600/30 bg-emerald-50 px-4 py-2.5 text-xs font-semibold text-emerald-700 hover:bg-emerald-600 hover:text-white transition-all shadow-sm cursor-pointer active:scale-95 disabled:opacity-50"
        >
          <span>{{ isSyncing ? 'Syncing...' : '🔄 Sync Google Sheets' }}</span>
        </button>

        <button 
          @click="handleForceResync" 
          :disabled="isSyncing"
          class="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-amber-600 to-orange-600 px-4 py-2.5 text-xs font-semibold text-white shadow-md shadow-amber-600/20 hover:from-amber-700 hover:to-orange-700 transition-all cursor-pointer active:scale-95 disabled:opacity-50"
          title="Clear SQLite inspection logs and re-pull all records from Google Sheets"
        >
          <span>⚠️ Force Re-sync</span>
        </button>
      </div>
    </div>

    <!-- Chief Security PIN Management -->
    <div v-if="props.canManagePins" class="rounded-2xl border border-slate-200/80 bg-white/90 p-5 shadow-sm backdrop-blur-md">
      <div class="flex flex-wrap items-center justify-between gap-4">
        <div>
        <h3 class="text-sm font-bold text-slate-900">Buat / Reset PIN Petugas Security</h3>
        <p class="text-[11px] text-slate-500">Jika belum ada PIN, sistem membuat PIN baru. Jika sudah ada, PIN lama diganti.</p>
        </div>
        <span v-if="securityUserSyncMessage" class="text-xs font-semibold text-blue-700">{{ securityUserSyncMessage }}</span>
        <span v-if="pinMessage" class="text-xs font-semibold text-emerald-700">{{ pinMessage }}</span>
      </div>
      <div class="mt-4 grid grid-cols-1 gap-3 md:grid-cols-4">
        <select v-model="selectedSecurityName" @change="checkSelectedPin" class="rounded-xl border border-slate-200 bg-white px-3 py-2 text-xs font-semibold">
          <option value="">Pilih nama security</option>
          <option v-for="employee in securityEmployees" :key="employee.nik" :value="employee.nama_lengkap">
            {{ employee.nama_lengkap }} — {{ employee.regu ? `${employee.jabatan || 'Anggota Security'} - ${employee.regu}` : (employee.jabatan || 'Anggota Security') }}
          </option>
        </select>
        <input v-model="newPin" type="password" inputmode="numeric" maxlength="6" placeholder="PIN baru 4-6 digit" class="rounded-xl border border-slate-200 px-3 py-2 text-xs">
        <input v-model="confirmPin" type="password" inputmode="numeric" maxlength="6" placeholder="Ulangi PIN baru" class="rounded-xl border border-slate-200 px-3 py-2 text-xs">
        <div class="flex gap-2">
          <button
            @click="resetSecurityPin"
            :disabled="isSavingPin || selectedPinLoading || !selectedSecurityName || selectedPinExists"
            class="flex-1 rounded-xl bg-emerald-700 px-4 py-2 text-xs font-bold text-white hover:bg-emerald-800 disabled:cursor-not-allowed disabled:opacity-40"
          >
            {{ selectedPinLoading ? 'Memeriksa...' : 'Buat PIN' }}
          </button>
          <button
            @click="openResetPinLogin"
            :disabled="isSavingPin || selectedPinLoading || !selectedSecurityName || !selectedPinExists"
            class="flex-1 rounded-xl bg-amber-600 px-4 py-2 text-xs font-bold text-white hover:bg-amber-700 disabled:cursor-not-allowed disabled:opacity-40"
          >
            Reset PIN
          </button>
        </div>
      </div>
      <form v-if="isResetLoginOpen" @submit.prevent="authorizeAndResetPin" class="mt-4 rounded-xl border border-amber-200 bg-amber-50 p-4">
        <p class="text-xs font-bold text-amber-900">Konfirmasi akun chief security</p>
        <p class="mt-1 text-[11px] text-amber-800">Masukkan kembali login untuk menghindari reset PIN dari aplikasi yang ditinggalkan terbuka.</p>
        <div class="mt-3 grid grid-cols-1 gap-3 md:grid-cols-3">
          <input v-model="resetUsername" autocomplete="username" placeholder="Username" class="rounded-xl border border-amber-300 bg-white px-3 py-2 text-xs">
          <input v-model="resetPassword" type="password" autocomplete="current-password" placeholder="Password" class="rounded-xl border border-amber-300 bg-white px-3 py-2 text-xs">
          <div class="flex gap-2">
            <button type="submit" :disabled="isAuthorizingReset" class="flex-1 rounded-xl bg-slate-900 px-4 py-2 text-xs font-bold text-white disabled:opacity-50">
              {{ isAuthorizingReset ? 'Memeriksa...' : 'Lanjutkan Reset' }}
            </button>
            <button type="button" @click="cancelResetPinLogin" class="rounded-xl border border-slate-300 bg-white px-3 py-2 text-xs font-semibold text-slate-700">Batal</button>
          </div>
        </div>
        <p v-if="resetLoginError" class="mt-2 text-xs font-semibold text-red-700">{{ resetLoginError }}</p>
      </form>
    </div>

    <!-- Dynamic facility metrics -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      <div 
        v-for="(stat, idx) in stats" 
        :key="idx" 
        class="group relative overflow-hidden rounded-2xl border border-slate-200/80 bg-white/90 p-5 shadow-sm transition-all duration-200 hover:-translate-y-1 hover:shadow-lg backdrop-blur-md hover:border-emerald-500/30"
      >
        <p class="text-[10px] font-bold text-slate-400 uppercase tracking-wider group-hover:text-emerald-600 transition-colors">{{ stat.label }}</p>
        <h3 class="text-3xl font-black text-slate-900 mt-2 tracking-tight">{{ stat.value }}</h3>
      </div>
    </div>

    <!-- Data Tables Grid & Inspector Preview -->
    <div class="grid grid-cols-1 xl:grid-cols-3 gap-6">
      
      <!-- Clean Inspection Summary Table (Left 2 Columns) -->
      <div class="xl:col-span-2 rounded-2xl border border-slate-200/80 bg-white/90 p-5 shadow-sm backdrop-blur-md">
        <div class="flex items-center justify-between mb-4">
          <div>
            <h3 class="text-sm font-bold text-slate-900">Log Inspeksi Lapangan</h3>
            <p class="text-[11px] text-slate-400">Klik baris untuk melihat detail foto & catatan</p>
          </div>
          <span class="text-[10px] font-bold uppercase tracking-wider text-emerald-600 bg-emerald-50 border border-emerald-100 px-2 py-1 rounded">Live SQLite</span>
        </div>

        <div class="overflow-hidden rounded-xl border border-slate-200">
          <div class="overflow-x-auto">
            <table class="w-full text-center border-collapse text-xs">
              <thead>
                <tr class="bg-slate-50 text-slate-500 text-[10px] uppercase font-bold border-b border-slate-200 tracking-wider">
                  <th class="py-3 px-3 text-left">Waktu</th>
                  <th class="py-3 px-2 text-left">Petugas</th>
                  <th class="py-3 px-2 text-left">Asset</th>
                  <th class="py-3 px-1 border-l">Selang</th>
                  <th class="py-3 px-1">Nozle</th>
                  <th class="py-3 px-1">Valve</th>
                  <th class="py-3 px-1">Lampu</th>
                  <th class="py-3 px-1">Bell</th>
                  <th class="py-3 px-1">Apar</th>
                  <th class="py-3 px-1 border-r">Kunci</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-slate-100 font-medium text-slate-700">
                <tr 
                  v-for="row in spreadsheetRows" 
                  :key="row.id" 
                  @click="selectedInspection = row"
                  class="hover:bg-emerald-50/50 transition cursor-pointer"
                  :class="{ 'bg-emerald-50/80 font-bold': selectedInspection?.id === row.id }"
                >
                  <td class="py-3 px-3 text-left text-slate-500 text-[11px] whitespace-nowrap">{{ formatDate(row.inspected_at) }}</td>
                  <td class="py-3 px-2 text-left text-slate-700 font-mono text-[11px]">{{ row.inspector_badge_id || 'SEC-01' }}</td>
                  <td class="py-3 px-2 text-left font-bold text-emerald-950">{{ row.asset_code }}</td>
                  
                  <td class="py-3 px-1 border-l" :class="getCellClass(row.selang)">{{ row.selang || '-' }}</td>
                  <td class="py-3 px-1" :class="getCellClass(row.nozle)">{{ row.nozle || '-' }}</td>
                  <td class="py-3 px-1" :class="getCellClass(row.valve)">{{ row.valve || '-' }}</td>
                  <td class="py-3 px-1" :class="getCellClass(row.lampu)">{{ row.lampu || '-' }}</td>
                  <td class="py-3 px-1" :class="getCellClass(row.bell)">{{ row.bell || '-' }}</td>
                  <td class="py-3 px-1" :class="getCellClass(row.apar)">{{ row.apar || '-' }}</td>
                  <td class="py-3 px-1 border-r" :class="getCellClass(row.kunci_pilar)">{{ row.kunci_pilar || '-' }}</td>
                </tr>
                <tr v-if="spreadsheetRows.length === 0">
                  <td colspan="10" class="py-12 text-center text-slate-400">
                    Tidak ada data inspeksi. Klik <strong class="text-emerald-600">Sync Google Sheets</strong>.
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
          
          <!-- Pagination Footer -->
          <div class="flex items-center justify-between px-4 py-3 bg-slate-50/80 border-t border-slate-200">
            <button 
              @click="prevPage"
              :disabled="currentPage === 1 || loading"
              class="px-3 py-1.5 text-xs font-bold text-slate-600 bg-white border border-slate-200 rounded-lg shadow-sm hover:bg-slate-50 disabled:opacity-50 disabled:cursor-not-allowed transition-all active:scale-95 cursor-pointer"
            >
              &larr; Prev
            </button>
            
            <span class="text-xs font-bold text-slate-500 tracking-wide">
              {{ currentPage }} / {{ totalPages }}
            </span>
            
            <button 
              @click="nextPage"
              :disabled="currentPage === totalPages || loading"
              class="px-3 py-1.5 text-xs font-bold text-slate-600 bg-white border border-slate-200 rounded-lg shadow-sm hover:bg-slate-50 disabled:opacity-50 disabled:cursor-not-allowed transition-all active:scale-95 cursor-pointer"
            >
              Next &rarr;
            </button>
          </div>
        </div>
      </div>

      <!-- Right Column: Photo Inspector & Detail Preview Card -->
      <div class="space-y-6">
        
        <!-- Inspection Detail Card -->
        <div class="rounded-2xl border border-slate-200/80 bg-white/90 p-5 shadow-sm backdrop-blur-md">
          <h3 class="text-sm font-bold text-slate-900 mb-3">Detail & Bukti Foto Lapangan</h3>
          
          <div v-if="selectedInspection" class="space-y-4 text-xs">
            <div class="bg-slate-50 p-3 rounded-xl border border-slate-100 space-y-1.5">
              <div class="flex justify-between">
                <span class="text-slate-400">Kode Asset:</span>
                <span class="font-bold text-emerald-950">{{ selectedInspection.asset_code }}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-slate-400">Petugas:</span>
                <span class="font-mono font-bold text-slate-700">{{ selectedInspection.inspector_badge_id || 'SEC-01' }}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-slate-400">Waktu:</span>
                <span class="text-slate-700">{{ formatDate(selectedInspection.inspected_at) }}</span>
              </div>
            </div>

            <!-- Photo Preview Box -->
            <div>
              <label class="block text-[11px] font-bold text-slate-500 mb-1.5 uppercase tracking-wider">Bukti Foto Lapangan</label>
              
              <!-- Jika photo_url berisi link Google Drive / http -->
              <div v-if="selectedInspection.photo_url && selectedInspection.photo_url.includes('http')" class="p-4 rounded-xl border border-emerald-200 bg-emerald-50/50 flex flex-col items-center justify-center gap-3 text-center">
                <span class="text-2xl">📸</span>
                <p class="text-xs font-semibold text-emerald-900">Foto dokumentasi tersedia di Google Drive</p>
                <a 
                  :href="selectedInspection.photo_url" 
                  target="_blank" 
                  class="px-4 py-2 bg-emerald-700 hover:bg-emerald-800 text-white font-bold rounded-xl shadow-sm text-xs transition flex items-center gap-2 cursor-pointer"
                >
                  <span>🔍 Buka Foto Ukuran Penuh</span>
                </a>
              </div>

              <!-- Jika kosong -->
              <div v-else class="h-28 rounded-xl border border-dashed border-slate-200 bg-slate-50 flex items-center justify-center text-slate-400 italic text-xs">
                Tidak ada foto diunggah
              </div>
            </div>

            <!-- Notes Box -->
            <div>
              <label class="block text-[11px] font-bold text-slate-500 mb-1 uppercase tracking-wider">Catatan Petugas</label>
              <div class="p-3 bg-slate-50 rounded-xl border border-slate-100 text-slate-700 italic min-h-[50px]">
                "{{ selectedInspection.notes || 'Tidak ada catatan khusus.' }}"
              </div>
            </div>
          </div>

          <div v-else class="py-12 text-center text-slate-400 text-xs">
            Pilih baris pada tabel untuk melihat detail.
          </div>
        </div>

        <!-- Reported Defect Summary -->
        <div class="rounded-2xl border border-slate-200/80 bg-white/90 p-5 shadow-sm backdrop-blur-md">
          <h3 class="text-sm font-bold text-slate-900 mb-3">Temuan Kerusakan Aktif</h3>
          <div class="divide-y divide-slate-100 max-h-48 overflow-y-auto">
            <div v-for="(prob, i) in problems" :key="i" class="py-2.5 flex items-start justify-between gap-2">
              <div>
                <h4 class="text-xs font-bold text-slate-900 leading-tight">{{ prob.title }}</h4>
                <p class="text-[11px] text-slate-500 mt-0.5">{{ prob.location }}</p>
              </div>
              <span class="shrink-0 px-2 py-0.5 rounded text-[9px] font-bold uppercase text-red-700 border border-red-200 bg-red-50 shadow-sm">
                {{ prob.severity }}
              </span>
            </div>
            <div v-if="problems.length === 0" class="py-6 text-center text-slate-400 text-xs">
              ✨ Tidak ada laporan kerusakan aktif.
            </div>
          </div>
        </div>

      </div>

    </div>
  </div>
</template>
