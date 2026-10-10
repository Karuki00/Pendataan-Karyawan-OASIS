import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export function useUpdater() {
  async function checkForUpdates() {
    try {
      const update = await check();
      if (update) {
        console.log(`Update ditemukan: ${update.version}`);
        const yes = confirm(
          `Versi baru v${update.version} telah tersedia!\n\nApakah Anda ingin memperbarui aplikasi sekarang?`
        );
        if (yes) {
          let downloaded = 0;
          let contentLength = 0;

          await update.downloadAndInstall((event) => {
            switch (event.event) {
              case 'Started':
                contentLength = event.data.contentLength || 0;
                break;
              case 'Progress':
                downloaded += event.data.chunkLength;
                break;
              case 'Finished':
                break;
            }
          });

          await relaunch();
        }
      }
    } catch (error) {
      console.error('Gagal memeriksa update:', error);
    }
  }

  return { checkForUpdates };
}