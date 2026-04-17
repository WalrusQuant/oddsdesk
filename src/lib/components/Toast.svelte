<script lang="ts">
  import { data } from '$lib/stores/data.svelte';

  interface ToastItem {
    id: number;
    message: string;
  }

  let visible = $state<ToastItem[]>([]);
  // Non-reactive on purpose: reading and writing $state inside the same
  // $effect risks a re-entry loop; plain let sidesteps reactive tracking.
  let seen = 0;
  let nextId = 0;

  $effect(() => {
    const errs = data.errors;
    if (errs.length <= seen) return;
    const msg = errs[errs.length - 1];
    seen = errs.length;
    const id = ++nextId;
    visible = [...visible, { id, message: msg }];
    const timer = setTimeout(() => {
      visible = visible.filter((v) => v.id !== id);
    }, 6000);
    return () => clearTimeout(timer);
  });

  function dismiss(id: number) {
    visible = visible.filter((v) => v.id !== id);
  }
</script>

{#if visible.length > 0}
  <div class="toasts" role="status" aria-live="polite">
    {#each visible as t (t.id)}
      <button class="toast" onclick={() => dismiss(t.id)} title="Dismiss">
        <span class="icon" aria-hidden="true">⚠</span>
        <span class="msg">{t.message}</span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .toasts {
    position: fixed;
    bottom: calc(var(--statusbar-h) + var(--sp-3));
    right: var(--sp-4);
    display: flex;
    flex-direction: column;
    gap: var(--sp-2);
    z-index: 200;
    max-width: 380px;
  }
  .toast {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: var(--sp-2);
    align-items: flex-start;
    padding: var(--sp-2) var(--sp-3);
    background: var(--surface);
    border: 1px solid var(--danger);
    border-left-width: 3px;
    border-radius: var(--r-2);
    color: var(--text);
    font-size: var(--fs-sm);
    text-align: left;
    cursor: pointer;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
    animation: slide-up 0.2s ease-out;
  }
  .toast:hover {
    background: var(--surface-2);
  }
  .icon {
    color: var(--danger);
    font-size: var(--fs-md);
    line-height: 1;
    margin-top: 1px;
  }
  .msg {
    word-break: break-word;
  }
  @keyframes slide-up {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
