<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { Employee } from "../types";

const props = defineProps<{
  isOpen: boolean;
  employeeToEdit?: Employee | null;
}>();

const emit = defineEmits<{
  close: [];
  saved: [warningMessage?: string];
}>();

const isSaving = ref(false);
const errorMessage = ref("");
const existingEmployees = ref<Employee[]>([]);

// Warning states (non-blocking)
const nikWarning = ref("");
const kkWarning = ref("");

const divisionJabatanMap: Record<string, string[]> = {
  STAFF: ["Manager", "Supervisor", "HRD & Legal", "Finance & Accounting", "Admin Pengelola"],
  ENGINEERING: ["Chief Engineering", "Ass. Chief Engineering", "Teknisi Shift", "Teknisi Non-Shift", "Danru Engineering"],
  SECURITY: ["Chief Security", "Danru Security", "Anggota Security (Shift)", "Patroli"],
  HOUSEKEEPING: ["Supervisor Housekeeping", "Leader Housekeeping", "Cleaner", "Gardener"],
};

const formFields: Array<{
  key: keyof Employee;
  label: string;
  type?: string;
  options?: string[];
}> = [
  { key: "nik", label: "NIK *" },
  { key: "nama_lengkap", label: "Nama Lengkap *" },
  { key: "jenis_kelamin", label: "Jenis Kelamin", options: ["Laki-laki", "Perempuan"] },
  { key: "tanggal_lahir", label: "Tanggal Lahir", type: "date" },
  { key: "golongan_darah", label: "Golongan Darah", options: ["A", "B", "AB", "O"] },
  { key: "nomor_kk", label: "Nomor KK" },
  { key: "bpjs_kesehatan", label: "Nomor BPJS Kesehatan" },
  { key: "bpjs_ketenagakerjaan", label: "Nomor BPJS Ketenagakerjaan" },
  { key: "nomor_telepon", label: "Nomor Telepon" },
  { key: "jabatan", label: "Jabatan" },
  { key: "divisi", label: "Divisi", options: ["SECURITY", "ENGINEERING", "HOUSEKEEPING", "STAFF"] },
  { key: "nama_ibu_kandung", label: "Nama Ibu Kandung" },
  { key: "nama_pasangan", label: "Nama Istri/Suami" },
  { key: "jumlah_anak", label: "Jumlah Anak", type: "number" },
  { key: "nomor_telp_keluarga", label: "Nomor Telp Keluarga" },
  { key: "pendidikan_terakhir", label: "Pendidikan Terakhir", options: ["SD", "SMP", "SMA", "SMK", "D3", "S1", "S2", "S3"] },
  { key: "perjanjian_kerja", label: "Perjanjian Kerja", options: ["PKWTT", "PKWT"] },
  { key: "status", label: "Status", options: ["AKTIF", "PENSIUN", "RESIGN"] },
  { key: "tempat_kerja", label: "Tempat Kerja", options: ["Lapangan", "Kantor"] },
];

const emptyEmployee = (): Employee => ({
  nik: "", nama_lengkap: "", jenis_kelamin: "Laki-laki", tanggal_lahir: "", golongan_darah: "A",
  nomor_kk: "", alamat_ktp: "", bpjs_kesehatan: "", bpjs_ketenagakerjaan: "", nomor_telepon: "",
  jabatan: "", divisi: "STAFF", nama_ibu_kandung: "", nama_pasangan: "", jumlah_anak: "",
  nomor_telp_keluarga: "", foto_ktp: "", foto_kk: "", foto_bpjs_kesehatan: "",
  foto_bpjs_ketenagakerjaan: "", pendidikan_terakhir: "SMA", perjanjian_kerja: "PKWTT",
  status: "AKTIF", tempat_kerja: "Lapangan",
});

const form = reactive<Employee>(emptyEmployee());
const isEditMode = computed(() => Boolean(props.employeeToEdit?.nik));
const originalNik = ref("");

const availableJabatans = computed(() => {
  if (!form.divisi) return [];
  return divisionJabatanMap[form.divisi] || [];
});

watch(() => form.divisi, (newDiv) => {
  if (newDiv && divisionJabatanMap[newDiv]) {
    const currentJabatan = form.jabatan || "";
    if (!divisionJabatanMap[newDiv].includes(currentJabatan)) {
      form.jabatan = divisionJabatanMap[newDiv][0] || "";
    }
  }
});

watch(() => props.isOpen, async (openState) => {
  if (openState) {
    try {
      existingEmployees.value = await invoke<Employee[]>("list_employees", { search: "" });
    } catch (e) {
      console.error(e);
    }

    if (props.employeeToEdit) {
      originalNik.value = props.employeeToEdit.nik || "";
      Object.assign(form, props.employeeToEdit);
    } else {
      originalNik.value = "";
      Object.assign(form, emptyEmployee());
    }
    nikWarning.value = "";
    kkWarning.value = "";
    errorMessage.value = "";
  }
});

