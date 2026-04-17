import type { GameRow, PropRow, EVBet, ArbBet, MiddleBet } from '../bindings';

type LoadingFlags = {
  games: boolean;
  props: boolean;
  ev: boolean;
  arbs: boolean;
  middles: boolean;
};

class DataStore {
  games = $state<GameRow[]>([]);
  props = $state<PropRow[]>([]);
  ev = $state<EVBet[]>([]);
  arbs = $state<ArbBet[]>([]);
  middles = $state<MiddleBet[]>([]);

  loading = $state<LoadingFlags>({
    games: false,
    props: false,
    ev: false,
    arbs: false,
    middles: false,
  });

  lastRefresh = $state<number | null>(null);
  errors = $state<string[]>([]);

  pushError(msg: string) {
    this.errors = [...this.errors.slice(-4), msg];
  }

  clearErrors() {
    this.errors = [];
  }
}

export const data = new DataStore();
