<script lang="ts">
  import { app, eq } from '$lib/store.svelte';
  import * as api from '$lib/api';
  import Equalizer from '$lib/components/Equalizer.svelte';
  import Status from '$lib/components/Status.svelte';
  import AutoEQ from '$lib/components/AutoEQ.svelte';

  async function handleConnect() {
    if (app.connected) {
      await api.disconnectDevice();
      app.connected = false;
      app.deviceName = '';
      return;
    }
    app.loading = true;
    app.error = null;
    try {
      const name = await api.connectDevice();
      app.connected = true;
      app.deviceName = name;
      await loadDeviceState();
    } catch (e: any) {
      app.error = e?.toString() || 'Connection failed';
    } finally {
      app.loading = false;
    }
  }

  async function loadDeviceState() {
    try {
      const [eqEnabled, eqCount, eqPreset, globalGain] = await Promise.all([
        api.getEqSwitch().catch(() => true),
        api.getEqCount().catch(() => 10),
        api.getEqPreset().catch(() => 240),
        api.getEqGlobalGain().catch(() => 0),
      ]);
      eq.enabled = eqEnabled;
      eq.count = eqCount;
      eq.preset = eqPreset;
      eq.globalGain = globalGain;

      const bands = await api.getAllEqBands().catch(() => []);
      eq.bands = bands;
    } catch (e) {
      console.error('Failed to load device state:', e);
    }
  }
</script>

