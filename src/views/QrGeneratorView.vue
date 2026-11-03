<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import QRCode from 'qrcode';
import QrCard from '../sidecomponents/QrCard.vue';
import { appConfig } from '../config';

const WEB_APP_BASE_URL = ref(appConfig.appsScriptUrl);
const SECURITY_TOKEN = ref(appConfig.inspectionAccessToken);

// Filters & Search
const selectedCategory = ref('ALL');
const searchQuery = ref('');

// Pagination Controls
const currentPage = ref(1);
const itemsPerPage = ref(6);
const printAllPages = ref(false);
const isPreparingPrint = ref(false);

interface QrItem {
  code: string;
  location: string;
  category: string;
  qrDataUrl?: string;
}

const assets = ref<QrItem[]>([]);

function generateAllAssets(): QrItem[] {
  const list: QrItem[] = [];

  const hldSpecialLocations: Record<number, string> = {
    1: 'Pos belakang', 2: 'Cucian mobil', 3: 'Parkir motor lt. dsr TC',
    4: 'Pos depan / kiri', 5: 'Pos depan / kanan', 6: 'Halte lt. dsr TA',
    7: 'Lapangan tenis TA', 8: 'Toilet lapangan tenis', 9: 'Pos anjungan', 10: 'Kolam renang'
  };
  for (let i = 1; i <= 10; i++) {
    list.push({ code: `HLD-${i}`, location: hldSpecialLocations[i], category: 'Lantai Dasar Special' });
  }

  const hldStandardLocations: Record<number, string> = {
    11: 'Ruang Kasuari TA', 12: 'Pos jaga Lobby TA', 13: 'Pos jaga Lobby TB',
    14: 'Ruang Fitnes', 15: 'Pos jaga Lobby TC', 16: 'Ruang Serbaguna TC'
  };
  for (let i = 11; i <= 16; i++) {
    list.push({ code: `HLD-${i}`, location: hldStandardLocations[i], category: 'Lantai Dasar Standard' });
  }

  const towerFloors = [2, 3, 5, 6, 7, 8, 9, 10, 11, 12, '12A', 14, 15, 16, 17, 18, 19, 20, 21, 22, 23];
  const towers = [
    { prefix: 'HTA', name: 'Tower A', category: 'Tower A' },
    { prefix: 'HTB', name: 'Tower B', category: 'Tower B' },
    { prefix: 'HTC', name: 'Tower C', category: 'Tower C' }
  ];

  for (const tower of towers) {
    for (const fl of towerFloors) {
      for (const suffix of ['A', 'B']) {
        list.push({
          code: `${tower.prefix}-${fl}${suffix}`,
          location: `${tower.name} Lt.${fl} Slot ${suffix}`,
          category: tower.category
        });
      }
    }
  }

  const hb1Locations: Record<number, string> = {
    1: 'B1A pintu lobby lift', 2: 'B1A lot parkir no.341', 3: 'B1A lot parkir no.378',
    4: 'B1A lot parkir no.281', 5: 'B1A lot parkir no.042', 6: 'B1B lot parkir no.229',
    7: 'B1B lot parkir no.310', 8: 'B1B lot parkir no.343', 9: 'B1B lobby lift',
    10: 'B1B lot parkir no.158', 11: 'B1C lot parkir no.074', 12: 'B1C lot parkir no.009',
    13: 'B1C Musholla', 14: 'B1C pintu lobby lift', 15: 'B1C r. tunggu supir',
    16: 'B1C arah klm renang', 17: 'B1C Kontrol room'
  };
  for (let i = 1; i <= 17; i++) {
    list.push({ code: `HB1-${i}`, location: hb1Locations[i] || `Basement 1 Slot ${i}`, category: 'Basement 1' });
  }

  for (let i = 1; i <= 13; i++) {
    list.push({ code: `HB2-${i}`, location: `Basement 2 Slot ${i}`, category: 'Basement 2' });
  }

  return list;
}

const filteredAssets = computed(() => {
  return assets.value.filter(item => {
    const matchesCategory = selectedCategory.value === 'ALL' || item.category === selectedCategory.value;
    const matchesSearch = item.code.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
                          item.location.toLowerCase().includes(searchQuery.value.toLowerCase());
    return matchesCategory && matchesSearch;
  });
});

const totalPages = computed(() => {
  return Math.max(1, Math.ceil(filteredAssets.value.length / itemsPerPage.value));
});

