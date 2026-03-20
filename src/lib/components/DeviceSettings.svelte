<script lang="ts">
  import { ble, showError } from '$lib/store.svelte';
  import * as api from '$lib/api';
  import Dropdown from './Dropdown.svelte';

  async function setSetting(name: string, fn: () => Promise<void>) {
    try { await fn(); } catch (e) { showError(name, e); }
  }

  // ---- Handlers ----

  function onInputSource(v: number) { ble.inputSource = v; setSetting('Input source', () => api.bleSetInputSource(v)); }

  function onLightSwitch(zone: number, on: boolean) {
    if (zone === api.ZONE_TOP) ble.topLightOn = on; else ble.knobLightOn = on;
    setSetting('Light switch', () => api.bleSetLightSwitch(zone, on));
  }
  function onLightMode(zone: number, mode: number) {
    if (zone === api.ZONE_TOP) ble.topLightMode = mode; else ble.knobLightMode = mode;
    setSetting('Light mode', () => api.bleSetLightMode(zone, mode));
  }
  function onLightColor(zone: number, color: number) {
    if (zone === api.ZONE_TOP) ble.topLightColor = color; else ble.knobLightColor = color;
    setSetting('Light color', () => api.bleSetLightColor(zone, color));
  }

  function getColorDot(value: number): string {
    const map: Record<number, string> = {
      0x00: '#888', 0x01: '#ef4444', 0x02: '#3b82f6', 0x03: '#06b6d4',
      0x04: '#a855f7', 0x05: '#eab308', 0x06: '#f5f5f5', 0x07: '#22c55e',
      0x08: 'conic-gradient(red, yellow, green, cyan, blue, magenta, red)',
    };
    return map[value] ?? '#888';
  }

</script>

<div class="settings">
  <div class="settings-header">
    <h2>Device Settings</h2>
    <span class="badge">BLE</span>
    {#if ble.firmwareVersion}
      <span class="fw-version">FW {ble.firmwareVersion}</span>
    {/if}
  </div>

  <!-- Input Source -->
  <section class="section">
    <h3>Input Source</h3>
    <div class="source-grid">
      {#each api.INPUT_SOURCES as src}
        <button
          class="source-btn"
          class:active={ble.inputSource === src.value}
          onclick={() => onInputSource(src.value)}
          aria-label={src.label}
        >
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
            {#if src.value === 0x01}
              <rect x="6" y="3" width="12" height="18" rx="2"/><line x1="10" y1="9" x2="14" y2="9"/><line x1="10" y1="13" x2="14" y2="13"/>
            {:else if src.value === 0x04}
              <circle cx="12" cy="12" r="3"/><circle cx="12" cy="12" r="7"/><circle cx="12" cy="12" r="10"/>
            {:else if src.value === 0x08}
              <path d="M2 12h4l3-9 6 18 3-9h4"/>
            {:else}
              <path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/>
            {/if}
          </svg>
          <span>{src.label}</span>
        </button>
      {/each}
    </div>
  </section>

  <!-- Indicator Lights -->
  <section class="section">
    <h3>Indicator Lights</h3>
    <div class="lights-grid">
      {#each [{ label: 'Top Light', zone: api.ZONE_TOP, on: ble.topLightOn, mode: ble.topLightMode, color: ble.topLightColor }, { label: 'Knob Light', zone: api.ZONE_KNOB, on: ble.knobLightOn, mode: ble.knobLightMode, color: ble.knobLightColor }] as light}
        <div class="light-card">
          <div class="light-card-header">
            <span class="light-label">{light.label}</span>
            <button class="toggle" class:on={light.on} onclick={() => onLightSwitch(light.zone, !light.on)} aria-label="{light.label} toggle">
              <span class="toggle-thumb"></span>
            </button>
          </div>
          {#if light.on}
            <div class="light-controls">
              <div class="control-row">
                <span class="control-label">Mode</span>
                <Dropdown options={api.LIGHT_MODES} value={light.mode} onchange={(v) => onLightMode(light.zone, v)} width="130px" />
              </div>
              <div class="control-row">
                <span class="control-label">Color</span>
                <div class="color-picker">
                  {#each api.LIGHT_COLORS as c}
                    <button class="color-swatch" class:active={light.color === c.value} style="background: {getColorDot(c.value)}" title={c.label} onclick={() => onLightColor(light.zone, c.value)} aria-label={c.label}></button>
                  {/each}
                </div>
              </div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </section>
</div>

<style>
  .settings { max-width: 640px; }

  .settings-header {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 24px;
  }
  .settings-header h2 { font-size: 18px; font-weight: 700; letter-spacing: -0.3px; }
  .badge { font-size: 10px; font-weight: 600; color: var(--accent); background: var(--accent-dim); padding: 2px 8px; border-radius: 4px; letter-spacing: 0.5px; }
  .fw-version { font-size: 10px; color: var(--text-3); margin-left: auto; font-family: var(--font-mono, monospace); }

  .section { margin-bottom: 28px; }
  .section h3 { font-size: 12px; font-weight: 600; color: var(--text-2); text-transform: uppercase; letter-spacing: 0.8px; margin-bottom: 12px; }

  /* Input Source */
  .source-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 8px; }
  .source-btn {
    display: flex; flex-direction: column; align-items: center; gap: 6px;
    padding: 14px 8px; background: var(--bg-2); border: 1px solid var(--border-subtle);
    border-radius: var(--radius-l); color: var(--text-2); font-size: 11px; font-weight: 500;
    transition: all 0.15s var(--ease);
  }
  .source-btn:hover { background: var(--bg-3); color: var(--text-1); border-color: var(--border-default); }
  .source-btn.active { background: var(--accent-dim); border-color: var(--accent); color: var(--accent); }
  .source-btn.active svg { color: var(--accent); }

  /* Controls */
  .control-row { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .control-label { font-size: 12px; color: var(--text-2); font-weight: 500; min-width: 70px; flex-shrink: 0; }

  /* Lights */
  .lights-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  .light-card { background: var(--bg-2); border: 1px solid var(--border-subtle); border-radius: var(--radius-l); padding: 16px; }
  .light-card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; }
  .light-label { font-size: 13px; font-weight: 600; color: var(--text-1); }
  .light-controls { display: flex; flex-direction: column; gap: 10px; }

  /* Toggle */
  .toggle { position: relative; width: 36px; height: 20px; background: var(--bg-3); border-radius: 10px; padding: 2px; transition: background 0.2s; border: 1px solid var(--border-subtle); }
  .toggle.on { background: var(--accent); border-color: var(--accent); }
  .toggle-thumb { display: block; width: 14px; height: 14px; background: white; border-radius: 50%; transition: transform 0.2s; }
  .toggle.on .toggle-thumb { transform: translateX(16px); }

  /* Color picker */
  .color-picker { display: flex; gap: 4px; flex-wrap: wrap; }
  .color-swatch { width: 20px; height: 20px; border-radius: 50%; border: 2px solid transparent; transition: all 0.15s; cursor: pointer; }
  .color-swatch:hover { transform: scale(1.15); }
  .color-swatch.active { border-color: var(--text-0); box-shadow: 0 0 0 2px var(--bg-0), 0 0 0 4px var(--text-0); transform: scale(1.1); }
</style>
