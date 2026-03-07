import type { EqBand } from './api';

export const app = $state({
  connected: false,
  deviceName: '',
  currentPage: 'equalizer' as 'equalizer' | 'config' | 'status' | 'autoeq',
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

export const config = $state({
  firmwareVersion: '',
  volMax: 120,
  volOutput: 0,
  volOutputSwitch: 0,
  micSwitch: 0,
  micMonitorVol: 0,
  screenOrientation: 0,
  channelBalance: 0,
});
