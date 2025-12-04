import * as VULFRAM_CORE from './napi';
import { pack, unpack } from 'msgpackr';
import type { VulframResult } from './enums';
import type { EngineBatchCmds, EngineBatchEvents } from './cmds';

export * from './cmds';
export * from './dev';
export * from './enums';
export * from './events';

// MARK: Benchmark types

export interface BenchmarkMetrics {
  /** Function name */
  name: string;
  /** Total number of calls */
  calls: number;
  /** Total time in milliseconds */
  totalMs: number;
  /** Average time in microseconds */
  avgUs: number;
  /** Average time in nanoseconds */
  avgNs: number;
  /** Minimum time in microseconds */
  minUs: number;
  /** Maximum time in microseconds */
  maxUs: number;
  /** Last call time in microseconds */
  lastUs: number;
}

interface BenchmarkData {
  calls: number;
  totalMs: number;
  minUs: number;
  maxUs: number;
  lastUs: number;
}

const benchmarks = new Map<string, BenchmarkData>();
let benchmarkEnabled = false;

/**
 * Enables or disables benchmark tracking for NAPI calls.
 * When enabled, tracks execution time for each function call.
 */
export function vulframSetBenchmark(enabled: boolean): void {
  benchmarkEnabled = enabled;
  if (!enabled) {
    benchmarks.clear();
  }
}

/**
 * Returns current benchmark metrics for all tracked functions.
 */
export function vulframGetBenchmarks(): BenchmarkMetrics[] {
  const results: BenchmarkMetrics[] = [];

  for (const [name, data] of benchmarks.entries()) {
    const avgMs = data.totalMs / data.calls;
    const avgUs = avgMs * 1000;
    const avgNs = avgUs * 1000;

    results.push({
      name,
      calls: data.calls,
      totalMs: data.totalMs,
      avgUs,
      avgNs,
      minUs: data.minUs,
      maxUs: data.maxUs,
      lastUs: data.lastUs,
    });
  }

  return results.sort((a, b) => b.totalMs - a.totalMs);
}

/**
 * Resets all benchmark statistics.
 */
export function vulframResetBenchmarks(): void {
  benchmarks.clear();
}

function trackBenchmark<T>(name: string, fn: () => T): T {
  if (!benchmarkEnabled) {
    return fn();
  }

  const start = performance.now();
  const result = fn();
  const end = performance.now();

  const durationMs = end - start;
  const durationUs = durationMs * 1000;

  const existing = benchmarks.get(name);
  if (existing) {
    existing.calls++;
    existing.totalMs += durationMs;
    existing.minUs = Math.min(existing.minUs, durationUs);
    existing.maxUs = Math.max(existing.maxUs, durationUs);
    existing.lastUs = durationUs;
  } else {
    benchmarks.set(name, {
      calls: 1,
      totalMs: durationMs,
      minUs: durationUs,
      maxUs: durationUs,
      lastUs: durationUs,
    });
  }

  return result;
}

/**
 * Initializes the Vulfram game engine.
 * Must be called before any other engine operations.
 *
 * @example
 * ```typescript
 * const result = vulframInit();
 * if (result === VulframResult.Success) {
 *   console.log('Engine initialized successfully');
 * }
 * ```
 */
export function vulframInit(): VulframResult {
  return trackBenchmark('vulframInit', () => VULFRAM_CORE.engineInit());
}

/**
 * Disposes the Vulfram engine and releases all resources.
 * Should be called when shutting down the application.
 *
 * @example
 * ```typescript
 * vulframDispose();
 * console.log('Engine disposed');
 * ```
 */
export function vulframDispose(): VulframResult {
  return trackBenchmark('vulframDispose', () => VULFRAM_CORE.engineDispose());
}

