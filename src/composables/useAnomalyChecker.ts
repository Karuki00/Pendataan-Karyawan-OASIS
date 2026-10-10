import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Employee } from "../types";

export function useAnomalyChecker() {
  const showAnomalyToast = ref(false);
  const warningEmployeesCount = ref(0);
  const anomalyDetails = ref<string[]>([]);

  async function checkDataAnomalies() {
    try {
      const employees = await invoke<Employee[]>("list_employees", { search: "" });
      const flaggedEmployees = employees.filter((emp) => emp.is_flagged);

      warningEmployeesCount.value = flaggedEmployees.length;

      if (flaggedEmployees.length > 0) {
        anomalyDetails.value = [
          `Terdapat ${flaggedEmployees.length} data karyawan yang perlu diperbaiki/dilengkapi.`
        ];
        showAnomalyToast.value = true;
      } else {
        showAnomalyToast.value = false;
      }
    } catch (err) {
      console.error("Gagal memeriksa flag data:", err);
    }
  }

  return {
    showAnomalyToast,
    warningEmployeesCount,
    anomalyDetails,
    checkDataAnomalies,
  };
}