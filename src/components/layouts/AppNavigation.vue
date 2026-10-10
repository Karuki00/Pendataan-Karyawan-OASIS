<script setup lang="ts">
export type Tab = "dashboard" | "master" | "absensi" | "cuti";

defineProps<{
  activeTab: Tab;
}>();

const emit = defineEmits<{
  (e: 'selectTab', tab: Tab): void;
}>();
</script>

<template>
  <nav class="flex items-center gap-1.5 rounded-2xl bg-white/80 p-1.5 shadow-md border border-amber-900/5">
    <button 
      class="flex items-center gap-2 rounded-xl px-5 py-2.5 text-sm font-semibold transition-all cursor-pointer"
      :class="activeTab === 'dashboard' ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-md' : 'text-slate-600 hover:bg-slate-100'" 
      @click="emit('selectTab', 'dashboard')"
    >
      <span>📊 Dashboard</span>
    </button>
    
    <button 
      class="flex items-center gap-2 rounded-xl px-5 py-2.5 text-sm font-semibold transition-all cursor-pointer"
      :class="activeTab === 'master' ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-md' : 'text-slate-600 hover:bg-slate-100'" 
      @click="emit('selectTab', 'master')"
    >
      <span>📋 Master Data Karyawan</span>
    </button>

    <button 
      class="flex items-center gap-2 rounded-xl px-5 py-2.5 text-sm font-semibold transition-all cursor-pointer"
      :class="activeTab === 'absensi' ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-md' : 'text-slate-600 hover:bg-slate-100'" 
      @click="emit('selectTab', 'absensi')"
    >
      <span>📅 Rekapitulasi Absensi</span>
    </button>

    <button 
      class="flex items-center gap-2 rounded-xl px-5 py-2.5 text-sm font-semibold transition-all cursor-pointer"
      :class="activeTab === 'cuti' ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-md' : 'text-slate-600 hover:bg-slate-100'" 
      @click="emit('selectTab', 'cuti')"
    >
      <span>🌴 Rekap Cuti</span>
    </button>
  </nav>
</template>