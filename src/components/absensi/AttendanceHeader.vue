<script setup lang="ts">
import { ref, computed } from 'vue';
import ExportPreviewModal from './ExportPreviewModal.vue';
import type { Employee } from '../../types';

const props = defineProps<{
  selectedYear: number;
  selectedMonth: number;
  employees: Employee[];
  attendanceMap: Record<string, string>;
}>();

const emit = defineEmits<{
  (e: 'imported'): void;
  (e: 'changeMonth', month: number): void;
  (e: 'changeYear', year: number): void;
}>();

const isExportPreviewOpen = ref(false);

// Month input format (YYYY-MM)
const monthInputValue = computed({
  get() {
    const m = String(props.selectedMonth).padStart(2, '0');
    return `${props.selectedYear}-${m}`;
  },
  set(val: string) {
    if (!val) return;
    const [y, m] = val.split('-').map(Number);
    emit('changeYear', y);
    emit('changeMonth', m);
  }
});
</script>

<template>
  <div class="flex flex-wrap items-center justify-between gap-4 bg-white p-4.5 rounded-2xl border border-gray-200/80 shadow-xs">
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-emerald-50 text-emerald-700 font-bold text-lg border border-emerald-100">
        📅
      </div>
      <div>
        <h2 class="text-base font-bold text-slate-800">Rekapitulasi Absensi Staff</h2>
        <p class="text-xs text-slate-500">Otomatis terhubung dengan Master Data Karyawan</p>
      </div>
    </div>

    <div class="flex items-center gap-3">
      <!-- Future-proof Month/Year Datepicker -->
      <div class="flex items-center gap-2 bg-slate-50 px-3 py-1.5 rounded-xl border border-slate-200">
        <span class="text-xs font-bold text-slate-500">Periode:</span>
        <input
          type="month"
          v-model="monthInputValue"
          class="bg-transparent text-xs font-bold text-slate-800 border-0 focus:outline-none cursor-pointer"
        />
      </div>

      <!-- Export Preview Button -->
      <button
        type="button"
        @click="isExportPreviewOpen = true"
        class="bg-emerald-700 hover:bg-emerald-800 text-white text-xs font-bold px-3.5 py-2 rounded-xl flex items-center gap-2 transition-all shadow-xs cursor-pointer"
      >
        <span>📊 Preview &amp; Export Excel</span>
      </button>
    </div>

    <ExportPreviewModal
      :is-open="isExportPreviewOpen"
      :selected-year="selectedYear"
      :selected-month="selectedMonth"
      :employees="employees"
      :attendance-map="attendanceMap"
      @close="isExportPreviewOpen = false"
    />
  </div>
</template>