<script setup lang="ts">
import logoUrl from "../../assets/LOGO_OASIS_V2.png";
import tauriConfig from "../../../src-tauri/tauri.conf.json";

defineEmits<{
  (e: 'openAbout'): void;
}>();

const appVersion = tauriConfig.version;
</script>

<template>
  <header class="top-0 z-30 border-b border-emerald-900/10 bg-gradient-to-r from-[#0d361f] via-[#115231] to-[#18603b] shadow-lg backdrop-blur-md">
    <div class="mx-auto flex max-w-7xl items-center justify-between px-6 py-3.5">
      <div class="flex items-center gap-4">
        <div class="flex items-center justify-center rounded-2xl bg-white/10 p-2 border border-white/10 shadow-inner">
          <img :src="logoUrl" alt="Logo Apartemen Oasis" class="h-12 w-auto object-contain" />
        </div>
        <div>
          <h1 class="text-xl font-bold text-white flex items-center gap-2">
            Database Karyawan
            <span class="rounded-full bg-emerald-400/20 px-2 py-0.5 text-[10px] text-emerald-200 border border-emerald-400/30">
              Oasis Portal
            </span>
          </h1>
          <p class="text-xs text-emerald-100/70">Sistem Pendataan &amp; Pusat Keputusan Manajemen</p>
        </div>
      </div>

      <button @click="$emit('openAbout')" class="flex items-center gap-2 rounded-xl border border-white/20 bg-white/10 px-3.5 py-2 text-xs font-semibold text-white hover:bg-white/20 cursor-pointer">
        <span>ℹ️ Tentang Aplikasi</span>
        <span class="rounded-lg bg-emerald-950/40 px-2 py-0.5 text-[10px] font-mono">v{{ appVersion }}</span>
      </button>
    </div>
  </header>
</template>