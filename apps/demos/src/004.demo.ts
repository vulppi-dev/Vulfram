import {
  Mount,
  World2D,
  World3D,
  closeWindow,
  createWindow,
  disposeEngine,
  initEngine,
  tick
} from '@galfus/engine';
import type { World3DId } from '@galfus/engine';
import { quat } from '@galfus/engine/math';
import { transportBunFfi } from '@galfus/transport-bun';

const RUN_DURATION_MS = 8_000;
const FRAME_TARGET_MS = 16;

async function main() {
  initEngine({ transport: transportBunFfi });
  const { windowId } = createWindow({
    title: 'Galfus Demo 004 - Realm2D Lights and Shadows',
    size: [1280, 720],
    position: [100, 100],
    borderless: false,
    resizable: true,
    transparent: false,
    initialState: 'maximized'
  });

  let totalMs = 0;

  const worldId = World2D.create2DWorld();
  Mount.mountWorld(worldId, { target: { kind: 'window', windowId } });

  const lightWorldId = worldId as unknown as World3DId;

  const quadGeometry = World2D.create2DGeometry(worldId, {
    type: 'primitive',
    shape: 'plane',
    label: 'Demo004Quad'
  });

  const receiverMaterial = World2D.create2DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo004Receiver',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.22, 0.45, 0.96, 1.0]
      }
    }
  });

  const casterMaterial = World2D.create2DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo004Caster',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.95, 0.33, 0.22, 1.0]
      }
    }
  });

  const floorMaterial = World2D.create2DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo004Floor',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.16, 0.16, 0.2, 1.0]
      }
    }
  });

  const camera = World2D.create2DEntity(worldId);
  World2D.update2DTransform(worldId, camera, { position: [0, 0, 5] });
  World2D.create2DCamera(worldId, camera, {
    kind: 'orthographic',
    near: 0.01,
    far: 100.0,
    orthoScale: 2.8,
    order: 0
  });

  const floor = World2D.create2DEntity(worldId);
  World2D.update2DTransform(worldId, floor, {
    position: [0, -1.45, 0],
    scale: [4.5, 0.5, 1]
  });
  World2D.create2DShape(worldId, floor, {
    geometryId: quadGeometry,
    materialId: floorMaterial,
    layer: 0
  });

  const receiverA = World2D.create2DEntity(worldId);
  const receiverB = World2D.create2DEntity(worldId);
  const casterA = World2D.create2DEntity(worldId);
  const casterB = World2D.create2DEntity(worldId);

  World2D.update2DTransform(worldId, receiverA, {
    position: [-1.0, -0.1, 0],
    scale: [0.9, 0.9, 1]
  });
  World2D.create2DSprite(worldId, receiverA, {
    geometryId: quadGeometry,
    materialId: receiverMaterial,
    layer: 1
  });

  World2D.update2DTransform(worldId, receiverB, {
    position: [1.1, 0.25, 0],
    scale: [0.8, 0.8, 1]
  });
  World2D.create2DSprite(worldId, receiverB, {
    geometryId: quadGeometry,
    materialId: receiverMaterial,
    layer: 1
  });

  World2D.update2DTransform(worldId, casterA, {
    position: [-0.1, 0.0, 0],
    scale: [0.45, 1.6, 1]
  });
  World2D.create2DShape(worldId, casterA, {
    geometryId: quadGeometry,
    materialId: casterMaterial,
    layer: 1
  });

  World2D.update2DTransform(worldId, casterB, {
    position: [0.55, -0.45, 0],
    scale: [0.5, 0.5, 1]
  });
  World2D.create2DShape(worldId, casterB, {
    geometryId: quadGeometry,
    materialId: casterMaterial,
    layer: 1
  });

  const lightA = World2D.create2DEntity(worldId);
  const lightB = World2D.create2DEntity(worldId);

  World2D.update2DTransform(worldId, lightA, { position: [-1.8, 1.3, 0.6] });
  World3D.create3DLight(lightWorldId, lightA, {
    kind: 'point',
    color: [0.25, 0.95, 1.0],
    intensity: 2.4,
    range: 5.5,
    castShadow: true
  });

  World2D.update2DTransform(worldId, lightB, { position: [1.8, 1.0, 0.6] });
  World3D.create3DLight(lightWorldId, lightB, {
    kind: 'point',
    color: [1.0, 0.45, 0.25],
    intensity: 2.2,
    range: 5.0,
    castShadow: true
  });

  const start = performance.now();
  let last = start;

  while (performance.now() - start < RUN_DURATION_MS) {
    const now = performance.now();
    const dtMs = now - last;
    last = now;
    totalMs += dtMs;

    const t = totalMs / 1000;

    const rotA = quat.fromEuler(quat.create(), 0, 0, (t * 70));
    const rotB = quat.fromEuler(quat.create(), 0, 0, (-t * 95));

    World2D.update2DTransform(worldId, casterA, {
      position: [Math.cos(t * 0.8) * 0.35, Math.sin(t * 1.2) * 0.25, 0],
      rotation: [rotA[0], rotA[1], rotA[2], rotA[3]],
      scale: [0.45, 1.6, 1]
    });
    World2D.update2DTransform(worldId, casterB, {
      position: [0.7 + Math.cos(t * 1.6) * 0.25, -0.45 + Math.sin(t * 1.9) * 0.2, 0],
      rotation: [rotB[0], rotB[1], rotB[2], rotB[3]],
      scale: [0.5, 0.5, 1]
    });

    World2D.update2DTransform(worldId, receiverA, {
      position: [-1.0 + Math.sin(t * 0.9) * 0.2, -0.1, 0],
      scale: [0.9, 0.9, 1]
    });
    World2D.update2DTransform(worldId, receiverB, {
      position: [1.1 + Math.cos(t * 1.0) * 0.2, 0.25, 0],
      scale: [0.8, 0.8, 1]
    });

    tick(totalMs, dtMs);
    const frameElapsed = performance.now() - now;
    await new Promise((resolve) =>
      setTimeout(resolve, Math.max(0, FRAME_TARGET_MS - frameElapsed))
    );
  }

  closeWindow(windowId);
  tick(totalMs + FRAME_TARGET_MS, FRAME_TARGET_MS);
  disposeEngine();
}

main().catch(console.error);