<div class="shell">
  <!-- Sidebar -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <div class="brand">
        <span class="brand-mark">K13</span>
        <span class="brand-model">R2R</span>
      </div>
      <span class="brand-sub">Desktop Control</span>
    </div>

    <nav class="sidebar-nav">
      <button
        class="nav-link"
        class:active={app.currentPage === 'equalizer'}
        onclick={() => (app.currentPage = 'equalizer')}
        disabled={!app.connected}
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
          <line x1="4" y1="21" x2="4" y2="14"/><line x1="4" y1="10" x2="4" y2="3"/><line x1="12" y1="21" x2="12" y2="12"/><line x1="12" y1="8" x2="12" y2="3"/><line x1="20" y1="21" x2="20" y2="16"/><line x1="20" y1="12" x2="20" y2="3"/><line x1="1" y1="14" x2="7" y2="14"/><line x1="9" y1="8" x2="15" y2="8"/><line x1="17" y1="16" x2="23" y2="16"/>
        </svg>
        Equalizer
      </button>
      <button
        class="nav-link"
        class:active={app.currentPage === 'autoeq'}
        onclick={() => (app.currentPage = 'autoeq')}
        disabled={!app.connected}
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
          <path d="M3 18v-6a9 9 0 0 1 18 0v6"/>
          <path d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3zM3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3z"/>
        </svg>
        Auto EQ
      </button>
      <button
        class="nav-link"
        class:active={app.currentPage === 'status'}
        onclick={() => (app.currentPage = 'status')}
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
          <circle cx="12" cy="12" r="10"/><line x1="12" y1="16" x2="12" y2="12"/><line x1="12" y1="8" x2="12.01" y2="8"/>
        </svg>
        Status
      </button>
    </nav>

    <div class="sidebar-footer">
      {#if app.connected}
        <div class="device-info">
          <div class="device-status">
            <span class="pulse"></span>
            <span class="device-label">Connected</span>
          </div>
          <span class="device-name">{app.deviceName}</span>
        </div>
      {/if}
      <button
        class="sidebar-connect"
        class:connected={app.connected}
        onclick={handleConnect}
        disabled={app.loading}
      >
        {#if app.loading}
          <span class="spinner"></span> Scanning...
        {:else if app.connected}
          Disconnect
        {:else}
          Connect USB
        {/if}
      </button>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="main">
    {#if app.error}
      <div class="toast-error">
        <span>{app.error}</span>
        <button onclick={() => (app.error = null)}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>
    {/if}

    {#if !app.connected}
      <div class="welcome">
        <div class="welcome-visual">
          <div class="rings">
            <div class="ring ring-1"></div>
            <div class="ring ring-2"></div>
            <div class="ring ring-3"></div>
            <div class="ring-center">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
                <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
                <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
                <line x1="12" y1="19" x2="12" y2="23"/>
                <line x1="8" y1="23" x2="16" y2="23"/>
              </svg>
            </div>
          </div>
        </div>
        <div class="welcome-text">
          <h1>FiiO K13 R2R</h1>
          <p>Connect your DAC via USB to configure equalizer, volume, and device settings.</p>
        </div>
        <button class="welcome-btn" onclick={handleConnect} disabled={app.loading}>
          {app.loading ? 'Scanning for devices...' : 'Connect Device'}
        </button>
        <div class="welcome-hint">
          <span>USB HID</span>
          <span class="dot"></span>
          <span>24-bit R2R DAC</span>
          <span class="dot"></span>
          <span>10-Band PEQ</span>
        </div>
      </div>
    {:else if app.currentPage === 'equalizer'}
      <Equalizer />
    {:else if app.currentPage === 'autoeq'}
      <AutoEQ />
    {:else}
      <Status />
    {/if}
  </main>
</div>

<style>
  .shell {
    display: flex;
    flex-direction: row;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--bg-0);
  }

  /* Sidebar */
  .sidebar {
    width: 220px;
    min-width: 220px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-1);
    border-right: 1px solid var(--border-subtle);
  }

  .sidebar-header {
    padding: 24px 20px 20px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 5px;
    margin-bottom: 2px;
  }

  .brand-mark {
    font-size: 20px;
    font-weight: 700;
    color: var(--text-0);
    letter-spacing: -1px;
  }

  .brand-model {
    font-size: 11px;
    font-weight: 600;
    color: var(--teal);
    background: var(--teal-dim);
    padding: 1px 6px;
    border-radius: 3px;
    letter-spacing: 0.5px;
  }

  .brand-sub {
    font-size: 11px;
    color: var(--text-3);
    letter-spacing: 0.3px;
  }

  .sidebar-nav {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 0 10px;
  }

  .nav-link {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 9px 12px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text-2);
    background: transparent;
    border-radius: var(--radius-m);
    transition: all 0.15s var(--ease);
  }

  .nav-link:hover:not(:disabled) {
    color: var(--text-1);
    background: var(--bg-3);
  }

  .nav-link.active {
    color: var(--text-0);
    background: var(--bg-3);
  }

  .nav-link.active::before {
    content: '';
    position: absolute;
    left: 0;
    width: 3px;
    height: 20px;
    background: var(--accent);
    border-radius: 0 2px 2px 0;
  }

  .nav-link:disabled {
    opacity: 0.25;
    cursor: default;
  }

  .nav-link svg {
    opacity: 0.6;
    flex-shrink: 0;
  }

  .nav-link.active svg {
    opacity: 1;
    color: var(--accent);
  }

  .sidebar-footer {
    padding: 16px;
    border-top: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .device-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .device-status {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .pulse {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--green);
    box-shadow: 0 0 6px var(--green);
    animation: pulse 2s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .device-label {
    font-size: 10px;
    font-weight: 600;
    color: var(--green);
    text-transform: uppercase;
    letter-spacing: 0.8px;
  }

  .device-name {
    font-size: 11px;
    color: var(--text-2);
    padding-left: 12px;
  }

  .sidebar-connect {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 8px;
    font-size: 12px;
    font-weight: 600;
    border-radius: var(--radius-m);
    transition: all 0.15s var(--ease);
    background: var(--accent);
    color: white;
  }

  .sidebar-connect:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .sidebar-connect.connected {
    background: var(--bg-3);
    color: var(--text-2);
  }

  .sidebar-connect.connected:hover {
    color: var(--rose);
    background: var(--rose-dim);
  }

  .sidebar-connect:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  .spinner {
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255,255,255,0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Main */
  .main {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 20px 24px;
    position: relative;
    min-width: 0;
    height: 100vh;
  }

  .toast-error {
    position: fixed;
    top: 16px;
    right: 16px;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    background: var(--bg-elevated);
    border: 1px solid var(--rose);
    border-radius: var(--radius-l);
    color: var(--rose);
    font-size: 12px;
    z-index: 100;
    box-shadow: 0 8px 32px rgba(0,0,0,0.4);
  }

  .toast-error button {
    background: none;
    color: var(--rose);
    padding: 2px;
    border-radius: 4px;
    display: flex;
  }

  /* Welcome */
  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 24px;
  }

  .welcome-visual {
    margin-bottom: 8px;
  }

  .rings {
    position: relative;
    width: 140px;
    height: 140px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .ring {
    position: absolute;
    border-radius: 50%;
    border: 1px solid var(--border-default);
  }

  .ring-1 { width: 140px; height: 140px; opacity: 0.3; }
  .ring-2 { width: 100px; height: 100px; opacity: 0.5; }
  .ring-3 { width: 66px; height: 66px; opacity: 0.7; border-color: var(--border-strong); }

  .ring-center {
    position: relative;
    z-index: 1;
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: var(--bg-3);
    color: var(--text-2);
  }

  .welcome-text {
    text-align: center;
  }

  .welcome-text h1 {
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.5px;
    margin-bottom: 6px;
  }

  .welcome-text p {
    color: var(--text-2);
    font-size: 13px;
    max-width: 340px;
    line-height: 1.6;
  }

  .welcome-btn {
    padding: 10px 28px;
    background: var(--accent);
    color: white;
    font-size: 13px;
    font-weight: 600;
    border-radius: var(--radius-m);
    transition: all 0.2s var(--ease);
  }

  .welcome-btn:hover:not(:disabled) {
    filter: brightness(1.1);
    transform: translateY(-1px);
    box-shadow: 0 4px 16px var(--accent-dim);
  }

  .welcome-btn:disabled { opacity: 0.5; cursor: wait; }

  .welcome-hint {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--text-3);
    font-size: 11px;
    margin-top: 4px;
  }

  .welcome-hint .dot {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: var(--text-3);
  }
</style>
