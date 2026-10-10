<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { Employee } from '../../types';

const props = defineProps<{
  employee: Employee;
  status: string;
  keterangan: string;
  activeDateStr: string;
}>();

const emit = defineEmits<{
  (e: 'change', payload: { status: string; keterangan: string }): void;
}>();

const localKeterangan = ref(props.keterangan);

watch(() => props.keterangan, (newVal) => {
  localKeterangan.value = newVal;
});

const statusOptions = [
  { value: 'ON', label: 'ON - Hadir' },
  { value: 'OFF', label: 'OFF - Libur' },
  { value: 'SAKIT', label: 'SAKIT (Tanpa SD)' },
  { value: 'SAKIT (SD)', label: 'SAKIT (SD - Surat Dokter)' },
  { value: 'IZIN', label: 'IZIN' },
  { value: 'CUTI', label: 'CUTI' },
  { value: 'ALPA', label: 'ALPA' },
];

const needsComment = computed(() => {
  return ['OFF', 'SAKIT', 'SAKIT (SD)', 'IZIN', 'CUTI'].includes(props.status);
});

const commentPlaceholder = computed(() => {
  switch (props.status) {
    case 'OFF':
      return 'Contoh: Tukar libur dengan [Nama Karyawan]';
    case 'SAKIT (SD)':
      return 'Contoh: Surat Dokter Klinik Kimia Farma';
    case 'SAKIT':
      return 'Contoh: Demam, tanpa surat dokter';
    case 'IZIN':
    case 'CUTI':
      return 'Contoh: Keperluan keluarga';
    default:
      return 'Keterangan tambahan (opsional)...';
  }
});

function handleStatusChange(event: Event) {
  const select = event.target as HTMLSelectElement;
  emit('change', { status: select.value, keterangan: localKeterangan.value });
}

function handleCommentBlur() {
  emit('change', { status: props.status, keterangan: localKeterangan.value });
}

function getDropdownColorClass(status: string) {
  switch (status) {
    case 'ON':
      return 'bg-emerald-50 text-emerald-800 border-emerald-300 font-bold';
    case 'OFF':
      return 'bg-slate-100 text-slate-800 border-slate-300 font-bold';
    case 'SAKIT (SD)':
    case 'SAKIT':
      return 'bg-amber-50 text-amber-800 border-amber-300 font-bold';
    case 'IZIN':
      return 'bg-sky-50 text-sky-800 border-sky-300 font-bold';
    case 'CUTI':
      return 'bg-purple-50 text-purple-800 border-purple-300 font-bold';
    case 'ALPA':
      return 'bg-rose-50 text-rose-800 border-rose-300 font-bold';
    default:
      return 'bg-slate-50 text-slate-400 border-slate-200';
  }
}
</script>

<template>
  <div class="bg-white p-4 rounded-2xl border border-slate-200/80 shadow-xs space-y-3 hover:border-slate-300 transition-all">
    <div class="flex items-center justify-between gap-4">
      <div>
        <h4 class="font-bold text-slate-900 text-xs sm:text-sm">{{ employee.nama_lengkap }}</h4>
        <p class="text-[11px] text-slate-500 font-medium mt-0.5">{{ employee.jabatan || 'Staff' }}</p>
      </div>

      <div class="relative min-w-[160px]">
        <select
          :value="status"
          @change="handleStatusChange"
          class="w-full text-xs px-3 py-2 rounded-xl border transition-all cursor-pointer focus:outline-none focus:ring-2 focus:ring-emerald-500/40"
          :class="getDropdownColorClass(status)"
        >
          <option value="" class="text-slate-400 bg-white">-- Belum Diisi --</option>
          <option v-for="opt in statusOptions" :key="opt.value" :value="opt.value" class="text-slate-800 bg-white font-semibold">
            {{ opt.label }}
          </option>
        </select>
      </div>
    </div>

    <!-- Conditional Comment Input Field -->
    <div v-if="needsComment || localKeterangan" class="pt-2 border-t border-slate-100">
      <div class="flex items-center gap-2">
        <span class="text-[10px] font-semibold uppercase text-slate-400 shrink-0">Ket:</span>
        <input
          v-model="localKeterangan"
          @blur="handleCommentBlur"
          @keyup.enter="handleCommentBlur"
          type="text"
          :placeholder="commentPlaceholder"
          class="w-full text-xs px-2.5 py-1.5 rounded-lg border border-slate-200 bg-slate-50/50 text-slate-700 placeholder:text-slate-400 focus:bg-white focus:border-emerald-500 focus:outline-none focus:ring-1 focus:ring-emerald-500"
        />
      </div>
    </div>
  </div>
</template>