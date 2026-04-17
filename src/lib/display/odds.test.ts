import { describe, it, expect } from 'vitest';
import { americanToDecimal, americanToImpliedProb, probToAmerican } from './odds';

describe('americanToDecimal', () => {
  it('positive odds', () => {
    expect(americanToDecimal(100)).toBeCloseTo(2.0, 9);
    expect(americanToDecimal(200)).toBeCloseTo(3.0, 9);
    expect(americanToDecimal(500)).toBeCloseTo(6.0, 9);
  });
  it('negative odds', () => {
    expect(americanToDecimal(-100)).toBeCloseTo(2.0, 9);
    expect(americanToDecimal(-200)).toBeCloseTo(1.5, 9);
  });
  it('zero', () => {
    expect(americanToDecimal(0)).toBe(1.0);
  });
});

describe('americanToImpliedProb', () => {
  it('even', () => {
    expect(americanToImpliedProb(100)).toBeCloseTo(0.5, 9);
    expect(americanToImpliedProb(-100)).toBeCloseTo(0.5, 9);
  });
  it('favorite', () => {
    expect(americanToImpliedProb(-200)).toBeCloseTo(2 / 3, 9);
  });
  it('underdog', () => {
    expect(americanToImpliedProb(200)).toBeCloseTo(1 / 3, 9);
  });
  it('zero', () => {
    expect(americanToImpliedProb(0)).toBe(0);
  });
});

describe('probToAmerican', () => {
  it('favorite', () => {
    expect(probToAmerican(2 / 3)).toBeCloseTo(-200, 0);
  });
  it('underdog', () => {
    expect(probToAmerican(1 / 3)).toBeCloseTo(200, 0);
  });
  it('even', () => {
    expect(probToAmerican(0.5)).toBeCloseTo(-100, 0);
  });
  it('edge cases', () => {
    expect(probToAmerican(0)).toBe(0);
    expect(probToAmerican(1)).toBe(0);
  });
});