/**
 * Sends a batch of commands to the engine for execution.
 * Commands are processed in the order they are sent.
 *
 * @example
 * ```typescript
 * const batch: EngineBatchCmds = {
 *   cmds: [{
 *     type: 'cmd-window-create',
 *     content: {
 *       title: 'My Game',
 *       size: { width: 1280, height: 720 },
 *       position: { x: 100, y: 100 },
 *       borderless: false,
 *       resizable: true,
 *       initialState: 'normal'
 *     }
 *   }]
 * };
 * vulframSendQueue(batch);
 * ```
 */
export function vulframSendQueue(batch: EngineBatchCmds): VulframResult {
  return trackBenchmark('vulframSendQueue', () => {
    const buffer = pack(batch);
    return VULFRAM_CORE.engineSendQueue(Buffer.from(buffer));
  });
}

/**
 * Receives and processes events from the engine.
 * Returns a batch of events that occurred since the last call.
 *
 * @example
 * ```typescript
 * const [events, result] = vulframReceiveQueue();
 * if (result === VulframResult.Success) {
 *   for (const event of events.events) {
 *     if (event.kind === 'window') {
 *       console.log('Window event:', event.content);
 *     }
 *   }
 * }
 * ```
 */
export function vulframReceiveQueue(): [EngineBatchEvents, VulframResult] {
  return trackBenchmark('vulframReceiveQueue', () => {
    const { buffer, result } = VULFRAM_CORE.engineReceiveQueue();
    const events = unpack(buffer) as EngineBatchEvents;
    return [events, result];
  });
}

/**
 * Advances the engine by one frame tick.
 * Should be called once per frame in your game loop.
 *
 * @example
 * ```typescript
 * let lastTime = Date.now();
 *
 * function gameLoop() {
 *   const currentTime = Date.now();
 *   const deltaTime = currentTime - lastTime;
 *   lastTime = currentTime;
 *
 *   vulframTick(currentTime, deltaTime);
 *   requestAnimationFrame(gameLoop);
 * }
 * ```
 */
export function vulframTick(time: number, deltaTime: number): VulframResult {
  return trackBenchmark('vulframTick', () =>
    VULFRAM_CORE.engineTick(time, deltaTime),
  );
}

/**
 * Uploads a buffer to the engine for GPU processing.
 * Used for textures, meshes, and other GPU resources.
 *
 * @example
 * ```typescript
 * const imageData = new Uint8Array([...]);
 * const bufferId = 1;
 *
 * const result = vulframUploadBuffer(bufferId, imageData);
 * if (result === VulframResult.Success) {
 *   console.log('Buffer uploaded successfully');
 * }
 * ```
 */
export function vulframUploadBuffer(
  id: number,
  buffer: Uint8Array,
): VulframResult {
  return trackBenchmark('vulframUploadBuffer', () =>
    VULFRAM_CORE.engineUploadBuffer(id, Buffer.from(buffer)),
  );
}

/**
 * Downloads a buffer from the engine.
 * Retrieves data that was previously uploaded or generated by the GPU.
 *
 * @example
 * ```typescript
 * const bufferId = 1;
 * const [data, result] = vulframDownloadBuffer(bufferId);
 *
 * if (result === VulframResult.Success) {
 *   console.log('Downloaded', data.length, 'bytes');
 * }
 * ```
 */
export function vulframDownloadBuffer(id: number): [Uint8Array, VulframResult] {
  return trackBenchmark('vulframDownloadBuffer', () => {
    const { buffer, result } = VULFRAM_CORE.engineDownloadBuffer(id);
    return [buffer, result];
  });
}

/**
 * Clears a buffer from the engine, freeing its GPU memory.
 * Should be called when a buffer is no longer needed.
 *
 * @example
 * ```typescript
 * const bufferId = 1;
 * vulframClearBuffer(bufferId);
 * console.log('Buffer cleared');
 * ```
 */
export function vulframClearBuffer(id: number): VulframResult {
  return trackBenchmark('vulframClearBuffer', () =>
    VULFRAM_CORE.engineClearBuffer(id),
  );
}
