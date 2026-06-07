import type { CommandResponseEnvelope, EngineCmd } from '../../types/cmds';
import type { EngineEvent } from '../../types/events';
import { engineState } from '../state';
import { getWorldOrThrow, requireInitialized } from './guards';

/**
 * Limits for Backpressure management.
 */
const MAX_BATCH_COMMANDS = 2048;
const MAX_BATCH_ESTIMATED_BYTES = 60 * 1024;

function estimateSerializedBytes(value: unknown): number {
  if (value === null || value === undefined) return 1;
  const valueType = typeof value;
  if (valueType === 'boolean') return 1;
  if (valueType === 'number') return 8;
  if (valueType === 'string') return 2 + (value as string).length * 2;
  if (ArrayBuffer.isView(value)) return (value as ArrayBufferView).byteLength + 8;
  if (value instanceof ArrayBuffer) return value.byteLength + 8;

  if (Array.isArray(value)) {
    let total = 8;
    for (let i = 0; i < value.length; i++) {
      total += estimateSerializedBytes(value[i]);
    }
    return total;
  }

  if (valueType === 'object') {
    const record = value as Record<string, unknown>;
    let total = 8;
    for (const [key, nested] of Object.entries(record)) {
      if (nested === undefined) continue;
      total += 2 + key.length * 2;
      total += estimateSerializedBytes(nested);
    }
    return total;
  }

  return 4;
}

function estimateEnvelopeBytes(envelope: { id: number; type: string; content: unknown }): number {
  return 24 + envelope.type.length * 2 + estimateSerializedBytes(envelope.content);
}

function maybeCompactQueue<T>(queue: T[], head: number): number {
  if (head <= 0) return head;
  if (head === queue.length) {
    queue.length = 0;
    return 0;
  }
  if (head >= 1024 && head * 2 >= queue.length) {
    queue.copyWithin(0, head);
    queue.length -= head;
    return 0;
  }
  return head;
}

function mergeNestedRecord(previous: unknown, next: unknown): Record<string, unknown> | undefined {
  const previousRecord =
    previous && typeof previous === 'object' ? (previous as Record<string, unknown>) : undefined;
  const nextRecord =
    next && typeof next === 'object' ? (next as Record<string, unknown>) : undefined;

  if (previousRecord && nextRecord) {
    return {
      ...previousRecord,
      ...nextRecord
    };
  }
  return nextRecord ?? previousRecord;
}

function mergeCommandContent(
  type: string,
  previous: Record<string, unknown>,
  next: Record<string, unknown>
): Record<string, unknown> {
  if (type === 'cmd-environment-upsert' || type === 'cmd-shadow3d-configure') {
    return {
      ...previous,
      ...next,
      config: mergeNestedRecord(previous.config, next.config)
    };
  }

  return {
    ...previous,
    ...next
  };
}

function isMergeableWindowStateContent(content: Record<string, unknown>): boolean {
  return (
    content.action === undefined &&
    content.getState !== true &&
    content.getDecorations !== true &&
    content.getResizable !== true
  );
}

