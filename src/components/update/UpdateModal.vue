<template>
  <div>
    <!-- Trigger Button (e.g. in Settings or App Header) -->
    <button @click="checkForUpdates(true)" :disabled="checking" class="btn-check">
      <span v-if="checking">Checking for updates...</span>
      <span v-else>Check for Updates</span>
    </button>

    <!-- Update Modal -->
    <div v-if="showModal" class="modal-overlay">
      <div class="modal-card">
        <h3>Update Available!</h3>
        <p class="version-info">
          Version <strong>{{ updateInfo?.version }}</strong> is now available (Current: {{ currentVersion }}).
        </p>

        <!-- Release Notes -->
        <div class="release-notes" v-if="updateInfo?.body">
          <h4>Release Notes:</h4>
          <p>{{ updateInfo.body }}</p>
        </div>

        <!-- Download & Install Progress Bar -->
        <div v-if="downloading" class="progress-container">
          <p v-if="totalBytes > 0">
            Downloading: {{ Math.round((downloadedBytes / totalBytes) * 100) }}% 
            ({{ (downloadedBytes / 1024 / 1024).toFixed(1) }} MB / {{ (totalBytes / 1024 / 1024).toFixed(1) }} MB)
          </p>
          <p v-else>Preparing download...</p>
          <progress :value="downloadedBytes" :max="totalBytes || 100"></progress>
        </div>

        <!-- Action Buttons -->
        <div class="modal-actions" v-if="!downloading">
          <button @click="startUpdate" class="btn-primary">Update Now</button>
          <button @click="closeModal" class="btn-secondary">Later</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

const checking = ref(false);
const showModal = ref(false);
const updateInfo = ref(null);
const downloading = ref(false);
const downloadedBytes = ref(0);
const totalBytes = ref(0);
const currentVersion = ref('1.0.0'); // Replace or load dynamically if needed

// Main Update Check Function
async function checkForUpdates(manualCheck = false) {
  checking.value = true;
  try {
    const update = await check(); // Checks endpoint defined in tauri.conf.json
    
    if (update) {
      updateInfo.value = update;
      showModal.value = true;
    } else if (manualCheck) {
      alert('You are already running the latest version of Pendataan Karyawan OASIS.');
    }
  } catch (error) {
    console.error('Failed to check for updates:', error);
    if (manualCheck) {
      alert('Failed to check for updates. Please check your internet connection.');
    }
  } finally {
    checking.value = false;
  }
}

// Start Download and Installation
async function startUpdate() {
  if (!updateInfo.value) return;
  
  downloading.value = true;
  downloadedBytes.value = 0;
  totalBytes.value = 0;

  try {
    // Tauri v2 update download and install with progress listener callback
    await updateInfo.value.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          totalBytes.value = event.data.contentLength || 0;
          break;
        case 'Progress':
          downloadedBytes.value += event.data.chunkLength;
          break;
        case 'Finished':
          console.log('Download finished. Installing...');
          break;
      }
    });

    // Automatically restart app to apply update
    await relaunch();
  } catch (error) {
    console.error('Update failed:', error);
    alert(`Update failed: ${error}`);
    downloading.value = false;
  }
}

function closeModal() {
  showModal.value = false;
}

// Automated check on startup
onMounted(() => {
  checkForUpdates(false);
});
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.modal-card {
  background: #fff;
  padding: 24px;
  border-radius: 8px;
  max-width: 450px;
  width: 100%;
  box-shadow: 0 4px 12px rgba(0,0,0,0.15);
}
.release-notes {
  background: #f4f5f7;
  padding: 12px;
  border-radius: 4px;
  max-height: 150px;
  overflow-y: auto;
  margin: 12px 0;
}
.progress-container {
  margin: 16px 0;
}
progress {
  width: 100%;
  height: 12px;
}
.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>