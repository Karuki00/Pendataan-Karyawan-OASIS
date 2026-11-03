import Database from '@tauri-apps/plugin-sql';
import type { StatMetric, FacilityProblem } from '../types/inspection';

let dbInstance: Database | null = null;

export async function getDb(): Promise<Database> {
  if (!dbInstance) {
    dbInstance = await Database.load('sqlite:oasis-karyawan.sqlite');
  }
  return dbInstance;
}

/**
 * Mengambil ringkasan statistik fasilitas yang benar-benar berasal dari database.
 */
export async function fetchDashboardStats(): Promise<StatMetric[]> {
  const db = await getDb();
  
  const totalAssets = await db.select<{ count: number }[]>('SELECT COUNT(*) as count FROM assets WHERE is_active = 1');
  const openIssues = await db.select<{ count: number }[]>('SELECT COUNT(*) as count FROM problems WHERE status = "open"');
  const pendingReviews = await db.select<{ count: number }[]>('SELECT COUNT(*) as count FROM inspections WHERE review_status = "pending_review"');

  return [
    { label: 'Active Facilities', value: totalAssets[0]?.count || 0, type: 'neutral' },
    { label: 'Open Issues', value: openIssues[0]?.count || 0, type: 'danger' },
    { label: 'Pending Reviews', value: pendingReviews[0]?.count || 0, type: 'warning' }
  ];
}

/**
 * Mengambil daftar masalah fasilitas terlaporkan
 */
export async function fetchReportedProblems(): Promise<FacilityProblem[]> {
  const db = await getDb();
  const rows = await db.select<any[]>(`
    SELECT 
      p.description as title,
      l.name as location,
      p.severity,
      p.reported_at as updated
    FROM problems p
    JOIN locations l ON p.location_id = l.id
    WHERE p.status = 'open'
    ORDER BY p.reported_at DESC
    LIMIT 5
  `);

  return rows.map(row => ({
    title: row.title,
    location: row.location,
    severity: row.severity,
    updated: row.updated ? new Date(row.updated).toLocaleDateString('id-ID') : 'Hari ini'
  }));
}

/**
 * Seeding data awal lokasi dan box hydrant ke SQLite
 */
export async function seedAllHydrantAssets() {
  const db = await getDb();

  // 1. Ensure Locations Exist
  const locCount = await db.select<{ count: number }[]>('SELECT COUNT(*) as count FROM locations');
  if (locCount[0]?.count === 0) {
    await db.execute(`
      INSERT INTO locations (id, name, type, building_code) VALUES 
      (1, 'Tower A', 'tower', 'TA'),
      (2, 'Tower B', 'tower', 'TB'),
      (3, 'Tower C', 'tower', 'TC'),
      (4, 'Lantai Dasar', 'floor', 'LD'),
      (5, 'Basement 1', 'floor', 'B1'),
      (6, 'Basement 2', 'floor', 'B2')
    `);
  }

  // 2. Ensure Category Exists
  const catCount = await db.select<{ count: number }[]>('SELECT COUNT(*) as count FROM asset_categories');
  if (catCount[0]?.count === 0) {
    await db.execute(`
      INSERT INTO asset_categories (id, name, description) VALUES 
      (1, 'Hydrant Box', 'Box Hydrant Equipment')
    `);
  }

  // 3. Build Full Asset Inventory
  const targetAssets: { category_id: number; location_id: number; code: string; name: string }[] = [];

  // Towers A, B, C (Floors 2-23, no 13, includes 12A)
  const towers = [
    { prefix: 'HTA', locId: 1, name: 'Tower A' },
    { prefix: 'HTB', locId: 2, name: 'Tower B' },
    { prefix: 'HTC', locId: 3, name: 'Tower C' },
  ];
  const floors = [2, 3, 5, 6, 7, 8, 9, 10, 11, 12, '12A', 14, 15, 16, 17, 18, 19, 20, 21, 22, 23];

  for (const tower of towers) {
    for (const fl of floors) {
      for (const suffix of ['A', 'B']) {
        targetAssets.push({
          category_id: 1,
          location_id: tower.locId,
          code: `${tower.prefix}-${fl}${suffix}`,
          name: `Box Hydrant ${tower.name} Lt.${fl}${suffix}`
        });
      }
    }
  }

  // HLD-1 through HLD-16
  const hldLocations: Record<number, string> = {
    1: 'Pos belakang', 2: 'Cucian mobil', 3: 'Parkir motor lt. dsr TC', 4: 'Pos depan / kiri',
    5: 'Pos depan / kanan', 6: 'Halte lt. dsr TA', 7: 'Lapangan tenis TA', 8: 'Toilet lapangan tenis',
    9: 'Pos anjungan', 10: 'Kolam renang', 11: 'Ruang Kasuari TA', 12: 'Pos jaga Lobby TA',
    13: 'Pos jaga Lobby TB', 14: 'Ruang Fitnes', 15: 'Pos jaga Lobby TC', 16: 'Ruang Serbaguna TC'
  };
  for (let i = 1; i <= 16; i++) {
    targetAssets.push({
      category_id: 1,
      location_id: 4,
      code: `HLD-${i}`,
      name: `Box Hydrant ${hldLocations[i] || 'Lantai Dasar'}`
    });
  }

  // HB1-1 through HB1-17
  for (let i = 1; i <= 17; i++) {
    targetAssets.push({
      category_id: 1,
      location_id: 5,
      code: `HB1-${i}`,
      name: `Box Hydrant Basement 1 Slot ${i}`
    });
  }

  // HB2-1 through HB2-13
  for (let i = 1; i <= 13; i++) {
    targetAssets.push({
      category_id: 1,
      location_id: 6,
      code: `HB2-${i}`,
      name: `Box Hydrant Basement 2 Slot ${i}`
    });
  }

  // 4. Upsert / Insert Missing Assets into SQLite
  for (const item of targetAssets) {
    await db.execute(
      `INSERT INTO assets (category_id, location_id, code, name, is_active)
       VALUES (?, ?, ?, ?, 1)
       ON CONFLICT(code) DO NOTHING;`,
      [item.category_id, item.location_id, item.code, item.name]
    );
  }

  console.log(`Seeded / verified ${targetAssets.length} hydrant assets in SQLite!`);
}