function getMergeableCommandKey(envelope: { type: string; content: unknown }): string | null {
  const content =
    envelope.content && typeof envelope.content === 'object'
      ? (envelope.content as Record<string, unknown>)
      : null;
  if (!content) return null;

  if (envelope.type === 'cmd-model3d-upsert') {
    const realmId = content.realmId;
    const modelId = content.modelId;
    if (typeof realmId === 'number' && typeof modelId === 'number') {
      return `${envelope.type}:${realmId}:${modelId}`;
    }
  }
  if (envelope.type === 'cmd-camera3d-upsert') {
    const realmId = content.realmId;
    const cameraId = content.cameraId;
    if (typeof realmId === 'number' && typeof cameraId === 'number') {
      return `${envelope.type}:${realmId}:${cameraId}`;
    }
  }
  if (envelope.type === 'cmd-camera2d-upsert') {
    const realmId = content.realmId;
    const cameraId = content.cameraId;
    if (typeof realmId === 'number' && typeof cameraId === 'number') {
      return `${envelope.type}:${realmId}:${cameraId}`;
    }
  }
  if (envelope.type === 'cmd-light3d-upsert') {
    const realmId = content.realmId;
    const lightId = content.lightId;
    if (typeof realmId === 'number' && typeof lightId === 'number') {
      return `${envelope.type}:${realmId}:${lightId}`;
    }
  }
  if (envelope.type === 'cmd-sprite2d-upsert') {
    const realmId = content.realmId;
    const spriteId = content.spriteId;
    if (typeof realmId === 'number' && typeof spriteId === 'number') {
      return `${envelope.type}:${realmId}:${spriteId}`;
    }
  }
  if (envelope.type === 'cmd-shape2d-upsert') {
    const realmId = content.realmId;
    const shapeId = content.shapeId;
    if (typeof realmId === 'number' && typeof shapeId === 'number') {
      return `${envelope.type}:${realmId}:${shapeId}`;
    }
  }
  if (envelope.type === 'cmd-target-layer-upsert') {
    const realmId = content.realmId;
    const targetId = content.targetId;
    if (typeof realmId === 'number' && typeof targetId === 'number') {
      return `${envelope.type}:${realmId}:${targetId}`;
    }
  }
  if (envelope.type === 'cmd-target-upsert') {
    const targetId = content.targetId;
    if (typeof targetId === 'number') {
      return `${envelope.type}:${targetId}`;
    }
  }
  if (envelope.type === 'cmd-window-measurement') {
    const windowId = content.windowId;
    if (typeof windowId === 'number') {
      return `${envelope.type}:${windowId}`;
    }
  }
  if (envelope.type === 'cmd-window-cursor') {
    const windowId = content.windowId;
    if (typeof windowId === 'number') {
      return `${envelope.type}:${windowId}`;
    }
  }
  if (envelope.type === 'cmd-window-state' && isMergeableWindowStateContent(content)) {
    const windowId = content.windowId;
    if (typeof windowId === 'number') {
      return `${envelope.type}:${windowId}`;
    }
  }
  if (envelope.type === 'cmd-environment-upsert') {
    const environmentId = content.environmentId;
    if (typeof environmentId === 'number') {
      return `${envelope.type}:${environmentId}`;
    }
  }
  if (envelope.type === 'cmd-shadow3d-configure') {
    const windowId = content.windowId;
    if (typeof windowId === 'number') {
      return `${envelope.type}:${windowId}`;
    }
  }

  return null;
}

function compactPendingCommands(
  queue: Array<{ id: number; type: string; content: unknown }>,
  head: number
): void {
  if (head >= queue.length) {
    return;
  }

  const mergedIndexByKey = new Map<string, number>();
  let writeIndex = head;

  for (let readIndex = head; readIndex < queue.length; readIndex++) {
    const envelope = queue[readIndex]!;
    const mergeKey = getMergeableCommandKey(envelope);
    if (!mergeKey) {
      queue[writeIndex] = envelope;
      writeIndex++;
      continue;
    }

    const previousIndex = mergedIndexByKey.get(mergeKey);
    if (previousIndex === undefined) {
      queue[writeIndex] = envelope;
      mergedIndexByKey.set(mergeKey, writeIndex);
      writeIndex++;
      continue;
    }

    const previous = queue[previousIndex]!;
    queue[previousIndex] = {
      id: envelope.id,
      type: envelope.type,
      content: mergeCommandContent(
        envelope.type,
        previous.content as Record<string, unknown>,
        envelope.content as Record<string, unknown>
      )
    };
  }

  queue.length = writeIndex;
}

export function markRoutingIndexDirty(): void {
  engineState.routingIndex.dirty = true;
}

/**
 * Enqueues a command to be sent to the core in the next tick.
 * Ensures the command is routed through the specific world.
 */
