import { fetch as tauriFetch } from '@tauri-apps/plugin-http';
import { invoke } from '@tauri-apps/api/core';
import type { Employee } from '../types';
import { appConfig } from '../config';

function normalizeSecurityJabatan(value: string): string {
  return value
    .replace(/\s*\(\s*shift\s*\)/gi, '')
    .replace(/\s*-\s*[A-Z0-9]+\s*$/i, '')
    .replace(/\s+regu\s+[A-Z0-9]+\s*$/i, '')
    .trim() || 'Anggota Security';
}

function extractSecurityRegu(jabatan: string, regu: string): string {
  const explicitRegu = (regu || '').trim();
  if (explicitRegu) return explicitRegu.toUpperCase();
  
  const dashMatch = jabatan.match(/\s*-\s*([A-Z0-9]+)\s*$/i);
  if (dashMatch) return dashMatch[1].toUpperCase();

  const wordMatch = jabatan.match(/regu\s+([A-Z0-9]+)/i);
  if (wordMatch) return wordMatch[1].toUpperCase();

  return '';
}

export async function syncSecurityUsers(employees: Employee[]): Promise<void> {
  if (!appConfig.isAdminBuild) return;

  // Gunakan Map untuk deduplikasi berdasarkan nama atau NIK agar tidak ada data ganda
  const uniqueUsersMap = new Map<string, { name: string; regu: string; active: boolean }>();

  employees.forEach((employee) => {
    const division = String(employee.divisi || '').trim().toUpperCase();
    const job = String(employee.jabatan || '').trim().toUpperCase();
    const isSecurity = division.includes('SECURITY') || division.includes('KEAMANAN') || job.includes('SECURITY') || job.includes('SATPAM');
    const isChief = job.includes('CHIEF SECURITY') || job.includes('CHIEF SATPAM');

    if (isSecurity && !isChief) {
      const name = employee.nama_lengkap.trim();
      if (name) {
        uniqueUsersMap.set(name.toLowerCase(), {
          name: employee.nama_lengkap,
          regu: extractSecurityRegu(employee.jabatan || '', employee.regu || ''),
          active: String(employee.status || 'AKTIF').toUpperCase() === 'AKTIF',
        });
      }
    }
  });

  const users = Array.from(uniqueUsersMap.values());

  if (!users.length) {
    console.warn(`No SECURITY employees found for Apps Script sync. HR employee count: ${employees.length}`);
    return;
  }

  if (!appConfig.appsScriptAdminUrl || !appConfig.appsScriptAdminSecret) {
    throw new Error('HR admin Apps Script is not configured for this build.');
  }

  const response = await tauriFetch(appConfig.appsScriptAdminUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      action: 'sync_security_users',
      syncSecret: appConfig.appsScriptAdminSecret,
      users,
    }),
  });

  const responseText = await response.text();
  if (!response.ok) throw new Error(`Apps Script sync failed: HTTP ${response.status} ${responseText}`);

  let result: { ok?: boolean; error?: string; synced?: number };
  try {
    result = JSON.parse(responseText);
  } catch {
    throw new Error(`Apps Script returned non-JSON response: ${responseText.slice(0, 300)}`);
  }

  if (result.ok !== true) {
    throw new Error(result.error || 'Apps Script rejected the security-user sync');
  }

  console.info(`SecurityUsers synced: ${result.synced ?? users.length}`);
}