const displayedAssets = computed(() => {
  if (printAllPages.value) return filteredAssets.value;
  const start = (currentPage.value - 1) * itemsPerPage.value;
  return filteredAssets.value.slice(start, start + itemsPerPage.value);
});

watch([selectedCategory, searchQuery, itemsPerPage], () => {
  currentPage.value = 1;
});

watch([WEB_APP_BASE_URL, SECURITY_TOKEN], () => {
  if (assets.value.length) void regenerateQrCodes();
});

function getFormUrl(code: string): string {
  // Browser account routes such as /macros/u/4/s/... are not portable.
  // Always generate the public web-app URL without the account index.
  const baseUrl = WEB_APP_BASE_URL.value.trim().replace(/[?&]+$/, '');
  return `${baseUrl}?token=${encodeURIComponent(SECURITY_TOKEN.value)}&asset=${encodeURIComponent(code)}`;
}

async function generateQrDataUrl(code: string): Promise<string> {
  return QRCode.toDataURL(getFormUrl(code), {
    width: 500,
    margin: 4,
    errorCorrectionLevel: 'M',
    color: { dark: '#0f172a', light: '#ffffff' },
  });
}

async function regenerateQrCodes() {
  const generated = generateAllAssets();
  assets.value = await Promise.all(generated.map(async (item) => ({
    ...item,
    qrDataUrl: await generateQrDataUrl(item.code),
  })));
}

async function resetQrConfiguration() {
  WEB_APP_BASE_URL.value = appConfig.appsScriptUrl;
  SECURITY_TOKEN.value = appConfig.inspectionAccessToken;
  await regenerateQrCodes();
}

async function prepareImagesForPrint() {
  const images = document.querySelectorAll<HTMLImageElement>('.qr-image-el');
  const promises = Array.from(images).map(img => {
    return new Promise<void>((resolve) => {
      if (img.complete && img.naturalWidth !== 0) {
        resolve();
      } else {
        img.onload = () => resolve();
        img.onerror = () => resolve();
      }
    });
  });
  await Promise.all(promises);
}

async function handlePrintCurrentPage() {
  isPreparingPrint.value = true;
  printAllPages.value = false;
  await prepareImagesForPrint();
  isPreparingPrint.value = false;
  window.print();
}

async function handlePrintAllPages() {
  isPreparingPrint.value = true;
  printAllPages.value = true;
  setTimeout(async () => {
    await prepareImagesForPrint();
    isPreparingPrint.value = false;
    window.print();
    printAllPages.value = false;
  }, 300);
}

onMounted(() => {
  void regenerateQrCodes();
});
</script>

