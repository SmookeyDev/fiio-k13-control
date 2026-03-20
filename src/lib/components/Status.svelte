<script lang="ts">
  import { app, eq, ble } from '$lib/store.svelte';
  import { EQ_PRESETS, INPUT_SOURCES, LIGHT_COLORS, BT_CODECS } from '$lib/api';

  const presetLabel = $derived(EQ_PRESETS.find(p => p.value === eq.preset)?.label || '—');
  const inputLabel = $derived(INPUT_SOURCES.find(s => s.value === ble.inputSource)?.label || '—');
  const topColorLabel = $derived(LIGHT_COLORS.find(c => c.value === ble.topLightColor)?.label || '—');
  const knobColorLabel = $derived(LIGHT_COLORS.find(c => c.value === ble.knobLightColor)?.label || '—');
  const btCodecLabel = $derived(BT_CODECS[ble.btCodec] || '—');
</script>

<div class="st-page">
  <h2>Device Status</h2>

  <div class="st-grid">
    <section class="st-card">
      <div class="card-head">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18 20V10"/><path d="M12 20V4"/><path d="M6 20v-6"/></svg>
        <h3>USB Connection</h3>
      </div>
      <div class="st-row">
        <span class="st-key">Status</span>
        <span class="st-val" class:st-green={app.connected}>{app.connected ? 'Connected' : 'Disconnected'}</span>
      </div>
      <div class="st-row">
        <span class="st-key">Device</span>
        <span class="st-val">{app.deviceName || '—'}</span>
      </div>
      <div class="st-row">
        <span class="st-key">Protocol</span>
        <span class="st-val mono">USB HID</span>
      </div>
    </section>

    <section class="st-card">
      <div class="card-head">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M4.5 16.5c-1.5 1.26-2 5-2 5s3.74-.5 5-2c.71-.84.7-2.13-.09-2.91a2.18 2.18 0 0 0-2.91-.09z"/><path d="M12 15l-3-3a22 22 0 0 1 2-3.95A12.88 12.88 0 0 1 22 2c0 2.72-.78 7.5-6 11a22.35 22.35 0 0 1-4 2z"/></svg>
        <h3>BLE Connection</h3>
      </div>
      <div class="st-row">
        <span class="st-key">Status</span>
        <span class="st-val" class:st-green={app.bleConnected} class:st-blue={app.bleConnected}>{app.bleConnected ? 'Connected' : 'Disconnected'}</span>
      </div>
      <div class="st-row">
        <span class="st-key">Device</span>
        <span class="st-val">{app.bleDeviceName || '—'}</span>
      </div>
      {#if ble.firmwareVersion}
        <div class="st-row">
          <span class="st-key">Firmware</span>
          <span class="st-val mono">{ble.firmwareVersion}</span>
        </div>
      {/if}
      <div class="st-row">
        <span class="st-key">Input</span>
        <span class="st-val">{inputLabel}</span>
      </div>
    </section>

    <section class="st-card">
      <div class="card-head">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><line x1="4" y1="21" x2="4" y2="14"/><line x1="4" y1="10" x2="4" y2="3"/><line x1="12" y1="21" x2="12" y2="12"/><line x1="12" y1="8" x2="12" y2="3"/><line x1="20" y1="21" x2="20" y2="16"/><line x1="20" y1="12" x2="20" y2="3"/></svg>
        <h3>Audio Engine</h3>
      </div>
      <div class="st-row">
        <span class="st-key">EQ</span>
        <span class="st-val" class:st-green={eq.enabled}>{eq.enabled ? 'Active' : 'Bypass'}</span>
      </div>
      <div class="st-row">
        <span class="st-key">Preset</span>
        <span class="st-val">{presetLabel}</span>
      </div>
      <div class="st-row">
        <span class="st-key">Bands</span>
        <span class="st-val mono">{eq.count}</span>
      </div>
      <div class="st-row">
        <span class="st-key">Pre-Amp</span>
        <span class="st-val mono">{eq.globalGain > 0 ? '+' : ''}{eq.globalGain.toFixed(1)} dB</span>
      </div>
    </section>

    <section class="st-card">
      <div class="card-head">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
        <h3>Indicator Lights</h3>
      </div>
      <div class="st-row">
        <span class="st-key">Top</span>
        <span class="st-val">{ble.topLightOn ? topColorLabel : 'Off'}</span>
      </div>
      <div class="st-row">
        <span class="st-key">Knob</span>
        <span class="st-val">{ble.knobLightOn ? knobColorLabel : 'Off'}</span>
      </div>
    </section>

    <section class="st-card full-width">
      <div class="card-head">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
        <h3>Hardware</h3>
      </div>
      <div class="st-row">
        <span class="st-key">DAC</span>
        <span class="st-val">24-bit R2R (192 resistors)</span>
      </div>
      <div class="st-row">
        <span class="st-key">Model</span>
        <span class="st-val">FiiO K13 R2R</span>
      </div>
    </section>
  </div>

  <div class="st-footer">
    Protocol reverse-engineered from FiiO Control &middot; <a href="https://github.com/SmookeyDev/fiio-k13-control" target="_blank" rel="noopener noreferrer" class="footer-link">Open Source on GitHub</a>
  </div>
</div>

<style>
  .st-page { display: flex; flex-direction: column; gap: 20px; }
  .st-page h2 { font-size: 17px; font-weight: 700; letter-spacing: -0.3px; }

  .st-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }

  .st-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-l);
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .st-card.full-width { grid-column: 1 / -1; }

  .card-head {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-2);
    margin-bottom: 4px;
  }

  .card-head h3 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--text-2);
  }

  .st-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 3px 0;
  }

  .st-key { font-size: 13px; color: var(--text-2); }

  .st-val {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-0);
  }

  .st-val.mono { font-family: var(--mono); font-size: 12px; }
  .st-green { color: var(--green) !important; }
  .st-blue { color: var(--accent) !important; }

  .st-footer {
    text-align: center;
    font-size: 11px;
    color: var(--text-3);
    padding: 8px 0;
  }

  .st-footer :global(.footer-link) {
    color: var(--accent-strong);
    text-decoration: none;
    transition: opacity 0.15s var(--ease);
  }

  .st-footer :global(.footer-link:hover) {
    text-decoration: underline;
    opacity: 0.85;
  }
</style>
