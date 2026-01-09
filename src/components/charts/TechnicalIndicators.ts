/**
 * Technical indicator calculations for chart overlays
 */

export interface IndicatorConfig {
  type: "sma" | "ema" | "rsi" | "macd" | "bollinger";
  period?: number;
  color?: string;
  visible?: boolean;
}

export interface PricePoint {
  time: number;
  close: number;
  high: number;
  low: number;
  volume: number;
}

/**
 * Simple Moving Average
 */
export function calculateSMA(data: PricePoint[], period: number): number[] {
  if (data.length < period) return [];
  
  const sma: number[] = [];
  for (let i = period - 1; i < data.length; i++) {
    const sum = data.slice(i - period + 1, i + 1).reduce((acc, p) => acc + p.close, 0);
    sma.push(sum / period);
  }
  return sma;
}

/**
 * Exponential Moving Average
 */
export function calculateEMA(data: PricePoint[], period: number): number[] {
  if (data.length < period) return [];
  
  const ema: number[] = [];
  const multiplier = 2 / (period + 1);
  
  // Start with SMA
  const firstSMA = data.slice(0, period).reduce((acc, p) => acc + p.close, 0) / period;
  ema.push(firstSMA);
  
  for (let i = period; i < data.length; i++) {
    const prevEMA = ema[ema.length - 1];
    const currentEMA = (data[i].close - prevEMA) * multiplier + prevEMA;
    ema.push(currentEMA);
  }
  
  return ema;
}

/**
 * Relative Strength Index
 */
export function calculateRSI(data: PricePoint[], period: number = 14): number[] {
  if (data.length < period + 1) return [];
  
  const rsi: number[] = [];
  const gains: number[] = [];
  const losses: number[] = [];
  
  // Calculate price changes
  for (let i = 1; i < data.length; i++) {
    const change = data[i].close - data[i - 1].close;
    gains.push(change > 0 ? change : 0);
    losses.push(change < 0 ? -change : 0);
  }
  
  // Calculate initial average gain/loss
  let avgGain = gains.slice(0, period).reduce((a, b) => a + b, 0) / period;
  let avgLoss = losses.slice(0, period).reduce((a, b) => a + b, 0) / period;
  
  // Calculate RSI for first period
  if (avgLoss === 0) {
    rsi.push(100);
  } else {
    const rs = avgGain / avgLoss;
    rsi.push(100 - (100 / (1 + rs)));
  }
  
  // Calculate RSI for remaining periods
  for (let i = period; i < gains.length; i++) {
    avgGain = (avgGain * (period - 1) + gains[i]) / period;
    avgLoss = (avgLoss * (period - 1) + losses[i]) / period;
    
    if (avgLoss === 0) {
      rsi.push(100);
    } else {
      const rs = avgGain / avgLoss;
      rsi.push(100 - (100 / (1 + rs)));
    }
  }
  
  return rsi;
}

/**
 * MACD (Moving Average Convergence Divergence)
 */
export interface MACDResult {
  macd: number[];
  signal: number[];
  histogram: number[];
}

export function calculateMACD(
  data: PricePoint[],
  fastPeriod: number = 12,
  slowPeriod: number = 26,
  signalPeriod: number = 9
): MACDResult {
  if (data.length < slowPeriod + signalPeriod) {
    return { macd: [], signal: [], histogram: [] };
  }
  
  const fastEMA = calculateEMA(data, fastPeriod);
  const slowEMA = calculateEMA(data, slowPeriod);
  
  // MACD line = fast EMA - slow EMA
  const macd: number[] = [];
  const offset = slowPeriod - fastPeriod;
  for (let i = 0; i < slowEMA.length; i++) {
    if (i + offset < fastEMA.length) {
      macd.push(fastEMA[i + offset] - slowEMA[i]);
    }
  }
  
  // Signal line = EMA of MACD
  const macdPoints: PricePoint[] = macd.map((value, idx) => ({
    time: data[slowPeriod - 1 + idx].time,
    close: value,
    high: value,
    low: value,
    volume: 0,
  }));
  
  const signal = calculateEMA(macdPoints, signalPeriod);
  
  // Histogram = MACD - Signal
  const histogram: number[] = [];
  const signalOffset = macd.length - signal.length;
  for (let i = 0; i < signal.length; i++) {
    if (i + signalOffset < macd.length) {
      histogram.push(macd[i + signalOffset] - signal[i]);
    }
  }
  
  return { macd, signal, histogram };
}

/**
 * Bollinger Bands
 */
export interface BollingerBandsResult {
  upper: number[];
  middle: number[];
  lower: number[];
}

export function calculateBollingerBands(
  data: PricePoint[],
  period: number = 20,
  stdDev: number = 2
): BollingerBandsResult {
  if (data.length < period) {
    return { upper: [], middle: [], lower: [] };
  }
  
  const sma = calculateSMA(data, period);
  const upper: number[] = [];
  const lower: number[] = [];
  
  for (let i = period - 1; i < data.length; i++) {
    const slice = data.slice(i - period + 1, i + 1);
    const mean = sma[i - period + 1];
    
    // Calculate standard deviation
    const variance = slice.reduce((acc, p) => acc + Math.pow(p.close - mean, 2), 0) / period;
    const sd = Math.sqrt(variance);
    
    upper.push(mean + stdDev * sd);
    lower.push(mean - stdDev * sd);
  }
  
  return {
    upper,
    middle: sma,
    lower,
  };
}

/**
 * Format indicator data for lightweight-charts
 */
export function formatIndicatorData(
  indicator: IndicatorConfig,
  priceData: PricePoint[],
  indicatorValues: number[] | MACDResult | BollingerBandsResult
): any[] {
  switch (indicator.type) {
    case "sma":
    case "ema": {
      const values = indicatorValues as number[];
      const offset = priceData.length - values.length;
      return values.map((value, idx) => ({
        time: priceData[offset + idx].time,
        value,
      }));
    }
    case "rsi": {
      const values = indicatorValues as number[];
      const offset = priceData.length - values.length;
      return values.map((value, idx) => ({
        time: priceData[offset + idx].time,
        value,
      }));
    }
    case "macd": {
      const macdResult = indicatorValues as MACDResult;
      const offset = priceData.length - macdResult.macd.length;
      return macdResult.macd.map((value, idx) => ({
        time: priceData[offset + idx].time,
        value,
      }));
    }
    case "bollinger": {
      const bbResult = indicatorValues as BollingerBandsResult;
      const offset = priceData.length - bbResult.upper.length;
      return bbResult.upper.map((_, idx) => ({
        time: priceData[offset + idx].time,
        upper: bbResult.upper[idx],
        middle: bbResult.middle[idx],
        lower: bbResult.lower[idx],
      }));
    }
    default:
      return [];
  }
}

