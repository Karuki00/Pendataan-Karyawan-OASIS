<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'resetComplete'): void;
}>();

const isDeleting = ref(false);
const confirmText = ref('');

async function handleReset() {
  if (confirmText.value.trim().toUpperCase() !== 'RESET') {
    alert('Ketik kata "RESET" untuk mengonfirmasi.');
    return;
  }

  if (!confirm('Apakah Anda benar-benar yakin? Seluruh data karyawan, absensi, dan cuti akan dihapus permanen!')) {
    return;
  }

  isDeleting.value = true;
  try {
    await invoke('reset_database');
    alert('Database berhasil dikosongkan!');
    confirmText.value = '';
    emit('resetComplete');
    emit('close');
  } catch (err) {
    console.error('Gagal reset database:', err);
    alert(`Gagal mengosongkan database: ${err}`);
  } finally {
    isDeleting.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="isOpen"
      class="fixed inset-0 z-[9999] flex items-center justify-center bg-slate-950/70 backdrop-blur-xs p-4"
      @click.self="emit('close')"
    >
      <div class="w-full max-w-md rounded-2xl bg-white p-6 shadow-2xl space-y-4 border border-rose-100">
        <div class="flex items-center gap-3 text-rose-600">
          <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-rose-100 font-bold text-xl">
            ⚠️
          </div>
          <div>
            <h3 class="text-base font-bold text-slate-900">Reset Seluruh Database</h3>
            <p class="text-xs text-rose-500 font-medium">Tindakan ini tidak dapat dibatalkan!</p>
          </div>
        </div>

        <p class="text-xs text-slate-600 leading-relaxed">
          Menghapus seluruh data <strong>Master Karyawan</strong>, <strong>Periode Absensi</strong>, <strong>Catatan Absensi Harian</strong>, dan <strong>Saldo Cuti</strong>.
        </p>

        <div class="bg-rose-50 p-3 rounded-xl border border-rose-200/80 space-y-1.5">
          <label class="text-[11px] font-bold text-rose-800 block">
            Ketik <span class="bg-rose-200 px-1 py-0.5 rounded text-rose-950">RESET</span> untuk konfirmasi:
          </label>
          <input
            v-model="confirmText"
            type="text"
            placeholder="RESET"
            class="w-full text-xs px-3 py-2 border rounded-lg bg-white focus:outline-none focus:ring-2 focus:ring-rose-500 uppercase font-mono font-bold"
          />
        </div>

        <div class="flex items-center justify-end gap-2 pt-2 border-t">
          <button
            type="button"
            @click="emit('close')"
            class="px-4 py-2 text-xs font-semibold text-slate-600 hover:bg-slate-100 rounded-xl cursor-pointer"
          >
            Batal
          </button>
          <button
            type="button"
            @click="handleReset"
            :disabled="confirmText.trim().toUpperCase() !== 'RESET' || isDeleting"
            class="px-4 py-2 text-xs font-bold text-white bg-rose-600 hover:bg-rose-700 disabled:opacity-40 disabled:cursor-not-allowed rounded-xl shadow-xs transition-all cursor-pointer"
          >
            {{ isDeleting ? 'Menghapus...' : 'Kosongkan Database' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>