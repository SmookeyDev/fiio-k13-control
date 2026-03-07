<script lang="ts">
  import { eq } from '$lib/store.svelte';
  import * as api from '$lib/api';
  import { EQ_PRESETS, FilterTypeLabels } from '$lib/api';
  import Dropdown from './Dropdown.svelte';

  const filterShortLabels: Record<number, string> = {
    0: 'PK', 1: 'LS', 2: 'HS', 3: 'BP', 4: 'LP', 5: 'HP', 6: 'AP',
  };

  const filterOptions = Object.entries(FilterTypeLabels).map(([val, label]) => ({ label, value: parseInt(val) }));

  let saving = $state(false);

  const freqLabel = (f: number) => f >= 1000 ? `${(f / 1000).toFixed(f % 1000 === 0 ? 0 : 1)}k` : `${f}`;

  async function onBandChange(index: number) {
    const band = eq.bands[index];
    if (!band) return;
    try { await api.setEqBand(index, band.frequency, band.gain, band.q_value, band.filter_type); }
    catch (e) { console.error('Failed to set EQ band:', e); }
  }

  async function onGlobalGainChange() {
    try { await api.setEqGlobalGain(eq.globalGain); } catch (e) { console.error(e); }
  }

  async function onEqSwitchChange() {
    eq.enabled = !eq.enabled;
    try { await api.setEqSwitch(eq.enabled); } catch (e) { console.error(e); }
  }

  async function handleSave() {
    saving = true;
    try { await api.saveEq(eq.preset); } catch (e) { console.error(e); }
    finally { saving = false; }
  }

  async function handleReset() {
    try {
      await api.resetEq();
      eq.bands = await api.getAllEqBands();
      eq.globalGain = await api.getEqGlobalGain().catch(() => 0);
    } catch (e) { console.error(e); }
  }

  function computeResponsePath(w: number, h: number): string {
    if (!eq.bands.length) return '';
    const minF = Math.log10(20), maxF = Math.log10(20000);
    const pts: string[] = [];
    for (let i = 0; i <= 250; i++) {
      const logF = minF + (i / 250) * (maxF - minF);
      const freq = Math.pow(10, logF);
      let gain = eq.globalGain;
      for (const b of eq.bands) {
        if (b.frequency <= 0 || b.q_value <= 0) continue;
        const x = Math.log2(freq / b.frequency) / (1 / b.q_value);
        gain += b.gain * Math.exp(-0.5 * x * x);
      }
      const xp = (i / 250) * w;
      const yp = Math.max(0, Math.min(h, h / 2 - (gain / 15) * (h / 2)));
      pts.push(`${i === 0 ? 'M' : 'L'}${xp.toFixed(1)},${yp.toFixed(1)}`);
    }
    return pts.join(' ');
  }

  function fillPath(w: number, h: number): string {
    const line = computeResponsePath(w, h);
    return line ? `${line} L${w},${h / 2} L0,${h / 2} Z` : '';
  }

  function cycleFilterType(index: number) {
    const band = eq.bands[index];
    if (!band) return;
    band.filter_type = (band.filter_type + 1) % 7;
    onBandChange(index);
  }

  const GW = 860, GH = 180;
  const freqMarks = [20, 50, 100, 200, 500, 1000, 2000, 5000, 10000, 20000];
  const dbMarks = [-12, -6, 0, 6, 12];
</script>

