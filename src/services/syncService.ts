import { fetch } from '@tauri-apps/plugin-http';
import { getDb } from './db';
import { appConfig } from '../config';

const GOOGLE_APPS_SCRIPT_URL = appConfig.appsScriptUrl;
const ACTIVE_DAILY_TOKEN = appConfig.inspectionAccessToken;

export async function syncGoogleSheetToSqlite(): Promise<{ synced: number; skipped: number; errors: number }> {
  let synced = 0;
  let skipped = 0;
  let errors = 0;

  try {
    if (!GOOGLE_APPS_SCRIPT_URL || !ACTIVE_DAILY_TOKEN) {
      throw new Error('Inspection sync is not configured. Set VITE_APPS_SCRIPT_URL and VITE_INSPECTION_ACCESS_TOKEN.');
    }

    const syncUrl = `${GOOGLE_APPS_SCRIPT_URL}?action=json&token=${encodeURIComponent(ACTIVE_DAILY_TOKEN)}`;
    const response = await fetch(syncUrl, { method: 'GET' });

    if (!response.ok) {
      throw new Error(`HTTP Error Status: ${response.status}`);
    }

    const records: any[] = await response.json();

    if (!Array.isArray(records) || records.length === 0) {
      return { synced: 0, skipped: 0, errors: 0 };
    }

    const db = await getDb();

    for (const record of records) {
      const sheetToken = String(record['Security Token'] || record['security_token'] || '').trim();
      if (sheetToken && sheetToken !== ACTIVE_DAILY_TOKEN) {
        skipped++;
        continue;
      }

      const assetCodeRaw = record['Asset Code'] || record['Kode Box Hydrant'] || record['asset'] || '';
      const assetCode = String(assetCodeRaw).trim().toUpperCase();

      if (!assetCode) {
        errors++;
        continue;
      }

      const assetRows = await db.select<{ id: number; location_id: number }[]>(
        'SELECT id, location_id FROM assets WHERE code = ?',
        [assetCode]
      );

      if (assetRows.length === 0) {
        errors++;
        continue;
      }

      const asset = assetRows[0];

      // AMBIL PHOTO URL SECARA CERDAS & DINAMIS (Mencari kunci atau nilai yang mengandung http/drive.google)
      let photoUrl = "";
      // Cek langsung dari header umum jika ada
      const directPhoto = record['Photo URL'] || record['photo_url'] || record['foto'] || '';
      if (String(directPhoto).includes('http')) {
        photoUrl = String(directPhoto).trim();
      } else {
        // Cari di semua kolom record secara fleksibel
        for (const key of Object.keys(record)) {
          const val = String(record[key] || '').trim();
          if ((val.includes('drive.google.com') || val.startsWith('http')) && !val.includes(',')) {
            photoUrl = val;
            break;
          }
        }
      }

      const badgeId = String(record['Security Name'] || record['Badge ID'] || record['badgeId'] || 'SECURITY').trim();
      const responseId = record.response_id || `RESP_${assetCode}_${Date.now()}`;
      const notesVal = record['Notes'] || record['notes'] || '';

      await db.execute(
        `INSERT INTO inspections (google_response_id, asset_id, inspector_badge_id, inspected_at, notes, review_status, photo_url)
         VALUES (?, ?, ?, ?, ?, 'pending_review', ?)
         ON CONFLICT(google_response_id) DO UPDATE SET
           asset_id = excluded.asset_id,
           inspector_badge_id = excluded.inspector_badge_id,
           inspected_at = excluded.inspected_at,
           notes = excluded.notes,
           photo_url = excluded.photo_url;`,
        [
          responseId,
          asset.id,
          badgeId, 
          new Date(record.Timestamp || Date.now()).toISOString(),
          notesVal,
          photoUrl
        ]
      );

      const inspectionRes = await db.select<{ id: number }[]>(
        'SELECT id FROM inspections WHERE google_response_id = ?',
        [responseId]
      );
      
      if (inspectionRes.length === 0) continue;
      const inspectionId = inspectionRes[0].id;

      await db.execute('DELETE FROM inspection_item_checks WHERE inspection_id = ?', [inspectionId]);

      const components = [
        { name: 'Selang', status: record.Selang, note: record['Selang Keterangan'] },
        { name: 'Nozle', status: record.Nozle || record.Nozzle, note: record['Nozle Keterangan'] },
        { name: 'Valve', status: record.Valve, note: record['Valve Keterangan'] },
        { name: 'Lampu', status: record.Lampu, note: record['Lampu Keterangan'] },
        { name: 'Bell', status: record.Bell, note: record['Bell Keterangan'] },
        { name: 'J.Inter', status: record['J.Inter'] || record['Jack Inter'], note: record['J.Inter Keterangan'] },
        { name: 'Apar', status: record.Apar || record.APAR, note: record['Apar Keterangan'] },
        { name: 'Pressure Gauge', status: record['Pressure Gauge'], note: record['Pressure Gauge Keterangan'] },
        { name: 'Kunci pilar', status: record['Kunci pilar'] || record['Kunci Box'], note: record['Kunci pilar Keterangan'] }
      ];

      let hasDefect = false;

      for (const item of components) {
        if (!item.status || item.status === '-') continue;

        await db.execute(
          `INSERT INTO inspection_item_checks (inspection_id, component_name, condition_code, remark)
           VALUES (?, ?, ?, ?)`,
          [inspectionId, item.name, item.status, item.note || notesVal]
        );

        if (item.status === 'X' || item.status === 'R') {
          hasDefect = true;
        }
      }

      if (hasDefect) {
        const problemCode = `PROB-${responseId}`;
        await db.execute(
          `INSERT OR REPLACE INTO problems (problem_code, inspection_id, location_id, category, severity, description, status)
           VALUES (?, ?, ?, 'fire_safety', 'high', ?, 'open')`,
          [
            problemCode,
            inspectionId,
            asset.location_id,
            `Kerusakan pada Box Hydrant ${assetCode}: ${notesVal || 'Memerlukan perbaikan'}`
          ]
        );

        await db.execute('UPDATE assets SET status = "needs_service" WHERE id = ?', [asset.id]);
      }

      synced++;
    }

    return { synced, skipped, errors };
  } catch (error) {
    console.error('Sync execution failed:', error);
    throw error;
  }
}