// Non-blocking Warning Evaluator
function checkWarnings(): string[] {
  const warnings: string[] = [];
  const cleanNik = (form.nik || "").trim();
  const cleanKk = (form.nomor_kk || "").trim();

  if (cleanNik && !/^\d{16}$/.test(cleanNik)) {
    nikWarning.value = "Format NIK tidak standar (seharusnya 16 digit).";
    warnings.push("NIK tidak bernilai 16 digit angka.");
  } else {
    nikWarning.value = "";
  }

  if (cleanKk && !/^\d{16}$/.test(cleanKk)) {
    kkWarning.value = "Format KK tidak standar (seharusnya 16 digit).";
    warnings.push("Nomor KK tidak bernilai 16 digit angka.");
  } else {
    kkWarning.value = "";
  }

  if (!form.bpjs_kesehatan || !form.bpjs_ketenagakerjaan) {
    warnings.push("Nomor BPJS Kesehatan atau Ketenagakerjaan belum diisi.");
  }

  return warnings;
}

async function saveEmployee() {
  if (!form.nik.trim() || !form.nama_lengkap.trim()) {
    errorMessage.value = "NIK dan Nama Lengkap wajib diisi.";
    return;
  }

  const warnings = checkWarnings();
  isSaving.value = true;
  errorMessage.value = "";

  try {
    await invoke("save_employee", { employee: form });
    
    // Kirim catatan warning ke komponen utama jika ada
    const warningSummary = warnings.length > 0 
      ? `Data tersimpan dengan catatan: ${warnings.join(" ")}`
      : undefined;

    emit("saved", warningSummary);
    emit("close");
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isSaving.value = false;
  }
}
</script>

