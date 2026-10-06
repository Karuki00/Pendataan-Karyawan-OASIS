<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import EmployeeDetailModal from "./EmployeeDetailModal.vue";
import type { Employee } from "../types";

const props = defineProps<{ refreshKey: number; initialSearch?: string }>();
const emit = defineEmits<{
  edit: [employee: Employee];
  resetSearch: [];
}>();
const employees = ref<Employee[]>([]);
const search = ref(props.initialSearch || "");
const division = ref("ALL");
const isLoading = ref(false);
const errorMessage = ref("");
const selectedEmployee = ref<Employee | null>(null);
function calculateAge(value: string | null | undefined): number | null {
  if (!value) return null;
  const birthDate = new Date(value);
  if (Number.isNaN(birthDate.getTime())) return null;
  const today = new Date();
  let age = today.getFullYear() - birthDate.getFullYear();
  if (today.getMonth() < birthDate.getMonth()
    || (today.getMonth() === birthDate.getMonth() && today.getDate() < birthDate.getDate())) age--;
  return age;
}
const filteredEmployees = computed(() => {
  let result = employees.value;
  if (search.value.toUpperCase() === "PENSIUN") {
    result = result.filter((employee) => {
      const age = calculateAge(employee.tanggal_lahir);
      return age !== null && age >= 55;
    });
  }
  return division.value === "ALL"
    ? result
    : result.filter((employee) => (employee.divisi || "").toUpperCase() === division.value);
});

async function loadEmployees() {
  isLoading.value = true;
  try {
    employees.value = await invoke<Employee[]>("list_employees", { search: search.value.toUpperCase() === "PENSIUN" ? "" : search.value });
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally { isLoading.value = false; }
}

function photoLinks(employee: Employee): Array<[string, string]> {
  return [["KTP", employee.foto_ktp], ["KK", employee.foto_kk], ["BPJS K", employee.foto_bpjs_kesehatan], ["BPJS TK", employee.foto_bpjs_ketenagakerjaan]]
    .filter((entry): entry is [string, string] => Boolean(entry[1]));
}

function openDetail(employee: Employee) {
  selectedEmployee.value = employee;
}

async function openLink(url: string) {
  await openUrl(url);
}

watch(search, (value) => {
  if (!value && props.initialSearch) emit("resetSearch");
  void loadEmployees();
});
watch(() => props.initialSearch, (value) => {
  search.value = value || "";
});
watch(() => props.refreshKey, () => void loadEmployees());
onMounted(() => void loadEmployees());
</script>

<template>
  <section class="space-y-4">
    <div class="flex flex-col gap-3 sm:flex-row sm:items-center">
      <input v-model="search" class="w-full rounded-lg border border-slate-300 bg-white px-4 py-2.5 outline-none focus:border-blue-500" placeholder="Cari nama, NIK, atau divisi...">
      <select v-model="division" class="rounded-lg border border-slate-300 bg-white px-3 py-2.5 outline-none focus:border-blue-500" aria-label="Filter divisi">
        <option value="ALL">Semua Divisi</option>
        <option value="SECURITY">SECURITY</option>
        <option value="ENGINEERING">ENGINEERING</option>
        <option value="HOUSEKEEPING">HOUSEKEEPING</option>
        <option value="STAFF">STAFF</option>
      </select>
      <span class="whitespace-nowrap text-sm text-slate-500">{{ filteredEmployees.length }} karyawan</span>
    </div>
    <p v-if="errorMessage" class="rounded-lg bg-red-50 p-3 text-sm text-red-700">{{ errorMessage }}</p>
    <div class="overflow-hidden rounded-2xl border border-slate-200 bg-white">
      <div class="overflow-auto">
        <table class="min-w-[1000px] w-full text-left text-sm">
          <thead class="bg-slate-100 text-xs uppercase text-slate-500"><tr><th class="w-16 px-4 py-3">No.</th><th class="px-4 py-3">Nama</th><th class="px-4 py-3">NIK</th><th class="px-4 py-3">Divisi / Jabatan</th><th class="px-4 py-3">Kontak</th><th class="px-4 py-3">Dokumen</th><th class="px-4 py-3">Status</th><th class="px-4 py-3"></th></tr></thead>
          <tbody>
            <tr v-if="isLoading"><td colspan="8" class="px-4 py-12 text-center text-slate-500">Memuat data...</td></tr>
            <tr v-else-if="!filteredEmployees.length"><td colspan="8" class="px-4 py-12 text-center text-slate-500">Belum ada data karyawan.</td></tr>
            <tr v-for="(employee, index) in filteredEmployees" v-else :key="employee.nik" class="cursor-pointer border-t border-slate-100 hover:bg-slate-50" @click="openDetail(employee)">
              <td class="px-4 py-3 font-semibold text-slate-400">{{ index + 1 }}</td><td class="px-4 py-3 font-medium text-slate-800">{{ employee.nama_lengkap }}</td><td class="px-4 py-3">{{ employee.nik }}</td><td class="px-4 py-3">{{ employee.divisi || "-" }}<br><span class="text-xs text-slate-500">{{ employee.jabatan || "-" }}</span></td><td class="px-4 py-3">{{ employee.nomor_telepon || "-" }}</td>
              <td class="px-4 py-3"><div class="flex flex-wrap gap-1"><button v-for="[label, url] in photoLinks(employee)" :key="label" class="text-xs text-blue-600 underline hover:text-blue-800" @click.stop="openLink(url)">{{ label }}</button><span v-if="!photoLinks(employee).length">-</span></div></td>
              <td class="px-4 py-3"><span class="rounded-full bg-emerald-100 px-2.5 py-1 text-xs font-semibold text-emerald-700">{{ employee.status || "AKTIF" }}</span></td>
              <td class="px-4 py-3"><div class="flex gap-2">
                <button class="rounded-md bg-blue-600 px-2.5 py-1.5 text-xs font-semibold text-white hover:bg-blue-700" @click.stop="openDetail(employee)">Detail</button>
                <button class="rounded-md bg-amber-100 px-2.5 py-1.5 text-xs font-semibold text-amber-800 hover:bg-amber-200" @click.stop="emit('edit', employee)">Edit</button>
              </div></td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
    <EmployeeDetailModal v-if="selectedEmployee" :employee="selectedEmployee" @close="selectedEmployee = null" />
  </section>
</template>
