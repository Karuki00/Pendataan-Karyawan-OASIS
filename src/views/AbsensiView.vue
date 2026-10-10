<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import AttendanceHeader from '../components/absensi/AttendanceHeader.vue';
import DailyAttendanceView from './DailyAttendanceView.vue';
import { useAbsensiStore } from '../composables/useAbsensiStore';
import type { Employee } from '../types';

const { 
  selectedYear, 
  selectedMonth, 
  attendanceMap, 
  keteranganMap, 
  setCellStatus, 
  setCellKeterangan, 
  clearAllState 
} = useAbsensiStore();

const currentPeriode = ref<any>(null);
const employees = ref<Employee[]>([]);
const isLoading = ref(false);

interface AbsensiCellData {
  status: string;
  keterangan: string;
}

async function loadData() {
  isLoading.value = true;
  try {
    const periode = await invoke('get_or_create_periode', {
      year: selectedYear.value,
      month: selectedMonth.value,
    });
    currentPeriode.value = periode;

    const empList = await invoke<Employee[]>('list_staff_employees_for_absensi');
    employees.value = empList;

    // Fetch entire month's DB status & keterangan to sync with localStorage
    const daysInMonth = new Date(selectedYear.value, selectedMonth.value, 0).getDate();
    const promises: Promise<void>[] = [];

    for (const emp of empList) {
      for (let day = 1; day <= daysInMonth; day++) {
        const m = String(selectedMonth.value).padStart(2, '0');
        const d = String(day).padStart(2, '0');
        const dateStr = `${selectedYear.value}-${m}-${d}`;
        const key = `${emp.nik}_${dateStr}`;

        // Fetch from DB if status or keterangan is missing in local reactive maps
        if (!(key in attendanceMap.value) || !(key in keteranganMap.value)) {
          promises.push(
            invoke<AbsensiCellData>('get_absensi_cell', {
              nik: emp.nik,
              tanggal: dateStr,
            }).then((cell) => {
              if (cell) {
                if (cell.status) setCellStatus(emp.nik, dateStr, cell.status);
                if (cell.keterangan) setCellKeterangan(emp.nik, dateStr, cell.keterangan);
              }
            }).catch(() => {})
          );
        }
      }
    }

    await Promise.all(promises);
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

watch([selectedYear, selectedMonth], () => {
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
      @imported="handleResetComplete"
      @change-month="(m) => { selectedMonth = m; }"
      @change-year="(y) => { selectedYear = y; }"
    />

    <div v-if="isLoading" class="p-8 text-center text-slate-500 bg-white rounded-2xl border shadow-xs">
      Memuat data absensi staff...
    </div>

    <DailyAttendanceView
      v-else
      :selected-year="selectedYear"
      :selected-month="selectedMonth"
      :employees="employees"
      :attendance-map="attendanceMap"
      :keterangan-map="keteranganMap"
      @update-cell="handleCellUpdate"
    />
  </div>
</template>