export function enqueueCommand<T extends EngineCmd['type']>(
  worldId: number,
  type: T,
  content: Extract<EngineCmd, { type: T }>['content']
): number {
  requireInitialized();
  const world = getWorldOrThrow(worldId);
  const id = engineState.nextCommandId++;

  // Contract: realmId resolves from world core realm; windowId may default to primary binding.
  const typedContent = {
    ...(content as Record<string, unknown>)
  } as Record<string, unknown>;
  if (
    'windowId' in typedContent &&
    typedContent.windowId == null &&
    world.primaryWindowId !== undefined
  ) {
    typedContent.windowId = world.primaryWindowId;
  }
  if ('realmId' in typedContent && typedContent.realmId == null) {
    // Avoid host-side readiness guards: keep logical-id flow stable.
    // Core validates realm existence and may apply internal fallbacks.
    typedContent.realmId = world.coreRealmId ?? world.worldId;
  }

  if (type === 'cmd-target-layer-upsert') {
    const cameraId = typedContent.cameraId;
    const enabledCameraIds = typedContent.enabledCameraIds;
    if (
      typeof cameraId === 'number' &&
      (!Array.isArray(enabledCameraIds) || enabledCameraIds.length === 0)
    ) {
      typedContent.enabledCameraIds = [cameraId];
    }
  }

  world.pendingCommands.push({
    id,
    type,
    content: typedContent as unknown as Extract<EngineCmd, { type: T }>['content']
  });
  return id;
}

/**
 * Enqueues a command not scoped to a specific world (window management, etc).
 */
export function enqueueGlobalCommand<T extends EngineCmd['type']>(
  type: T,
  content: Extract<EngineCmd, { type: T }>['content']
): number {
  requireInitialized();
  const id = engineState.nextCommandId++;
  engineState.globalCommandTracker.add(id);
  engineState.globalPendingCommands.push({ id, type, content });
  return id;
}

/**
 * Collects pending commands from all worlds into the global engine batch.
 * Implements BatchAggregatorSystem and basic BackpressureSystem logic.
 */
export function collectCommands(): void {
  let collectedCount = 0;
  let collectedBytes = 0;

  compactPendingCommands(engineState.globalPendingCommands, engineState.globalPendingCommandsHead);

  const globalAvailable =
    engineState.globalPendingCommands.length - engineState.globalPendingCommandsHead;
  if (globalAvailable > 0) {
    const start = engineState.globalPendingCommandsHead;
    let end = start;
    while (
      end < engineState.globalPendingCommands.length &&
      collectedCount < MAX_BATCH_COMMANDS &&
      collectedBytes < MAX_BATCH_ESTIMATED_BYTES
    ) {
      const cmd = engineState.globalPendingCommands[end]!;
      const estimateBytes = estimateEnvelopeBytes(cmd);
      if (collectedBytes > 0 && collectedBytes + estimateBytes > MAX_BATCH_ESTIMATED_BYTES) {
        break;
      }
      engineState.commandBatch.push(cmd);
      collectedBytes += estimateBytes;
      collectedCount++;
      end++;
    }
    engineState.globalPendingCommandsHead = end;
    engineState.globalPendingCommandsHead = maybeCompactQueue(
      engineState.globalPendingCommands,
      engineState.globalPendingCommandsHead
    );
  }

  for (const [worldId, world] of engineState.worlds) {
    if (collectedCount >= MAX_BATCH_COMMANDS || collectedBytes >= MAX_BATCH_ESTIMATED_BYTES) {
      // Backpressure: If we reached the limit, stop collecting for this frame.
      // Remaining commands will wait for the next frame.
      break;
    }

    compactPendingCommands(world.pendingCommands, world.pendingCommandsHead);

    const worldAvailable = world.pendingCommands.length - world.pendingCommandsHead;
    if (worldAvailable > 0) {
      const start = world.pendingCommandsHead;
      let end = start;
      while (
        end < world.pendingCommands.length &&
        collectedCount < MAX_BATCH_COMMANDS &&
        collectedBytes < MAX_BATCH_ESTIMATED_BYTES
      ) {
        const cmd = world.pendingCommands[end]!;
        const estimateBytes = estimateEnvelopeBytes(cmd);
        if (collectedBytes > 0 && collectedBytes + estimateBytes > MAX_BATCH_ESTIMATED_BYTES) {
          break;
        }
        engineState.commandBatch.push(cmd);
        engineState.commandTracker.set(cmd.id, worldId);
        collectedBytes += estimateBytes;
        collectedCount++;
        end++;
      }
      world.pendingCommandsHead = end;
      world.pendingCommandsHead = maybeCompactQueue(
        world.pendingCommands,
        world.pendingCommandsHead
      );
    }
  }
}

