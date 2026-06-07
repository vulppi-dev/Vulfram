import { Mount, World2D, World3D, createWindow, initEngine, tick } from '@galfus/engine';
import type { World2DId, World3DId } from '@galfus/engine';
import { quat } from '@galfus/engine/math';
import { initWasmTransport, transportWasm } from '@galfus/transport-browser';

const FRAME_MS = 16;
const SHADOW_CONFIG = {
  tileResolution: 1024,
  atlasTilesW: 16,
  atlasTilesH: 16,
  atlasLayers: 2,
  virtualGridSize: 1,
  smoothing: 2,
  normalBias: 0.05
} as const;

type DemoUpdate = (dtSeconds: number) => void;
type DemoId = '001' | '002' | '003' | '004';

function getDemoId(): DemoId {
  const hash = window.location.hash.replace('#', '');
  const normalized = hash.startsWith('demo-') ? hash.replace('demo-', '') : hash;
  if (normalized === '002') return '002';
  if (normalized === '003') return '003';
  if (normalized === '004') return '004';
  return '001';
}

function createMountedWorld(demoId: DemoId): { world3dId?: World3DId; world2dId?: World2DId } {
  const { windowId } = createWindow({
    title: `Galfus Demo ${demoId}`,
    size: [1280, 720],
    position: [0, 0],
    resizable: true,
    canvasId: 'galfus-canvas'
  });

  if (demoId === '003' || demoId === '004') {
    const world2dId = World2D.create2DWorld();
    Mount.mountWorld(world2dId, { target: { kind: 'window', windowId } });
    return { world2dId };
  }

  const world3dId = World3D.create3DWorld();
  Mount.mountWorld(world3dId, { target: { kind: 'window', windowId } });
  return { world3dId };
}

function setupDemo001(worldId: World3DId): DemoUpdate {
  World3D.configure3DEnvironment(worldId, {
    clearColor: [0, 0, 0, 1],
    post: {
      outlineEnabled: true,
      outlineStrength: 0.5,
      outlineThreshold: 0.25,
      outlineWidth: 1.25
    }
  });
  World3D.configure3DShadows(worldId, SHADOW_CONFIG);

  const cubeGeometryId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'cube',
    label: 'Demo001Cube'
  });
  const floorGeometryId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'plane',
    label: 'Demo001FloorPlane'
  });

  const standardMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo001Standard',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.92, 0.35, 0.32, 1]
      }
    }
  });
  const pbrMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'pbr',
    label: 'Demo001Pbr',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.25, 0.86, 0.62, 1],
        metallic: [0.55, 0, 0, 0],
        roughness: [0.35, 0, 0, 0],
        ao: [1, 0, 0, 0],
        normalScale: [1, 0, 0, 0]
      }
    }
  });
  const customSimpleMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo001CustomSimplePlaceholder',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.25, 0.45, 0.98, 1],
        emissiveColor: [0, 0, 0, 0],
        specColor: [1, 1, 1, 1],
        specPower: [64, 0, 0, 0]
      }
    }
  });
  const floorMaterialId = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    label: 'Demo001Floor',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.24, 0.24, 0.26, 1],
        specColor: [0.05, 0.05, 0.05, 1],
        specPower: [8, 0, 0, 0]
      }
    }
  });

  const cameraEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, cameraEntity, {
    position: [0, 2, 7],
    rotation: [-0.1305262, 0, 0, 0.9914449]
  });
  World3D.create3DCamera(worldId, cameraEntity, {
    kind: 'perspective',
    near: 0.1,
    far: 120,
    order: 0
  });

  const lightEntity = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, lightEntity, { position: [3, 5, 5] });
  World3D.create3DLight(worldId, lightEntity, {
    kind: 'point',
    color: [1, 1, 1],
    intensity: 4,
    range: 24,
    castShadow: true
  });

  const cubeA = World3D.create3DEntity(worldId);
  const cubeB = World3D.create3DEntity(worldId);
  const cubeC = World3D.create3DEntity(worldId);
  const floor = World3D.create3DEntity(worldId);

  World3D.update3DTransform(worldId, cubeA, { position: [-2, 0, 0], scale: [1, 1, 1] });
  World3D.update3DTransform(worldId, cubeB, { position: [0, 0, 0], scale: [1, 1, 1] });
  World3D.update3DTransform(worldId, cubeC, { position: [2, 0, 0], scale: [1, 1, 1] });
  World3D.update3DTransform(worldId, floor, {
    position: [0, -1, 0],
    rotation: [-0.7071068, 0, 0, 0.7071068],
    scale: [20, 20, 1]
  });

  World3D.create3DModel(worldId, cubeA, {
    geometryId: cubeGeometryId,
    materialId: standardMaterialId
  });
  World3D.create3DModel(worldId, cubeB, { geometryId: cubeGeometryId, materialId: pbrMaterialId });
  World3D.create3DModel(worldId, cubeC, {
    geometryId: cubeGeometryId,
    materialId: customSimpleMaterialId
  });
  World3D.create3DModel(worldId, floor, {
    geometryId: floorGeometryId,
    materialId: floorMaterialId
  });

  let elapsedSeconds = 0;
  let nextShadowRefreshSeconds = 0;
  return (dtSeconds) => {
    elapsedSeconds += dtSeconds;
    const angleA = elapsedSeconds * 1.8;
    const angleB = elapsedSeconds * 2.5 + 0.6;
    const angleC = elapsedSeconds * 1.4 + 1.2;
    const qa = quat.fromEuler(quat.create(), 0, (angleA * 180) / Math.PI, 0);
    const qb = quat.fromEuler(quat.create(), 0, (angleB * 180) / Math.PI, 0);
    const qc = quat.fromEuler(quat.create(), 0, (angleC * 180) / Math.PI, 0);
    World3D.update3DTransform(worldId, cubeA, { rotation: [qa[0], qa[1], qa[2], qa[3]] });
    World3D.update3DTransform(worldId, cubeB, { rotation: [qb[0], qb[1], qb[2], qb[3]] });
    World3D.update3DTransform(worldId, cubeC, { rotation: [qc[0], qc[1], qc[2], qc[3]] });
    if (elapsedSeconds <= 1.5 && elapsedSeconds >= nextShadowRefreshSeconds) {
      World3D.configure3DShadows(worldId, SHADOW_CONFIG);
      nextShadowRefreshSeconds += 0.25;
    }
  };
}

