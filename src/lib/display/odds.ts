export function americanToDecimal(american: number): number {
  if (american === 0) return 1.0;
  if (american >= 100) return american / 100 + 1;
  return 100 / Math.abs(american) + 1;
}

export function americanToImpliedProb(american: number): number {
  if (american === 0) return 0;
  if (american < 0) return Math.abs(american) / (Math.abs(american) + 100);
  return 100 / (american + 100);
}

export function probToAmerican(prob: number): number {
  if (prob <= 0 || prob >= 1) return 0;
  if (prob >= 0.5) return -(prob / (1 - prob)) * 100;
  return ((1 - prob) / prob) * 100;
}
