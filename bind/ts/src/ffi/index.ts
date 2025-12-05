import { dlopen, ptr, read, type Pointer } from 'bun:ffi';

const { symbols: VULFRAM_CORE, close } = dlopen('src/ffi/vulfram_core.dll', {
  engine_clear_buffer: { args: ['u64'], returns: 'u32' },
  engine_dispose: { args: [], returns: 'u32' },
  engine_download_buffer: { args: ['u64', 'ptr', 'ptr'], returns: 'u32' },
  engine_init: { args: [], returns: 'u32' },
  engine_receive_queue: { args: ['ptr', 'ptr'], returns: 'u32' },
  engine_receive_events: { args: ['ptr', 'ptr'], returns: 'u32' },
  engine_send_queue: { args: ['ptr', 'usize'], returns: 'u32' },
  engine_tick: { args: ['u64', 'u32'], returns: 'u32' },
  engine_upload_buffer: { args: ['u64', 'ptr', 'usize'], returns: 'u32' },
  engine_get_profiling: { args: ['ptr', 'ptr'], returns: 'u32' },
});

process.once('beforeExit', () => {
  close();
});

export interface BufferResult {
  buffer: Buffer;
  result: number;
}

export function engineClearBuffer(id: number): number {
  return VULFRAM_CORE.engine_clear_buffer(BigInt(id));
}

export function engineDispose(): number {
  return VULFRAM_CORE.engine_dispose();
}

export function engineDownloadBuffer(id: number): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = VULFRAM_CORE.engine_download_buffer(
    BigInt(id),
    ptr(ptrHolder),
    ptr(sizeHolder),
  );
  if (!sizeHolder[0]) {
    return { buffer: Buffer.alloc(0), result };
  }
  const srcPtr = Number(ptrHolder[0]) as Pointer;
  if (!srcPtr) {
    return { buffer: Buffer.alloc(0), result };
  }
  const buffer = Buffer.alloc(Number(sizeHolder[0]));
  for (let i = 0; i < buffer.length; i++) {
    buffer[i] = read.u8(srcPtr, i);
  }

  return { buffer, result };
}

export function engineInit(): number {
  return VULFRAM_CORE.engine_init();
}

export function engineReceiveQueue(): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = VULFRAM_CORE.engine_receive_queue(
    ptr(ptrHolder),
    ptr(sizeHolder),
  );
  if (!sizeHolder[0]) {
    return { buffer: Buffer.alloc(0), result };
  }
  const srcPtr = Number(ptrHolder[0]) as Pointer;
  if (!srcPtr) {
    return { buffer: Buffer.alloc(0), result };
  }
  const buffer = Buffer.alloc(Number(sizeHolder[0]));
  for (let i = 0; i < buffer.length; i++) {
    buffer[i] = read.u8(srcPtr, i);
  }

  return { buffer, result };
}

export function engineReceiveEvents(): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = VULFRAM_CORE.engine_receive_events(
    ptr(ptrHolder),
    ptr(sizeHolder),
  );
  if (!sizeHolder[0]) {
    return { buffer: Buffer.alloc(0), result };
  }
  const srcPtr = Number(ptrHolder[0]) as Pointer;
  if (!srcPtr) {
    return { buffer: Buffer.alloc(0), result };
  }
  const buffer = Buffer.alloc(Number(sizeHolder[0]));
  for (let i = 0; i < buffer.length; i++) {
    buffer[i] = read.u8(srcPtr, i);
  }

  return { buffer, result };
}

export function engineSendQueue(data: Buffer): number {
  return VULFRAM_CORE.engine_send_queue(ptr(data), data.length);
}

export function engineTick(time: number, deltaTime: number): number {
  return VULFRAM_CORE.engine_tick(time, deltaTime);
}

export function engineUploadBuffer(id: number, data: Buffer): number {
  return VULFRAM_CORE.engine_upload_buffer(BigInt(id), ptr(data), data.length);
}

export function engineGetProfiling(): BufferResult {
  const ptrHolder = new BigUint64Array(1);
  const sizeHolder = new BigUint64Array(1);
  const result = VULFRAM_CORE.engine_get_profiling(
    ptr(ptrHolder),
    ptr(sizeHolder),
  );
  if (!sizeHolder[0]) {
    return { buffer: Buffer.alloc(0), result };
  }
  const srcPtr = Number(ptrHolder[0]) as Pointer;
  if (!srcPtr) {
    return { buffer: Buffer.alloc(0), result };
  }
  const buffer = Buffer.alloc(Number(sizeHolder[0]));
  for (let i = 0; i < buffer.length; i++) {
    buffer[i] = read.u8(srcPtr, i);
  }

  return { buffer, result };
}