export async function syncSecurityEmployees(employees: Employee[]): Promise<void> {
  if (!appConfig.appsScriptAdminUrl || !appConfig.appsScriptAdminSecret) {
    throw new Error('Apps Script admin URL/secret belum dikonfigurasi untuk roster Security.');
  }

  // Gunakan Map berdasarkan NIK untuk memastikan tidak ada NIK ganda yang terkirim ke spreadsheet
  const uniqueEmployeesMap = new Map<string, { nik: string; name: string; jabatan: string; regu: string; active: boolean }>();

  employees.forEach((employee) => {
    const division = String(employee.divisi || '').trim().toUpperCase();
    const job = String(employee.jabatan || '').trim().toUpperCase();
    const isSecurity = division.includes('SECURITY') || division.includes('KEAMANAN') || job.includes('SECURITY') || job.includes('SATPAM');

    if (isSecurity && employee.nik) {
      uniqueEmployeesMap.set(employee.nik.trim(), {
        nik: employee.nik,
        name: employee.nama_lengkap,
        jabatan: normalizeSecurityJabatan(employee.jabatan || ''),
        regu: extractSecurityRegu(employee.jabatan || '', employee.regu || ''),
        active: String(employee.status || 'AKTIF').toUpperCase() === 'AKTIF',
      });
    }
  });

  const securityEmployees = Array.from(uniqueEmployeesMap.values());

  const response = await tauriFetch(appConfig.appsScriptAdminUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      action: 'sync_security_employees',
      syncSecret: appConfig.appsScriptAdminSecret,
      employees: securityEmployees,
    }),
  });
  
  const result = await response.json() as { ok?: boolean; error?: string };
  if (!response.ok || result.ok !== true) throw new Error(result.error || 'SecurityEmployees sync failed');
}

export async function syncSecurityAuthFromHr(): Promise<void> {
  if (!appConfig.appsScriptAdminUrl || !appConfig.appsScriptAdminSecret) {
    throw new Error('Apps Script admin URL/secret belum dikonfigurasi untuk sinkronisasi akun Security.');
  }
  const accounts = await invoke<Array<{ username: string; role: string; password_hash: string }>>('get_security_auth_accounts');
  const response = await tauriFetch(appConfig.appsScriptAdminUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      action: 'sync_security_auth',
      syncSecret: appConfig.appsScriptAdminSecret,
      accounts,
    }),
  });
  const result = await response.json() as { ok?: boolean; error?: string };
  if (!response.ok || result.ok !== true) throw new Error(result.error || 'Security auth sync failed');
}

export async function loginSecurityFromAppsScript(username: string, password: string): Promise<string> {
  if (!appConfig.appsScriptUrl || !appConfig.inspectionAccessToken) {
    throw new Error('Apps Script Security belum dikonfigurasi pada perangkat ini.');
  }
  const response = await tauriFetch(appConfig.appsScriptUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      action: 'security_login',
      token: appConfig.inspectionAccessToken,
      username,
      password,
    }),
  });
  const result = await response.json() as { ok?: boolean; role?: string; error?: string };
  if (!response.ok || result.ok !== true) throw new Error(result.error || 'Kredensial Security tidak valid.');
  return result.role || 'security';
}

export async function fetchSecurityRosterFromAppsScript(): Promise<Employee[]> {
  if (!appConfig.appsScriptUrl || !appConfig.inspectionAccessToken) return [];
  const query = new URLSearchParams({ action: 'security_employees', token: appConfig.inspectionAccessToken });
  const response = await tauriFetch(`${appConfig.appsScriptUrl}?${query.toString()}`);
  const result = await response.json() as { ok?: boolean; employees?: Array<{ nik: string; name: string; jabatan?: string; regu?: string }>; error?: string };
  if (!response.ok || result.ok !== true) throw new Error(result.error || 'Roster Security tidak dapat dimuat.');
  return (result.employees || []).map((employee) => ({
    nik: employee.nik,
    nama_lengkap: employee.name,
    jabatan: normalizeSecurityJabatan(employee.jabatan || 'Security'),
    divisi: 'SECURITY',
    regu: employee.regu || '',
    status: 'AKTIF',
  }));
}

export async function fetchSecurityAttendanceFromAppsScript(year: number, month: number): Promise<{
  employees: Employee[];
  cells: Array<{ nik: string; tanggal: string; status: string; keterangan: string }>;
}> {
  if (!appConfig.appsScriptAdminUrl || !appConfig.appsScriptAdminSecret) {
    throw new Error('Apps Script HR Auth/Admin belum dikonfigurasi.');
  }
  
  const response = await tauriFetch(appConfig.appsScriptAdminUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      action: 'security_rekap',
      syncSecret: appConfig.appsScriptAdminSecret,
      year: year,
      month: month,
    })
  });

  const result = await response.json() as {
    ok?: boolean;
    employees?: Array<{ nik: string; name: string; jabatan?: string; regu?: string }>;
    cells?: Array<{ nik: string; tanggal: string; status: string; keterangan?: string }>;
    error?: string;
  };

  if (!response.ok || result.ok !== true) throw new Error(result.error || 'Rekap Security tidak dapat dimuat dari HR Auth.');
  
  return {
    employees: (result.employees || []).map((employee) => ({
      nik: employee.nik,
      nama_lengkap: employee.name,
      jabatan: normalizeSecurityJabatan(employee.jabatan || 'Security'),
      divisi: 'SECURITY',
      regu: employee.regu || '',
      status: 'AKTIF',
    })),
    cells: (result.cells || []).map((cell) => ({ ...cell, keterangan: cell.keterangan || '' })),
  };
}

