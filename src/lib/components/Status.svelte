<script lang="ts">
  import { app, config, eq } from '$lib/store.svelte';
  import { EQ_PRESETS } from '$lib/api';

  const presetLabel = $derived(EQ_PRESETS.find(p => p.value === eq.preset)?.label || '—');
</script>

<div class="st-page">
  <h2>Device Status</h2>

  <div class="st-grid">
    <section class="st-card">
      <div class="card-head">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M18 20V10"/><path d="M12 20V4"/><path d="M6 20v-6"/></svg>
        <h3>Connection</h3>
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
        <span class="st-key">Firmware</span>
        <span class="st-val mono">{config.firmwareVersion || '—'}</span>
      </div>
      <div class="st-row">
        <span class="st-key">Protocol</span>
        <span class="st-val mono">USB HID</span>
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
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/></svg>
        <h3>Output</h3>
      </div>
      <div class="st-row">
        <span class="st-key">Max Volume</span>
        <span class="st-val mono">{config.volMax}</span>
      </div>
      <div class="st-row">
        <span class="st-key">Current Volume</span>
        <span class="st-val mono">{config.volOutput}</span>
      </div>
      <div class="st-row">
        <span class="st-key">Output Mode</span>
        <span class="st-val">{config.volOutputSwitch === 1 ? 'Fixed' : 'Variable'}</span>
      </div>
      <div class="st-row">
        <span class="st-key">Balance</span>
        <span class="st-val mono">
          {#if config.channelBalance < 0}L{Math.abs(config.channelBalance)}{:else if config.channelBalance > 0}R{config.channelBalance}{:else}0{/if}
        </span>
      </div>
    </section>

    <section class="st-card">
      <div class="card-head">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
        <h3>Hardware</h3>
      </div>
      <div class="st-row">
        <span class="st-key">DAC</span>
        <span class="st-val">24-bit R2R (192 resistors)</span>
      </div>
      <div class="st-row">
        <span class="st-key">Microphone</span>
        <span class="st-val">{config.micSwitch === 1 ? 'Enabled' : 'Disabled'}</span>
      </div>
      <div class="st-row">
        <span class="st-key">Display</span>
        <span class="st-val">{config.screenOrientation === 0 ? 'Normal' : 'Rotated'}</span>
      </div>
    </section>
  </div>

  <div class="st-footer">
    Protocol reverse-engineered from fiiocontrol.fiio.com &middot; Built with Tauri + Svelte
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

  .st-footer {
    text-align: center;
    font-size: 11px;
    color: var(--text-3);
    padding: 8px 0;
  }
</style>
