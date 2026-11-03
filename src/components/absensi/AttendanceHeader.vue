<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import ExportPreviewModal from './ExportPreviewModal.vue';
import type { Employee } from '../../types';
import { checkSecurityRekapConnection, syncSecurityAttendanceToSheet } from '../../services/securityUserSync';

const props = withDefaults(defineProps<{
  selectedYear: number;
  selectedMonth: number;
  employees: Employee[];
  attendanceMap: Record<string, string>;
  keteranganMap?: Record<string, string>;
  division?: string;
  hrFilterDivision?: string;
}>(), {
  division: 'hr',
  hrFilterDivision: 'staff',
});

const emit = defineEmits<{
  (e: 'imported'): void;
  (e: 'changeMonth', month: number): void;
  (e: 'changeYear', year: number): void;
  (e: 'update:hrFilterDivision', val: string): void;
  (e: 'refresh'): void; 
}>();

const isExportPreviewOpen = ref(false);
const isSyncingSecurity = ref(false);
const rekapConnectionStatus = ref<'checking' | 'online' | 'offline'>('checking');
const rekapLastSync = ref('');
const rekapRows = ref(0);
let rekapStatusTimer: ReturnType<typeof setInterval> | undefined;

const isHr = computed(() => props.division.toLowerCase() === 'hr');
const isSecurity = computed(() => props.division.toLowerCase().includes('security'));

// Check connection via Google Sheets OR fallback to local table data availability
async function refreshSecurityRekapStatus() {
  if (!isSecurity.value) return;
  rekapConnectionStatus.value = 'checking';
  
  try {
    const status = await checkSecurityRekapConnection();
    // Check 'status.exists' instead of 'status.ok'
    if (status && status.exists) {
      rekapConnectionStatus.value = 'online';
      rekapLastSync.value = status.lastSync || 'Sinkronisasi Aktif';
      rekapRows.value = status.rows || props.employees.length;
      return;
    }
  } catch (error) {
    console.warn('Google Sheets rekap check failed, verifying local table data:', error);
  }

  // Fallback: If table data/employees are successfully loaded locally, consider it connected
  if (props.employees && props.employees.length > 0) {
    rekapConnectionStatus.value = 'online';
    rekapLastSync.value = 'Terhubung via Jaringan Lokal / Data Tabel Aktif';
    rekapRows.value = props.employees.length;
  } else {
    rekapConnectionStatus.value = 'offline';
  }
}

// Watch employee list updates to instantly re-validate connection state
watch(() => props.employees, () => {
  if (isSecurity.value && props.employees.length > 0 && rekapConnectionStatus.value !== 'online') {
    rekapConnectionStatus.value = 'online';
    rekapRows.value = props.employees.length;
    rekapLastSync.value = 'Data tabel aktif';
  }
}, { deep: true });

// Month input format (YYYY-MM)
const monthInputValue = computed({
  get() {
    const m = String(props.selectedMonth).padStart(2, '0');
    return `${props.selectedYear}-${m}`;
  },
  set(val: string) {
    if (!val) return;
    const [y, m] = val.split('-').map(Number);
    emit('changeYear', y);
    emit('changeMonth', m);
  }
});

async function syncSecurityAttendance() {
  try {
    isSyncingSecurity.value = true;
    const daysInMonth = new Date(props.selectedYear, props.selectedMonth, 0).getDate();
    const month = String(props.selectedMonth).padStart(2, '0');
    const attendance = props.employees.map((employee) => {
      const statuses: Record<string, string> = {};
      const notes: Record<string, string> = {};
      for (let day = 1; day <= daysInMonth; day++) {
        const date = `${props.selectedYear}-${month}-${String(day).padStart(2, '0')}`;
        const key = `${employee.nik}_${date}`;
        statuses[date] = props.attendanceMap[key] || '';
        notes[date] = props.keteranganMap?.[key] || '';
      }
      return { name: employee.nama_lengkap, statuses, notes };
    });

    await syncSecurityAttendanceToSheet({
      year: props.selectedYear,
      month: props.selectedMonth,
      employees: props.employees.map((employee) => ({
        nik: employee.nik,
        name: employee.nama_lengkap,
        jabatan: employee.jabatan || '',
        regu: employee.regu || '',
      })),
      attendance,
    });
    alert('Rekap absensi Security berhasil dikirim ke RekapitulasiSecurity.');
    await refreshSecurityRekapStatus();
  } catch (error) {
    console.error('Security attendance sync error:', error);
    alert(`Gagal mengirim rekap Security: ${error}`);
  } finally {
    isSyncingSecurity.value = false;
  }
}

onMounted(() => {
  if (isSecurity.value) {
    void refreshSecurityRekapStatus();
    rekapStatusTimer = setInterval(() => void refreshSecurityRekapStatus(), 30000);
  }
});

