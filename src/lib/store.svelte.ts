import type { EqBand } from './api';

export const app = $state({
  connected: false,
  deviceName: '',
  currentPage: 'equalizer' as 'equalizer' | 'status' | 'autoeq',
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
