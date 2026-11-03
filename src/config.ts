export const appConfig = {
  appsScriptUrl: import.meta.env.VITE_APPS_SCRIPT_URL || '',
  appsScriptAdminUrl: import.meta.env.VITE_APPS_SCRIPT_ADMIN_URL || '',
  appsScriptAdminSecret: import.meta.env.VITE_APPS_SCRIPT_ADMIN_SECRET || '',
  securityAttendanceSecret: import.meta.env.VITE_SECURITY_ATTENDANCE_SECRET || '',
  inspectionAccessToken: import.meta.env.VITE_INSPECTION_ACCESS_TOKEN || '',
  isAdminBuild: import.meta.env.VITE_ADMIN_BUILD === 'true',
  isMasterNode: import.meta.env.VITE_MASTER_NODE === 'true',
};