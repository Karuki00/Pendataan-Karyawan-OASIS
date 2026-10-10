<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import AttendanceCell from './AttendanceCell.vue';
import type { Employee } from '../../types';

const props = defineProps<{
  periode: any;
  employees: Employee[];
  attendanceMap: Record<string, string>;
}>();

const emit = defineEmits<{
  (e: 'refresh'): void;
}>();

function buildDateRange(startStr: string, endStr: string) {
  const list: Date[] = [];
  if (!startStr || !endStr) return list;
  let curr = new Date(startStr);
  const end = new Date(endStr);
  while (curr <= end) {
    list.push(new Date(curr));
    curr.setDate(curr.getDate() + 1);
  }
  return list;
}

const formatDateKey = (d: Date) => d.toISOString().split('T')[0];

async function removeOrDeactivate(emp: Employee) {
  const yes = confirm(`Sembunyikan/Nonaktifkan ${emp.nama_lengkap} dari rekap absensi?`);
  if (!yes) return;

  try {
    await invoke('toggle_employee_active_status', {
      nik: emp.nik,
      newStatus: 'NONAKTIF',
    });
    emit('refresh');
  } catch (err) {
    alert(`Gagal mengubah status: ${err}`);
  }
}
</script>

<template>
  <div v-if="periode" class="overflow-x-auto border border-slate-200/80 rounded-2xl shadow-xs bg-white">
    <table class="min-w-full divide-y divide-slate-200 text-xs">
      <thead class="bg-slate-50/90 sticky top-0 z-20">
        <tr>
          <!-- Column 1: Employee Name & Jabatan Subtitle -->
          <th class="px-3.5 py-2.5 text-left font-bold text-slate-700 sticky left-0 bg-slate-50 z-30 border-r border-slate-200 min-w-[220px]">
            NAMA / JABATAN
          </th>

          <!-- Daily Columns 1-31 -->
          <th 
            v-for="d in buildDateRange(periode.tanggal_mulai, periode.tanggal_selesai)" 
            :key="d.toISOString()"
            class="px-1 py-1.5 text-center font-semibold text-slate-600 min-w-[48px] border-r border-slate-100"
          >
            <div>{{ d.getDate() }}</div>
            <div class="text-[9px] text-slate-400 uppercase font-mono">
              {{ d.toLocaleDateString('id-ID', { weekday: 'narrow' }) }}
            </div>
          </th>

          <!-- Action Column -->
          <th class="px-3 py-2 text-center font-bold text-slate-700 sticky right-0 bg-slate-50 z-30 border-l border-slate-200 min-w-[80px]">
            AKSI
          </th>
        </tr>
      </thead>
      <tbody class="divide-y divide-slate-100">
        <tr v-for="emp in employees" :key="emp.nik" class="hover:bg-slate-50/80 transition-colors">
          <!-- Name Top, Jabatan Bottom -->
          <td class="px-3.5 py-2 sticky left-0 bg-white z-10 border-r border-slate-200 shadow-[2px_0_5px_-2px_rgba(0,0,0,0.04)]">
            <div class="font-bold text-slate-900 truncate max-w-[200px] text-xs">
              {{ emp.nama_lengkap }}
            </div>
            <div class="text-[10px] text-slate-500 font-medium truncate max-w-[200px] mt-0.5">
              {{ emp.jabatan || 'Staff' }}
            </div>
          </td>

          <!-- Daily Status Cell -->
          <td 
            v-for="d in buildDateRange(periode.tanggal_mulai, periode.tanggal_selesai)" 
            :key="d.toISOString()" 
            class="px-0.5 py-1 text-center border-r border-slate-50"
          >
            <AttendanceCell
              :nik="emp.nik"
              :periode-id="periode.id"
              :tanggal="formatDateKey(d)"
              :current-status="attendanceMap[`${emp.nik}_${formatDateKey(d)}`] || 'ON'"
              @updated="emit('refresh')"
            />
          </td>

          <!-- Actions -->
          <td class="px-2 py-1 text-center sticky right-0 bg-white z-10 border-l border-slate-200">
            <button
              type="button"
              @click="removeOrDeactivate(emp)"
              class="text-[10px] font-bold text-rose-600 hover:text-rose-800 bg-rose-50 hover:bg-rose-100 border border-rose-200 px-2 py-1 rounded-lg transition-all cursor-pointer"
              title="Set Nonaktif"
            >
              Hapus
            </button>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>