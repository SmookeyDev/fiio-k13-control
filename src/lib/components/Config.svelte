<script lang="ts">
  import { config } from '$lib/store.svelte';
  import * as api from '$lib/api';

  async function setVolMax() { await api.setVolMax(config.volMax).catch(console.error); }
  async function setVolOutput() { await api.setVolOutput(config.volOutput).catch(console.error); }
  async function setChannelBalance() { await api.setChannelBalance(config.channelBalance).catch(console.error); }
  async function setMicMonitorVol() { await api.setMicMonitorVol(config.micMonitorVol).catch(console.error); }

  async function toggleVolOutputSwitch() {
    config.volOutputSwitch = config.volOutputSwitch === 0 ? 1 : 0;
    await api.setVolOutputSwitch(config.volOutputSwitch).catch(console.error);
  }
  async function toggleMicSwitch() {
    config.micSwitch = config.micSwitch === 0 ? 1 : 0;
    await api.setMicSwitch(config.micSwitch).catch(console.error);
  }
  async function toggleScreenOrientation() {
    config.screenOrientation = config.screenOrientation === 0 ? 1 : 0;
    await api.setScreenOrientation(config.screenOrientation).catch(console.error);
  }
</script>

<div class="cfg-page">
  <h2>Configuration</h2>

  <div class="cfg-grid">
    <!-- Volume Section -->
    <section class="cfg-card">
      <div class="card-head">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/></svg>
        <h3>Volume</h3>
      </div>

      <div class="cfg-row">
        <div class="cfg-row-head">
          <span class="cfg-name">Maximum Volume</span>
          <span class="cfg-val">{config.volMax}</span>
        </div>
        <input type="range" min="0" max="120" step="1" bind:value={config.volMax} onchange={setVolMax} />
      </div>

      <div class="cfg-row">
        <div class="cfg-row-head">
          <span class="cfg-name">Output Volume</span>
          <span class="cfg-val">{config.volOutput}</span>
        </div>
        <input type="range" min="0" max="120" step="1" bind:value={config.volOutput} onchange={setVolOutput} />
      </div>

      <div class="cfg-row">
        <div class="cfg-row-head">
          <span class="cfg-name">Output Mode</span>
        </div>
        <div class="segmented">
          <button class:seg-active={config.volOutputSwitch === 0} onclick={config.volOutputSwitch === 0 ? undefined : toggleVolOutputSwitch}>Variable</button>
          <button class:seg-active={config.volOutputSwitch === 1} onclick={config.volOutputSwitch === 1 ? undefined : toggleVolOutputSwitch}>Fixed</button>
        </div>
      </div>

      <div class="cfg-row">
        <div class="cfg-row-head">
          <span class="cfg-name">Channel Balance</span>
          <span class="cfg-val">
            {#if config.channelBalance < 0}L {Math.abs(config.channelBalance)}{:else if config.channelBalance > 0}R {config.channelBalance}{:else}Center{/if}
          </span>
        </div>
        <input type="range" min="-10" max="10" step="1" bind:value={config.channelBalance} onchange={setChannelBalance} />
      </div>
    </section>

    <!-- Microphone Section -->
    <section class="cfg-card">
      <div class="card-head">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/></svg>
        <h3>Microphone</h3>
      </div>

      <div class="cfg-row">
        <div class="cfg-row-head">
          <span class="cfg-name">Microphone Input</span>
        </div>
        <div class="segmented">
          <button class:seg-active={config.micSwitch === 0} onclick={config.micSwitch === 0 ? undefined : toggleMicSwitch}>Off</button>
          <button class:seg-active={config.micSwitch === 1} onclick={config.micSwitch === 1 ? undefined : toggleMicSwitch}>On</button>
        </div>
      </div>

      <div class="cfg-row">
        <div class="cfg-row-head">
          <span class="cfg-name">Monitor Volume</span>
          <span class="cfg-val">{config.micMonitorVol}</span>
        </div>
        <input type="range" min="0" max="100" step="1" bind:value={config.micMonitorVol} onchange={setMicMonitorVol} />
      </div>
    </section>

    <!-- Display Section -->
    <section class="cfg-card">
      <div class="card-head">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>
        <h3>Display</h3>
      </div>

      <div class="cfg-row">
        <div class="cfg-row-head">
          <span class="cfg-name">Screen Orientation</span>
        </div>
        <div class="segmented">
          <button class:seg-active={config.screenOrientation === 0} onclick={config.screenOrientation === 0 ? undefined : toggleScreenOrientation}>Normal</button>
          <button class:seg-active={config.screenOrientation === 1} onclick={config.screenOrientation === 1 ? undefined : toggleScreenOrientation}>Rotated 180</button>
        </div>
      </div>
    </section>
  </div>
</div>

<style>
  .cfg-page { display: flex; flex-direction: column; gap: 20px; }
  .cfg-page h2 { font-size: 17px; font-weight: 700; letter-spacing: -0.3px; }

  .cfg-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
  }

  .cfg-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-l);
    padding: 18px 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .card-head {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-2);
  }

  .card-head h3 {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--text-2);
  }

  .cfg-row { display: flex; flex-direction: column; gap: 8px; }

  .cfg-row-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .cfg-name { font-size: 13px; color: var(--text-1); }

  .cfg-val {
    font-family: var(--mono);
    font-size: 12px;
    font-weight: 500;
    color: var(--accent-strong);
  }

  /* Segmented control */
  .segmented {
    display: flex;
    gap: 0;
    background: var(--bg-2);
    border-radius: var(--radius-m);
    padding: 2px;
  }

  .segmented button {
    flex: 1;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 500;
    color: var(--text-3);
    background: transparent;
    border-radius: var(--radius-s);
    transition: all 0.15s var(--ease);
  }

  .segmented button:hover { color: var(--text-2); }

  .seg-active {
    background: var(--bg-4) !important;
    color: var(--text-0) !important;
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  }
</style>
