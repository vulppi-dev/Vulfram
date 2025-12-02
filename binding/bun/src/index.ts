import { decode, encode } from 'cbor2';
import { dlopen, ptr } from 'bun:ffi';
import { VULFRAM_CORE_PATH } from './paths';
import type { VulframResult } from './enums';
import type { EngineBatchCmds, EngineBatchEvents } from './cmds';

export * from './cmds';
export * from './dev';
export * from './enums';
export * from './events';
export * from './paths';

if (!VULFRAM_CORE_PATH) {
  throw new Error(
    `Unsupported platform or architecture: ${process.platform} ${process.arch}`,
  );
}

const { symbols: VULFRAM_CORE, close: vulframCoreClose } = dlopen(
  VULFRAM_CORE_PATH,
  {
    engine_init: {
      args: [],
      returns: 'u32',
    },
    engine_dispose: {
      args: [],
      returns: 'u32',
    },
    engine_send_queue: {
      args: ['ptr', 'usize'] as [buffer: 'ptr', length: 'usize'],
      returns: 'u32',
    },
    engine_receive_queue: {
      args: ['ptr', 'ptr'] as [buffer: 'ptr', out_length: 'ptr'],
      returns: 'u32',
    },
    engine_upload_buffer: {
      args: ['u64', 'ptr', 'usize'] as [
        id: 'u64',
        buffer: 'ptr',
        length: 'usize',
      ],
      returns: 'u32',
    },
    engine_download_buffer: {
      args: ['u64', 'ptr', 'ptr'] as [
        id: 'u64',
        buffer: 'ptr',
        out_length: 'ptr',
      ],
      returns: 'u32',
    },
    engine_clear_buffer: {
      args: ['u64'] as [id: 'u64'],
      returns: 'u32',
    },
    engine_tick: {
      args: ['u64', 'u32'] as [time: 'u64', delta_time: 'u32'],
      returns: 'u32',
    },
  },
);

process.on('beforeExit', () => {
  vulframCoreClose();
});

export function vulframInit(): VulframResult {
  return VULFRAM_CORE.engine_init();
}

export function vulframDispose(): VulframResult {
  return VULFRAM_CORE.engine_dispose();
}

export function vulframSendQueue(batch: EngineBatchCmds): VulframResult {
  const buffer = encode(batch);
  const bufferPtr = ptr(buffer);
  const length = buffer.byteLength;

  return VULFRAM_CORE.engine_send_queue(bufferPtr, length);
}

export function vulframReceiveQueue(): [EngineBatchEvents, VulframResult] {
  const lengthHolder = new BigUint64Array(1);
  const lengthHolderPtr = ptr(lengthHolder);

  let result = VULFRAM_CORE.engine_receive_queue(null, lengthHolderPtr);
  if (result !== 0 || lengthHolder[0] === 0n) {
    return [[], result];
  }
  const buffer = new Uint8Array(Number(lengthHolder[0]));
  const bufferPtr = ptr(buffer);
  result = VULFRAM_CORE.engine_receive_queue(bufferPtr, lengthHolderPtr);
  if (result !== 0) {
    return [[], result];
  }
  const events = decode(buffer) as EngineBatchEvents;
  return [events, result];
}

export function vulframTick(time: bigint, deltaTime: number): VulframResult {
  return VULFRAM_CORE.engine_tick(time, deltaTime);
}

export function vulframUploadBuffer(
  id: bigint,
  buffer: Uint8Array,
): VulframResult {
  const bufferPtr = ptr(buffer);
  const length = buffer.byteLength;
  return VULFRAM_CORE.engine_upload_buffer(id, bufferPtr, length);
}

export function vulframDownloadBuffer(id: bigint): [Uint8Array, VulframResult] {
  const lengthHolder = new BigUint64Array(1);
  const lengthHolderPtr = ptr(lengthHolder);

  let result = VULFRAM_CORE.engine_download_buffer(id, null, lengthHolderPtr);
  if (result !== 0 || lengthHolder[0] === 0n) {
    return [new Uint8Array(0), result];
  }
  const buffer = new Uint8Array(Number(lengthHolder[0]));
  const bufferPtr = ptr(buffer);
  result = VULFRAM_CORE.engine_download_buffer(id, bufferPtr, lengthHolderPtr);
  if (result !== 0) {
    return [new Uint8Array(0), result];
  }
  return [buffer, result];
}

export function vulframClearBuffer(id: bigint): VulframResult {
  return VULFRAM_CORE.engine_clear_buffer(id);
}
