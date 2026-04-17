<script lang="ts">
  import { budget } from '$lib/stores/budget.svelte';

  const b = $derived(budget.current);
</script>

{#if b?.is_critical}
  <div class="banner danger">
    <span class="icon">⚠</span>
    <span>{b.warning_text}</span>
  </div>
{:else if b?.is_low}
  <div class="banner warn">
    <span class="icon">⚠</span>
    <span>{b.warning_text}</span>
  </div>
{/if}

<style>
  .banner {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    padding: 6px var(--sp-4);
    font-size: var(--fs-xs);
    font-weight: 500;
    letter-spacing: 0.02em;
  }
  .banner.danger {
    background: rgba(248, 113, 113, 0.12);
    color: var(--danger);
    border-bottom: 1px solid rgba(248, 113, 113, 0.3);
  }
  .banner.warn {
    background: rgba(251, 191, 36, 0.08);
    color: var(--warning);
    border-bottom: 1px solid rgba(251, 191, 36, 0.2);
  }
  .icon {
    font-size: var(--fs-md);
    line-height: 1;
  }
</style>
