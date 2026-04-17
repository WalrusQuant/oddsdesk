<script lang="ts">
  import { app } from '$lib/stores/app.svelte';
  import Topbar from '$lib/components/Topbar.svelte';
  import GamesTable from '$lib/components/GamesTable.svelte';
  import PropsTable from '$lib/components/PropsTable.svelte';
  import SidePanels from '$lib/components/SidePanels.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import SettingsDrawer from '$lib/components/SettingsDrawer.svelte';
  import CreditBanner from '$lib/components/CreditBanner.svelte';
  import Toast from '$lib/components/Toast.svelte';
</script>

<div class="shell">
  <Topbar />
  <CreditBanner />
  <main class="main">
    <section class="content">
      {#if app.viewMode === 'games'}
        <GamesTable />
      {:else}
        <PropsTable />
      {/if}
    </section>
    <SidePanels />
  </main>
  <StatusBar />
  {#if app.settingsDrawerOpen}
    <SettingsDrawer />
  {/if}
  <Toast />
</div>

<style>
  .shell {
    display: grid;
    grid-template-rows: var(--topbar-h) auto 1fr var(--statusbar-h);
    height: 100vh;
    min-height: 0;
  }
  .main {
    display: grid;
    grid-template-columns: 1fr var(--panel-w);
    min-height: 0;
    overflow: hidden;
  }
  .content {
    overflow: auto;
    background: var(--bg);
  }
</style>
