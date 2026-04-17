/**
 * Run `fn` immediately, then re-run every `intervalMs` ms.
 * Returns a cleanup function that clears the interval.
 *
 * Errors thrown by `fn` are caught and logged so one bad tick doesn't
 * tear down the whole loop.
 */
export function startPolling(
  fn: () => void | Promise<void>,
  intervalMs: number,
): () => void {
  const tick = async () => {
    try {
      await fn();
    } catch (err) {
      console.error('[polling] tick failed', err);
    }
  };
  tick();
  const id = setInterval(tick, intervalMs);
  return () => clearInterval(id);
}
