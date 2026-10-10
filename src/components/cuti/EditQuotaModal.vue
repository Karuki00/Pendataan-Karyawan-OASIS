<script setup lang="ts">
import { ref, watch } from 'vue';

export interface LeaveSummary {
  nik: string;
  nama_lengkap: string;
  jabatan: string;
  join_date?: string;
  jatah_awal: number;
  cuti_bersama: number;
  monthly_a: number[];
  monthly_b: number[];
  total_cuti_terpakai: number;
  total_potongan_gaji: number;
  sisa_cuti: number;
}

const props = defineProps<{
  isOpen: boolean;
  employee: LeaveSummary | null;
  selectedYear: number;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', payload: { nik: string; jatahAwal: number; cutiBersama: number }): void;
}>();

const formJatahAwal = ref(12);
const formCutiBersama = ref(6);

watch(() => props.employee, (emp) => {
  if (emp) {
    formJatahAwal.value = emp.jatah_awal;
    formCutiBersama.value = emp.cuti_bersama;
  }
}, { immediate: true });

function handleSave() {
  if (!props.employee) return;
  emit('save', {
    nik: props.employee.nik,
    jatahAwal: formJatahAwal.value,
    cutiBersama: formCutiBersama.value,
  });
}
</script>

<template>
  <Teleport to="body">
    <div v-if="isOpen && employee" class="fixed inset-0 z-[9999] flex items-center justify-center bg-slate-950/60 p-4 backdrop-blur-xs">
      <div class="bg-white rounded-3xl p-6 w-full max-w-md shadow-2xl border border-slate-200 space-y-4">
        
        <!-- Modal Header -->
        <div class="flex items-center justify-between border-b pb-3">
          <h3 class="text-sm font-bold text-slate-900">Atur Kuota Cuti - {{ employee.nama_lengkap }}</h3>
          <button type="button" @click="emit('close')" class="text-slate-400 hover:text-slate-700 text-lg cursor-pointer">&times;</button>
        </div>

        <!-- Inputs & Formula -->
        <div class="space-y-3 text-xs">
          <div>
            <label class="block font-bold text-slate-700 mb-1">Jatah Cuti Tahunan (Hari):</label>
            <input
              v-model.number="formJatahAwal"
              type="number"
              min="0"
              max="30"
              class="w-full px-3 py-2 rounded-xl border border-slate-300 focus:outline-none focus:ring-2 focus:ring-emerald-500"
            />
          </div>

          <div>
            <label class="block font-bold text-slate-700 mb-1">Cuti Bersama (Hari):</label>
            <input
              v-model.number="formCutiBersama"
              type="number"
              min="0"
              max="20"
              class="w-full px-3 py-2 rounded-xl border border-slate-300 focus:outline-none focus:ring-2 focus:ring-emerald-500"
            />
          </div>

          <div class="p-3 bg-slate-50 rounded-xl space-y-1 text-slate-600 font-mono">
            <p>Rumus Sisa Cuti ({{ selectedYear }}):</p>
            <p class="font-bold text-emerald-700">
              Sisa = {{ formJatahAwal }} - {{ employee.total_cuti_terpakai }} (Terpakai) - {{ formCutiBersama }} (Bersama) = {{ formJatahAwal - employee.total_cuti_terpakai - formCutiBersama }}
            </p>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="flex items-center justify-end gap-2 pt-2 border-t">
          <button
            type="button"
            @click="emit('close')"
            class="px-4 py-2 text-xs font-bold text-slate-600 hover:bg-slate-100 rounded-xl cursor-pointer"
          >
            Batal
          </button>
          <button
            type="button"
            @click="handleSave"
            class="px-4 py-2 text-xs font-bold bg-emerald-700 hover:bg-emerald-800 text-white rounded-xl shadow-xs cursor-pointer"
          >
            Simpan Saldo
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>