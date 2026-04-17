export function formatOdds(price: number): string {
  const rounded = Math.round(price);
  return rounded >= 0 ? `+${rounded}` : `${rounded}`;
}

export function formatEv(pct: number): string {
  const sign = pct >= 0 ? '+' : '';
  return `${sign}${pct.toFixed(1)}%`;
}

export function truncate(s: string, n: number): string {
  if (s.length <= n) return s;
  return s.slice(0, n - 1) + '…';
}

export function formatMinutesAgo(iso: string | null): string {
  if (!iso) return '';
  const dt = new Date(iso).getTime();
  if (Number.isNaN(dt)) return '';
  const mins = Math.floor((Date.now() - dt) / 60_000);
  if (mins < 1) return 'now';
  if (mins < 60) return `${mins}m`;
  return `${Math.floor(mins / 60)}h${mins % 60}m`;
}

export function formatTime(iso: string): string {
  const d = new Date(iso);
  const h = d.getHours();
  const m = d.getMinutes();
  const h12 = h % 12 || 12;
  const ampm = h >= 12 ? 'pm' : 'am';
  return `${h12}:${m.toString().padStart(2, '0')}${ampm}`;
}
