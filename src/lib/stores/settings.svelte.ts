import type { Settings } from '../bindings';

class SettingsStore {
  current = $state<Settings | null>(null);
  loading = $state<boolean>(false);
}

export const settings = new SettingsStore();
