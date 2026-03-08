<script lang="ts">
  import { eq } from '$lib/store.svelte';
  import * as api from '$lib/api';
  import type { AutoEqHeadphone, AutoEqProfile } from '$lib/api';
  import { EQ_PRESETS } from '$lib/api';
  import Dropdown from './Dropdown.svelte';

  let headphones = $state<AutoEqHeadphone[]>([]);
  let filtered = $state<AutoEqHeadphone[]>([]);
  let search = $state('');
  let loading = $state(false);
  let loadingProfile = $state(false);
  let selectedHp = $state<AutoEqHeadphone | null>(null);
  let profile = $state<AutoEqProfile | null>(null);
  let applying = $state(false);
  let savePreset = $state(160); // USER 1

  const userPresets = EQ_PRESETS.filter(p => p.value >= 160 && p.value <= 169);

  async function loadIndex() {
    if (headphones.length > 0) return;
    loading = true;
    try {
      headphones = await api.fetchAutoEqIndex();
      filterList();
    } catch (e) {
      console.error('Failed to load AutoEQ index:', e);
    } finally {
      loading = false;
    }
  }

  function filterList() {
    const q = search.toLowerCase().trim();
    if (!q) {
      filtered = headphones.slice(0, 100);
    } else {
      filtered = headphones
        .filter(h => h.name.toLowerCase().includes(q) || h.source.toLowerCase().includes(q))
        .slice(0, 100);
    }
  }

  async function selectHeadphone(hp: AutoEqHeadphone) {
    selectedHp = hp;
    loadingProfile = true;
    profile = null;
    try {
      profile = await api.fetchAutoEqProfile(hp.path);
    } catch (e) {
      console.error('Failed to load profile:', e);
    } finally {
      loadingProfile = false;
    }
  }

  async function applyToDevice() {
    if (!profile) return;
    applying = true;
    try {
      // Set preset to user slot first
      await api.setEqPreset(savePreset);
      eq.preset = savePreset;

      // Set global gain (preamp)
      await api.setEqGlobalGain(profile.preamp);
      eq.globalGain = profile.preamp;

      // Apply each filter to device bands (K13 has 10 bands)
      for (let i = 0; i < Math.min(profile.filters.length, 10); i++) {
        const f = profile.filters[i];
        const filterType = f.filter_type === 'LSC' ? 1 : f.filter_type === 'HSC' ? 2 : 0;
        await api.setEqBand(i, Math.round(f.frequency), f.gain, f.q, filterType);
      }

      // Zero out remaining bands
      for (let i = profile.filters.length; i < 10; i++) {
        await api.setEqBand(i, 1000, 0, 1, 0);
      }

      // Save to device
      await api.saveEq(savePreset);

      // Read back actual state from device
      eq.bands = await api.getAllEqBands();
      eq.globalGain = await api.getEqGlobalGain().catch(() => profile!.preamp);
    } catch (e) {
      console.error('Failed to apply profile:', e);
    } finally {
      applying = false;
    }
  }

  function computeProfilePath(prof: AutoEqProfile): string {
    const minF = Math.log10(20), maxF = Math.log10(20000);
    const pts: string[] = [];
    for (let i = 0; i <= 200; i++) {
      const logF = minF + (i / 200) * (maxF - minF);
      const freq = Math.pow(10, logF);
      let gain = prof.preamp;
      for (const f of prof.filters) {
        if (f.frequency <= 0 || f.q <= 0) continue;
        const x = Math.log2(freq / f.frequency) / (1 / f.q);
        gain += f.gain * Math.exp(-0.5 * x * x);
      }
      const xp = (i / 200) * 400;
      const yp = Math.max(0, Math.min(120, 60 - (gain / 15) * 60));
      pts.push(`${i === 0 ? 'M' : 'L'}${xp.toFixed(1)},${yp.toFixed(1)}`);
    }
    return pts.join(' ');
  }

  // Load index on mount
  $effect(() => { loadIndex(); });
  $effect(() => { search; filterList(); });
</script>

