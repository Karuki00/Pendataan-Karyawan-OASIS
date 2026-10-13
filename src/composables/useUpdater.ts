import { ref } from 'vue';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export function useUpdater() {
  const isChecking = ref(false);
  const isDownloading = ref(false);
  const updateAvailable = ref(false);
  const downloadProgress = ref(0);
  const newVersion = ref('');

  /**
   * Checks for available updates.
   * If autoApply is true, it immediately downloads, installs, and restarts the app.
   */
  async function checkForUpdates(
    autoApply = true,
    onProgress?: (progress: number, statusText: string) => void
  ): Promise<boolean> {
    isChecking.value = true;
    try {
      const update = await check();
      if (update?.available) {
        updateAvailable.value = true;
        newVersion.value = update.version;

        if (autoApply) {
          await downloadAndInstallUpdate(update, onProgress);
          return true;
        }
      }
      return false;
    } catch (err) {
      console.error('Gagal memeriksa pembaruan:', err);
      return false;
    } finally {
      isChecking.value = false;
    }
  }

  /**
   * Downloads and applies the update with live progress updates.
   */
  async function downloadAndInstallUpdate(
    updateObj?: Update | null,
    onProgress?: (progress: number, statusText: string) => void
  ) {
    try {
      const activeUpdate = updateObj || (await check());
      if (!activeUpdate?.available) return;

      isDownloading.value = true;
      downloadProgress.value = 0;

      let downloadedBytes = 0;
      let totalBytes = 0;

      if (onProgress) {
        onProgress(0, `Mengunduh pembaruan v${activeUpdate.version}...`);
      }

      await activeUpdate.downloadAndInstall((event: any) => {
        switch (event.event) {
          case 'Started':
            totalBytes = event.data.contentLength || 0;
            break;
          case 'Progress':
            downloadedBytes += event.data.chunkLength || 0;
            if (totalBytes > 0) {
              const pct = Math.min(
                100,
                Math.round((downloadedBytes / totalBytes) * 100)
              );
              downloadProgress.value = pct;
              if (onProgress) {
                onProgress(
                  pct,
                  `Mengunduh pembaruan v${activeUpdate.version} (${pct}%)...`
                );
              }
            }
            break;
          case 'Finished':
            downloadProgress.value = 100;
            if (onProgress) {
              onProgress(100, 'Memasang pembaruan & memuat ulang...');
            }
            break;
        }
      });

      // Automatically relaunch app after installation
      await relaunch();
    } catch (err) {
      console.error('Gagal memasang pembaruan:', err);
      throw err;
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