<template>
  <div class="p-6 space-y-6 bg-slate-50 min-h-screen print:p-0 print:bg-white">
    
    <!-- Controls Header -->
    <div class="print:hidden flex flex-col lg:flex-row lg:items-center justify-between gap-4 bg-white/90 backdrop-blur-md p-5 rounded-2xl border border-slate-200/80 shadow-sm">
      <div>
        <h2 class="text-xl font-bold tracking-tight text-slate-900">Generator & Cetak QR Code Hydrant</h2>
        <p class="text-xs text-slate-500 mt-0.5">
          Menampilkan {{ displayedAssets.length }} dari {{ filteredAssets.length }} Box Hydrant
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-3">
        <input 
          v-model="searchQuery"
          type="text"
          placeholder="Cari kode (mis. HTA-12A)..."
          class="px-3 py-2.5 text-xs border border-slate-200 rounded-xl w-44 focus:outline-none focus:ring-2 focus:ring-emerald-500/20 shadow-sm transition-shadow"
        />

        <select 
          v-model="selectedCategory" 
          class="px-3 py-2.5 text-xs border border-slate-200 rounded-xl bg-white focus:outline-none focus:ring-2 focus:ring-emerald-500/20 shadow-sm cursor-pointer"
        >
          <option value="ALL">Semua Kategori ({{ assets.length }})</option>
          <option value="Lantai Dasar Special">Lantai Dasar Special (HLD 1-10)</option>
          <option value="Lantai Dasar Standard">Lantai Dasar Standard (HLD 11-16)</option>
          <option value="Tower A">Tower A (HTA)</option>
          <option value="Tower B">Tower B (HTB)</option>
          <option value="Tower C">Tower C (HTC)</option>
          <option value="Basement 1">Basement 1 (HB1)</option>
          <option value="Basement 2">Basement 2 (HB2)</option>
        </select>

        <select 
          v-model="itemsPerPage" 
          class="px-3 py-2.5 text-xs border border-slate-200 rounded-xl bg-white font-semibold shadow-sm cursor-pointer"
        >
          <option :value="6">6 per Hal (Besar)</option>
          <option :value="12">12 per Hal (Standar)</option>
          <option :value="15">15 per Hal (Kompak)</option>
        </select>

        <div class="flex items-center space-x-2 border-l border-slate-200 pl-3">
          <button 
            @click="handlePrintCurrentPage"
            :disabled="isPreparingPrint"
            class="px-4 py-2.5 bg-emerald-50 hover:bg-emerald-600 border border-emerald-600/30 text-emerald-700 hover:text-white text-xs font-semibold rounded-xl shadow-sm transition-all active:scale-95 cursor-pointer disabled:opacity-50"
          >
            <span>{{ isPreparingPrint ? 'Menyiapkan...' : 'Cetak Hal Ini 🖨️' }}</span>
          </button>
          <button 
            @click="handlePrintAllPages"
            :disabled="isPreparingPrint"
            class="px-4 py-2.5 bg-gradient-to-r from-amber-600 to-orange-600 hover:from-amber-700 hover:to-orange-700 text-white text-xs font-semibold rounded-xl shadow-md shadow-amber-600/20 transition-all active:scale-95 cursor-pointer disabled:opacity-50"
          >
            Cetak Semua ({{ filteredAssets.length }})
          </button>
        </div>
      </div>
    </div>

    <!-- Web App Config Bar -->
    <div class="print:hidden bg-amber-50 border border-amber-200 rounded-xl p-4 flex items-end justify-between text-xs text-amber-900 space-x-4 shadow-sm">
      <div class="flex-1 space-y-1">
        <label class="font-bold block">Base Web App URL (Google Apps Script)</label>
        <input 
          v-model="WEB_APP_BASE_URL" 
          type="text" 
          class="w-full px-3 py-2 bg-white border border-amber-300 rounded-lg font-mono text-[11px] shadow-sm"
        />
      </div>
      <div class="w-48 space-y-1">
        <label class="font-bold block">Token Keamanan</label>
        <input 
          v-model="SECURITY_TOKEN" 
          type="text" 
          class="w-full px-3 py-2 bg-white border border-amber-300 rounded-lg font-mono text-[11px] shadow-sm"
        />
      </div>
      <button
        type="button"
        @click="resetQrConfiguration"
        class="shrink-0 px-3 py-2 bg-white border border-amber-300 rounded-lg font-bold text-amber-800 hover:bg-amber-100 transition-colors"
      >
        Reset URL &amp; QR
      </button>
    </div>

    <!-- Modular Cards Grid -->
    <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-6 print:grid-cols-2 print:gap-4">
      <QrCard 
        v-for="item in displayedAssets" 
        :key="item.code" 
        :item="item" 
        :qr-image-url="item.qrDataUrl || ''"
      />
    </div>

    <!-- Sleek Pagination Footer -->
    <div v-if="totalPages > 1" class="print:hidden flex items-center justify-between px-5 py-3 bg-white/90 backdrop-blur-md border border-slate-200/80 shadow-sm rounded-xl">
      <button 
        @click="currentPage--"
        :disabled="currentPage === 1"
        class="px-4 py-2 text-xs font-bold text-slate-600 bg-white border border-slate-200 rounded-xl shadow-sm hover:bg-slate-50 disabled:opacity-50 disabled:cursor-not-allowed transition-all active:scale-95 cursor-pointer"
      >
        &larr; Prev
      </button>
      
      <span class="text-xs font-bold text-slate-500 tracking-wide">
        Halaman <span class="text-emerald-700">{{ currentPage }}</span> dari {{ totalPages }}
      </span>
      
      <button 
        @click="currentPage++"
        :disabled="currentPage === totalPages"
        class="px-4 py-2 text-xs font-bold text-slate-600 bg-white border border-slate-200 rounded-xl shadow-sm hover:bg-slate-50 disabled:opacity-50 disabled:cursor-not-allowed transition-all active:scale-95 cursor-pointer"
      >
        Next &rarr;
      </button>
    </div>

  </div>
</template>

<style scoped>
@media print {
  @page {
    margin: 10mm;
  }

  body {
    background: white !important;
  }

  .qr-card {
    break-inside: avoid;
    page-break-inside: avoid;
  }
}
</style>