<div class="aeq-page">
  <div class="aeq-toolbar">
    <h2>AutoEQ Profiles</h2>
    <span class="aeq-count">{headphones.length > 0 ? `${headphones.length} headphones` : ''}</span>
  </div>

  <div class="aeq-layout">
    <!-- Left: Search & List -->
    <div class="aeq-list-panel">
      <div class="aeq-search-wrap">
        <svg class="aeq-search-icon" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
        <input
          class="aeq-search"
          type="text"
          placeholder="Search headphones..."
          bind:value={search}
        />
      </div>

      {#if loading}
        <div class="aeq-loading">
          <span class="spinner-sm"></span>
          <span>Loading AutoEQ database...</span>
        </div>
      {:else}
        <div class="aeq-list">
          {#each filtered as hp}
            <button
              class="aeq-item"
              class:aeq-item-active={selectedHp?.path === hp.path}
              onclick={() => selectHeadphone(hp)}
            >
              <span class="aeq-item-name">{hp.name}</span>
              {#if hp.source}
                <span class="aeq-item-source">{hp.source}</span>
              {/if}
            </button>
          {/each}
          {#if filtered.length === 0 && search}
            <div class="aeq-empty">No headphones matching "{search}"</div>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Right: Profile detail -->
    <div class="aeq-detail-panel">
      {#if !selectedHp}
        <div class="aeq-placeholder">
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round">
            <path d="M3 18v-6a9 9 0 0 1 18 0v6"/>
            <path d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3zM3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3z"/>
          </svg>
          <p>Select a headphone to view its AutoEQ profile</p>
        </div>
      {:else if loadingProfile}
        <div class="aeq-placeholder">
          <span class="spinner-sm"></span>
          <p>Loading EQ profile...</p>
        </div>
      {:else if profile}
        <div class="aeq-profile">
          <div class="aeq-profile-head">
            <div>
              <h3>{selectedHp.name}</h3>
              {#if selectedHp.source}
                <span class="aeq-source-badge">{selectedHp.source}</span>
              {/if}
            </div>
            <div class="aeq-apply-row">
              <Dropdown options={userPresets} bind:value={savePreset} width="100px" />
              <button class="tb-btn primary" onclick={applyToDevice} disabled={applying}>
                {applying ? 'Applying...' : 'Apply to Device'}
              </button>
            </div>
          </div>

          <div class="aeq-preamp">
            <span class="aeq-preamp-label">Preamp</span>
            <span class="aeq-preamp-val" class:pos={profile.preamp > 0} class:neg={profile.preamp < 0}>
              {profile.preamp > 0 ? '+' : ''}{profile.preamp.toFixed(1)} dB
            </span>
          </div>

          <!-- Mini EQ preview graph -->
          <div class="aeq-graph-wrap">
            <svg viewBox="0 0 400 120" preserveAspectRatio="none" class="aeq-graph">
              <line x1="0" y1="60" x2="400" y2="60" class="aeq-g-zero" />
              <path d="{computeProfilePath(profile)} L400,60 L0,60 Z" fill="var(--accent-dim)" />
              <path d={computeProfilePath(profile)} fill="none" stroke="var(--accent)" stroke-width="1.5" />
            </svg>
          </div>

          <!-- Filters table -->
          <div class="aeq-filters">
            <div class="aeq-filter-header">
              <span class="aeq-fh">#</span>
              <span class="aeq-fh">Type</span>
              <span class="aeq-fh aeq-fh-r">Freq</span>
              <span class="aeq-fh aeq-fh-r">Gain</span>
              <span class="aeq-fh aeq-fh-r">Q</span>
            </div>
            {#each profile.filters as f, i}
              <div class="aeq-filter-row" class:aeq-filter-disabled={!f.enabled}>
                <span class="aeq-fc aeq-fc-idx">{i + 1}</span>
                <span class="aeq-fc aeq-fc-type">{f.filter_type}</span>
                <span class="aeq-fc aeq-fc-r">{f.frequency >= 1000 ? `${(f.frequency/1000).toFixed(1)}k` : f.frequency.toFixed(0)} Hz</span>
                <span class="aeq-fc aeq-fc-r" class:pos={f.gain > 0} class:neg={f.gain < 0}>{f.gain > 0 ? '+' : ''}{f.gain.toFixed(1)} dB</span>
                <span class="aeq-fc aeq-fc-r">{f.q.toFixed(2)}</span>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="aeq-placeholder">
          <p>Could not load EQ profile for this headphone.</p>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .aeq-page { display: flex; flex-direction: column; gap: 12px; height: 100%; }

  .aeq-toolbar {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }
  .aeq-toolbar h2 { font-size: 17px; font-weight: 700; letter-spacing: -0.3px; }
  .aeq-count { font-size: 11px; color: var(--text-3); font-family: var(--mono); }

  .aeq-layout {
    display: flex;
    gap: 14px;
    flex: 1;
    min-height: 0;
  }

  /* Search & List */
  .aeq-list-panel {
    width: 320px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .aeq-search-wrap {
    position: relative;
    display: flex;
    align-items: center;
  }

  .aeq-search-icon {
    position: absolute;
    left: 10px;
    color: var(--text-3);
    pointer-events: none;
  }

  .aeq-search {
    width: 100%;
    padding: 8px 12px 8px 32px;
    background: var(--bg-surface);
    color: var(--text-0);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-m);
    font-size: 12px;
    font-family: var(--font);
    transition: border-color 0.15s var(--ease);
  }

  .aeq-search:focus { border-color: var(--accent); outline: none; }
  .aeq-search::placeholder { color: var(--text-3); }

  .aeq-loading {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 20px;
    color: var(--text-2);
    font-size: 12px;
    justify-content: center;
  }

  .spinner-sm {
    width: 14px;
    height: 14px;
    border: 2px solid var(--bg-4);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }

  @keyframes spin { to { transform: rotate(360deg); } }

  .aeq-list {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-l);
    background: var(--bg-surface);
    padding: 4px;
  }

  .aeq-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 8px 10px;
    background: transparent;
    color: var(--text-1);
    text-align: left;
    border-radius: var(--radius-s);
    transition: all 0.1s var(--ease);
  }

  .aeq-item:hover { background: var(--bg-3); }

  .aeq-item-active {
    background: var(--accent-dim) !important;
    color: var(--text-0);
  }

  .aeq-item-name {
    font-size: 12px;
    font-weight: 500;
    line-height: 1.3;
  }

  .aeq-item-source {
    font-size: 10px;
    color: var(--text-3);
  }

  .aeq-item-active .aeq-item-source { color: var(--accent-strong); }

  .aeq-empty {
    padding: 20px;
    text-align: center;
    color: var(--text-3);
    font-size: 12px;
  }

  /* Detail panel */
  .aeq-detail-panel {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .aeq-placeholder {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    color: var(--text-3);
    font-size: 13px;
  }

  .aeq-profile {
    display: flex;
    flex-direction: column;
    gap: 12px;
    height: 100%;
  }

  .aeq-profile-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
  }

  .aeq-profile-head h3 {
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.2px;
  }

  .aeq-source-badge {
    font-size: 10px;
    font-weight: 500;
    color: var(--teal);
    background: var(--teal-dim);
    padding: 1px 6px;
    border-radius: 3px;
    letter-spacing: 0.3px;
  }

  .aeq-apply-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .tb-btn {
    padding: 7px 16px;
    font-size: 12px;
    font-weight: 600;
    border-radius: var(--radius-m);
    transition: all 0.15s var(--ease);
  }

  .tb-btn.primary {
    background: var(--accent);
    color: white;
  }

  .tb-btn.primary:hover { filter: brightness(1.1); }
  .tb-btn:disabled { opacity: 0.4; cursor: not-allowed; }

  .aeq-preamp {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 14px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-m);
  }

  .aeq-preamp-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .aeq-preamp-val {
    font-family: var(--mono);
    font-size: 13px;
    font-weight: 500;
    color: var(--text-0);
  }

  .pos { color: var(--accent-strong) !important; }
  .neg { color: var(--rose) !important; }

  /* Graph */
  .aeq-graph-wrap {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-l);
    padding: 8px 12px;
  }

  .aeq-graph { width: 100%; height: 120px; }

  .aeq-g-zero { stroke: var(--border-strong); stroke-width: 0.5; }

  /* Filters table */
  .aeq-filters {
    flex: 1;
    overflow-y: auto;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-l);
    padding: 4px;
  }

  .aeq-filter-header {
    display: flex;
    padding: 6px 10px;
    gap: 8px;
  }

  .aeq-fh {
    font-size: 9px;
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.6px;
    flex: 1;
  }

  .aeq-fh-r { text-align: right; }

  .aeq-filter-row {
    display: flex;
    padding: 5px 10px;
    gap: 8px;
    border-radius: var(--radius-s);
    transition: background 0.1s var(--ease);
  }

  .aeq-filter-row:hover { background: var(--bg-3); }

  .aeq-filter-disabled { opacity: 0.4; }

  .aeq-fc {
    flex: 1;
    font-size: 12px;
    font-family: var(--mono);
    color: var(--text-1);
  }

  .aeq-fc-idx {
    color: var(--text-3);
    font-weight: 600;
    flex: 0 0 20px;
  }

  .aeq-fc-type {
    font-weight: 500;
    color: var(--text-2);
  }

  .aeq-fc-r { text-align: right; }
</style>
