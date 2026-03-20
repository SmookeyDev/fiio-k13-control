import type { EqBand, BleEqBand } from './api';

/** Log error to console and show in UI toast. Auto-disconnects BLE on connection errors. */
export function showError(context: string, e: unknown) {
  const msg = `${context}: ${e}`;
  const errStr = String(e);
  console.error(`[ERROR] ${msg}`);
  app.error = msg;

  if (errStr.includes('Not connected') || errStr.includes('NotConnected')) {
    app.bleConnected = false;
    app.bleDeviceName = '';
    if (app.currentPage === 'settings') {
      app.currentPage = 'equalizer';
    }
  }
}

export const app = $state({
  connected: false,
  deviceName: '',
  bleConnected: false,
  bleDeviceName: '',
  bleLoading: false,
  currentPage: 'equalizer' as 'equalizer' | 'status' | 'autoeq' | 'settings',
  loading: false,
  error: null as string | null,
});

export const eq = $state({
  enabled: true,
  bands: [] as EqBand[],
  count: 10,
  preset: 240,
  globalGain: 0,
  selectedBand: 0,
});

export const ble = $state({
  inputSource: 0x01 as number,
  topLightOn: true,
  topLightMode: 0 as number,
  topLightColor: 0 as number,
  knobLightOn: true,
  knobLightMode: 0 as number,
  knobLightColor: 0 as number,
  firmwareVersion: '' as string,
  btCodec: 0 as number,
});
