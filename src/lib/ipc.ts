import { commands, type Result } from './bindings';
import type { Settings } from './bindings';

async function unwrap<T>(p: Promise<Result<T, string>>): Promise<T> {
  const r = await p;
  if (r.status === 'error') throw new Error(r.error);
  return r.data;
}

export const api = {
  listSports: () => unwrap(commands.listSports()),
  loadGames: (sport: string) => unwrap(commands.loadGames(sport)),
  loadProps: (sport: string) => unwrap(commands.loadProps(sport)),
  findEv: (sport: string) => unwrap(commands.findEv(sport)),
  findPropEv: (sport: string) => unwrap(commands.findPropEv(sport)),
  findArbs: (sport: string) => unwrap(commands.findArbs(sport)),
  findPropArbs: (sport: string) => unwrap(commands.findPropArbs(sport)),
  findMiddles: (sport: string) => unwrap(commands.findMiddles(sport)),
  findPropMiddles: (sport: string) => unwrap(commands.findPropMiddles(sport)),
  storedEv: (sport: string, isProps: boolean) => unwrap(commands.storedEv(sport, isProps)),
  getBudget: () => unwrap(commands.getBudget()),
  getSettings: () => unwrap(commands.getSettings()),
  saveSettings: (update: Settings) => unwrap(commands.saveSettings(update)),
  setAltLines: (enabled: boolean) => unwrap(commands.setAltLines(enabled)),
  forceRefresh: (sport: string) => unwrap(commands.forceRefresh(sport)),
};