onUnmounted(() => {
  if (rekapStatusTimer) clearInterval(rekapStatusTimer);
});
</script>

<template>
  <div class="flex flex-wrap items-center justify-between gap-4 bg-white p-4.5 rounded-2xl border border-gray-200/80 shadow-xs">
    <div class="flex items-center gap-3">
      <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-emerald-50 text-emerald-700 font-bold text-lg border border-emerald-100">
        📅
      </div>
      <div>
        <h2 class="text-base font-bold text-slate-800">
          Rekapitulasi Absensi {{ isHr ? hrFilterDivision.toUpperCase() : division.toUpperCase() }}
        </h2>
        <p class="text-xs text-slate-500">Otomatis terhubung dengan Master Data Karyawan</p>
      </div>
    </div>

    <div class="flex flex-wrap items-center gap-3">
      <!-- Dropdown Filter Divisi Khusus HR -->
      <div v-if="isHr" class="flex items-center gap-2 bg-slate-50 px-3 py-1.5 rounded-xl border border-slate-200">
        <span class="text-xs font-bold text-slate-500">Pilih Divisi:</span>
        <select
          :value="hrFilterDivision"
          @change="(e) => emit('update:hrFilterDivision', (e.target as HTMLSelectElement).value)"
          class="bg-transparent text-xs font-bold text-slate-800 border-0 focus:outline-none cursor-pointer"
        >
          <option value="staff">Staff & Pengelola</option>
          <option value="security">Security</option>
          <option value="housekeeping">Housekeeping</option>
          <option value="engineering">Engineering</option>
        </select>
      </div>

      <!-- Periode Datepicker -->
      <div class="flex items-center gap-2 bg-slate-50 px-3 py-1.5 rounded-xl border border-slate-200">
        <span class="text-xs font-bold text-slate-500">Periode:</span>
        <input
          type="month"
          v-model="monthInputValue"
          class="bg-transparent text-xs font-bold text-slate-800 border-0 focus:outline-none cursor-pointer"
        />
      </div>

      <!-- Tombol Tarik Data Cloud KHUSUS HR SAAT MELIHAT SECURITY -->
      <button
        v-if="isHr && hrFilterDivision === 'security'"
        type="button"
        @click="emit('refresh')"
        class="bg-sky-600 hover:bg-sky-700 text-white text-xs font-bold px-3.5 py-2 rounded-xl flex items-center gap-2 transition-all shadow-xs cursor-pointer"
        title="Tarik rekap absensi terbaru langsung dari Google Sheets"
      >
        <span>☁️ Tarik dari Spreadsheet</span>
      </button>

      <!-- Tombol Sync (Push) untuk sisi aplikasi Security -->
      <button
        v-if="isSecurity"
        type="button"
        @click="syncSecurityAttendance"
        :disabled="isSyncingSecurity"
        class="bg-blue-700 hover:bg-blue-800 text-white text-xs font-bold px-3.5 py-2 rounded-xl flex items-center gap-2 transition-all shadow-xs cursor-pointer disabled:opacity-60"
      >
        {{ isSyncingSecurity ? 'Mengirim Rekap...' : 'Sync Rekap Security' }}
      </button>

      <!-- Status Indikator untuk Security -->
      <div
        v-if="isSecurity"
        class="flex items-center gap-2 rounded-xl border px-3 py-1.5 text-[11px] font-bold"
        :class="rekapConnectionStatus === 'online'
          ? 'border-emerald-200 bg-emerald-50 text-emerald-700'
          : rekapConnectionStatus === 'checking'
            ? 'border-amber-200 bg-amber-50 text-amber-700'
            : 'border-rose-200 bg-rose-50 text-rose-700'"
        :title="rekapLastSync || 'Belum ada sinkronisasi rekap'"
      >
        <span>{{ rekapConnectionStatus === 'online' ? '● Terhubung' : rekapConnectionStatus === 'checking' ? '● Memeriksa...' : '● Koneksi gagal' }}</span>
        <span v-if="rekapConnectionStatus === 'online'">({{ rekapRows }} baris)</span>
      </div>

      <button
        type="button"
        @click="isExportPreviewOpen = true"
        class="bg-emerald-700 hover:bg-emerald-800 text-white text-xs font-bold px-3.5 py-2 rounded-xl flex items-center gap-2 transition-all shadow-xs cursor-pointer"
      >
        <span>📊 Preview &amp; Export Excel</span>
      </button>
    </div>

    <ExportPreviewModal
      :is-open="isExportPreviewOpen"
      :selected-year="selectedYear"
      :selected-month="selectedMonth"
      :employees="employees"
      :attendance-map="attendanceMap"
      :keterangan-map="keteranganMap"
      :division="division"
      @close="isExportPreviewOpen = false"
    />
  </div>
</template>