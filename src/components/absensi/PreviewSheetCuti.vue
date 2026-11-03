<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface LeaveSummary {
  nik: string;
  nama_lengkap: string;
  jabatan: string;
  join_date?: string;
  jatah_awal: number;
  cuti_bersama: number;
  monthly_a: number[]; // Cuti dari Quota (Status = CUTI)
  monthly_b: number[]; // Cuti Potong Gaji (Status = IZIN/ALPA dsb tergantung rules)
  total_cuti_terpakai: number;
  total_potongan_gaji: number;
  sisa_cuti: number;
}

const props = defineProps<{
  selectedYear: number;
}>();

const leaveData = ref<LeaveSummary[]>([]);
const isLoading = ref(false);

// ⬇️ UPDATE: State reaktif untuk filter Divisi (Default: Staff)
const selectedDivision = ref('staff');

const monthHeaders = ['JAN', 'FEB', 'MAR', 'APR', 'MEI', 'JUN', 'JUL', 'AGS', 'SEP', 'OKT', 'NOV', 'DES'];

async function loadLeaveData() {
  isLoading.value = true;
  try {
    // ⬇️ UPDATE: Mengirim parameter `divisi` ke backend Rust
    const res = await invoke<LeaveSummary[]>('get_rekap_cuti_tahun', {
      tahun: props.selectedYear,
      divisi: selectedDivision.value
    });
    leaveData.value = res;
  } catch (err) {
    console.error('Gagal memuat preview rekap cuti:', err);
  } finally {
    isLoading.value = false;
  }
}

// ⬇️ UPDATE: Menjalankan ulang fetch DB lokal setiap kali Tahun ATAU Divisi diubah
watch([() => props.selectedYear, selectedDivision], () => {
  void loadLeaveData();
});

onMounted(() => {
  void loadLeaveData();
});
</script>

<template>
  <div class="bg-white p-6 rounded-2xl border border-slate-200/80 shadow-xs space-y-4">
    
    <!-- ⬇️ UPDATE: Header dengan Dropdown Filter Divisi -->
    <div class="flex flex-col sm:flex-row sm:items-center justify-between pb-4 border-b gap-4">
      <div>
        <h2 class="text-lg font-bold text-slate-900 uppercase">
          REKAPITULASI CUTI {{ selectedDivision === 'staff' ? 'STAFF PENGELOLA' : selectedDivision }}
        </h2>
        <h3 class="text-sm font-semibold text-emerald-800">Periode Tahun: {{ selectedYear }}</h3>
      </div>
      
      <div class="flex items-center gap-2 bg-slate-50 px-3 py-1.5 rounded-xl border border-slate-200">
        <span class="text-xs font-bold text-slate-500">Pilih Divisi:</span>
        <select
          v-model="selectedDivision"
          class="bg-transparent text-xs font-bold text-slate-800 border-0 focus:outline-none cursor-pointer"
        >
          <option value="staff">Staff & Pengelola</option>
          <option value="security">Security</option>
          <option value="housekeeping">Housekeeping</option>
          <option value="engineering">Engineering</option>
        </select>
      </div>
    </div>

    <div v-if="isLoading" class="p-8 text-center text-slate-400 font-medium">
      Memuat preview Rekap Cuti...
    </div>

    <div v-else-if="leaveData.length === 0" class="p-8 text-center text-slate-400">
      Tidak ada data cuti untuk ditampilkan pada divisi ini di tahun {{ selectedYear }}.
    </div>

    <div v-else class="overflow-x-auto border rounded-xl">
      <table class="min-w-full text-center divide-y divide-slate-200 text-[11px]">
        <thead class="bg-emerald-50 text-slate-700 font-bold">
          <tr>
            <th rowspan="2" class="px-2 py-2 border-r">NO</th>
            <th rowspan="2" class="px-3 py-2 text-left border-r min-w-[160px]">NAMA</th>
            <th rowspan="2" class="px-3 py-2 text-left border-r min-w-[120px]">JABATAN</th>
            <th rowspan="2" class="px-2 py-2 border-r min-w-[80px]">JATAH CUTI</th>
            <th v-for="m in monthHeaders" :key="m" colspan="2" class="px-2 py-1 border-r border-b">
              {{ m }}
            </th>
            <th rowspan="2" class="px-2 py-2 bg-purple-100 border-r text-purple-900">CUTI TERPAKAI</th>
            <th rowspan="2" class="px-2 py-2 bg-amber-100 border-r text-amber-900">POT. GAJI</th>
            <th rowspan="2" class="px-2 py-2 bg-emerald-100 text-emerald-900 min-w-[80px]">SISA CUTI</th>
          </tr>
          <tr>
            <template v-for="m in 12" :key="'sub_'+m">
              <th class="px-1 py-1 border-r text-[10px] bg-purple-50 text-purple-800" title="A: Pengambilan Cuti Quota">A</th>
              <th class="px-1 py-1 border-r text-[10px] bg-amber-50 text-amber-800" title="B: Potongan Gaji">B</th>
            </template>
          </tr>
        </thead>
        <tbody class="divide-y divide-slate-100 font-mono">
          <tr v-for="(row, idx) in leaveData" :key="row.nik" class="hover:bg-slate-50">
            <td class="px-2 py-2 border-r text-slate-500 font-sans">{{ idx + 1 }}</td>
            <td class="px-3 py-2 text-left font-bold text-slate-900 border-r font-sans">{{ row.nama_lengkap }}</td>
            <td class="px-3 py-2 text-left text-slate-600 border-r font-sans">{{ row.jabatan }}</td>
            <td class="px-2 py-2 border-r font-bold text-slate-700">{{ row.jatah_awal }}</td>

            <!-- Monthly A / B -->
            <template v-for="mIdx in 12" :key="'col_'+mIdx">
              <td class="px-1 py-2 border-r text-slate-700 font-semibold" :class="row.monthly_a[mIdx-1] > 0 ? 'bg-purple-50 text-purple-900 font-extrabold' : ''">
                {{ row.monthly_a[mIdx - 1] || '-' }}
              </td>
              <td class="px-1 py-2 border-r text-slate-700 font-semibold" :class="row.monthly_b[mIdx-1] > 0 ? 'bg-amber-50 text-amber-900 font-extrabold' : ''">
                {{ row.monthly_b[mIdx - 1] || '-' }}
              </td>
            </template>

            <!-- Totals -->
            <td class="px-2 py-2 border-r bg-purple-50/80 font-bold text-purple-900">{{ row.total_cuti_terpakai }}</td>
            <td class="px-2 py-2 border-r bg-amber-50/80 font-bold text-amber-900">{{ row.total_potongan_gaji }}</td>
            <td 
              class="px-2 py-2 font-bold"
              :class="row.sisa_cuti < 0 ? 'bg-rose-100 text-rose-800 font-extrabold' : 'bg-emerald-50 text-emerald-900'"
            >
              {{ row.sisa_cuti }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>