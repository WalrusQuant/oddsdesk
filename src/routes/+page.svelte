<script lang="ts">
  let ticks = $state(0);
  let now = $derived(new Date().toLocaleTimeString());

  function bump() {
    ticks += 1;
  }
</script>

<main class="shell">
  <header>
    <h1>OddsDesk</h1>
    <span class="tag">Phase 0 scaffold</span>
  </header>

  <section class="status">
    <div>Runes check: <strong>{ticks}</strong> clicks</div>
    <div>Clock: {now}</div>
    <button onclick={bump}>+1</button>
  </section>

  <p class="hint">Tauri v2 + SvelteKit (SPA) + Svelte 5 runes. Ready for Phase 1.</p>
</main>

<style>
  :global(:root) {
    color-scheme: dark;
    font-family: "JetBrains Mono", ui-monospace, SFMono-Regular, Menlo, monospace;
    background: #0b0d10;
    color: #e6e8eb;
  }

  :global(body) {
    margin: 0;
  }

  .shell {
    max-width: 960px;
    margin: 0 auto;
    padding: 48px 32px;
  }

  header {
    display: flex;
    align-items: baseline;
    gap: 16px;
    border-bottom: 1px solid #1f2937;
    padding-bottom: 12px;
    margin-bottom: 24px;
  }

  h1 {
    margin: 0;
    font-size: 24px;
    letter-spacing: 0.02em;
  }

  .tag {
    font-size: 11px;
    text-transform: uppercase;
    color: #6b7280;
    letter-spacing: 0.1em;
  }

  .status {
    display: grid;
    grid-template-columns: 1fr 1fr auto;
    gap: 16px;
    align-items: center;
    padding: 16px 20px;
    background: #11151a;
    border: 1px solid #1f2937;
    border-radius: 8px;
    margin-bottom: 16px;
  }

  button {
    background: #1f2937;
    color: #e6e8eb;
    border: 1px solid #374151;
    border-radius: 6px;
    padding: 6px 14px;
    font-family: inherit;
    cursor: pointer;
  }

  button:hover {
    border-color: #60a5fa;
  }

  .hint {
    color: #6b7280;
    font-size: 13px;
  }
</style>
