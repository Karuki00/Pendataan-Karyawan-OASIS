<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { resolveMaster, forgetMaster } from '../services/masterDiscovery';
import AttendanceHeader from '../components/absensi/AttendanceHeader.vue';
import DailyAttendanceView from './DailyAttendanceView.vue';
import { useAbsensiStore } from '../composables/useAbsensiStore';
import type { Employee } from '../types';
import { fetchSecurityAttendanceFromAppsScript, fetchSecurityRosterFromAppsScript } from '../services/securityUserSync';

const props = withDefaults(defineProps<{
  division?: string; // 'hr', 'security', 'housekeeping', 'engineering'
}>(), {
  division: 'hr',
});

const { 
  selectedYear, 
  selectedMonth, 
  attendanceMap, 
  keteranganMap, 
  setCellStatus, 
  clearAllState 
} = useAbsensiStore();

const currentPeriode = ref<any>(null);
const employees = ref<Employee[]>([]);
const isLoading = ref(false);

const hrFilterDivision = ref<string>('staff');

// --- Pagination States ---
const currentPage = ref(1);
const itemsPerPage = 10;

const totalPages = computed(() => {
  return Math.max(1, Math.ceil(employees.value.length / itemsPerPage));
});

const paginatedEmployees = computed(() => {
  const start = (currentPage.value - 1) * itemsPerPage;
  return employees.value.slice(start, start + itemsPerPage);
});

interface AbsensiCellData {
  status: string;
  keterangan: string;
}

interface NetworkAttendanceCell extends AbsensiCellData {
  nik: string;
  tanggal: string;
}

async function loadData(forceDiscovery = false) {
  isLoading.value = true;
  clearAllState();
  try {
    // 1. Tentukan target divisi
    let targetDivisionKeyword = 'staff';
    const activeDiv = props.division.toLowerCase();

    if (activeDiv === 'hr') {
      targetDivisionKeyword = hrFilterDivision.value;
    } else {
      if (activeDiv.includes('security')) targetDivisionKeyword = 'security';
      else if (activeDiv.includes('housekeeping') || activeDiv.includes('hk')) targetDivisionKeyword = 'housekeeping';
      else if (activeDiv.includes('engineering') || activeDiv.includes('teknisi')) targetDivisionKeyword = 'engineering';
    }

    let empList: Employee[] = [];
    let isDataLoadedFromCloud = false;

    // 2. PRIORITAS UTAMA: Spreadsheet (Hanya untuk Security)
    if (targetDivisionKeyword === 'security') {
      try {
        const cloudRekap = await fetchSecurityAttendanceFromAppsScript(selectedYear.value, selectedMonth.value);
        empList = cloudRekap.employees;
        for (const cell of cloudRekap.cells) {
          setCellStatus(cell.nik, cell.tanggal, cell.status, cell.keterangan);
        }
        isDataLoadedFromCloud = empList.length > 0;
      } catch (cloudError) {
        console.warn('Gagal memuat dari Spreadsheet, jatuh kembali ke mode LAN/Lokal:', cloudError);
      }
    }

    // 3. Cari master di jaringan, lalu ambil / buat periode
    //    Dilewati kalau data sudah dari Spreadsheet, supaya Security tidak menunggu scan jaringan.
    let masterUrl: string | null = null;
    let isLanAvailable = false;
    let periode: any;

    if (!isDataLoadedFromCloud) {
      masterUrl = await resolveMaster(forceDiscovery);
      if (masterUrl) {
        try {
          const res = await tauriFetch(
            `${masterUrl}/api/attendance/periode?year=${selectedYear.value}&month=${selectedMonth.value}`,
            { connectTimeout: 3000 },
          );
          if (res.ok) {
            periode = await res.json();
            isLanAvailable = true;
          }
        } catch {
          forgetMaster();
        }
      }
    }

    if (!periode) {
      periode = await invoke('get_or_create_periode', { year: selectedYear.value, month: selectedMonth.value });
    }
    currentPeriode.value = periode;

    // 4. LAN & OFFLINE FALLBACK
    if (!isDataLoadedFromCloud) {

      // --- A. Fetch Karyawan (Roster) ---
      if (isLanAvailable && masterUrl) {
        try {
          const response = await tauriFetch(
            `${masterUrl}/api/employees/division?divisi=${encodeURIComponent(targetDivisionKeyword)}`,
            { connectTimeout: 3000 },
          );
          if (response.ok) {
            empList = await response.json();
          } else {
            isLanAvailable = false;
          }
        } catch {
          isLanAvailable = false;
          forgetMaster();
        }
      }

      if (!isLanAvailable) {
        let commandName = 'list_staff_employees_for_absensi';
        if (targetDivisionKeyword === 'security') commandName = 'list_security_employees_for_absensi';
        else if (targetDivisionKeyword === 'housekeeping') commandName = 'list_housekeeping_employees_for_absensi';
        else if (targetDivisionKeyword === 'engineering') commandName = 'list_engineering_employees_for_absensi';

        try {
          empList = await invoke<Employee[]>(commandName);
          if (targetDivisionKeyword === 'security' && empList.length === 0) {
            empList = await fetchSecurityRosterFromAppsScript();
          }
        } catch (localError) {
          console.error('Gagal fetch employee dari offline DB:', localError);
        }
      }

      // --- B. Fetch Sel Absensi ---
      if (isLanAvailable && masterUrl) {
        try {
          const response = await tauriFetch(
            `${masterUrl}/api/attendance/cells?periode_id=${currentPeriode.value.id}`,
            { connectTimeout: 3000 },
          );
          if (response.ok) {
            const cells = await response.json() as NetworkAttendanceCell[];
            for (const cell of cells) {
              setCellStatus(cell.nik, cell.tanggal, cell.status, cell.keterangan);
            }
          }
        } catch (e) {
          console.error('Gagal fetch cells via LAN', e);
          forgetMaster();
        }
      } else {
        // Mode Offline: baca tiap sel dari SQLite
        const daysInMonth = new Date(selectedYear.value, selectedMonth.value, 0).getDate();
        const promises: Promise<void>[] = [];
        for (const emp of empList) {
          for (let day = 1; day <= daysInMonth; day++) {
            const m = String(selectedMonth.value).padStart(2, '0');
            const d = String(day).padStart(2, '0');
            const dateStr = `${selectedYear.value}-${m}-${d}`;
            promises.push(
              invoke<AbsensiCellData | null>('get_absensi_cell', { nik: emp.nik, tanggal: dateStr })
                .then((cell) => { if (cell) setCellStatus(emp.nik, dateStr, cell.status, cell.keterangan); })
                .catch(() => {}),
            );
          }
        }
        await Promise.all(promises);
      }
    }

    employees.value = empList;
    if (currentPage.value > Math.ceil(empList.length / itemsPerPage)) {
      currentPage.value = 1;
    }
  } catch (err) {
    console.error('Gagal memuat data absensi:', err);
  } finally {
    isLoading.value = false;
  }
}

