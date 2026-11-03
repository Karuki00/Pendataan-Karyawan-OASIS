export interface StatMetric {
  label: string;
  value: number | string;
  type?: 'neutral' | 'danger' | 'warning' | 'success';
}

export interface FacilityProblem {
  title: string;
  location: string;
  severity: 'critical' | 'high' | 'medium' | 'low';
  severity_class?: string;
  updated: string;
}

export interface GoogleSheetInspectionRow {
  Timestamp: string;
  'Security Token': string;
  'Asset Code': string;
  Selang: 'V' | 'X' | 'R';
  Nozzle: 'V' | 'X' | 'R';
  Valve: 'V' | 'X' | 'R';
  Lampu: 'V' | 'X' | 'R';
  Bell: 'V' | 'X' | 'R';
  'Jack Inter': 'V' | 'X' | 'R';
  APAR: 'V' | 'X' | 'R';
  Notes: string;
  response_id: string;
}
