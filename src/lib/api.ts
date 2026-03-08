import { invoke } from '@tauri-apps/api/core';

export interface EqBand {
  index: number;
  gain: number;
  frequency: number;
  q_value: number;
  filter_type: number;
}

export const FilterTypeLabels: Record<number, string> = {
  0: 'Peak',
  1: 'Low Shelf',
  2: 'High Shelf',
  3: 'Band Pass',
  4: 'Low Pass',
  5: 'High Pass',
  6: 'All Pass',
};

export const EQ_PRESETS = [
  { label: 'BYPASS', value: 240 },
  { label: 'Jazz', value: 0 },
  { label: 'Pop', value: 1 },
  { label: 'Rock', value: 2 },
  { label: 'Dance', value: 3 },
  { label: 'R&B', value: 4 },
  { label: 'Classic', value: 5 },
  { label: 'HipHop', value: 6 },
  { label: 'Retro', value: 8 },
  { label: 'sDamp-1', value: 9 },
  { label: 'sDamp-2', value: 10 },
  { label: 'USER 1', value: 160 },
  { label: 'USER 2', value: 161 },
  { label: 'USER 3', value: 162 },
  { label: 'USER 4', value: 163 },
  { label: 'USER 5', value: 164 },
  { label: 'USER 6', value: 165 },
  { label: 'USER 7', value: 166 },
  { label: 'USER 8', value: 167 },
  { label: 'USER 9', value: 168 },
  { label: 'USER 10', value: 169 },
];

// Connection
export const connectDevice = () => invoke<string>('connect_device');
export const disconnectDevice = () => invoke<void>('disconnect_device');
export const isConnected = () => invoke<boolean>('is_connected');
export const getDeviceName = () => invoke<string | null>('get_device_name');

// EQ
export const getEqCount = () => invoke<number>('get_eq_count');
export const getEqBand = (index: number) => invoke<EqBand>('get_eq_band', { index });
export const getAllEqBands = () => invoke<EqBand[]>('get_all_eq_bands');
export const setEqBand = (index: number, frequency: number, gain: number, qValue: number, filterType: number) =>
  invoke<void>('set_eq_band', { index, frequency, gain, qValue, filterType });
export const getEqPreset = () => invoke<number>('get_eq_preset');
export const setEqPreset = (preset: number) => invoke<void>('set_eq_preset', { preset });
export const getEqGlobalGain = () => invoke<number>('get_eq_global_gain');
export const setEqGlobalGain = (gain: number) => invoke<void>('set_eq_global_gain', { gain });
export const getEqSwitch = () => invoke<boolean>('get_eq_switch');
export const setEqSwitch = (enabled: boolean) => invoke<void>('set_eq_switch', { enabled });
export const saveEq = (preset: number) => invoke<void>('save_eq', { preset });
export const resetEq = () => invoke<void>('reset_eq');

// Preset names
export const getPresetName = (index: number) => invoke<string>('get_preset_name', { index });
export const setPresetName = (index: number, name: string) => invoke<void>('set_preset_name', { index, name });

// AutoEQ
export interface AutoEqHeadphone {
  name: string;
  path: string;
  source: string;
}

export interface AutoEqFilter {
  index: number;
  enabled: boolean;
  filter_type: string;
  frequency: number;
  gain: number;
  q: number;
}

export interface AutoEqProfile {
  preamp: number;
  filters: AutoEqFilter[];
}

export const fetchAutoEqIndex = () => invoke<AutoEqHeadphone[]>('fetch_autoeq_index');
export const fetchAutoEqProfile = (path: string) => invoke<AutoEqProfile>('fetch_autoeq_profile', { path });
