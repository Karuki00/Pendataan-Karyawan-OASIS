import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { resolveMaster, forgetMaster } from './masterDiscovery';

let sessionToken = '';

const CONNECT_TIMEOUT_MS = 3000;
const OFFLINE_MESSAGE =
  'Komputer HR tidak ditemukan di jaringan ini. ' +
  'Pastikan komputer HR menyala dan terhubung ke jaringan yang sama.';

async function request(path: string, body?: unknown) {
  for (let attempt = 0; attempt < 2; attempt++) {
    // Second attempt forces a fresh network scan, ignoring the "not found" cooldown.
    const baseUrl = await resolveMaster(attempt > 0);
    if (!baseUrl) continue;

    let response: Response;
    try {
      response = await tauriFetch(`${baseUrl}${path}`, {
        method: body === undefined ? 'GET' : 'POST',
        headers: body === undefined ? undefined : { 'Content-Type': 'application/json' },
        body: body === undefined ? undefined : JSON.stringify(body),
        connectTimeout: CONNECT_TIMEOUT_MS,
      });
    } catch {
      // Real connection failure: forget the address and retry once.
      forgetMaster();
      continue;
    }

    const data = await response.json().catch(() => ({}));

    // Server answered: report its message as-is, do not retry.
    if (!response.ok) {
      throw new Error(data.message || 'Permintaan ke server HR gagal.');
    }
    return data;
  }

  throw new Error(OFFLINE_MESSAGE);
}

export async function getMasterPasswordSetupStatus(): Promise<boolean> {
  const data = await request('/api/auth/setup-status');
  return Boolean(data.needs_setup);
}

export async function loginToMaster(username: string, password: string): Promise<string> {
  const data = await request('/api/auth/login', { username, password });
  sessionToken = data.session_token as string;
  return data.role as string;
}

export function clearMasterSession() {
  sessionToken = '';
}

export async function changeMasterPassword(
  _username: string,
  _currentPassword: string,
  newPassword: string,
) {
  await request('/api/auth/change-password', {
    session_token: sessionToken,
    new_password: newPassword,
  });
}

export async function changeOwnCredentials(
  currentPassword: string,
  newUsername: string,
  newPassword: string,
): Promise<string> {
  const data = await request('/api/auth/change-credentials', {
    session_token: sessionToken,
    current_password: currentPassword,
    new_username: newUsername,
    new_password: newPassword,
  });
  return String(data.username || newUsername);
}

// Matches the server route /api/auth/reset-division (HR session only).
export async function resetDivisionCredentials(
  actorPassword: string,
  targetRole: string,
  newUsername: string,
  newPassword: string,
) {
  await request('/api/auth/reset-division', {
    session_token: sessionToken,
    actor_password: actorPassword,
    target_role: targetRole,
    new_username: newUsername,
    new_password: newPassword,
  });
}

export async function listDivisionAccounts(): Promise<Array<{ username: string; role: string }>> {
  const data = await request('/api/auth/list-accounts', {
    session_token: sessionToken,
  });
  return Array.isArray(data.accounts) ? data.accounts : [];
}

export async function recoverMasterPassword(recoveryCode: string, newPassword: string) {
  await request('/api/auth/recover-master', {
    recovery_code: recoveryCode,
    new_password: newPassword,
  });
}