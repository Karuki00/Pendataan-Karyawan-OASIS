<script setup lang="ts">
import { onMounted, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import AppHeader from "./components/layouts/AppHeader.vue";
import AppNavigation, { type Tab } from "./components/layouts/AppNavigation.vue";
import AnomalyToast from "./components/layouts/AnomalyToast.vue";

import DashboardView, { type MetricFilterKey } from "./components/DashboardView.vue";
import EmployeeTable from "./components/EmployeeTable.vue";
import AbsensiView from "./views/AbsensiView.vue"; 
import RekapCutiView from "./views/RekapCutiView.vue";

// New Inspection Views
import SecurityDashboardView from "./views/SecurityDashboardView.vue";
import QrGeneratorView from "./views/QrGeneratorView.vue";

import ExcelImporter from "./components/ExcelImporter.vue";
import EmployeeModal from "./components/EmployeeModal.vue";
import AboutModal from "./components/AboutModal.vue";

import { useUpdater } from "./composables/useUpdater";
import { useAnomalyChecker } from "./composables/useAnomalyChecker";
import type { Employee } from "./types";
import { syncSecurityAuthFromHr, syncSecurityEmployees, syncSecurityUsers } from "./services/securityUserSync";
import { loginSecurityFromAppsScript } from "./services/securityUserSync";
import { appConfig } from "./config";
import {
  loginToMaster,
  recoverMasterPassword,
  clearMasterSession,
  changeOwnCredentials,
  resetDivisionCredentials,
  listDivisionAccounts,
} from "./services/masterAuth";
import { masterStatus, resolveMaster } from "./services/masterDiscovery";
import logoUrl from "./assets/LOGO_OASIS_V1-removebg-preview.png";

// --- Loading Intro State ---
const isInitializing = ref(true);
const loadingProgress = ref(15);
const loadingStatus = ref("Menyiapkan aplikasi...");

// --- Auth State ---
const isLoggedIn = ref(false);
const userRole = ref<string | null>(null); // 'hr', 'security', 'engineering', 'housekeeping'
const loginUsername = ref('');
const loginPassword = ref('');
const loginError = ref('');
const isLoggingIn = ref(false);
const needsPasswordSetup = ref(false);
const runtimeIsMasterNode = ref(false);
const setupCompleted = ref(false);
const isSettingUpPasswords = ref(false);
const setupError = ref('');
const setupSuccess = ref('');
const recoveryCode = ref('');
const recoveryInputCode = ref('');
const isRecoveryOpen = ref(false);
const recoveryNewPassword = ref('');
const recoveryConfirmPassword = ref('');
const recoveryError = ref('');
const recoveryMessage = ref('');
const isRecovering = ref(false);

// --- Password Change / Reset State ---
const isPasswordModalOpen = ref(false);
const passwordCurrent = ref(''); 
const passwordNew = ref('');
const passwordConfirm = ref('');
const passwordTargetRole = ref('security'); 
const passwordNewUsername = ref(''); 
const passwordMessage = ref('');
const passwordError = ref('');
const isSavingPassword = ref(false);
const divisionAccounts = ref<Array<{ username: string; role: string }>>([]);

const setupAccounts = reactive([
  { role: 'hr', label: 'HR Admin', username: 'hr_admin', password: '', confirm: '' },
  { role: 'security', label: 'Chief Security', username: 'chief_security', password: '', confirm: '' },
  { role: 'engineering', label: 'Chief Engineering', username: 'chief_engineering', password: '', confirm: '' },
  { role: 'housekeeping', label: 'Chief Housekeeping', username: 'chief_hk', password: '', confirm: '' },
]);

const activeTab = ref<string>("dashboard");
const refreshKey = ref(0);
const isSyncingSecurityData = ref(false);
const securityDataSyncMessage = ref('');

const isModalOpen = ref(false);
const isImportModalOpen = ref(false);
const showAboutModal = ref(false);
const employeeToEdit = ref<Employee | null>(null);

const masterSearchQuery = ref("");
const activeMetricKey = ref<MetricFilterKey | null>(null);

const { checkForUpdates } = useUpdater();
const { showAnomalyToast, warningEmployeesCount, anomalyDetails, checkDataAnomalies } = useAnomalyChecker();

// Bootstrap / Initial App Hydration

async function bootstrapApp() {
  try {
    runtimeIsMasterNode.value = await invoke<boolean>(
      'get_or_initialize_node_mode',
      {
        preferredMaster: appConfig.isMasterNode,
      }
    );
    console.info(
      `[node] runtime mode = ${runtimeIsMasterNode.value ? 'MASTER' : 'CLIENT'} ` +
      `(build flag isMasterNode = ${appConfig.isMasterNode})`
    );

    // Start the API server only on the master PC.
    if (runtimeIsMasterNode.value) {
      try {
        await invoke('start_master_server');
        console.info('[node] MASTER: API server started on port 3000, advertising _oasis._tcp');
      } catch (error) {
        console.error('[node] MASTER: API server FAILED to start:', error);
      }

      // When a division changes its credentials through the master server,
      // push the updated Security accounts to Apps Script.
      void listen('credentials-changed', () => {
        void syncSecurityAuthFromHr().catch((error) =>
          console.error('Security auth Apps Script sync failed:', error)
        );
      }).catch((error) => console.error('Failed to listen for credentials-changed:', error));
    }

    // Both roles look for a master, so the badge is live from the first screen.
    void resolveMaster(true);

    // First-run password setup only exists on the master. A client's local
    // database never completes it, so asking there would block startup forever.
    needsPasswordSetup.value = runtimeIsMasterNode.value
      ? await invoke<boolean>('get_password_setup_status')
      : false;

    if (needsPasswordSetup.value) {
      loadingStatus.value = 'Menunggu pembuatan password divisi...';
      loadingProgress.value = 100;
      return;
    }

    loadingStatus.value = 'Memeriksa integritas data & anomali...';
    loadingProgress.value = 30;
    await checkDataAnomalies();

    void syncSecurityUsersFromHr().catch((error) =>
      console.error('SecurityUsers Apps Script sync failed:', error)
    );

    if (runtimeIsMasterNode.value) {
      void syncSecurityAuthFromHr().catch((error) =>
        console.error('Security auth Apps Script sync failed:', error)
      );
    }

    loadingStatus.value = 'Memeriksa pembaruan aplikasi...';
    loadingProgress.value = 60;

    // Never auto-update the dev client; it would install a real release over the test setup.
    const isUpdating = import.meta.env.DEV
      ? false
      : await checkForUpdates(true, (progress, statusText) => {
          loadingStatus.value = statusText;
          loadingProgress.value = 60 + Math.round((progress / 100) * 40);
        });

    if (isUpdating) return;

    loadingStatus.value = 'Menyiapkan antarmuka...';
    loadingProgress.value = 100;

    await new Promise((resolve) => setTimeout(resolve, 300));
  } catch (error) {
    console.error('Gagal melakukan inisialisasi aplikasi:', error);
  } finally {
    isInitializing.value = false;
  }
}

async function handlePasswordSetup() {
  setupError.value = '';
  setupSuccess.value = '';

  for (const account of setupAccounts) {
    if (!account.username || account.username.trim().length < 3) {
      setupError.value = `Username ${account.label} minimal 3 karakter.`;
      return;
    }
    if (!account.password || account.password.length < 8) {
      setupError.value = `Password ${account.label} minimal 8 karakter.`;
      return;
    }
    if (account.password !== account.confirm) {
      setupError.value = `Konfirmasi password ${account.label} tidak sama.`;
      return;
    }
  }

  const usernames = setupAccounts.map(a => a.username.trim().toLowerCase());
  if (new Set(usernames).size !== usernames.length) {
    setupError.value = 'Username tiap divisi tidak boleh ada yang sama.';
    return;
  }

  isSettingUpPasswords.value = true;
  try {
    const generatedRecoveryCode = await invoke<string>('setup_division_passwords', {
      accounts: setupAccounts.map((account) => ({
        username: account.username.trim(),
        password: account.password,
        role_division: account.role,
      })),
    });
    
    setupCompleted.value = true;
    
    void syncSecurityAuthFromHr().catch((error) => console.error('Security auth Apps Script sync failed:', error));
    recoveryCode.value = generatedRecoveryCode;
    setupSuccess.value = `Password berhasil dibuat. KODE PEMULIHAN MASTER: ${generatedRecoveryCode} — simpan di tempat aman dan jangan dibagikan.`;
    
    for (const account of setupAccounts) {
      account.password = '';
      account.confirm = '';
    }
  } catch (error) {
    setupError.value = error instanceof Error ? error.message : String(error);
  } finally {
    isSettingUpPasswords.value = false;
  }
}

async function recoverMaster() {
  recoveryError.value = ''; recoveryMessage.value = '';
  if (!recoveryNewPassword.value || recoveryNewPassword.value.length < 8) { recoveryError.value = 'Password baru minimal 8 karakter.'; return; }
  if (recoveryNewPassword.value !== recoveryConfirmPassword.value) { recoveryError.value = 'Konfirmasi password tidak sama.'; return; }
  isRecovering.value = true;
  try {
    await recoverMasterPassword(recoveryInputCode.value, recoveryNewPassword.value);
    recoveryMessage.value = 'Password HR berhasil dipulihkan. Silakan login dengan password baru.';
    loginUsername.value = ''; 
    loginPassword.value = ''; 
    isRecoveryOpen.value = false;
  } catch (error) { recoveryError.value = error instanceof Error ? error.message : String(error); }
  finally { isRecovering.value = false; }
}

async function handleLogin() {
  if (!loginUsername.value || !loginPassword.value) {
    loginError.value = "Username dan password wajib diisi.";
    return;
  }

  isLoggingIn.value = true;
  loginError.value = '';

  try {
    let role: string;
    try {
      role = await loginSecurityFromAppsScript(loginUsername.value, loginPassword.value);
      console.info('[login] OK via Apps Script');
      try {
        await loginToMaster(loginUsername.value, loginPassword.value);
        console.info('[login] LAN session established for account changes');
      } catch (lanSessionError) {
        console.warn('[login] LAN session unavailable after Apps Script login:', lanSessionError);
      }
    } catch (appsScriptError) {
      console.warn('[login] Apps Script login failed, trying master LAN:', appsScriptError);
      try {
        role = await loginToMaster(loginUsername.value, loginPassword.value);
        console.info('[login] OK via master (LAN fallback)');
      } catch (lanError) {
        console.warn('[login] master LAN login failed:', lanError);
        throw appsScriptError;
      }
    }
    
    setLoggedInRole(role);
  } catch (error) {
    loginError.value = String(error);
  } finally {
    isLoggingIn.value = false;
  }
}

// --- Account settings (every division; HR also manages the others) ---

const isEditingOtherDivision = () =>
  userRole.value === 'hr' && passwordTargetRole.value !== 'hr';

function usernameOf(role: string): string {
  return divisionAccounts.value.find((account) => account.role === role)?.username ?? '';
}

async function loadDivisionAccounts() {
  if (userRole.value !== 'hr') return;
  try {
    divisionAccounts.value = await listDivisionAccounts();
  } catch {
    divisionAccounts.value = [];
  }
}

// HR picking another division shows that division's current username.
watch(passwordTargetRole, () => {
  if (userRole.value === 'hr') {
    passwordNewUsername.value = usernameOf(passwordTargetRole.value);
  }
});

async function openPasswordModal() {
  passwordCurrent.value = '';
  passwordNew.value = '';
  passwordConfirm.value = '';
  passwordMessage.value = '';
  passwordError.value = '';
  passwordTargetRole.value = userRole.value || 'security';
  passwordNewUsername.value = loginUsername.value;
  isPasswordModalOpen.value = true;

  if (userRole.value === 'hr') {
    await loadDivisionAccounts();
    passwordNewUsername.value = usernameOf(passwordTargetRole.value) || loginUsername.value;
  }
}

async function savePasswordChange() {
  passwordMessage.value = '';
  passwordError.value = '';

  const newUsername = passwordNewUsername.value.trim();
  if (newUsername.length < 3) { passwordError.value = 'Username minimal 3 karakter.'; return; }
  if (passwordNew.value.length < 8) { passwordError.value = 'Password baru minimal 8 karakter.'; return; }
  if (passwordNew.value !== passwordConfirm.value) { passwordError.value = 'Konfirmasi password tidak sama.'; return; }

  isSavingPassword.value = true;
  try {
    if (isEditingOtherDivision()) {
      // HR superadmin: reset another division's username and password.
      await resetDivisionCredentials(
        passwordCurrent.value,
        passwordTargetRole.value,
        newUsername,
        passwordNew.value,
      );
      passwordMessage.value = `Akun divisi ${passwordTargetRole.value.toUpperCase()} diperbarui. Username baru: ${newUsername}`;
      await loadDivisionAccounts();
    } else {
      // Anyone, for their own account (including HR's own).
      const savedUsername = await changeOwnCredentials(
        passwordCurrent.value,
        newUsername,
        passwordNew.value,
      );
      loginUsername.value = savedUsername;
      passwordMessage.value = 'Username dan password Anda berhasil diperbarui.';
      if (userRole.value === 'hr') await loadDivisionAccounts();
    }

    passwordCurrent.value = ''; passwordNew.value = ''; passwordConfirm.value = '';
    if (runtimeIsMasterNode.value) {
      void syncSecurityAuthFromHr().catch((error) => console.error('Security auth Apps Script sync failed:', error));
    }
  } catch (error) {
    passwordError.value = error instanceof Error ? error.message : String(error);
  } finally {
    isSavingPassword.value = false;
  }
}

function setLoggedInRole(role: string) {
  userRole.value = role;
  isLoggedIn.value = true;
  loginError.value = '';

  if (role === 'hr') {
    activeTab.value = 'dashboard';
  } else if (role === 'security') {
    activeTab.value = 'security_inspection';
  } else if (role === 'engineering') {
    activeTab.value = 'engineering_absensi';
  } else if (role === 'housekeeping') {
    activeTab.value = 'housekeeping_absensi';
  } else {
    activeTab.value = 'dashboard';
  }
}

function logout() {
  clearMasterSession();
  divisionAccounts.value = [];
  isPasswordModalOpen.value = false;
  isLoggedIn.value = false;
  userRole.value = null;
  loginUsername.value = '';
  loginPassword.value = '';
}

onMounted(() => {
  void bootstrapApp();
});

watch(refreshKey, () => {
  void checkDataAnomalies();
});

function filterWarningData() {
  activeMetricKey.value = "flagged" as any; 
  masterSearchQuery.value = "";
  activeTab.value = "master";
  showAnomalyToast.value = false;
}

async function syncSecurityUsersFromHr() {
  if (!runtimeIsMasterNode.value) return;
  const employees = await invoke<Employee[]>("list_employees", { search: "" });
  await syncSecurityUsers(employees);
  await syncSecurityEmployees(employees);
}

async function syncSecurityDatabaseFromHr() {
  securityDataSyncMessage.value = '';
  isSyncingSecurityData.value = true;
  try {
    await syncSecurityUsersFromHr();
    await syncSecurityAuthFromHr();
    securityDataSyncMessage.value = 'Database Security berhasil disinkronkan.';
  } catch (error) {
    securityDataSyncMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isSyncingSecurityData.value = false;
  }
}

function handleSaved() {
  refreshKey.value++;
  void syncSecurityUsersFromHr().catch((error) => console.error('SecurityUsers Apps Script sync failed:', error));
}
function openCreate() { employeeToEdit.value = null; isModalOpen.value = true; }
function openEdit(employee: Employee) { employeeToEdit.value = employee; isModalOpen.value = true; }
function openMaster() { masterSearchQuery.value = ""; activeMetricKey.value = null; activeTab.value = "master"; }
function imported() {
  refreshKey.value++;
  isImportModalOpen.value = false;
  openMaster();
  void syncSecurityUsersFromHr().catch((error) => console.error('SecurityUsers Apps Script sync failed:', error));
}

function switchTab(tab: string) {
  if (tab === 'master') openMaster();
  else {
    masterSearchQuery.value = "";
    activeTab.value = tab;
  }
}
</script>

<template>
  <div class="min-h-screen text-slate-900 bg-gradient-to-br from-[#fef3e2] via-[#fce6bd] to-[#f7d6a0] antialiased relative">
    
    <!-- Intro Loading Screen Overlay -->
    <Transition name="splash-fade">
      <div 
        v-if="isInitializing" 
        class="fixed inset-0 z-[99999] flex flex-col items-center justify-center bg-gradient-to-br from-[#1e293b] via-[#0f172a] to-[#020617] p-6 select-none"
      >
        <div class="flex flex-col items-center gap-6 max-w-sm text-center">
          <div class="relative flex items-center justify-center w-20 h-20 rounded-3xl bg-amber-500/10 border border-amber-500/20 shadow-2xl shadow-amber-900/30">
            <span class="text-4xl animate-bounce">🏢</span>
            <div class="absolute inset-0 rounded-3xl border border-amber-400/20 animate-ping"></div>
          </div>
          <div>
            <h1 class="text-xl font-extrabold tracking-wide text-white">OASIS One</h1>
            <p class="text-xs text-amber-200/70 mt-1 font-medium">Apartemen Mitra Oasis Sarana</p>
          </div>
          <div class="w-full space-y-2.5">
            <div class="h-2 w-full bg-slate-800/80 rounded-full overflow-hidden border border-slate-700/60 p-0.5">
              <div 
                class="h-full bg-gradient-to-r from-amber-500 via-emerald-400 to-teal-400 rounded-full transition-all duration-300 ease-out"
                :style="{ width: `${loadingProgress}%` }"
              ></div>
            </div>
            <p class="text-[11px] font-mono text-amber-300/90 tracking-wide">
              {{ loadingStatus }}
            </p>
          </div>
        </div>
      </div>
    </Transition>

    <!-- First-run password setup (Only for Master Node) -->
    <div v-if="!isInitializing && !isLoggedIn && (needsPasswordSetup || setupCompleted) && runtimeIsMasterNode" class="fixed inset-0 z-50 flex items-center justify-center overflow-y-auto bg-gradient-to-br from-[#1c3323] via-[#0f1c13] to-[#050a07] p-4">
      <div class="my-6 w-full max-w-4xl space-y-6 rounded-3xl border border-slate-100 bg-white p-8 shadow-2xl">
        <div class="text-center space-y-2">
          <div class="mx-auto flex h-12 w-12 items-center justify-center rounded-2xl bg-amber-100 text-amber-800 text-xl font-bold shadow-inner">🔐</div>
          <h2 class="text-2xl font-extrabold text-slate-900">Penyiapan Kredensial Divisi</h2>
          <p class="text-xs font-medium text-slate-500">Ubah username dan atur password untuk setiap divisi demi keamanan privasi sebelum aplikasi digunakan.</p>
        </div>

        <form v-if="!setupCompleted" @submit.prevent="handlePasswordSetup" class="space-y-4">
          <div v-for="account in setupAccounts" :key="account.role" class="rounded-2xl border border-slate-200 bg-slate-50 p-4">
            <p class="mb-3 text-xs font-bold uppercase tracking-wider text-slate-700">{{ account.label }}</p>
            <div class="grid gap-3 sm:grid-cols-3">
              <input v-model="account.username" type="text" required placeholder="Username custom" class="w-full rounded-xl border border-slate-200 bg-white px-3.5 py-2.5 text-sm outline-none focus:border-emerald-500 transition-colors" />
              <input v-model="account.password" type="password" minlength="8" required autocomplete="new-password" placeholder="Password min 8 karakter" class="w-full rounded-xl border border-slate-200 bg-white px-3.5 py-2.5 text-sm outline-none focus:border-emerald-500 transition-colors" />
              <input v-model="account.confirm" type="password" minlength="8" required autocomplete="new-password" placeholder="Ulangi password" class="w-full rounded-xl border border-slate-200 bg-white px-3.5 py-2.5 text-sm outline-none focus:border-emerald-500 transition-colors" />
            </div>
          </div>

          <p class="rounded-xl border border-amber-200 bg-amber-50 p-3 text-xs text-amber-800">
            Password asli tidak disimpan di source code. Setelah disimpan, hanya hasil hash yang masuk ke database komputer ini.
          </p>
          <p v-if="setupError" class="rounded-xl border border-rose-200 bg-rose-50 p-3 text-xs font-semibold text-rose-700">{{ setupError }}</p>

          <button type="submit" :disabled="isSettingUpPasswords" class="w-full rounded-xl bg-emerald-800 py-3 text-sm font-bold text-white shadow-lg transition hover:bg-emerald-900 disabled:opacity-50 cursor-pointer">
            {{ isSettingUpPasswords ? 'Menyimpan Kredensial...' : 'Simpan Kredensial Divisi' }}
          </button>
        </form>

        <div v-else class="space-y-4 max-w-2xl mx-auto">
          <div class="rounded-2xl border border-emerald-300 bg-emerald-50 p-5 text-center space-y-3">
            <span class="text-3xl">🎉</span>
            <h3 class="text-sm font-bold text-emerald-900">Kredensial Divisi Berhasil Disimpan!</h3>
            <p class="text-xs text-emerald-700">Simpan <b>Kode Pemulihan Master</b> berikut di tempat yang aman. Kode ini wajib dicatat untuk memulihkan password jika suatu saat lupa:</p>
            
            <div class="rounded-xl bg-white border border-emerald-300 p-3 font-mono text-base font-extrabold text-emerald-950 tracking-wider select-all shadow-inner">
              {{ recoveryCode }}
            </div>
          </div>

          <button 
            type="button" 
            @click="needsPasswordSetup = false; setupCompleted = false;" 
            class="w-full rounded-xl bg-emerald-800 py-3 text-sm font-bold text-white shadow-lg transition hover:bg-emerald-950 cursor-pointer"
          >
            Lanjut ke Halaman Login ➔
          </button>
        </div>
      </div>
    </div>

    <!-- Login Gate Overlay -->
    <div v-else-if="!isInitializing && !isLoggedIn" class="fixed inset-0 z-50 flex items-center justify-center bg-gradient-to-br from-[#1c3323] via-[#0f1c13] to-[#050a07] p-4">
      <div class="bg-orange-100 p-8 rounded-3xl shadow-2xl w-full max-w-md border border-slate-100 space-y-6">
        <div class="text-center space-y-1 text-transparent">
            <img :src="logoUrl" alt="Logo OASIS" class="mx-auto mb-3 block h-48 w-48 object-contain" />
            🛡️
          <h2 class="text-2xl font-extrabold text-slate-900">OASIS Enterprise Hub</h2>
          <p class="text-xs text-slate-500 font-medium">Silakan masuk sesuai kredensial divisi Anda</p>
        </div>

        <form @submit.prevent="handleLogin" class="space-y-4">
          <div>
            <label class="block text-xs font-bold text-slate-700 mb-1">Username Divisi</label>
            <input 
              v-model="loginUsername" 
              type="text" 
              class="w-full px-4 py-2.5 border rounded-xl text-sm bg-slate-50 outline-none focus:ring-2 focus:ring-emerald-700 focus:bg-white transition" 
              placeholder="Masukkan username Anda..." 
            />
          </div>
          <div>
            <label class="block text-xs font-bold text-slate-700 mb-1">Password</label>
            <input 
              v-model="loginPassword" 
              type="password" 
              class="w-full px-4 py-2.5 border rounded-xl text-sm bg-slate-50 outline-none focus:ring-2 focus:ring-emerald-700 focus:bg-white transition" 
              placeholder="••••••••" 
            />
          </div>

          <p v-if="loginError" class="text-xs text-red-600 font-semibold bg-red-50 p-2.5 rounded-lg border border-red-100 text-center">
            {{ loginError }}
          </p>

          <button 
            type="submit" 
            :disabled="isLoggingIn" 
            class="w-full py-3 bg-emerald-800 hover:bg-emerald-900 text-white font-bold text-sm rounded-xl shadow-lg shadow-emerald-900/20 transition cursor-pointer disabled:opacity-50"
          >
            {{ isLoggingIn ? 'Memverifikasi Akses...' : 'Masuk Sistem' }}
          </button>
        </form>

        <button
          type="button"
          @click="resolveMaster(true)"
          class="w-full text-center text-[11px] font-mono text-slate-600 hover:text-slate-900 cursor-pointer"
          title="Klik untuk mencari ulang master"
        >
          {{ runtimeIsMasterNode ? 'MASTER' : 'CLIENT' }} ·
          {{ masterStatus.mode === 'connected' ? masterStatus.url : masterStatus.mode }}
        </button>

        <!-- Trigger Button untuk Modal Pemulihan -->
        <template v-if="runtimeIsMasterNode">
          <button 
            type="button" 
            @click="isRecoveryOpen = true; recoveryError = ''; recoveryMessage = ''" 
            class="w-full text-center text-xs font-semibold text-amber-700 hover:text-amber-900 cursor-pointer mt-2"
          >
            Semua password lupa? Gunakan kode pemulihan master
          </button>
        </template>
      </div>

      <!-- Dedicated Master Recovery Modal (Popup) -->
      <div v-if="isRecoveryOpen" class="fixed inset-0 z-[100] flex items-center justify-center bg-slate-950/70 p-4">
        <div class="w-full max-w-md rounded-3xl bg-white p-6 shadow-2xl space-y-4 border border-slate-100 relative">
          
          <!-- Tombol Tutup (X) -->
          <button 
            type="button" 
            @click="isRecoveryOpen = false" 
            class="absolute top-4 right-4 text-xl text-slate-400 hover:text-slate-700 cursor-pointer"
          >
            &times;
          </button>

          <div>
            <h3 class="text-lg font-extrabold text-slate-900">Pulihkan Seluruh Sistem (Master Reset)</h3>
            <p class="mt-1 text-xs text-slate-500">Gunakan kode pemulihan master yang diberikan saat pertama kali setup untuk reset darurat.</p>
          </div>

          <form @submit.prevent="recoverMaster" class="space-y-3">
            <div class="space-y-1">
              <label class="text-xs font-bold text-slate-700">Kode Pemulihan Master</label>
              <input v-model="recoveryInputCode" type="text" required placeholder="Masukkan kode pemulihan..." class="w-full rounded-xl border border-slate-200 px-3.5 py-2.5 text-sm uppercase outline-none focus:border-amber-500" />
            </div>
            <div class="space-y-1">
              <label class="text-xs font-bold text-slate-700">Password Baru untuk HR</label>
              <input v-model="recoveryNewPassword" type="password" minlength="8" required placeholder="Minimal 8 karakter..." class="w-full rounded-xl border border-slate-200 px-3.5 py-2.5 text-sm outline-none focus:border-amber-500" />
            </div>
            <div class="space-y-1">
              <label class="text-xs font-bold text-slate-700">Ulangi Password Baru</label>
              <input v-model="recoveryConfirmPassword" type="password" minlength="8" required placeholder="Ulangi password baru..." class="w-full rounded-xl border border-slate-200 px-3.5 py-2.5 text-sm outline-none focus:border-emerald-500" />
            </div>

            <p v-if="recoveryError" class="rounded-lg bg-rose-50 p-2 text-xs font-semibold text-rose-700">{{ recoveryError }}</p>
            <p v-if="recoveryMessage" class="rounded-lg bg-emerald-50 p-2 text-xs font-semibold text-emerald-700">{{ recoveryMessage }}</p>

            <div class="flex gap-2 pt-2">
              <button type="button" @click="isRecoveryOpen = false" class="flex-1 rounded-xl bg-slate-100 hover:bg-slate-200 transition py-2.5 text-sm font-bold text-slate-700 cursor-pointer">Batal</button>
              <button type="submit" :disabled="isRecovering" class="flex-1 rounded-xl bg-amber-700 hover:bg-amber-800 transition py-2.5 text-sm font-bold text-white cursor-pointer disabled:opacity-50">
                {{ isRecovering ? 'Memulihkan...' : 'Pulihkan Sistem' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>

    <!-- Main Application Canvas (Visible after login) -->
    <template v-if="!isInitializing && isLoggedIn">
      <AnomalyToast 
        :show="showAnomalyToast" 
        :count="warningEmployeesCount" 
        :details="anomalyDetails"
        @close="showAnomalyToast = false"
        @view-warning="filterWarningData"
      />

      <AppHeader @open-about="showAboutModal = true" />
      <AboutModal v-if="showAboutModal" @close="showAboutModal = false" />

      <!-- Role-Based Navigation Header -->
      <div class="bg-emerald-900 text-white px-6 py-2 flex items-center justify-between text-xs font-medium border-b border-emerald-950 shadow-sm">
        <div class="flex items-center space-x-2">
          <span class="px-2 py-0.5 rounded bg-emerald-800 border border-emerald-700 uppercase font-mono font-bold tracking-wider">
            Divisi: {{ userRole }}
          </span>
          <span class="px-2 py-0.5 rounded bg-emerald-950/60 border border-emerald-700 font-mono text-[10px]">
            {{ runtimeIsMasterNode ? 'MASTER' : 'CLIENT' }} ·
            {{ masterStatus.mode === 'connected' ? masterStatus.url : masterStatus.mode }}
          </span>
          <span class="text-emerald-200/80">| Sistem Terintegrasi Apartemen Oasis Mitra Sarana</span>
        </div>
        <div class="flex items-center gap-2">
          <template v-if="userRole === 'hr' && appConfig.isAdminBuild">
            <span v-if="securityDataSyncMessage" class="max-w-xs truncate text-emerald-100" :title="securityDataSyncMessage">
              {{ securityDataSyncMessage }}
            </span>
            <button
              @click="syncSecurityDatabaseFromHr"
              :disabled="isSyncingSecurityData"
              class="px-3 py-1 rounded-lg bg-blue-500/90 hover:bg-blue-500 text-white font-bold transition shadow-sm cursor-pointer disabled:opacity-50"
            >
              {{ isSyncingSecurityData ? 'Menyinkronkan...' : 'Sync Database Security' }}
            </button>
          </template>
          
          <button
            @click="openPasswordModal"
            class="px-3 py-1 bg-amber-500/90 hover:bg-amber-500 text-white rounded-lg font-bold transition shadow-sm cursor-pointer"
          >
            {{ userRole === 'hr' ? 'Kelola Akun Divisi' : 'Pengaturan Akun' }}
          </button>

          <button 
            @click="logout" 
            class="px-3 py-1 bg-red-600/80 hover:bg-red-600 text-white rounded-lg font-bold transition shadow-sm cursor-pointer"
          >
            Keluar (Logout)
          </button>
        </div>
      </div>

      <!-- PASSWORD MODAL: HR (Admin Panel with Dropdown) vs Non-HR (Self-Service Profile/Username/Password) -->
      <div v-if="isPasswordModalOpen" class="fixed inset-0 z-[100] flex items-center justify-center bg-slate-950/60 p-4">
        <form @submit.prevent="savePasswordChange" class="w-full max-w-md space-y-4 rounded-2xl bg-white p-6 shadow-2xl">
          <div>
            <h2 class="text-lg font-extrabold text-slate-900">
              {{ userRole === 'hr' ? 'Kelola Akun & Reset Divisi' : 'Pengaturan Akun & Keamanan' }}
            </h2>
            <p class="mt-1 text-xs text-slate-500">
              {{ userRole === 'hr' ? 'Pilih divisi untuk mengedit username atau mereset passwordnya.' : 'Perbarui username custom dan password untuk privasi divisi Anda.' }}
            </p>
          </div>

          <div v-if="userRole === 'hr'" class="space-y-1">
            <label class="text-xs font-bold text-slate-700">Pilih Divisi Target</label>
            <select v-model="passwordTargetRole" class="w-full rounded-xl border border-slate-200 px-3 py-2.5 text-sm bg-white outline-none focus:border-emerald-500">
              <option value="hr">HR Admin — {{ usernameOf('hr') }}</option>
              <option value="security">Chief Security — {{ usernameOf('security') }}</option>
              <option value="engineering">Chief Engineering — {{ usernameOf('engineering') }}</option>
              <option value="housekeeping">Chief Housekeeping — {{ usernameOf('housekeeping') }}</option>
            </select>
          </div>

          <div class="space-y-1">
            <label class="text-xs font-bold text-slate-700">Username Custom</label>
            <input v-model="passwordNewUsername" type="text" required placeholder="Masukkan username baru..." class="w-full rounded-xl border border-slate-200 px-3 py-2.5 text-sm outline-none focus:border-emerald-500" />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-bold text-slate-700">
              {{ isEditingOtherDivision() ? 'Password HR Anda (Otorisasi)' : 'Password Lama Anda' }}
            </label>
            <input v-model="passwordCurrent" type="password" required autocomplete="current-password" placeholder="Masukkan password saat ini..." class="w-full rounded-xl border border-slate-200 px-3 py-2.5 text-sm outline-none focus:border-emerald-500" />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-bold text-slate-700">Password Baru</label>
            <input v-model="passwordNew" type="password" autocomplete="new-password" minlength="8" required placeholder="Minimal 8 karakter" class="w-full rounded-xl border border-slate-200 px-3 py-2.5 text-sm outline-none focus:border-emerald-500" />
          </div>

          <div class="space-y-1">
            <label class="text-xs font-bold text-slate-700">Ulangi Password Baru</label>
            <input v-model="passwordConfirm" type="password" autocomplete="new-password" minlength="8" required placeholder="Ulangi password baru" class="w-full rounded-xl border border-slate-200 px-3 py-2.5 text-sm outline-none focus:border-emerald-500" />
          </div>

          <p v-if="passwordError" class="rounded-lg bg-rose-50 p-2.5 text-xs font-semibold text-rose-700">{{ passwordError }}</p>
          <p v-if="passwordMessage" class="rounded-lg bg-emerald-50 p-2.5 text-xs font-semibold text-emerald-700">{{ passwordMessage }}</p>
          
          <div class="flex gap-2 mt-4">
            <button type="button" @click="isPasswordModalOpen = false" class="flex-1 rounded-xl bg-slate-100 hover:bg-slate-200 transition py-2.5 text-sm font-bold text-slate-700 cursor-pointer">Tutup</button>
            <button type="submit" :disabled="isSavingPassword" class="flex-1 rounded-xl bg-emerald-800 hover:bg-emerald-900 transition py-2.5 text-sm font-bold text-white cursor-pointer disabled:opacity-50">{{ isSavingPassword ? 'Menyimpan...' : 'Simpan Perubahan' }}</button>
          </div>
        </form>
      </div>

      <main class="mx-auto max-w-7xl space-y-6 px-6 py-8">
        <!-- HR Navigation & Views -->
        <template v-if="userRole === 'hr'">
          <AppNavigation :active-tab="activeTab as Tab" :show-security-admin="false" @select-tab="switchTab" />

          <DashboardView 
            v-if="activeTab === 'dashboard'" 
            :refresh-key="refreshKey" 
            @import="isImportModalOpen = true" 
            @add="openCreate" 
            @open-master="openMaster" 
            @open-master-with-filter="(q: string) => { masterSearchQuery = q; activeTab = 'master'; }" 
            @select-metric="(k: MetricFilterKey) => { activeMetricKey = k; activeTab = 'master'; }" 
          />

          <EmployeeTable 
            v-if="activeTab === 'master'" 
            :refresh-key="refreshKey" 
            :initial-search="masterSearchQuery" 
            :active-metric-key="activeMetricKey || undefined" 
            @reset-search="masterSearchQuery = ''" 
            @edit="openEdit" 
            @refresh="handleSaved"
            @clear-metric-filter="activeMetricKey = null" 
          />

          <AbsensiView v-if="activeTab === 'absensi'" division="hr" />
          <RekapCutiView v-if="activeTab === 'cuti'" />
        </template>

        <!-- Security Navigation & Views -->
        <template v-if="userRole === 'security'">
          <div class="flex space-x-2 border-b border-emerald-900/20 pb-4">
            <button 
              @click="activeTab = 'security_inspection'" 
              :class="activeTab === 'security_inspection' ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-md' : 'bg-white text-slate-700 hover:bg-slate-100'"
              class="px-4 py-2 rounded-xl text-xs font-bold transition cursor-pointer border border-slate-200"
            >
              🚨 Dasbor Inspeksi Hydrant
            </button>
            <button 
              @click="activeTab = 'security_absensi'" 
              :class="activeTab === 'security_absensi' ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-md' : 'bg-white text-slate-700 hover:bg-slate-100'"
              class="px-4 py-2 rounded-xl text-xs font-bold transition cursor-pointer border border-slate-200"
            >
              📅 Rekapitulasi Absensi Security
            </button>
            <button 
              @click="activeTab = 'qr_generator'" 
              :class="activeTab === 'qr_generator' ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-md' : 'bg-white text-slate-700 hover:bg-slate-100'"
              class="px-4 py-2 rounded-xl text-xs font-bold transition cursor-pointer border border-slate-200"
            >
              🖨️ Cetak QR Code Asset
            </button>
          </div>
        
          <SecurityDashboardView v-if="activeTab === 'security_inspection'" :can-manage-pins="true" />
          <AbsensiView v-if="activeTab === 'security_absensi'" division="security" />
          <QrGeneratorView v-if="activeTab === 'qr_generator'" />
        </template>

        <!-- Engineering Navigation & Views -->
        <template v-if="userRole === 'engineering'">
          <div class="flex space-x-2 border-b border-emerald-900/20 pb-4">
            <button 
              @click="activeTab = 'engineering_absensi'" 
              :class="activeTab === 'engineering_absensi' ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-md' : 'bg-white text-slate-700 hover:bg-slate-100'"
              class="px-4 py-2 rounded-xl text-xs font-bold transition cursor-pointer border border-slate-200"
            >
              📅 Rekapitulasi Absensi Engineering
            </button>
          </div>

          <AbsensiView v-if="activeTab === 'engineering_absensi'" division="engineering" />
        </template>

        <!-- Housekeeping Navigation & Views -->
        <template v-if="userRole === 'housekeeping'">
          <div class="flex space-x-2 border-b border-emerald-900/20 pb-4">
            <button 
              @click="activeTab = 'housekeeping_absensi'" 
              :class="activeTab === 'housekeeping_absensi' ? 'bg-gradient-to-r from-emerald-600 to-teal-600 text-white shadow-md' : 'bg-white text-slate-700 hover:bg-slate-100'"
              class="px-4 py-2 rounded-xl text-xs font-bold transition cursor-pointer border border-slate-200"
            >
              📅 Rekapitulasi Absensi Housekeeping
            </button>
          </div>

          <AbsensiView v-if="activeTab === 'housekeeping_absensi'" division="housekeeping" />
        </template>
      </main>

      <EmployeeModal 
        :is-open="isModalOpen" 
        :employee-to-edit="employeeToEdit"
        @close="isModalOpen = false"
        @saved="handleSaved"
      />

      <Teleport to="body">
        <div v-if="isImportModalOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/60 p-4" @click.self="isImportModalOpen = false">
          <div class="max-h-[92vh] w-full max-w-6xl overflow-auto rounded-3xl bg-slate-50 p-6 shadow-2xl space-y-4">
            <div class="flex items-center justify-between border-b pb-4">
              <h2 class="text-lg font-bold text-slate-900">Import Data Karyawan</h2>
              <button type="button" class="text-xl text-slate-400 hover:text-slate-700 cursor-pointer" @click="isImportModalOpen = false">&times;</button>
            </div>
            <ExcelImporter @saved="imported" />
          </div>
        </div>
      </Teleport>
    </template>
  </div>
</template>

<style scoped>
/* Smooth Splash Screen Fade-out Transition */
.splash-fade-leave-active {
  transition: opacity 0.4s ease-in-out, transform 0.4s ease-in-out;
}
.splash-fade-leave-to {
  opacity: 0;
  transform: scale(0.97);
}
</style>