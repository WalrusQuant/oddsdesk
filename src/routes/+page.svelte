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
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
  }
  .main {
    display: grid;
    grid-template-columns: 1fr var(--panel-w);
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  .content {
    overflow: auto;
    background: var(--bg);
    min-height: 0;
  }
</style>
