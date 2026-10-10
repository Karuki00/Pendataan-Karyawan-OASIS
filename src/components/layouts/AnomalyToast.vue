<script setup lang="ts">
defineProps<{
  show: boolean;
  count: number;
  details: string[];
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'viewWarning'): void;
}>();
</script>

<template>
  <Teleport to="body">
    <Transition name="toast">
      <div 
        v-if="show" 
        class="fixed top-6 right-6 z-[9999] flex items-start gap-3.5 rounded-2xl bg-slate-900/95 p-4.5 text-white shadow-2xl border border-amber-500/40 backdrop-blur-md max-w-md"
      >
        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-amber-500/20 text-amber-400 text-xl border border-amber-500/30">
          ⚠️
        </div>
        <div class="flex-1 min-w-0">
          <div class="flex items-center justify-between gap-2">
            <h4 class="text-xs font-bold text-amber-300 uppercase tracking-wider">
              Pengingat Data Karyawan (Warning)
            </h4>
            <button 
              @click="emit('close')" 
              title="Tutup Notifikasi"
              class="text-slate-400 hover:text-white transition-colors p-0.5 text-sm cursor-pointer"
            >
              ✕
            </button>
          </div>
          
          <p class="text-[11px] text-slate-300 mt-1">
            Ditemukan <strong class="text-amber-300 font-bold">{{ count }} data karyawan</strong> yang memerlukan perhatian:
          </p>

          <ul class="mt-1.5 space-y-1 text-xs text-amber-100/90 list-disc list-inside bg-amber-950/40 p-2 rounded-lg border border-amber-500/20">
            <li v-for="(detail, index) in details" :key="index" class="truncate">
              {{ detail }}
            </li>
          </ul>

          <div class="mt-3 flex items-center gap-2 pt-2 border-t border-slate-800">
            <button 
              @click="emit('viewWarning')"
              class="rounded-lg bg-amber-500 px-3 py-1.5 text-[11px] font-bold text-slate-950 hover:bg-amber-400 transition-colors cursor-pointer flex items-center gap-1.5"
            >
              <span>🔍 Lihat Data Warning</span>
            </button>
            <button 
              @click="emit('close')"
              class="rounded-lg bg-slate-800 px-3 py-1.5 text-[11px] font-semibold text-slate-300 hover:bg-slate-700 transition-colors cursor-pointer"
            >
              Nanti Saja
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.35s cubic-bezier(0.16, 1, 0.3, 1);
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(-1rem) scale(0.95);
}
</style>