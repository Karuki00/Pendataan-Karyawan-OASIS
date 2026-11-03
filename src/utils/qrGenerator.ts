import QRCode from 'qrcode';

const FORM_BASE_URL = 'https://docs.google.com/forms/d/e/YOUR_FORM_ID/viewform?usp=pp_url';

// Entry parameters from Google Form "Get pre-filled link"
const ENTRY_TOKEN = 'entry.987654321';
const ENTRY_ASSET = 'entry.456789123';

/**
 * Builds a pre-filled Google Form URL with Token and Asset Code.
 */
export function generatePreFilledUrl(token: string, assetCode: string): string {
  const url = new URL(FORM_BASE_URL);
  url.searchParams.append(ENTRY_TOKEN, token);
  url.searchParams.append(ENTRY_ASSET, assetCode);
  return url.toString();
}

/**
 * Generates a base64 Data URL QR Code image for display or printing.
 */
export async function generateInspectionQR(token: string, assetCode: string): Promise<string> {
  const preFilledUrl = generatePreFilledUrl(token, assetCode);
  try {
    return await QRCode.toDataURL(preFilledUrl, {
      width: 300,
      margin: 2,
      color: {
        dark: '#0f172a',
        light: '#ffffff'
      }
    });
  } catch (err) {
    console.error('Failed to generate QR code:', err);
    throw err;
  }
}