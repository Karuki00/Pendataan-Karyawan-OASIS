<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import type { Employee } from '../../types';
import PreviewSheetAbsen from './PreviewSheetAbsen.vue';
import PreviewSheetCuti from './PreviewSheetCuti.vue';

const props = defineProps<{
  isOpen: boolean;
  selectedYear: number;
  selectedMonth: number;
  employees: Employee[];
  attendanceMap: Record<string, string>;
  keteranganMap?: Record<string, string>;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
}>();

const isExporting = ref(false);
const activeSheetTab = ref<'absen' | 'cuti'>('absen');

const monthNames = [
  'Januari', 'Februari', 'Maret', 'April', 'Mei', 'Juni', 
  'Juli', 'Agustus', 'September', 'Oktober', 'November', 'Desember'
];

async function triggerExcelExport() {
  try {
    const monthName = monthNames[props.selectedMonth - 1];
    const defaultFileName = `Rekap_Absen_Pengelola_${monthName}_${props.selectedYear}.xlsx`;

    const filePath = await save({
      defaultPath: defaultFileName,
      filters: [{ name: 'Excel Workbook', extensions: ['xlsx'] }]
    });

    if (!filePath) return;

    isExporting.value = true;

    await invoke('export_rekap_absen_excel_1to1', {
      filePath,
      logoPath: null,
      year: props.selectedYear,
      month: props.selectedMonth,
    });

    alert('File Excel Rekap Absensi & Rekap Cuti berhasil diekspor!');
    emit('close');
  } catch (err) {
    console.error('Export Error:', err);
    alert(`Gagal mengekspor file: ${err}`);
  } finally {
    isExporting.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <div v-if="isOpen" class="fixed inset-0 z-[9999] flex items-center justify-center bg-slate-950/70 p-4 backdrop-blur-xs">
      <div class="flex flex-col max-h-[92vh] w-full max-w-7xl rounded-3xl bg-white shadow-2xl border border-slate-200 overflow-hidden">
        
        <!-- Modal Top Bar -->
        <div class="flex flex-wrap items-center justify-between border-b bg-slate-50 px-6 py-4 gap-4">
          <div>
            <h3 class="text-base font-bold text-slate-900">Preview Export Workbook (.xlsx)</h3>
            <p class="text-xs text-slate-500">
              Periode: {{ monthNames[selectedMonth - 1] }} {{ selectedYear }}
            </p>
          </div>

          <!-- Sheet Switcher Tabs -->
          <div class="flex items-center gap-1.5 p-1 bg-slate-200/70 rounded-2xl">
            <button
              type="button"
              @click="activeSheetTab = 'absen'"
              class="px-4 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer flex items-center gap-1.5"
              :class="activeSheetTab === 'absen' ? 'bg-white text-emerald-800 shadow-xs' : 'text-slate-600 hover:text-slate-900'"
            >
              <span>📋</span>
              <span>Sheet 1: Rekap Absen</span>
            </button>
            <button
              type="button"
              @click="activeSheetTab = 'cuti'"
              class="px-4 py-1.5 rounded-xl text-xs font-bold transition-all cursor-pointer flex items-center gap-1.5"
              :class="activeSheetTab === 'cuti' ? 'bg-white text-purple-800 shadow-xs' : 'text-slate-600 hover:text-slate-900'"
            >
              <span>🌴</span>
              <span>Sheet 2: Rekap Cuti</span>
            </button>
          </div>

          <!-- Action Buttons -->
          <div class="flex items-center gap-3">
            <button
              type="button"
              @click="triggerExcelExport"
              :disabled="isExporting"
              class="bg-emerald-700 hover:bg-emerald-800 text-white text-xs font-bold px-4 py-2.5 rounded-xl flex items-center gap-2 transition-all shadow-xs cursor-pointer"
            >
              <span>📥 {{ isExporting ? 'Mengekspor...' : 'Download Excel 2 Sheet' }}</span>
            </button>
            <button @click="emit('close')" class="text-slate-400 hover:text-slate-700 text-lg p-1 cursor-pointer">&times;</button>
          </div>
        </div>

        <!-- Sheet Preview Content -->
        <div class="flex-1 overflow-auto p-6 bg-slate-100/60">
          <PreviewSheetAbsen
            v-if="activeSheetTab === 'absen'"
            :selected-year="selectedYear"
            :selected-month="selectedMonth"
            :employees="employees"
            :attendance-map="attendanceMap"
            :keterangan-map="keteranganMap"
          />

          <PreviewSheetCuti
            v-else-if="activeSheetTab === 'cuti'"
            :selected-year="selectedYear"
          />
        </div>
      </div>
    </div>
  </Teleport>
</template>