<div class="eq-page">
  <!-- Top bar -->
  <div class="eq-toolbar">
    <div class="toolbar-left">
      <h2>Parametric EQ</h2>
      <button class="eq-toggle" class:on={eq.enabled} onclick={onEqSwitchChange}>
        <span class="toggle-track"><span class="toggle-thumb"></span></span>
        {eq.enabled ? 'Active' : 'Bypass'}
      </button>
    </div>
    <div class="toolbar-right">
      <button class="tb-btn ghost" onclick={handleReset}>Reset</button>
      <button class="tb-btn primary" onclick={handleSave} disabled={saving}>
        {saving ? 'Saving...' : 'Save to Device'}
      </button>
    </div>
  </div>

  <!-- Controls row -->
  <div class="controls-bar">
    <div class="ctrl-group">
      <label class="ctrl-label">Preset</label>
      <Dropdown options={EQ_PRESETS} bind:value={eq.preset} onchange={async (val) => {
        try {
          await api.setEqPreset(val);
          if (val !== 240) {
            eq.bands = await api.getAllEqBands();
            eq.globalGain = await api.getEqGlobalGain().catch(() => 0);
          }
        } catch (e) { console.error(e); }
      }} width="140px" />
    </div>
    <div class="ctrl-group">
      <label class="ctrl-label">Pre-Amp</label>
      <div class="inline-slider">
        <input type="range" min="-12" max="12" step="0.5" bind:value={eq.globalGain} onchange={onGlobalGainChange} />
        <span class="mono-val" class:pos={eq.globalGain > 0} class:neg={eq.globalGain < 0}>
          {eq.globalGain > 0 ? '+' : ''}{eq.globalGain.toFixed(1)} dB
        </span>
      </div>
    </div>
  </div>

  <!-- Graph -->
  <div class="graph-wrap">
    <svg viewBox="0 0 {GW} {GH}" preserveAspectRatio="none" class="graph-svg">
      {#each dbMarks as db}
        {@const y = GH / 2 - (db / 15) * (GH / 2)}
        <line x1="0" y1={y} x2={GW} y2={y} class="g-line" class:g-zero={db === 0} />
        <text x={GW - 4} y={y - 4} class="g-db">{db > 0 ? '+' : ''}{db}</text>
      {/each}
      {#each freqMarks as f}
        {@const x = ((Math.log10(f) - Math.log10(20)) / (Math.log10(20000) - Math.log10(20))) * GW}
        <line x1={x} y1="0" x2={x} y2={GH} class="g-line g-vert" />
        <text x={x} y={GH - 4} class="g-freq">{freqLabel(f)}</text>
      {/each}
      <defs>
        <linearGradient id="fillGrad" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color="var(--accent)" stop-opacity="0.2"/>
          <stop offset="100%" stop-color="var(--accent)" stop-opacity="0"/>
        </linearGradient>
      </defs>
      <path d={fillPath(GW, GH)} fill="url(#fillGrad)" />
      <path d={computeResponsePath(GW, GH)} class="curve" />
      {#each eq.bands as band, i}
        {@const bx = ((Math.log10(Math.max(20, band.frequency)) - Math.log10(20)) / (Math.log10(20000) - Math.log10(20))) * GW}
        {@const by = Math.max(6, Math.min(GH - 6, GH / 2 - ((band.gain + eq.globalGain) / 15) * (GH / 2)))}
        <circle cx={bx} cy={by} r={eq.selectedBand === i ? 7 : 4.5}
          class="dot" class:dot-sel={eq.selectedBand === i}
          role="button" tabindex="0"
          onclick={() => (eq.selectedBand = i)}
          onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') eq.selectedBand = i; }}
        />
        {#if eq.selectedBand === i}
          <text x={bx} y={by - 12} class="dot-label">{i + 1}</text>
        {/if}
      {/each}
    </svg>
  </div>

  <!-- All bands grid -->
  <div class="bands-grid">
    {#each eq.bands as band, i}
      <div class="band-col" class:band-selected={eq.selectedBand === i} onclick={() => eq.selectedBand = i} role="button" tabindex="0" onkeydown={(e) => { if (e.key === 'Enter') eq.selectedBand = i; }}>
        <span class="band-num">{i + 1}</span>

        <div class="band-slider-wrap">
          <input type="range" class="v-slider" min="-12" max="12" step="0.5"
            bind:value={band.gain}
            oninput={() => onBandChange(i)}
          />
          <span class="band-gain-label" class:pos={band.gain > 0} class:neg={band.gain < 0}>
            {band.gain > 0 ? '+' : ''}{band.gain.toFixed(1)}
          </span>
        </div>

        <div class="band-inputs">
          <div class="band-field">
            <label>Freq</label>
            <input type="number" min="20" max="20000" step="1"
              bind:value={band.frequency}
              onchange={() => onBandChange(i)}
            />
          </div>
          <div class="band-field">
            <label>Q</label>
            <input type="number" min="0.1" max="10" step="0.01"
              bind:value={band.q_value}
              onchange={() => onBandChange(i)}
            />
          </div>
        </div>

        <button class="filter-btn" onclick={() => cycleFilterType(i)} title={FilterTypeLabels[band.filter_type]}>
          {filterShortLabels[band.filter_type] || 'PK'}
        </button>
      </div>
    {/each}
  </div>

  <!-- Selected band detail row -->
  {#if eq.bands[eq.selectedBand]}
    {@const band = eq.bands[eq.selectedBand]}
    {@const idx = eq.selectedBand}
    <div class="detail-bar">
      <span class="detail-title">Band {idx + 1}</span>
      <div class="detail-param">
        <label>Frequency</label>
        <input type="range" min="20" max="20000" step="1" bind:value={band.frequency} oninput={() => onBandChange(idx)} />
        <span class="detail-val">{freqLabel(band.frequency)} Hz</span>
      </div>
      <div class="detail-param">
        <label>Gain</label>
        <input type="range" min="-12" max="12" step="0.5" bind:value={band.gain} oninput={() => onBandChange(idx)} />
        <span class="detail-val" class:pos={band.gain > 0} class:neg={band.gain < 0}>{band.gain > 0 ? '+' : ''}{band.gain.toFixed(1)} dB</span>
      </div>
      <div class="detail-param">
        <label>Q</label>
        <input type="range" min="0.1" max="10" step="0.1" bind:value={band.q_value} oninput={() => onBandChange(idx)} />
        <span class="detail-val">{band.q_value.toFixed(2)}</span>
      </div>
      <div class="detail-param detail-type">
        <label>Type</label>
        <Dropdown options={filterOptions} bind:value={band.filter_type} onchange={() => onBandChange(idx)} width="110px" />
      </div>
    </div>
  {/if}
</div>

<style>
  .eq-page { display: flex; flex-direction: column; gap: 12px; height: 100%; }

  .eq-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .toolbar-left { display: flex; align-items: center; gap: 14px; }
  .toolbar-left h2 { font-size: 17px; font-weight: 700; letter-spacing: -0.3px; }

  .toolbar-right { display: flex; gap: 8px; }

  .eq-toggle {
    display: flex;
    align-items: center;
    gap: 7px;
    background: none;
    color: var(--text-3);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.3px;
    padding: 0;
  }

  .eq-toggle.on { color: var(--green); }

  .toggle-track {
    display: flex;
    align-items: center;
    width: 32px;
    height: 18px;
    background: var(--bg-4);
    border-radius: 10px;
    padding: 2px;
    transition: background 0.2s var(--ease);
  }

  .eq-toggle.on .toggle-track { background: var(--green); }

  .toggle-thumb {
    width: 14px;
    height: 14px;
    background: white;
    border-radius: 50%;
    transition: transform 0.2s var(--ease);
    box-shadow: 0 1px 3px rgba(0,0,0,0.3);
  }

  .eq-toggle.on .toggle-thumb { transform: translateX(14px); }

  .tb-btn {
    padding: 7px 16px;
    font-size: 12px;
    font-weight: 600;
    border-radius: var(--radius-m);
    transition: all 0.15s var(--ease);
  }

  .tb-btn.ghost {
    background: var(--bg-3);
    color: var(--text-2);
  }

  .tb-btn.ghost:hover { background: var(--bg-4); color: var(--text-1); }

  .tb-btn.primary {
    background: var(--accent);
    color: white;
  }

  .tb-btn.primary:hover { filter: brightness(1.1); }
  .tb-btn:disabled { opacity: 0.4; cursor: not-allowed; }

  /* Controls bar */
  .controls-bar {
    display: flex;
    align-items: center;
    gap: 28px;
    padding: 10px 16px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-l);
  }

  .ctrl-group { display: flex; align-items: center; gap: 10px; }
  .ctrl-label { font-size: 11px; color: var(--text-3); font-weight: 500; text-transform: uppercase; letter-spacing: 0.6px; }

  .inline-slider { display: flex; align-items: center; gap: 10px; }
  .inline-slider input[type="range"] { width: 120px; }

  .mono-val {
    font-family: var(--mono);
    font-size: 12px;
    color: var(--text-1);
    min-width: 60px;
    text-align: right;
  }

  .mono-val.pos, .pos { color: var(--accent-strong); }
  .mono-val.neg, .neg { color: var(--rose); }

  /* Graph */
  .graph-wrap {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-l);
    padding: 10px 16px 6px;
    flex-shrink: 0;
  }

  .graph-svg { width: 100%; height: 180px; }

  .g-line { stroke: var(--border-default); stroke-width: 0.5; }
  .g-line.g-zero { stroke: var(--border-strong); stroke-width: 0.8; }
  .g-line.g-vert { stroke-dasharray: 2 5; }
  .g-db { font-size: 8px; fill: var(--text-3); font-family: var(--mono); text-anchor: end; }
  .g-freq { font-size: 8px; fill: var(--text-3); font-family: var(--mono); text-anchor: middle; }

  .curve {
    fill: none;
    stroke: var(--accent);
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
    filter: drop-shadow(0 0 6px var(--accent-dim));
  }

  .dot {
    fill: var(--bg-surface);
    stroke: var(--accent);
    stroke-width: 2;
    cursor: pointer;
    transition: all 0.12s var(--ease);
  }

  .dot-sel {
    fill: var(--accent);
    stroke: white;
    stroke-width: 2;
    filter: drop-shadow(0 0 8px var(--accent-dim));
  }

  .dot:hover { r: 7; }

  .dot-label {
    font-size: 9px;
    fill: var(--accent-strong);
    text-anchor: middle;
    font-weight: 700;
    font-family: var(--mono);
  }

  /* Bands grid */
  .bands-grid {
    display: flex;
    gap: 4px;
  }

  .band-col {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 10px 4px 8px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-m);
    cursor: pointer;
    transition: all 0.12s var(--ease);
  }

  .band-col:hover { border-color: var(--border-strong); }

  .band-selected {
    border-color: rgba(59, 130, 246, 0.3) !important;
    background: rgba(59, 130, 246, 0.04);
  }

  .band-num {
    font-size: 10px;
    font-weight: 700;
    font-family: var(--mono);
    color: var(--text-3);
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    background: var(--bg-3);
  }

  .band-selected .band-num {
    background: var(--accent-dim);
    color: var(--accent-strong);
  }

  .band-slider-wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    height: 110px;
  }

  .v-slider {
    writing-mode: vertical-lr;
    direction: rtl;
    -webkit-appearance: slider-vertical;
    appearance: slider-vertical;
    width: 20px;
    height: 90px;
  }

  .band-gain-label {
    font-size: 10px;
    font-family: var(--mono);
    font-weight: 500;
    color: var(--text-2);
  }

  .band-inputs {
    display: flex;
    flex-direction: column;
    gap: 3px;
    width: 100%;
    padding: 0 2px;
  }

  .band-field {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .band-field label {
    font-size: 8px;
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    text-align: center;
  }

  .band-field input[type="number"] {
    width: 100%;
    padding: 3px 2px;
    font-size: 10px;
    font-family: var(--mono);
    text-align: center;
    background: var(--bg-3);
    color: var(--text-1);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-s);
  }

  .band-field input[type="number"]:focus {
    border-color: var(--accent);
    outline: none;
  }

  /* Hide number spinners */
  .band-field input[type="number"]::-webkit-inner-spin-button,
  .band-field input[type="number"]::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }
  .band-field input[type="number"] {
    -moz-appearance: textfield;
  }

  .filter-btn {
    font-size: 10px;
    font-weight: 700;
    font-family: var(--mono);
    padding: 3px 8px;
    background: var(--bg-3);
    color: var(--text-2);
    border-radius: var(--radius-s);
    transition: all 0.1s var(--ease);
    letter-spacing: 0.3px;
  }

  .filter-btn:hover {
    background: var(--accent-dim);
    color: var(--accent-strong);
  }

  /* Detail bar */
  .detail-bar {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 10px 16px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-l);
  }

  .detail-title {
    font-size: 12px;
    font-weight: 700;
    color: var(--accent-strong);
    font-family: var(--mono);
    min-width: 50px;
  }

  .detail-param {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .detail-param label {
    font-size: 10px;
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    min-width: 30px;
  }

  .detail-param input[type="range"] {
    flex: 1;
    min-width: 60px;
  }

  .detail-val {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--text-1);
    min-width: 55px;
    text-align: right;
  }

  .detail-type {
    flex: 0 0 auto;
  }
</style>
