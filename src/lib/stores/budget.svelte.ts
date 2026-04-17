import type { BudgetState } from '../bindings';

class BudgetStore {
  current = $state<BudgetState | null>(null);
  loading = $state<boolean>(false);
}

export const budget = new BudgetStore();
