<script setup>
import { invoke } from '@tauri-apps/api/core';

const props = defineProps({
  nik: String,
  periodeId: Number,
  tanggal: String,
  currentStatus: { type: String, default: 'ON' }
});

const emit = defineEmits(['updated']);

const statusOptions = [
  { label: 'ON', color: 'bg-emerald-100 text-emerald-800 border-emerald-300' },
  { label: 'OFF', color: 'bg-slate-100 text-slate-600 border-slate-300' },
  { label: 'SAKIT (SD)', color: 'bg-amber-100 text-amber-800 border-amber-300' },
  { label: 'IZIN', color: 'bg-sky-100 text-sky-800 border-sky-300' },
  { label: 'ALPA', color: 'bg-rose-100 text-rose-800 border-rose-300' },
  { label: 'CUTI', color: 'bg-purple-100 text-purple-800 border-purple-300' },
];

async function onChange(event) {
  const newStatus = event.target.value;
  try {
    await invoke('update_absensi_cell', {
      nik: props.nik,
      periodeId: props.periodeId,
      tanggal: props.tanggal,
      status: newStatus
    });
    emit('updated');
  } catch (err) {
    console.error("Gagal update absensi:", err);
  }
}
</script>

<template>
  <select
    :value="currentStatus"
    @change="onChange"
    class="text-[11px] font-bold rounded px-1 py-0.5 border text-center cursor-pointer focus:outline-none focus:ring-1 focus:ring-indigo-500"
    :class="statusOptions.find(o => o.label === currentStatus)?.color || 'bg-gray-50 border-gray-200'"
  >
    <option v-for="opt in statusOptions" :key="opt.label" :value="opt.label">
      {{ opt.label }}
    </option>
  </select>
</template>