<script lang="ts">
  type Option = { label: string; value: number };

  let {
    options,
    value = $bindable(),
    onchange,
    placeholder = 'Select...',
    width = 'auto',
  }: {
    options: Option[];
    value: number;
    onchange?: (val: number) => void;
    placeholder?: string;
    width?: string;
  } = $props();

  let open = $state(false);
  let el: HTMLDivElement;

  const selected = $derived(options.find(o => o.value === value));

  function toggle() { open = !open; }

  function pick(opt: Option) {
    value = opt.value;
    open = false;
    onchange?.(opt.value);
  }

  function handleClickOutside(e: MouseEvent) {
    if (el && !el.contains(e.target as Node)) open = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') open = false;
  }
</script>

<svelte:window onclick={handleClickOutside} onkeydown={handleKeydown} />

<div class="dd" bind:this={el} style:width={width} class:dd-open={open}>
  <button class="dd-trigger" onclick={toggle} type="button">
    <span class="dd-text">{selected?.label || placeholder}</span>
    <svg class="dd-chevron" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><polyline points="6 9 12 15 18 9"/></svg>
  </button>

  {#if open}
    <div class="dd-menu">
      {#each options as opt}
        <button
          class="dd-item"
          class:dd-item-active={opt.value === value}
          onclick={() => pick(opt)}
          type="button"
        >
          {opt.label}
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .dd {
    position: relative;
    display: inline-flex;
  }

  .dd-trigger {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    width: 100%;
    padding: 6px 10px;
    background: var(--bg-3);
    color: var(--text-0);
    border: 1px solid var(--border-default);
    border-radius: var(--radius-m);
    font-size: 12px;
    font-weight: 500;
    font-family: var(--font);
    cursor: pointer;
    transition: border-color 0.15s var(--ease);
  }

  .dd-trigger:hover { border-color: var(--border-strong); }
  .dd-open .dd-trigger { border-color: var(--accent); }

  .dd-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dd-chevron {
    flex-shrink: 0;
    opacity: 0.4;
    transition: transform 0.15s var(--ease);
  }

  .dd-open .dd-chevron { transform: rotate(180deg); }

  .dd-menu {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    min-width: 100%;
    max-height: 240px;
    overflow-y: auto;
    background: var(--bg-elevated, var(--bg-3));
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-m);
    padding: 3px;
    z-index: 50;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
  }

  .dd-item {
    display: block;
    width: 100%;
    padding: 6px 10px;
    font-size: 12px;
    font-weight: 400;
    color: var(--text-1);
    background: transparent;
    border-radius: var(--radius-s);
    text-align: left;
    cursor: pointer;
    transition: all 0.1s var(--ease);
    font-family: var(--font);
    white-space: nowrap;
  }

  .dd-item:hover {
    background: var(--bg-4);
    color: var(--text-0);
  }

  .dd-item-active {
    color: var(--accent-strong) !important;
    background: var(--accent-dim);
    font-weight: 600;
  }
</style>
