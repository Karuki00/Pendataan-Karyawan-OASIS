<script setup lang="ts">
// Props can be added later if active route/tab handling is needed
</script>

<template>
  <aside class="w-64 bg-[#1c3323] text-slate-300 flex flex-col justify-between shrink-0 h-screen print:hidden">
    <div>
      <!-- Logo Header -->
      <div class="h-20 flex items-center px-6 space-x-3 border-b border-emerald-900/30">
        <div class="w-8 h-8 rounded bg-white/20 border border-white/30 flex items-center justify-center text-white font-bold text-xs">
          OA
        </div>
        <div>
          <h1 class="text-sm font-bold text-white tracking-wide leading-tight">OASIS</h1>
          <p class="text-[10px] text-emerald-300/70 font-medium tracking-wider uppercase">MITRA SARANA</p>
        </div>
      </div>

      <!-- Navigation Items -->
      <nav class="px-3 space-y-1 mt-4">
        <a href="#" class="flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium bg-[#2a4732] text-white">
          <svg class="w-4 h-4 text-emerald-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z"></path></svg>
          <span class="flex-1">Dashboard</span>
          <span class="w-1 h-4 bg-amber-400 rounded-full"></span>
        </a>

        <a href="#" class="flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium text-slate-300 hover:bg-[#233f2b] transition">
          <svg class="w-4 h-4 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16m14 0h2m-2 0h-5m-9 0H3m2 0h5m0 0h5m0 0v-5a1 1 0 011-1h2a1 1 0 011 1v5"></path></svg>
          <span>Facilities & Hydrants</span>
        </a>

        <a href="#" class="flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium text-slate-300 hover:bg-[#233f2b] transition">
          <svg class="w-4 h-4 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path></svg>
          <span>Inspection Logs</span>
        </a>

        <a href="#" class="flex items-center space-x-3 px-3 py-2.5 rounded-lg text-sm font-medium text-slate-300 hover:bg-[#233f2b] transition">
          <svg class="w-4 h-4 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path></svg>
          <span>Reported Defects</span>
        </a>
      </nav>
    </div>

    <!-- User Profile at bottom -->
    <div class="p-4 border-t border-emerald-900/40 flex items-center justify-between">
      <div class="flex items-center space-x-3">
        <div class="w-9 h-9 rounded-full bg-emerald-800 border border-white/20 flex items-center justify-center font-bold text-white text-xs">
          AD
        </div>
        <div>
          <p class="text-xs font-bold text-white leading-tight">Admin Security</p>
          <p class="text-[10px] text-slate-400">Chief Inspector</p>
        </div>
      </div>
    </div>
  </aside>
</template>