function setupDemo002(worldId: World3DId): DemoUpdate {
  World3D.configure3DEnvironment(worldId, {
    clearColor: [0.01, 0.01, 0.02, 1],
    post: { bloomEnabled: true, bloomIntensity: 0.35, bloomThreshold: 0.75, bloomScatter: 0.25 }
  });
  const cubeGeometryId = World3D.create3DGeometry(worldId, {
    type: 'primitive',
    shape: 'cube'
  });
  const matA = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.1, 0.85, 1.0, 0.65]
      }
    }
  });
  const matB = World3D.create3DMaterial(worldId, {
    kind: 'standard',
    options: {
      type: 'schema',
      content: { baseColor: [0.95, 0.4, 0.2, 1.0] }
    }
  });
  const matC = World3D.create3DMaterial(worldId, {
    kind: 'pbr',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.3, 0.45, 1.0, 1.0],
        metallic: [0.35, 0, 0, 0],
        roughness: [0.25, 0, 0, 0],
        ao: [1.0, 0, 0, 0],
        normalScale: [1.0, 0, 0, 0]
      }
    }
  });

  const camera = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, camera, {
    position: [0, 2.3, 7.5],
    rotation: [-0.1305262, 0, 0, 0.9914449]
  });
  World3D.create3DCamera(worldId, camera, { kind: 'perspective', near: 0.1, far: 120, order: 0 });

  const light = World3D.create3DEntity(worldId);
  World3D.update3DTransform(worldId, light, { position: [0, 4, 0] });
  World3D.create3DLight(worldId, light, {
    kind: 'directional',
    color: [1, 1, 1],
    intensity: 3.2,
    castShadow: true,
    direction: [-0.45, -1, -0.35]
  });

  const a = World3D.create3DEntity(worldId);
  const b = World3D.create3DEntity(worldId);
  const c = World3D.create3DEntity(worldId);
  World3D.create3DModel(worldId, a, { geometryId: cubeGeometryId, materialId: matA });
  World3D.create3DModel(worldId, b, { geometryId: cubeGeometryId, materialId: matB });
  World3D.create3DModel(worldId, c, { geometryId: cubeGeometryId, materialId: matC });

  let t = 0;
  return (dt) => {
    t += dt;
    const ySpin = t * 1.8;
    const radius = 1.8;
    const qA = quat.fromEuler(quat.create(), 0, (ySpin * 180) / Math.PI, 0);
    const qB = quat.fromEuler(quat.create(), 0, ((ySpin + 1.2) * 180) / Math.PI, 0);
    const qC = quat.fromEuler(quat.create(), 0, ((ySpin + 2.4) * 180) / Math.PI, 0);
    World3D.update3DTransform(worldId, a, {
      position: [Math.cos(t * 1.3) * radius, 0, Math.sin(t * 1.3) * radius],
      rotation: [qA[0], qA[1], qA[2], qA[3]]
    });
    World3D.update3DTransform(worldId, b, {
      position: [Math.cos(t * 1.3 + 2.09) * radius, 0, Math.sin(t * 1.3 + 2.09) * radius],
      rotation: [qB[0], qB[1], qB[2], qB[3]]
    });
    World3D.update3DTransform(worldId, c, {
      position: [Math.cos(t * 1.3 + 4.18) * radius, 0, Math.sin(t * 1.3 + 4.18) * radius],
      rotation: [qC[0], qC[1], qC[2], qC[3]]
    });
  };
}

