<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    onclick?: (e: MouseEvent) => void;
    title?: string;
    active?: boolean;
    children: Snippet;
  }

  let { onclick, title, active = false, children }: Props = $props();
</script>

<button class="icon-btn" class:active {onclick} {title} aria-label={title}>
  {@render children()}
</button>

<style>
  .icon-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: 1px solid transparent;
    border-radius: var(--r-2);
    background: transparent;
    color: var(--text-muted);
    cursor: pointer;
    transition: color 0.12s, background 0.12s, border-color 0.12s;
  }
  .icon-btn:hover {
    color: var(--text);
    background: var(--surface-2);
    border-color: var(--border);
  }
  .icon-btn.active {
    color: var(--accent);
    background: var(--surface-2);
    border-color: var(--border);
  }
</style>