function ensureRoutingIndex(): void {
  if (!engineState.routingIndex.dirty) return;

  const byWindowId = new Map<number, number[]>();
  const byRealmId = new Map<number, number[]>();
  const byTargetId = new Map<number, number[]>();

  for (const [worldId, world] of engineState.worlds) {
    for (const boundWindowId of world.boundWindowIds) {
      const ids = byWindowId.get(boundWindowId);
      if (ids) ids.push(worldId);
      else byWindowId.set(boundWindowId, [worldId]);
    }

    if (typeof world.coreRealmId === 'number') {
      const ids = byRealmId.get(world.coreRealmId);
      if (ids) ids.push(worldId);
      else byRealmId.set(world.coreRealmId, [worldId]);
    }

    for (const targetId of world.targetLayerBindings.keys()) {
      const ids = byTargetId.get(targetId);
      if (ids) ids.push(worldId);
      else byTargetId.set(targetId, [worldId]);
    }
  }

  engineState.routingIndex.byWindowId = byWindowId;
  engineState.routingIndex.byRealmId = byRealmId;
  engineState.routingIndex.byTargetId = byTargetId;
  engineState.routingIndex.dirty = false;
}

function extractEventScopeIds(event: EngineEvent): {
  windowId?: number;
  realmId?: number;
  targetId?: number;
} {
  const content = event.content as { data?: Record<string, unknown> } | Record<string, unknown>;
  const scopedDataCandidate =
    'data' in content && content.data && typeof content.data === 'object' ? content.data : content;

  if (!scopedDataCandidate || typeof scopedDataCandidate !== 'object') {
    return {};
  }
  const scopedData = scopedDataCandidate as Record<string, unknown>;

  const windowId = typeof scopedData.windowId === 'number' ? scopedData.windowId : undefined;
  const realmId = typeof scopedData.realmId === 'number' ? scopedData.realmId : undefined;
  const targetId = typeof scopedData.targetId === 'number' ? scopedData.targetId : undefined;

  return { windowId, realmId, targetId };
}

/**
 * Routes core events to their respective worlds based on windowId.
 */
export function routeEvents(events: EngineEvent[]): void {
  ensureRoutingIndex();
  const worldsByWindowId = engineState.routingIndex.byWindowId;
  const worldsByRealmId = engineState.routingIndex.byRealmId;
  const worldsByTargetId = engineState.routingIndex.byTargetId;

  for (const event of events) {
    const { windowId, realmId, targetId } = extractEventScopeIds(event);

    if (windowId !== undefined) {
      const worldIds = worldsByWindowId.get(windowId);
      if (!worldIds) {
        continue;
      }
      for (let i = 0; i < worldIds.length; i++) {
        const world = engineState.worlds.get(worldIds[i]!);
        if (world) {
          world.inboundEvents.push(event);
        }
      }
      continue;
    }

    if (realmId !== undefined) {
      const worldIds = worldsByRealmId.get(realmId);
      if (!worldIds) {
        continue;
      }
      for (let i = 0; i < worldIds.length; i++) {
        const world = engineState.worlds.get(worldIds[i]!);
        if (world) {
          world.inboundEvents.push(event);
        }
      }
      continue;
    }

    if (targetId !== undefined) {
      const worldIds = worldsByTargetId.get(targetId);
      if (!worldIds) {
        continue;
      }
      for (let i = 0; i < worldIds.length; i++) {
        const world = engineState.worlds.get(worldIds[i]!);
        if (world) {
          world.inboundEvents.push(event);
        }
      }
      continue;
    }

    // Broadcast non-scoped events (system/audio notifications).
    for (const world of engineState.worlds.values()) {
      world.inboundEvents.push(event);
    }
  }
}

/**
 * Routes core responses back to the world that initiated the command.
 */
export function routeResponses(responses: CommandResponseEnvelope[]): void {
  for (const res of responses) {
    const worldId = engineState.commandTracker.get(res.id);
    if (worldId !== undefined) {
      const world = engineState.worlds.get(worldId);
      if (world) {
        world.inboundResponses.push(res);
      }
      engineState.commandTracker.delete(res.id);
      continue;
    }
    if (engineState.globalCommandTracker.has(res.id)) {
      engineState.globalInboundResponses.push(res);
      engineState.globalCommandTracker.delete(res.id);
    }
  }
}