function setupDemo003(worldId: World2DId): DemoUpdate {
  const quadA = World2D.create2DGeometry(worldId, { type: 'primitive', shape: 'plane' });
  const quadB = World2D.create2DGeometry(worldId, { type: 'primitive', shape: 'plane' });
  const red = World2D.create2DMaterial(worldId, {
    kind: 'standard',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.95, 0.25, 0.35, 1]
      }
    }
  });
  const blue = World2D.create2DMaterial(worldId, {
    kind: 'standard',
    options: {
      type: 'schema',
      content: {
        baseColor: [0.25, 0.45, 0.95, 1]
      }
    }
  });
  const camera = World2D.create2DEntity(worldId);
  World2D.update2DTransform(worldId, camera, { position: [0, 0, 5] });
  World2D.create2DCamera(worldId, camera, {
    kind: 'orthographic',
    near: 0.01,
    far: 100,
    orthoScale: 2.5,
    order: 0
  });

  const spriteA = World2D.create2DEntity(worldId);
  const spriteB = World2D.create2DEntity(worldId);
  const shapeA = World2D.create2DEntity(worldId);
  World2D.update2DTransform(worldId, spriteA, { position: [-0.8, 0, 0], scale: [0.9, 0.9, 1] });
  World2D.update2DTransform(worldId, spriteB, { position: [0.9, -0.5, 0], scale: [0.7, 0.7, 1] });
  World2D.update2DTransform(worldId, shapeA, { position: [0, 0.35, 0], scale: [0.5, 1.4, 1] });
  World2D.create2DSprite(worldId, spriteA, { geometryId: quadA, materialId: red, layer: 1 });
  World2D.create2DSprite(worldId, spriteB, { geometryId: quadB, materialId: blue, layer: 2 });
  World2D.create2DShape(worldId, shapeA, { geometryId: quadA, materialId: red, layer: 0 });

  let t = 0;
  return (dt) => {
    t += dt;
    const xA = Math.sin(t * 1.2) * 0.55;
    const yB = Math.cos(t * 1.8) * 0.35;
    const rot = t * 1.7;
    const rotA = quat.fromEuler(quat.create(), 0, 0, (rot * 180) / Math.PI);
    const rotShape = quat.fromEuler(quat.create(), 0, 0, (-rot * 0.7 * 180) / Math.PI);
    World2D.update2DTransform(worldId, spriteA, {
      position: [xA, 0, 0],
      rotation: [rotA[0], rotA[1], rotA[2], rotA[3]],
      scale: [0.9, 0.9, 1]
    });
    World2D.update2DTransform(worldId, spriteB, { position: [0.9, yB, 0], scale: [0.7, 0.7, 1] });
    World2D.update2DTransform(worldId, shapeA, {
      position: [0, 0.35, 0],
      rotation: [rotShape[0], rotShape[1], rotShape[2], rotShape[3]],
      scale: [0.5, 1.4, 1]
    });
    World2D.update2DTransform(worldId, camera, { position: [Math.sin(t * 0.4) * 0.1, 0, 5] });
  };
}