export async function checkSecurityRekapConnection(): Promise<{
  exists: boolean;
  lastSync: string;
  period: string;
  rows: number;
}> {
  if (!appConfig.appsScriptAdminUrl || !appConfig.appsScriptAdminSecret) {
    throw new Error('Apps Script HR Auth/Admin belum dikonfigurasi.');
  }
  
  const response = await tauriFetch(appConfig.appsScriptAdminUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      action: 'security_rekap_status',
      syncSecret: appConfig.appsScriptAdminSecret
    })
  });

  const result = await response.json() as {
    ok?: boolean;
    exists?: boolean;
    lastSync?: string;
    period?: string;
    rows?: number;
    error?: string;
  };

  if (!response.ok || result.ok !== true) throw new Error(result.error || 'Status rekap Security tidak dapat dibaca dari HR Auth.');
  
  return {
    exists: result.exists === true,
    lastSync: result.lastSync || '',
    period: result.period || '',
    rows: result.rows || 0,
  };
}

export async function updateSecurityPin(name: string, pin: string, regu = ''): Promise<void> {
  if (!appConfig.appsScriptAdminUrl || !appConfig.appsScriptAdminSecret) {
    throw new Error('Apps Script admin URL/secret belum dikonfigurasi untuk PIN Security.');
  }

  const response = await tauriFetch(appConfig.appsScriptAdminUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      action: 'set_security_pin',
      syncSecret: appConfig.appsScriptAdminSecret,
      name,
      pin,
      regu,
    }),
  });

  const responseText = await response.text();
  if (!response.ok) throw new Error(`Apps Script PIN update failed: HTTP ${response.status}`);
  const result = JSON.parse(responseText) as { ok?: boolean; error?: string };
  if (result.ok !== true) throw new Error(result.error || 'Apps Script rejected the PIN update');
  await invoke('cache_security_pin', { name, pin });
}

export async function getSecurityPinStatus(name: string): Promise<{ exists: boolean; hasPin: boolean; active: boolean }> {
  if (appConfig.appsScriptAdminUrl && appConfig.appsScriptAdminSecret) {
    const response = await tauriFetch(appConfig.appsScriptAdminUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        action: 'get_security_pin_status',
        syncSecret: appConfig.appsScriptAdminSecret,
        name,
      }),
    });
    const result = await response.json() as { ok?: boolean; exists?: boolean; hasPin?: boolean; active?: boolean; error?: string };
    if (!response.ok || result.ok !== true) throw new Error(result.error || 'Status PIN Security tidak dapat dibaca.');
    return { exists: result.exists === true, hasPin: result.hasPin === true, active: result.active !== false };
  }
  const hasPin = await invoke<boolean>('get_security_pin_cache_status', { name });
  return { exists: hasPin, hasPin, active: true };
}

export async function syncSecurityAttendanceToSheet(payload: {
  year: number;
  month: number;
  employees: Array<{ nik: string; name: string; jabatan?: string; regu?: string }>;
  attendance: Array<{ name: string; statuses: Record<string, string>; notes: Record<string, string> }>;
}): Promise<void> {
  if (!appConfig.appsScriptAdminUrl || !appConfig.securityAttendanceSecret) {
    throw new Error('URL dan secret rekap Security belum diatur pada build Security.');
  }
  const response = await tauriFetch(appConfig.appsScriptAdminUrl, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      action: 'sync_security_attendance',
      syncSecret: appConfig.securityAttendanceSecret,
      ...payload,
    }),
  });
  const result = await response.json() as { ok?: boolean; error?: string };
  if (!response.ok || result.ok !== true) throw new Error(result.error || 'Rekap Security gagal disinkronkan.');
}