<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Employee } from '../types';
import DailyAttendanceCard from '../components/absensi/DailyAttendanceCard.vue';

export interface DailyAttendanceProps {
  selectedYear: number;
  selectedMonth: number;
  employees: Employee[];
  attendanceMap: Record<string, string>;
  keteranganMap?: Record<string, string>;
}

const props = withDefaults(defineProps<DailyAttendanceProps>(), {
  keteranganMap: () => ({}),
});

const emit = defineEmits<{
  (e: 'updateCell', payload: { nik: string; dateStr: string; status: string; keterangan: string }): void;
}>();

const STORAGE_KEY_DAY = 'oasis_absensi_day';
const savedDay = Number(localStorage.getItem(STORAGE_KEY_DAY)) || 1;
const selectedDay = ref(savedDay);

watch(selectedDay, (newDay) => {
  localStorage.setItem(STORAGE_KEY_DAY, String(newDay));
});

const totalDaysInMonth = computed(() => {
  return new Date(props.selectedYear, props.selectedMonth, 0).getDate();
});

const activeDateStr = computed(() => {
  const m = String(props.selectedMonth).padStart(2, '0');
  const d = String(selectedDay.value).padStart(2, '0');
  return `${props.selectedYear}-${m}-${d}`;
});

function getStatusForEmployee(nik: string): string {
  return props.attendanceMap[`${nik}_${activeDateStr.value}`] || '';
}

function getKeteranganForEmployee(nik: string): string {
  return props.keteranganMap[`${nik}_${activeDateStr.value}`] || '';
}

async function handleCardChange(nik: string, payload: { status: string; keterangan: string }) {
  const dateStr = activeDateStr.value;

  // 1. Instantly update Vue Store & LocalStorage
  emit('updateCell', { nik, dateStr, status: payload.status, keterangan: payload.keterangan });

  // 2. Save to SQLite
  try {
    const periode = await invoke<any>('get_or_create_periode', {
      year: props.selectedYear,
      month: props.selectedMonth,
    });

    await invoke('update_absensi_cell', {
      nik,
      periodeId: periode.id,
      tanggal: dateStr,
      status: payload.status,
      keterangan: payload.keterangan,
    });
  } catch (err) {
    console.error('Gagal menyimpan ke SQLite:', err);
  }
}
</script>

<template>
  <div class="space-y-4">
    <!-- Day Selector Ribbon -->
    <div class="bg-white p-4 rounded-2xl border border-slate-200/80 shadow-xs flex flex-wrap items-center justify-between gap-4">
      <div class="flex items-center gap-3 overflow-hidden">
        <span class="text-xs font-bold text-slate-500 shrink-0 uppercase tracking-wider">Pilih Tanggal:</span>
        <div class="flex items-center gap-1.5 overflow-x-auto py-1 max-w-2xl scrollbar-thin">
          <button
            v-for="day in totalDaysInMonth"
            :key="day"
            type="button"
            @click="selectedDay = day"
            class="h-9 w-9 shrink-0 rounded-xl text-xs font-bold transition-all cursor-pointer flex items-center justify-center"
            :class="selectedDay === day 
              ? 'bg-emerald-700 text-white shadow-md scale-105 ring-2 ring-emerald-600/30' 
              : 'bg-slate-100 text-slate-700 hover:bg-slate-200'"
          >
            {{ day }}
          </button>
        </div>
      </div>

      <div class="text-right pl-3 border-l border-slate-200">
        <span class="text-[10px] font-medium text-slate-400 block uppercase">Tanggal Aktif Rekap</span>
        <span class="text-xs font-bold text-slate-800 font-mono">{{ activeDateStr }}</span>
      </div>
    </div>

    <!-- Staff Cards List -->
    <div v-if="employees.length === 0" class="p-8 text-center text-slate-400 bg-white rounded-2xl border shadow-xs">
      Tidak ada karyawan aktif divisi STAFF / Pengelola.
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
      <DailyAttendanceCard
        v-for="emp in employees"
        :key="emp.nik"
        :employee="emp"
        :status="getStatusForEmployee(emp.nik)"
        :keterangan="getKeteranganForEmployee(emp.nik)"
        :activeDateStr="activeDateStr"
        @change="handleCardChange(emp.nik, $event)"
      />
    </div>
  </div>
</template>