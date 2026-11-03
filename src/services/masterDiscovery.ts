import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { fetch as tauriFetch } from '@tauri-apps/plugin-http';

const CONNECT_TIMEOUT_MS = 3000;
const NOT_FOUND_COOLDOWN_MS = 15000;

export type MasterMode = 'searching' | 'connected' | 'offline';
export const masterStatus = ref<{ mode: MasterMode; url: string }>({ mode: 'searching', url: '' });

const log = (...args: unknown[]) => console.info('[master-discovery]', ...args);

let cachedBaseUrl = '';
let lastFailedAt = 0;
let inFlight: Promise<string | null> | null = null;

async function isHealthy(baseUrl: string): Promise<boolean> {
  try {
    const res = await tauriFetch(`${baseUrl}/api/health`, { connectTimeout: CONNECT_TIMEOUT_MS });
    return res.ok;
  } catch (error) {
    console.warn('[master-discovery] health check failed:', baseUrl, error);
    return false;
  }
}

async function discover(): Promise<string | null> {
  masterStatus.value = { mode: 'searching', url: '' };
  log('scanning network (up to ~4s)...');
  try {
    const candidates = await invoke<string[]>('discover_master');
    log('mDNS candidates:', candidates);
    for (const candidate of candidates) {
      const base = candidate.replace(/\/+$/, '');
      const healthy = await isHealthy(base);
      log('health check', base, healthy ? 'OK' : 'NO ANSWER');
      if (healthy) {
        cachedBaseUrl = base;
        lastFailedAt = 0;
        masterStatus.value = { mode: 'connected', url: base };
        return base;
      }
    }
  } catch (error) {
    console.warn('[master-discovery] discover_master failed:', error);
  }
  lastFailedAt = Date.now();
  masterStatus.value = { mode: 'offline', url: '' };
  log('NO MASTER FOUND (next automatic scan allowed in 15s)');
  return null;
}

export async function resolveMaster(force = false): Promise<string | null> {
  if (cachedBaseUrl && !force && (await isHealthy(cachedBaseUrl))) {
    return cachedBaseUrl;
  }
  if (cachedBaseUrl) log('cached master no longer answers:', cachedBaseUrl);
  cachedBaseUrl = '';

  if (!force && lastFailedAt && Date.now() - lastFailedAt < NOT_FOUND_COOLDOWN_MS) {
    log('skipped scan: failed recently (cooldown)');
    masterStatus.value = { mode: 'offline', url: '' };
    return null;
  }

  if (!inFlight) {
    inFlight = discover().finally(() => {
      inFlight = null;
    });
  } else {
    log('scan already running, waiting for it');
  }
  return inFlight;
}

export function forgetMaster() {
  log('forgetMaster() called, cache cleared');
  cachedBaseUrl = '';
  masterStatus.value = { mode: 'offline', url: '' };
}