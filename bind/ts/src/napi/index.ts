import { createRequire } from 'module';

const require = createRequire(import.meta.url);
const VULFRAM_CORE = require('./vulfram_core.node');

export interface BufferResult {
  buffer: Buffer;
  result: number;
}

export function engineClearBuffer(id: number): number {
  return VULFRAM_CORE.engineClearBuffer(id);
}

export function engineDispose(): number {
  return VULFRAM_CORE.engineDispose();
}

export function engineDownloadBuffer(id: number): BufferResult {
  return VULFRAM_CORE.engineDownloadBuffer(id);
}

export function engineInit(): number {
  return VULFRAM_CORE.engineInit();
}

export function engineReceiveQueue(): BufferResult {
  return VULFRAM_CORE.engineReceiveQueue();
}

export function engineReceiveEvents(): BufferResult {
  return VULFRAM_CORE.engineReceiveEvents();
}

export function engineSendQueue(data: Buffer): number {
  return VULFRAM_CORE.engineSendQueue(data);
}

export function engineTick(time: number, deltaTime: number): number {
  return VULFRAM_CORE.engineTick(time, deltaTime);
}

export function engineUploadBuffer(id: number, data: Buffer): number {
  return VULFRAM_CORE.engineUploadBuffer(id, data);
}

export function engineGetProfiling(): BufferResult {
  return VULFRAM_CORE.engineGetProfiling();
}
