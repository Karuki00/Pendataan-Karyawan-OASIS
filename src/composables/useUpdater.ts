import { ref } from 'vue';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export function useUpdater() {
  const isChecking = ref(false);
  const isDownloading = ref(false);
  const updateAvailable = ref(false);
  const downloadProgress = ref(0);
  const newVersion = ref('');

  async function checkForUpdates(silent = true) {
    isChecking.value = true;
    try {
      const update = await check();
      if (update?.available) {
        updateAvailable.value = true;
        newVersion.value = update.version;

        if (!silent) {
          const confirmUpdate = window.confirm(
            `Versi baru (${update.version}) telah tersedia!\n\nApakah Anda ingin mengunduh dan memasang pembaruan sekarang?`
          );

          if (confirmUpdate) {
            await downloadAndInstallUpdate(update);
          }
        }
      } else if (!silent) {
        alert('Aplikasi sudah menggunakan versi terbaru.');
      }
    } catch (err) {
      console.error('Gagal memeriksa pembaruan:', err);
      if (!silent) {
        alert(`Gagal memeriksa pembaruan: ${err}`);
      }
    } finally {
      isChecking.value = false;
    }
  }

  async function downloadAndInstallUpdate(updateObj?: any) {
    try {
      const activeUpdate = updateObj || (await check());
      if (!activeUpdate?.available) return;

      isDownloading.value = true;
      downloadProgress.value = 0;

      let downloadedBytes = 0;
      let totalBytes = 0;

      await activeUpdate.downloadAndInstall((event: any) => {
        switch (event.event) {
          case 'Started':
            totalBytes = event.data.contentLength || 0;
            break;
          case 'Progress':
            downloadedBytes += event.data.chunkLength || 0;
            if (totalBytes > 0) {
              downloadProgress.value = Math.min(
                100,
                Math.round((downloadedBytes / totalBytes) * 100)
              );
            }
            break;
          case 'Finished':
            downloadProgress.value = 100;
            break;
        }
      });

      alert('Pembaruan berhasil diunduh! Aplikasi akan dimuat ulang.');
      await relaunch();
    } catch (err) {
      console.error('Gagal memasang pembaruan:', err);
      alert(`Gagal memasang pembaruan: ${err}`);
    } finally {
      isDownloading.value = false;
    }
  }

  return {
    isChecking,
    isDownloading,
    updateAvailable,
    downloadProgress,
    newVersion,
    checkForUpdates,
    downloadAndInstallUpdate,
  };
}