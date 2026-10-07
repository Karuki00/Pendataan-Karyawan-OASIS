<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import tauriConfig from "../../src-tauri/tauri.conf.json";

defineEmits<{
  close: [];
}>();

import logoUrl from "../assets/LOGO_OASIS_V2.png";

// App metadata
const appVersion = tauriConfig.version;
const appTitle = tauriConfig.productName || "Pendataan Karyawan OASIS";
const developer = "Apartemen Oasis Mitra Sarana";
const techStack = [
  { name: "Tauri v2", role: "Desktop App Framework" },
  { name: "Rust & rusqlite", role: "Backend Core & Local Database" },
  { name: "Vue 3 & Vite", role: "Frontend UI Engine" },
  { name: "Tailwind CSS", role: "Styling & UI Components" },
  { name: "SQLite", role: "Local-First Storage (AppData)" },
];

async function openGitHub() {
  try {
    await openUrl("https://github.com/Karuki00/Pendataan-Karyawan-OASIS");
  } catch (error) {
    console.error("Gagal membuka URL:", error);
  }
}
</script>

<template>
  <!-- Modal Overlay -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4 backdrop-blur-sm" @click.self="$emit('close')">
    <div class="w-full max-w-md overflow-hidden rounded-2xl bg-white shadow-2xl border border-slate-100 animate-in fade-in zoom-in-95 duration-150">
      
      <!-- Header Banner -->
      <div class="bg-gradient-to-r from-blue-600 to-indigo-700 p-6 text-white text-center relative">
        <button 
          @click="$emit('close')" 
          class="absolute top-4 right-4 text-white/70 hover:text-white text-lg font-bold w-8 h-8 rounded-full hover:bg-white/10 transition-colors"
        >
          ✕
        </button>
        <div class="flex items-center justify-center">
          <img 
          :src="logoUrl" 
          alt="Logo Apartemen Oasis" 
          class="h-[10vh] w-auto items-center justify-center object-contain"
            />
        </div>
        <h3 class="text-xl font-extrabold tracking-tight">{{ appTitle }}</h3>
        <p class="text-xs text-blue-100 mt-1 font-medium">Sistem Informasi Master Data Karyawan</p>
        <span class="mt-3 inline-block rounded-full bg-white/20 px-3 py-0.5 text-xs font-mono font-semibold text-white">
          v{{ appVersion }}
        </span>
      </div>

      <!-- Content Details -->
      <div class="p-6 space-y-5 text-sm text-slate-600">
        
        <!-- App Info Summary -->
        <div class="rounded-xl bg-slate-50 p-4 border border-slate-200/80 space-y-2">
          <div class="flex justify-between items-center text-xs">
            <span class="text-slate-500 font-medium">Pengembang:</span>
            <span class="font-semibold text-slate-800">{{ developer }}</span>
          </div>
          <div class="flex justify-between items-center text-xs">
            <span class="text-slate-500 font-medium">Arsitektur:</span>
            <span class="font-semibold text-slate-800">Local-First Desktop App</span>
          </div>
          <div class="flex justify-between items-center text-xs">
            <span class="text-slate-500 font-medium">Database:</span>
            <span class="font-semibold text-slate-800">SQLite (AppData Managed)</span>
          </div>
        </div>

        <!-- Tech Stack Badges -->
        <div>
          <h4 class="text-xs font-bold uppercase tracking-wider text-slate-400 mb-2">Teknologi Terpasang</h4>
          <div class="grid grid-cols-2 gap-2">
            <div v-for="tech in techStack" :key="tech.name" class="rounded-lg border border-slate-200 p-2 text-xs">
              <div class="font-bold text-slate-800">{{ tech.name }}</div>
              <div class="text-[10px] text-slate-400">{{ tech.role }}</div>
            </div>
          </div>
        </div>

        <!-- Footer Notes & Links -->
        <div class="pt-2 text-center text-xs text-slate-400 border-t border-slate-100 space-y-2">
          <p>© 2026 Apartemen Oasis Mitra Sarana. All rights reserved.</p>
          <button @click="openGitHub" class="text-blue-600 hover:underline font-semibold inline-flex items-center gap-1">
            <span>🔗 Repositori GitHub</span>
          </button>
        </div>

      </div>

      <!-- Action Button -->
      <div class="bg-slate-50 px-6 py-3 border-t border-slate-100 text-right">
        <button 
          @click="$emit('close')" 
          class="rounded-lg bg-slate-800 px-4 py-2 text-xs font-semibold text-white hover:bg-slate-900 transition-colors"
        >
          Tutup
        </button>
      </div>

    </div>
  </div>
</template>