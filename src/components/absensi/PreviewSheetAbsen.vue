<script setup lang="ts">
import { computed } from 'vue';
import type { Employee } from '../../types';

const props = defineProps<{
  selectedYear: number;
  selectedMonth: number;
  employees: Employee[];
  attendanceMap: Record<string, string>;
  keteranganMap?: Record<string, string>;
}>();

const daysInMonth = computed(() => {
  return new Date(props.selectedYear, props.selectedMonth, 0).getDate();
});

function getStatus(nik: string, day: number): string {
  const m = String(props.selectedMonth).padStart(2, '0');
  const d = String(day).padStart(2, '0');
  const key = `${nik}_${props.selectedYear}-${m}-${d}`;
  return props.attendanceMap[key] || '';
}

function getKeterangan(nik: string, day: number): string {
  if (!props.keteranganMap) return '';
  const m = String(props.selectedMonth).padStart(2, '0');
  const d = String(day).padStart(2, '0');
  const key = `${nik}_${props.selectedYear}-${m}-${d}`;
  return props.keteranganMap[key] || '';
}

function getSummary(nik: string) {
  let on = 0, off = 0, alpa = 0, izin = 0, sakit = 0, cuti = 0, filledDays = 0;
  for (let day = 1; day <= daysInMonth.value; day++) {
    const st = getStatus(nik, day);
    if (st) filledDays++;

    if (st === 'ON') on++;
    else if (st === 'OFF') off++;
    else if (st === 'ALPA') alpa++;
    else if (st === 'IZIN') izin++;
    else if (st.startsWith('SAKIT')) sakit++;
    else if (st === 'CUTI') cuti++;
  }
  
  const isComplete = filledDays === daysInMonth.value;
  return { on, off, alpa, izin, sakit, cuti, filledDays, isComplete };
}

const hasIncompleteEntries = computed(() => {
  return props.employees.some((emp) => !getSummary(emp.nik).isComplete);
});
</script>

<template>
  <div class="space-y-4">
    <!-- Incomplete Warning Banner -->
    <div v-if="hasIncompleteEntries" class="bg-amber-50 border border-amber-200 rounded-xl px-4 py-2.5 flex items-center gap-2 text-xs text-amber-800 font-medium">
      <span class="text-base">⚠️</span>
      <span>Beberapa karyawan memiliki hari yang belum diisi status absensinya (Total hari terisi &lt; {{ daysInMonth }} hari).</span>
    </div>

    <div class="bg-white p-6 rounded-2xl border border-slate-200/80 shadow-xs space-y-4">
      <div class="pb-2 border-b">
        <h2 class="text-lg font-bold text-slate-900">REKAPITULASI ABSENSI STAFF PENGELOLA</h2>
        <h3 class="text-sm font-semibold text-emerald-800">APARTEMEN MITRA OASIS SARANA</h3>
      </div>

      <div class="overflow-x-auto border rounded-xl">
        <table class="min-w-full divide-y divide-slate-200 text-[11px]">
          <thead class="bg-emerald-50 text-slate-700 font-bold">
            <tr>
              <th class="px-2 py-2 text-center border-r">NO</th>
              <th class="px-3 py-2 text-left border-r min-w-[180px]">NAMA ANGGOTA</th>
              <th class="px-3 py-2 text-left border-r min-w-[140px]">JABATAN</th>
              <th v-for="day in daysInMonth" :key="day" class="px-1.5 py-1 text-center border-r min-w-[32px]">
                {{ day }}
              </th>
              <th class="px-2 py-2 text-center bg-emerald-100 border-r">KERJA</th>
              <th class="px-2 py-2 text-center bg-slate-100 border-r">OFF</th>
              <th class="px-2 py-2 text-center bg-rose-100 border-r">ALPA</th>
              <th class="px-2 py-2 text-center bg-sky-100 border-r">IZIN</th>
              <th class="px-2 py-2 text-center bg-amber-100 border-r">SAKIT</th>
              <th class="px-2 py-2 text-center bg-purple-100 border-r">CUTI</th>
              <th class="px-2 py-2 text-center bg-emerald-200 min-w-[60px]">TOTAL</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-slate-100 font-mono">
            <tr v-for="(emp, idx) in employees" :key="emp.nik" class="hover:bg-slate-50">
              <td class="px-2 py-1.5 text-center border-r text-slate-500 font-sans">{{ idx + 1 }}</td>
              <td class="px-3 py-1.5 font-bold text-slate-900 border-r font-sans">{{ emp.nama_lengkap }}</td>
              <td class="px-3 py-1.5 text-slate-600 border-r font-sans">{{ emp.jabatan || 'Staff' }}</td>
              
              <!-- Daily Cells -->
              <td 
                v-for="day in daysInMonth" 
                :key="day" 
                class="px-1 py-1 text-center border-r font-bold text-[10px] relative cursor-help"
                :class="{
                  'text-emerald-700 bg-emerald-50': getStatus(emp.nik, day) === 'ON',
                  'text-slate-700 bg-slate-100': getStatus(emp.nik, day) === 'OFF',
                  'text-amber-800 bg-amber-100': getStatus(emp.nik, day).startsWith('SAKIT'),
                  'text-sky-800 bg-sky-100': getStatus(emp.nik, day) === 'IZIN',
                  'text-purple-800 bg-purple-100': getStatus(emp.nik, day) === 'CUTI',
                  'text-rose-800 bg-rose-100': getStatus(emp.nik, day) === 'ALPA',
                  'text-slate-300 bg-slate-50': !getStatus(emp.nik, day),
                }"
                :title="getKeterangan(emp.nik, day) || undefined"
              >
                {{ getStatus(emp.nik, day) || '-' }}
                <span v-if="getKeterangan(emp.nik, day)" class="absolute top-0 right-0 w-1.5 h-1.5 bg-amber-500 rounded-full"></span>
              </td>

              <!-- Calculations -->
              <td class="px-2 py-1.5 text-center font-bold border-r bg-emerald-50/80">{{ getSummary(emp.nik).on }}</td>
              <td class="px-2 py-1.5 text-center font-bold border-r bg-slate-50">{{ getSummary(emp.nik).off }}</td>
              <td class="px-2 py-1.5 text-center font-bold border-r bg-rose-50 text-rose-700">{{ getSummary(emp.nik).alpa }}</td>
              <td class="px-2 py-1.5 text-center font-bold border-r bg-sky-50 text-sky-700">{{ getSummary(emp.nik).izin }}</td>
              <td class="px-2 py-1.5 text-center font-bold border-r bg-amber-50 text-amber-700">{{ getSummary(emp.nik).sakit }}</td>
              <td class="px-2 py-1.5 text-center font-bold border-r bg-purple-50 text-purple-700">{{ getSummary(emp.nik).cuti }}</td>
              
              <td 
                class="px-2 py-1.5 text-center font-bold font-mono"
                :class="getSummary(emp.nik).isComplete 
                  ? 'bg-emerald-100 text-slate-900' 
                  : 'bg-rose-100 text-rose-800 border border-rose-300 font-extrabold'"
              >
                {{ getSummary(emp.nik).filledDays }} / {{ daysInMonth }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>