<script lang="ts" generics="T extends string">
  interface Option {
    key: T;
    label: string;
  }

  interface Props {
    options: Option[];
    value: T;
    onchange: (v: T) => void;
    compact?: boolean;
  }

  let { options, value, onchange, compact = false }: Props = $props();
</script>

<div class="segmented" class:compact>
  {#each options as opt (opt.key)}
    <button
      class="seg"
      class:active={opt.key === value}
      onclick={() => onchange(opt.key)}
    >
      {opt.label}
    </button>
  {/each}
</div>

<style>
  .segmented {
    display: inline-flex;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    padding: 2px;
    gap: 2px;
  }
  .seg {
    padding: 4px 10px;
    border: 1px solid transparent;
    border-radius: var(--r-1);
    background: transparent;
    color: var(--text-muted);
    font-size: var(--fs-xs);
    font-weight: 500;
    cursor: pointer;
    transition: color 0.12s, background 0.12s;
    letter-spacing: 0.03em;
  }
  .seg:hover:not(.active) {
    color: var(--text);
    background: var(--surface-2);
  }
  .seg.active {
    background: var(--surface-2);
    color: var(--text);
    border-color: var(--border-hover);
  }
  .segmented.compact .seg {
    padding: 3px 8px;
    font-size: 10px;
  }
</style>
