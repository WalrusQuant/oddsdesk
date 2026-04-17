<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import EvPanel from './panels/EvPanel.svelte';
  import ArbPanel from './panels/ArbPanel.svelte';
  import MiddlesPanel from './panels/MiddlesPanel.svelte';
</script>

<aside class="panels">
  {#if app.visiblePanels.ev}
    <EvPanel />
  {/if}
  {#if app.visiblePanels.arb}
    <ArbPanel />
  {/if}
  {#if app.visiblePanels.middles}
    <MiddlesPanel />
  {/if}
  {#if !app.visiblePanels.ev && !app.visiblePanels.arb && !app.visiblePanels.middles}
    <div class="empty">
      <div class="title">Panels closed</div>
      <div class="hint">
        Press <kbd>e</kbd>, <kbd>a</kbd>, or <kbd>m</kbd> to show EV / arb / middles
      </div>
    </div>
  {/if}
</aside>

<style>
  .panels {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--border);
    overflow-y: auto;
    border-left: 1px solid var(--border);
  }
  .empty {
    padding: var(--sp-5) var(--sp-4);
    background: var(--bg);
    color: var(--text-muted);
    text-align: center;
    flex: 1;
  }
  .empty .title {
    font-size: var(--fs-sm);
    color: var(--text);
    margin-bottom: var(--sp-1);
  }
  .empty .hint {
    font-size: var(--fs-xs);
    color: var(--text-dim);
  }
  kbd {
    display: inline-block;
    padding: 1px 4px;
    border: 1px solid var(--border);
    border-radius: 3px;
    font-family: var(--font-mono);
    font-size: 10px;
    background: var(--surface);
    color: var(--text-muted);
  }
</style>