<template>
  <Teleport to="body">
    <div 
      v-if="isOpen" 
      class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/60 backdrop-blur-xs p-4" 
      @click.self="emit('close')"
    >
      <form 
        class="max-h-[90vh] w-full max-w-3xl overflow-auto rounded-3xl bg-white p-7 shadow-2xl border border-slate-100 space-y-6" 
        @submit.prevent="saveEmployee"
      >
        <div class="flex items-center justify-between border-b border-slate-100 pb-4">
          <div class="flex items-center gap-3">
            <div class="flex h-10 w-10 items-center justify-center rounded-xl bg-emerald-50 text-emerald-700 text-xl">
              👤
            </div>
            <div>
              <h2 class="text-lg font-bold text-slate-900">
                {{ isEditMode ? 'Edit Data Karyawan' : 'Tambah Karyawan Baru' }}
              </h2>
              <p class="text-xs text-slate-500">Isi formulir data master karyawan</p>
            </div>
          </div>
          <button type="button" class="text-xl text-slate-400 hover:text-slate-700 cursor-pointer" @click="emit('close')">&times;</button>
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <!-- Input NIK dengan Warning Visual Non-blocking -->
          <label class="block text-xs font-semibold uppercase tracking-wider text-slate-600">
            NIK (16 Digit) *
            <input 
              v-model="form.nik" 
              type="text" 
              maxlength="16"
              @input="checkWarnings"
              placeholder="3175080402560005"
              class="mt-1.5 w-full rounded-xl border px-3.5 py-2.5 text-sm font-normal text-slate-800 outline-none transition-all"
              :class="nikWarning ? 'border-amber-400 bg-amber-50/40 focus:border-amber-500' : 'border-slate-200 bg-slate-50/50 focus:border-emerald-500 focus:bg-white'"
            />
            <span v-if="nikWarning" class="mt-1 block text-[11px] font-semibold text-amber-700 normal-case">
              ⚠️ {{ nikWarning }}
            </span>
          </label>

          <!-- Nama Lengkap -->
          <label class="block text-xs font-semibold uppercase tracking-wider text-slate-600">
            Nama Lengkap *
            <input 
              v-model="form.nama_lengkap" 
              type="text" 
              placeholder="Nama sesuai KTP"
              class="mt-1.5 w-full rounded-xl border border-slate-200 bg-slate-50/50 px-3.5 py-2.5 text-sm font-normal text-slate-800 outline-none transition-all focus:border-emerald-500 focus:bg-white"
            />
          </label>

          <!-- Nomor KK dengan Warning Non-blocking -->
          <label class="block text-xs font-semibold uppercase tracking-wider text-slate-600">
            Nomor KK (16 Digit)
            <input 
              v-model="form.nomor_kk" 
              type="text" 
              maxlength="16"
              @input="checkWarnings"
              placeholder="3175080402560000"
              class="mt-1.5 w-full rounded-xl border px-3.5 py-2.5 text-sm font-normal text-slate-800 outline-none transition-all"
              :class="kkWarning ? 'border-amber-400 bg-amber-50/40 focus:border-amber-500' : 'border-slate-200 bg-slate-50/50 focus:border-emerald-500 focus:bg-white'"
            />
            <span v-if="kkWarning" class="mt-1 block text-[11px] font-semibold text-amber-700 normal-case">
              ⚠️ {{ kkWarning }}
            </span>
          </label>

          <!-- Divisi -->
          <label class="block text-xs font-semibold uppercase tracking-wider text-slate-600">
            Divisi *
            <select v-model="form.divisi" class="mt-1.5 w-full rounded-xl border border-slate-200 bg-slate-50/50 px-3.5 py-2.5 text-sm font-normal text-slate-800 outline-none transition-all focus:border-emerald-500 focus:bg-white">
              <option value="STAFF">STAFF</option>
              <option value="ENGINEERING">ENGINEERING</option>
              <option value="SECURITY">SECURITY</option>
              <option value="HOUSEKEEPING">HOUSEKEEPING</option>
            </select>
          </label>

          <!-- Cascading Jabatan -->
          <label class="block text-xs font-semibold uppercase tracking-wider text-slate-600">
            Jabatan *
            <select v-model="form.jabatan" class="mt-1.5 w-full rounded-xl border border-slate-200 bg-slate-50/50 px-3.5 py-2.5 text-sm font-normal text-slate-800 outline-none transition-all focus:border-emerald-500 focus:bg-white">
              <option value="">Pilih Jabatan (Sesuai Divisi)</option>
              <option v-for="jabatanOption in availableJabatans" :key="jabatanOption" :value="jabatanOption">
                {{ jabatanOption }}
              </option>
            </select>
          </label>

          <!-- Dynamic Field List -->
          <template v-for="field in formFields" :key="field.key">
            <label 
              v-if="!['nik', 'nama_lengkap', 'nomor_kk', 'divisi', 'jabatan'].includes(field.key)"
              class="block text-xs font-semibold uppercase tracking-wider text-slate-600"
            >
              {{ field.label }}
              <select 
                v-if="field.options" 
                v-model="form[field.key]" 
                class="mt-1.5 w-full rounded-xl border border-slate-200 bg-slate-50/50 px-3.5 py-2.5 text-sm font-normal text-slate-800 outline-none transition-all focus:border-emerald-500 focus:bg-white"
              >
                <option value="">Pilih {{ field.label.replace(" *", "") }}</option>
                <option v-for="option in field.options" :key="option" :value="option">{{ option }}</option>
              </select>
              <input 
                v-else 
                v-model="form[field.key]" 
                :type="field.type || 'text'" 
                class="mt-1.5 w-full rounded-xl border border-slate-200 bg-slate-50/50 px-3.5 py-2.5 text-sm font-normal text-slate-800 outline-none transition-all focus:border-emerald-500 focus:bg-white"
              />
            </label>
          </template>

          <!-- Alamat KTP -->
          <label class="block text-xs font-semibold uppercase tracking-wider text-slate-600 sm:col-span-2">
            Alamat Sesuai KTP
            <textarea 
              v-model="form.alamat_ktp" 
              rows="2" 
              class="mt-1.5 w-full rounded-xl border border-slate-200 bg-slate-50/50 px-3.5 py-2.5 text-sm font-normal text-slate-800 outline-none transition-all focus:border-emerald-500 focus:bg-white" 
            />
          </label>

          <!-- URL Links -->
          <label 
            v-for="field in [
              { key: 'foto_ktp', label: 'URL Foto KTP' }, 
              { key: 'foto_kk', label: 'URL Foto KK' }, 
              { key: 'foto_bpjs_kesehatan', label: 'URL Foto BPJS Kesehatan' }, 
              { key: 'foto_bpjs_ketenagakerjaan', label: 'URL Foto BPJS Ketenagakerjaan' }
            ]" 
            :key="field.key" 
            class="block text-xs font-semibold uppercase tracking-wider text-slate-600 sm:col-span-2"
          >
            {{ field.label }}
            <input 
              v-model="form[field.key as keyof Employee]" 
              type="url" 
              placeholder="https://..."
              class="mt-1.5 w-full rounded-xl border border-slate-200 bg-slate-50/50 px-3.5 py-2.5 text-sm font-normal text-slate-800 outline-none transition-all focus:border-emerald-500 focus:bg-white"
            />
          </label>
        </div>

        <p v-if="errorMessage" class="rounded-xl bg-rose-50 p-3.5 text-xs text-rose-700 border border-rose-200">⚠️ {{ errorMessage }}</p>

        <div class="flex items-center justify-end gap-3 pt-4 border-t border-slate-100">
          <button type="button" class="rounded-xl border border-slate-200 bg-white px-5 py-2.5 text-xs font-semibold text-slate-600 hover:bg-slate-50 cursor-pointer" @click="emit('close')">Batal</button>
          <button type="submit" class="inline-flex items-center gap-2 rounded-xl bg-gradient-to-r from-emerald-600 to-teal-600 px-6 py-2.5 text-xs font-semibold text-white shadow-md hover:from-emerald-700 hover:to-teal-700 disabled:opacity-50 cursor-pointer" :disabled="isSaving">
            <span>{{ isSaving ? "Menyimpan..." : "Simpan Data Karyawan" }}</span>
          </button>
        </div>
      </form>
    </div>
  </Teleport>
</template>