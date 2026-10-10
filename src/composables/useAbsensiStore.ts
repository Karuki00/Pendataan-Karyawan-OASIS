import { ref, watch } from 'vue';

const STORAGE_KEY_MAP = 'oasis_absensi_map';
const STORAGE_KEY_KETERANGAN = 'oasis_absensi_keterangan';
const STORAGE_KEY_YEAR = 'oasis_absensi_year';
const STORAGE_KEY_MONTH = 'oasis_absensi_month';

// Shared reactive state outside function scope
const selectedYear = ref<number>(
  Number(localStorage.getItem(STORAGE_KEY_YEAR)) || new Date().getFullYear()
);
const selectedMonth = ref<number>(
  Number(localStorage.getItem(STORAGE_KEY_MONTH)) || (new Date().getMonth() + 1)
);

// Helper to safely parse JSON from localStorage
const readLocalStorageObject = (key: string): Record<string, string> => {
  try {
    const cached = localStorage.getItem(key);
    return cached ? JSON.parse(cached) : {};
  } catch {
    return {};
  }
};

// Attendance map storing "nik_YYYY-MM-DD" -> "STATUS"
const attendanceMap = ref<Record<string, string>>(readLocalStorageObject(STORAGE_KEY_MAP));

// Keterangan map storing "nik_YYYY-MM-DD" -> "KETERANGAN"
const keteranganMap = ref<Record<string, string>>(readLocalStorageObject(STORAGE_KEY_KETERANGAN));

// Watchers for persistent storage
watch(
  selectedYear,
  (newY) => localStorage.setItem(STORAGE_KEY_YEAR, String(newY)),
  { immediate: true }
);

watch(
  selectedMonth,
  (newM) => localStorage.setItem(STORAGE_KEY_MONTH, String(newM)),
  { immediate: true }
);

watch(
  attendanceMap,
  (newMap) => localStorage.setItem(STORAGE_KEY_MAP, JSON.stringify(newMap)),
  { deep: true }
);

watch(
  keteranganMap,
  (newMap) => localStorage.setItem(STORAGE_KEY_KETERANGAN, JSON.stringify(newMap)),
  { deep: true }
);

export function useAbsensiStore() {
  function setCellStatus(nik: string, dateStr: string, status: string, keterangan?: string) {
    const key = `${nik}_${dateStr}`;
    attendanceMap.value = {
      ...attendanceMap.value,
      [key]: status,
    };

    if (keterangan !== undefined) {
      keteranganMap.value = {
        ...keteranganMap.value,
        [key]: keterangan,
      };
    }
  }

  function setCellKeterangan(nik: string, dateStr: string, keterangan: string) {
    const key = `${nik}_${dateStr}`;
    keteranganMap.value = {
      ...keteranganMap.value,
      [key]: keterangan,
    };
  }

  function clearAllState() {
    attendanceMap.value = {};
    keteranganMap.value = {};
    localStorage.removeItem(STORAGE_KEY_MAP);
    localStorage.removeItem(STORAGE_KEY_KETERANGAN);
  }

  return {
    selectedYear,
    selectedMonth,
    attendanceMap,
    keteranganMap,
    setCellStatus,
    setCellKeterangan,
    clearAllState,
  };
}