function setupDemo004(worldId: World2DId): DemoUpdate {
  const lightWorldId = worldId as unknown as World3DId;
  const quad = World2D.create2DGeometry(worldId, { type: 'primitive', shape: 'plane' });
  const receiver = World2D.create2DMaterial(worldId, {
    kind: 'standard',
    options: { type: 'schema', content: { baseColor: [0.22, 0.45, 0.96, 1.0] } }
  });
  const caster = World2D.create2DMaterial(worldId, {
    kind: 'standard',
    options: { type: 'schema', content: { baseColor: [0.95, 0.33, 0.22, 1.0] } }
  });
  const floor = World2D.create2DMaterial(worldId, {
    kind: 'standard',
    options: { type: 'schema', content: { baseColor: [0.16, 0.16, 0.2, 1.0] } }
  });

  const camera = World2D.create2DEntity(worldId);
  World2D.update2DTransform(worldId, camera, { position: [0, 0, 5] });
  World2D.create2DCamera(worldId, camera, {
    kind: 'orthographic',
    near: 0.01,
    far: 100,
    orthoScale: 2.8,
    order: 0
  });

  const floorEntity = World2D.create2DEntity(worldId);
  World2D.update2DTransform(worldId, floorEntity, { position: [0, -1.45, 0], scale: [4.5, 0.5, 1] });
  World2D.create2DShape(worldId, floorEntity, { geometryId: quad, materialId: floor, layer: 0 });

  const receiverA = World2D.create2DEntity(worldId);
  const receiverB = World2D.create2DEntity(worldId);
  const casterA = World2D.create2DEntity(worldId);
  const casterB = World2D.create2DEntity(worldId);

  World2D.update2DTransform(worldId, receiverA, { position: [-1.0, -0.1, 0], scale: [0.9, 0.9, 1] });
  World2D.create2DSprite(worldId, receiverA, { geometryId: quad, materialId: receiver, layer: 1 });
  World2D.update2DTransform(worldId, receiverB, { position: [1.1, 0.25, 0], scale: [0.8, 0.8, 1] });
  World2D.create2DSprite(worldId, receiverB, { geometryId: quad, materialId: receiver, layer: 1 });
  World2D.update2DTransform(worldId, casterA, { position: [-0.1, 0, 0], scale: [0.45, 1.6, 1] });
  World2D.create2DShape(worldId, casterA, { geometryId: quad, materialId: caster, layer: 1 });
  World2D.update2DTransform(worldId, casterB, { position: [0.55, -0.45, 0], scale: [0.5, 0.5, 1] });
  World2D.create2DShape(worldId, casterB, { geometryId: quad, materialId: caster, layer: 1 });

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

  let t = 0;
  return (dt) => {
    t += dt;
    const rotA = quat.fromEuler(quat.create(), 0, 0, t * 70);
    const rotB = quat.fromEuler(quat.create(), 0, 0, -t * 95);

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
  };
}

function startRenderLoop(update: DemoUpdate): void {
  let last = performance.now();
  let totalMs = 0;
  const frame = (now: number) => {
    const dtMs = now - last;
    last = now;
    totalMs += dtMs;
    update(dtMs / 1000);
    tick(totalMs, dtMs);
    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);
}

async function main(): Promise<void> {
  await initWasmTransport();
  initEngine({ transport: transportWasm });

  const demoId = getDemoId();
  const mounted = createMountedWorld(demoId);
  let update: DemoUpdate;
  if (demoId === '004') {
    update = setupDemo004(mounted.world2dId!);
  } else if (demoId === '003') {
    update = setupDemo003(mounted.world2dId!);
  } else if (demoId === '002') {
    update = setupDemo002(mounted.world3dId!);
  } else {
    update = setupDemo001(mounted.world3dId!);
  }
  setTimeout(() => tick(FRAME_MS, FRAME_MS), 0);
  startRenderLoop(update);
}

main().catch((error) => {
  console.error('[Galfus WASM] Fatal error:', error);
});