function handleCellUpdate(payload: { nik: string; dateStr: string; status: string; keterangan?: string }) {
  setCellStatus(payload.nik, payload.dateStr, payload.status, payload.keterangan);
}

function handleResetComplete() {
  clearAllState();
  void loadData();
}

function prevPage() {
  if (currentPage.value > 1) currentPage.value--;
}

function nextPage() {
  if (currentPage.value < totalPages.value) currentPage.value++;
}

watch([selectedYear, selectedMonth, () => props.division, hrFilterDivision], () => {
  currentPage.value = 1;
  void loadData();
});

onMounted(() => {
  void loadData();
});
</script>

<template>
  <div class="space-y-4">
    <AttendanceHeader
      :selected-year="selectedYear"
      :selected-month="selectedMonth"
      :employees="employees"
      :attendance-map="attendanceMap"
      :keterangan-map="keteranganMap"
      :division="division"
      :hr-filter-division="hrFilterDivision"
      @update:hr-filter-division="(val) => hrFilterDivision = val"
      @imported="handleResetComplete"
      @change-month="(m) => { selectedMonth = m; }"
      @change-year="(y) => { selectedYear = y; }"
      @refresh="loadData(true)"
    />

    <div v-if="isLoading" class="p-8 text-center text-slate-500 bg-white rounded-2xl border shadow-xs">
      Memuat data absensi...
    </div>

    <template v-else>
      <DailyAttendanceView
        :selected-year="selectedYear"
        :selected-month="selectedMonth"
        :employees="paginatedEmployees"
        :attendance-map="attendanceMap"
        :keterangan-map="keteranganMap"
        @update-cell="handleCellUpdate"
      />

      <!-- Pagination Footer Bar -->
      <div v-if="employees.length > 0" class="flex items-center justify-between px-5 py-3 bg-white rounded-2xl border border-slate-200/80 shadow-xs">
        <span class="text-xs font-semibold text-slate-500">
          Menampilkan <span class="font-bold text-slate-800">{{ paginatedEmployees.length ? (currentPage - 1) * itemsPerPage + 1 : 0 }}</span>
          - <span class="font-bold text-slate-800">{{ Math.min(currentPage * itemsPerPage, employees.length) }}</span> dari <span class="font-bold text-slate-800">{{ employees.length }}</span> total karyawan
        </span>

        <div class="flex items-center gap-2">
          <button 
            @click="prevPage" 
            :disabled="currentPage === 1"
            class="px-3 py-1.5 text-xs font-bold text-slate-700 bg-slate-100 hover:bg-slate-200 rounded-xl transition disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
          >
            &larr; Prev
          </button>
          
          <span class="text-xs font-bold text-slate-600 px-2 font-mono">
            {{ currentPage }} / {{ totalPages }}
          </span>

          <button 
            @click="nextPage" 
            :disabled="currentPage === totalPages"
            class="px-3 py-1.5 text-xs font-bold text-slate-700 bg-slate-100 hover:bg-slate-200 rounded-xl transition disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer"
          >
            Next &rarr;
          </button>
        </div>
      </div>
    </template>
  </div>
</template>