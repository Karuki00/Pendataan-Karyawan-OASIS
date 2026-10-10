<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useCutiStore } from '../composables/useCutiStore';
import YearSelector from '../components/cuti/YearSelector.vue';
import EditQuotaModal, { type LeaveSummary } from '../components/cuti/EditQuotaModal.vue';

const { selectedLeaveYear } = useCutiStore();
const leaveData = ref<LeaveSummary[]>([]);
const isLoading = ref(false);

// Modal state
const isModalOpen = ref(false);
const editingEmployee = ref<LeaveSummary | null>(null);

const monthHeaders = ['JAN', 'FEB', 'MAR', 'APR', 'MEI', 'JUN', 'JUL', 'AGS', 'SEP', 'OKT', 'NOV', 'DES'];

async function loadLeaveData() {
  isLoading.value = true;
  try {
    const res = await invoke<LeaveSummary[]>('get_rekap_cuti_tahun', {
      tahun: selectedLeaveYear.value,
    });
    leaveData.value = res;
  } catch (err) {
    console.error('Gagal memuat rekap cuti:', err);
  } finally {
    isLoading.value = false;
  }
}

function openEditModal(emp: LeaveSummary) {
  editingEmployee.value = emp;
  isModalOpen.value = true;
}

async function handleSaveQuota(payload: { nik: string; jatahAwal: number; cutiBersama: number }) {
  try {
    await invoke('update_saldo_cuti_karyawan', {
      nik: payload.nik,
      tahun: selectedLeaveYear.value,
      jatahAwal: payload.jatahAwal,
      cutiBersama: payload.cutiBersama,
    });
    isModalOpen.value = false;
    await loadLeaveData();
  } catch (err) {
    alert(`Gagal memperbarui saldo cuti: ${err}`);
  }
}

watch(selectedLeaveYear, () => {
  void loadLeaveData();
});

onMounted(() => {
  void loadLeaveData();
});
</script>

<template>
  <div class="space-y-4">
    <!-- Header Controls -->
    <div class="bg-white p-4 rounded-2xl border border-slate-200/80 shadow-xs flex flex-wrap items-center justify-between gap-4">
      <div>
        <h2 class="text-base font-bold text-slate-900">Rekapitulasi Cuti Staff Pengelola</h2>
        <p class="text-xs text-slate-500">Apartemen Mitra Oasis Sarana - Tahun {{ selectedLeaveYear }}</p>
      </div>

      <!-- Modular Future-Proof Year Selector -->
      <YearSelector v-model="selectedLeaveYear" />
    </div>

    <!-- Table Content -->
    <div v-if="isLoading" class="p-8 text-center text-slate-500 bg-white rounded-2xl border shadow-xs">
      Memuat data rekap cuti...
    </div>

    <div v-else-if="leaveData.length === 0" class="p-8 text-center text-slate-400 bg-white rounded-2xl border shadow-xs">
      Tidak ada data karyawan aktif untuk rekap cuti.
    </div>

    <div v-else class="bg-white rounded-2xl border border-slate-200/80 shadow-xs overflow-hidden">
      <div class="overflow-x-auto">
        <table class="min-w-full text-center divide-y divide-slate-200 text-[11px]">
          <thead class="bg-emerald-50 text-slate-700 font-bold">
            <tr>
              <th rowspan="2" class="px-2 py-2 border-r">NO</th>
              <th rowspan="2" class="px-3 py-2 text-left border-r min-w-[160px]">NAMA ANGGOTA</th>
              <th rowspan="2" class="px-3 py-2 text-left border-r min-w-[120px]">JABATAN</th>
              <th rowspan="2" class="px-2 py-2 border-r min-w-[80px]">JATAH CUTI</th>
              <th v-for="m in monthHeaders" :key="m" colspan="2" class="px-2 py-1 border-r border-b">
                {{ m }}
              </th>
              <th rowspan="2" class="px-2 py-2 bg-purple-100 border-r text-purple-900">CUTI TERPAKAI</th>
              <th rowspan="2" class="px-2 py-2 bg-amber-100 border-r text-amber-900">POT. GAJI</th>
              <th rowspan="2" class="px-2 py-2 bg-emerald-100 text-emerald-900 min-w-[80px]">SISA CUTI</th>
              <th rowspan="2" class="px-2 py-2 text-center min-w-[60px]">AKSI</th>
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

              <!-- 12 Months Dual Column (A = Cuti Quota, B = Potongan Gaji) -->
              <template v-for="mIdx in 12" :key="'col_'+mIdx">
                <td class="px-1 py-2 border-r text-slate-700 font-semibold" :class="row.monthly_a[mIdx-1] > 0 ? 'bg-purple-50 text-purple-900 font-extrabold' : ''">
                  {{ row.monthly_a[mIdx - 1] || '-' }}
                </td>
                <td class="px-1 py-2 border-r text-slate-700 font-semibold" :class="row.monthly_b[mIdx-1] > 0 ? 'bg-amber-50 text-amber-900 font-extrabold' : ''">
                  {{ row.monthly_b[mIdx - 1] || '-' }}
                </td>
              </template>

              <!-- Calculations -->
              <td class="px-2 py-2 border-r bg-purple-50/80 font-bold text-purple-900">{{ row.total_cuti_terpakai }}</td>
              <td class="px-2 py-2 border-r bg-amber-50/80 font-bold text-amber-900">{{ row.total_potongan_gaji }}</td>
              <td 
                class="px-2 py-2 font-bold"
                :class="row.sisa_cuti < 0 ? 'bg-rose-100 text-rose-800 font-extrabold' : 'bg-emerald-50 text-emerald-900'"
              >
                {{ row.sisa_cuti }}
              </td>

              <!-- Action Button -->
              <td class="px-2 py-2 text-center font-sans">
                <button
                  type="button"
                  @click="openEditModal(row)"
                  class="px-2 py-1 text-[10px] font-bold bg-slate-100 hover:bg-slate-200 text-slate-700 rounded-lg transition-all cursor-pointer"
                >
                  ⚙️ Kuota
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Modular Edit Quota Modal -->
    <EditQuotaModal
      :is-open="isModalOpen"
      :employee="editingEmployee"
      :selected-year="selectedLeaveYear"
      @close="isModalOpen = false"
      @save="handleSaveQuota"
    />
  </div>
</template>