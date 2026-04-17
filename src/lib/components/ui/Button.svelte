<script lang="ts">
  import type { Snippet } from 'svelte';

  type Variant = 'primary' | 'subtle' | 'ghost';

  interface Props {
    variant?: Variant;
    disabled?: boolean;
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
    title?: string;
  }

  let {
    variant = 'subtle',
    disabled = false,
    onclick,
    children,
    title,
  }: Props = $props();
</script>

<button class="btn {variant}" {disabled} {onclick} {title}>
  {@render children()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 6px 12px;
    border: 1px solid var(--border);
    border-radius: var(--r-2);
    background: var(--surface);
    color: var(--text);
    font-size: var(--fs-sm);
    cursor: pointer;
    transition: border-color 0.12s, background 0.12s;
  }
  .btn:hover:not(:disabled) {
    border-color: var(--border-hover);
    background: var(--surface-2);
  }
  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .btn.primary {
    background: var(--accent);
    border-color: var(--accent);
    color: #0b0d10;
    font-weight: 500;
  }
  .btn.primary:hover:not(:disabled) {
    background: #7cb4fb;
    border-color: #7cb4fb;
  }
  .btn.ghost {
    background: transparent;
    border-color: transparent;
  }
  .btn.ghost:hover:not(:disabled) {
    background: var(--surface-2);
    border-color: transparent;
  }
</style>
