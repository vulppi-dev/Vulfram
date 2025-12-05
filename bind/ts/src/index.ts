import * as VULFRAM_CORE from './napi';
import { pack, unpack } from 'msgpackr';
import type { VulframResult } from './enums';
import type {
  EngineBatchCmds,
  EngineBatchEvents,
  EngineBatchResponses,
} from './cmds';

export * from './cmds';
export * from './dev';
export * from './enums';
export * from './events';

// MARK: Profiling types

export interface ProfilingData {
  /** Time spent processing gamepad events in microseconds */
  gamepadProcessingUs: number;
  /** Time spent in event loop pump in microseconds */
  eventLoopPumpUs: number;
  /** Time spent requesting redraws in microseconds */
  requestRedrawUs: number;
  /** Time spent serializing events in microseconds */
  serializationUs: number;
  /** Total number of events dispatched */
  totalEventsDispatched: number;
  /** Total number of events cached (not dispatched due to no change) */
  totalEventsCached: number;
}

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
 * Receives and processes command responses from the engine.
 * Returns a batch of responses to commands sent via vulframSendQueue.
 *
 * @example
 * ```typescript
 * const [responses, result] = vulframReceiveQueue();
 * if (result === VulframResult.Success) {
 *   for (const response of responses) {
 *     if (response.response.type === 'window-create') {
 *       console.log('Window created:', response.response.content);
 *     }
 *   }
 * }
 * ```
 */
export function vulframReceiveQueue(): [EngineBatchResponses, VulframResult] {
  return trackBenchmark('vulframReceiveQueue', () => {
    const { buffer, result } = VULFRAM_CORE.engineReceiveQueue();

    // If buffer is empty, return empty array
    if (buffer.length === 0) {
      return [[], result];
    }

    const responses = unpack(buffer) as EngineBatchResponses;
    return [responses, result];
  });
}

/**
 * Receives and processes spontaneous events from the engine.
 * Returns a batch of events (input, window changes, system events) that occurred since the last call.
 *
 * @example
 * ```typescript
 * const [events, result] = vulframReceiveEvents();
 * if (result === VulframResult.Success) {
 *   for (const event of events) {
 *     if (event.type === 'window') {
 *       console.log('Window event:', event.content);
 *     }
 *   }
 * }
 * ```
 */
export function vulframReceiveEvents(): [EngineBatchEvents, VulframResult] {
  return trackBenchmark('vulframReceiveEvents', () => {
    const { buffer, result } = VULFRAM_CORE.engineReceiveEvents();

    // If buffer is empty, return empty array
    if (buffer.length === 0) {
      return [[], result];
    }

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

/**
 * Gets detailed profiling data from the last engine tick.
 * Provides breakdown of time spent in different tick phases.
 *
 * @example
 * ```typescript
 * const [profiling, result] = vulframGetProfiling();
 * if (result === VulframResult.Success) {
 *   console.log('Gamepad:', profiling.gamepadProcessingUs, 'µs');
 *   console.log('Event Loop:', profiling.eventLoopPumpUs, 'µs');
 *   console.log('Serialization:', profiling.serializationUs, 'µs');
 *   console.log('Events Dispatched:', profiling.totalEventsDispatched);
 *   console.log('Events Cached:', profiling.totalEventsCached);
 * }
 * ```
 */
export function vulframGetProfiling(): [ProfilingData, VulframResult] {
  return trackBenchmark('vulframGetProfiling', () => {
    const { buffer, result } = VULFRAM_CORE.engineGetProfiling();

    if (buffer.length === 0) {
      return [
        {
          gamepadProcessingUs: 0,
          eventLoopPumpUs: 0,
          requestRedrawUs: 0,
          serializationUs: 0,
          totalEventsDispatched: 0,
          totalEventsCached: 0,
        },
        result,
      ];
    }

    const data = unpack(buffer) as ProfilingData;

    return [data, result];
  });
}

/**
 * Starts the main loop for the engine, calling the provided loop function at approximately 244 FPS.
 */
export function startLoop(loop: () => void): () => void {
  let running = true;
  const targetFrameTime = 1000 / 244; // 244 FPS
  function frame() {
    if (!running) return;

    const start = performance.now();
    loop();
    const end = performance.now();
    const delta = end - start;
    const delay = Math.max(0, targetFrameTime - delta);

    if (delay === 0) {
      console.warn(
        'Frame took longer than target frame time of',
        targetFrameTime.toFixed(2),
        'ms',
        ` (actual frame time: ${delta.toFixed(2)} ms)`,
      );
    }
    setTimeout(frame, delay);
  }
  frame();

  return function stopLoop() {
    running = false;
  };
}
