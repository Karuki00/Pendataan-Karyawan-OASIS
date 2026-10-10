import { ref, watch } from 'vue';

const STORAGE_KEY_LEAVE_YEAR = 'oasis_leave_year';

const selectedLeaveYear = ref<number>(
  Number(localStorage.getItem(STORAGE_KEY_LEAVE_YEAR)) || new Date().getFullYear()
);

watch(
  selectedLeaveYear,
  (newY) => localStorage.setItem(STORAGE_KEY_LEAVE_YEAR, String(newY)),
  { immediate: true }
);

export function useCutiStore() {
  return {
    selectedLeaveYear,
  };
}