import type { EnvironmentConfig } from '../../types/cmds/environment';
import type { Shadow3dConfig } from '../../types/cmds/shadow';
import { vec3, vec4 } from '../../math/index';
import { enqueueCommand } from '../bridge/dispatch';
import type { System } from '../ecs';
import { toVec3, toVec4 } from './utils';

const WORLD_LIFECYCLE_INTENT_TYPES = [
  'configure-environment',
  'configure-shadows',
  'send-notification'
] as const;

const DEFAULT_CLEAR_COLOR = vec4.create();
const DEFAULT_MSAA_ENABLED = true;
const DEFAULT_MSAA_SAMPLE_COUNT = 4;
const DEFAULT_SKYBOX_INTENSITY = 1;
const DEFAULT_SKYBOX_ROTATION = 0;
const DEFAULT_GROUND_COLOR = vec3.fromValues(0.02, 0.03, 0.04);
const DEFAULT_HORIZON_COLOR = vec3.fromValues(0.12, 0.16, 0.22);
const DEFAULT_SKY_COLOR = vec3.fromValues(0.2, 0.35, 0.6);
const DEFAULT_HORIZON_GROUND_THRESHOLD = 0.45;
const DEFAULT_HORIZON_SKY_THRESHOLD = 0.55;
const DEFAULT_SUN_SOLID_SIZE = 0.0018;
const DEFAULT_SUN_GRADIENT_SIZE = 0.0287;
const DEFAULT_POST = {
  filterEnabled: true,
  filterExposure: 1,
  filterGamma: 1,
  filterSaturation: 1,
  filterContrast: 1,
  filterVignette: 0,
  filterGrain: 0,
  filterChromaticAberration: 0,
  filterBlur: 0,
  filterSharpen: 0,
  filterTonemapMode: 0,
  outlineEnabled: false,
  outlineStrength: 0,
  outlineThreshold: 0.2,
  outlineWidth: 1,
  outlineQuality: 1,
  filterPosterizeSteps: 0,
  cellShading: false,
  ssaoEnabled: false,
  ssaoStrength: 1,
  ssaoRadius: 0.75,
  ssaoBias: 0.025,
  ssaoPower: 1.5,
  ssaoBlurRadius: 2,
  ssaoBlurDepthThreshold: 0.02,
  bloomEnabled: false,
  bloomThreshold: 1,
  bloomKnee: 0.5,
  bloomIntensity: 0.8,
  bloomScatter: 0.7
} as const;
const DEFAULT_SHADOW = {
  tileResolution: 2048,
  atlasTilesW: 16,
  atlasTilesH: 16,
  atlasLayers: 2,
  virtualGridSize: 1,
  smoothing: 2,
  normalBias: 0.03
} as const;

function hasOwnKeys(value: object): boolean {
  return Object.keys(value).length > 0;
}

function sameTuple(value: ArrayLike<number>, expected: ArrayLike<number>, epsilon = 1e-6): boolean {
  if (value.length !== expected.length) return false;
  for (let i = 0; i < expected.length; i++) {
    const expectedValue = expected[i];
    if (expectedValue === undefined) {
      return false;
    }
    if (Math.abs((value[i] ?? 0) - expectedValue) > epsilon) {
      return false;
    }
  }
  return true;
}

function clampOutlineThreshold(value: number): number {
  return Math.max(0, Math.min(value, 0.999999));
}

function normalizeEnvironmentConfig(config: EnvironmentConfig): EnvironmentConfig {
  const payload: EnvironmentConfig = {};

  if (config.msaa) {
    const msaa: NonNullable<EnvironmentConfig['msaa']> = {};
    if (config.msaa.enabled !== undefined && config.msaa.enabled !== DEFAULT_MSAA_ENABLED) {
      msaa.enabled = config.msaa.enabled;
    }
    if (
      config.msaa.sampleCount !== undefined &&
      config.msaa.sampleCount !== DEFAULT_MSAA_SAMPLE_COUNT
    ) {
      msaa.sampleCount = config.msaa.sampleCount;
    }
    if (hasOwnKeys(msaa)) {
      payload.msaa = msaa;
    }
  }

  if (config.skybox) {
    const skybox: NonNullable<EnvironmentConfig['skybox']> = {};
    if (config.skybox.mode !== undefined && config.skybox.mode !== 'none') {
      skybox.mode = config.skybox.mode;
    }
    if (
      config.skybox.intensity !== undefined &&
      config.skybox.intensity !== DEFAULT_SKYBOX_INTENSITY
    ) {
      skybox.intensity = config.skybox.intensity;
    }
    if (
      config.skybox.rotation !== undefined &&
      config.skybox.rotation !== DEFAULT_SKYBOX_ROTATION
    ) {
      skybox.rotation = config.skybox.rotation;
    }
    if (
      config.skybox.groundColor !== undefined &&
      !sameTuple(config.skybox.groundColor, DEFAULT_GROUND_COLOR)
    ) {
      skybox.groundColor = toVec3(config.skybox.groundColor);
    }
    if (
      config.skybox.horizonColor !== undefined &&
      !sameTuple(config.skybox.horizonColor, DEFAULT_HORIZON_COLOR)
    ) {
      skybox.horizonColor = toVec3(config.skybox.horizonColor);
    }
    if (
      config.skybox.skyColor !== undefined &&
      !sameTuple(config.skybox.skyColor, DEFAULT_SKY_COLOR)
    ) {
      skybox.skyColor = toVec3(config.skybox.skyColor);
    }
    if (
      config.skybox.horizonGroundThreshold !== undefined &&
      config.skybox.horizonGroundThreshold !== DEFAULT_HORIZON_GROUND_THRESHOLD
    ) {
      skybox.horizonGroundThreshold = config.skybox.horizonGroundThreshold;
    }
    if (
      config.skybox.horizonSkyThreshold !== undefined &&
      config.skybox.horizonSkyThreshold !== DEFAULT_HORIZON_SKY_THRESHOLD
    ) {
      skybox.horizonSkyThreshold = config.skybox.horizonSkyThreshold;
    }
    if (config.skybox.directionalLights && config.skybox.directionalLights.length > 0) {
      const directionalLights = config.skybox.directionalLights.map((light) => {
        const normalized = { lightId: light.lightId } as {
          lightId: number;
          solidSize?: number;
          gradientSize?: number;
        };
        if (light.solidSize !== undefined && light.solidSize !== DEFAULT_SUN_SOLID_SIZE) {
          normalized.solidSize = light.solidSize;
        }
        if (light.gradientSize !== undefined && light.gradientSize !== DEFAULT_SUN_GRADIENT_SIZE) {
          normalized.gradientSize = light.gradientSize;
        }
        return normalized;
      });
      if (directionalLights.length > 0) {
        skybox.directionalLights = directionalLights;
      }
    }
    if (config.skybox.cubemapTextureId !== undefined) {
      skybox.cubemapTextureId = config.skybox.cubemapTextureId;
    }
    if (hasOwnKeys(skybox)) {
      payload.skybox = skybox;
    }
  }

  if (config.clearColor !== undefined && !sameTuple(config.clearColor, DEFAULT_CLEAR_COLOR)) {
    payload.clearColor = toVec4(config.clearColor);
  }

  if (config.post) {
    const post: NonNullable<EnvironmentConfig['post']> = {};
    if (
      config.post.filterEnabled !== undefined &&
      config.post.filterEnabled !== DEFAULT_POST.filterEnabled
    ) {
      post.filterEnabled = config.post.filterEnabled;
    }
    if (
      config.post.filterExposure !== undefined &&
      config.post.filterExposure !== DEFAULT_POST.filterExposure
    ) {
      post.filterExposure = config.post.filterExposure;
    }
    if (
      config.post.filterGamma !== undefined &&
      config.post.filterGamma !== DEFAULT_POST.filterGamma
    ) {
      post.filterGamma = config.post.filterGamma;
    }
    if (
      config.post.filterSaturation !== undefined &&
      config.post.filterSaturation !== DEFAULT_POST.filterSaturation
    ) {
      post.filterSaturation = config.post.filterSaturation;
    }
    if (
      config.post.filterContrast !== undefined &&
      config.post.filterContrast !== DEFAULT_POST.filterContrast
    ) {
      post.filterContrast = config.post.filterContrast;
    }
    if (
      config.post.filterVignette !== undefined &&
      config.post.filterVignette !== DEFAULT_POST.filterVignette
    ) {
      post.filterVignette = config.post.filterVignette;
    }
    if (
      config.post.filterGrain !== undefined &&
      config.post.filterGrain !== DEFAULT_POST.filterGrain
    ) {
      post.filterGrain = config.post.filterGrain;
    }
    if (
      config.post.filterChromaticAberration !== undefined &&
      config.post.filterChromaticAberration !== DEFAULT_POST.filterChromaticAberration
    ) {
      post.filterChromaticAberration = config.post.filterChromaticAberration;
    }
    if (
      config.post.filterBlur !== undefined &&
      config.post.filterBlur !== DEFAULT_POST.filterBlur
    ) {
      post.filterBlur = config.post.filterBlur;
    }
    if (
      config.post.filterSharpen !== undefined &&
      config.post.filterSharpen !== DEFAULT_POST.filterSharpen
    ) {
      post.filterSharpen = config.post.filterSharpen;
    }
    if (
      config.post.filterTonemapMode !== undefined &&
      config.post.filterTonemapMode !== DEFAULT_POST.filterTonemapMode
    ) {
      post.filterTonemapMode = config.post.filterTonemapMode;
    }
    if (
      config.post.outlineEnabled !== undefined &&
      config.post.outlineEnabled !== DEFAULT_POST.outlineEnabled
    ) {
      post.outlineEnabled = config.post.outlineEnabled;
    }
    if (
      config.post.outlineStrength !== undefined &&
      config.post.outlineStrength !== DEFAULT_POST.outlineStrength
    ) {
      post.outlineStrength = config.post.outlineStrength;
    }
    if (config.post.outlineThreshold !== undefined) {
      const outlineThreshold = clampOutlineThreshold(config.post.outlineThreshold);
      if (outlineThreshold !== DEFAULT_POST.outlineThreshold) {
        post.outlineThreshold = outlineThreshold;
      }
    }
    if (
      config.post.outlineWidth !== undefined &&
      config.post.outlineWidth !== DEFAULT_POST.outlineWidth
    ) {
      post.outlineWidth = config.post.outlineWidth;
    }
    if (
      config.post.outlineQuality !== undefined &&
      config.post.outlineQuality !== DEFAULT_POST.outlineQuality
    ) {
      post.outlineQuality = config.post.outlineQuality;
    }
    if (
      config.post.filterPosterizeSteps !== undefined &&
      config.post.filterPosterizeSteps !== DEFAULT_POST.filterPosterizeSteps
    ) {
      post.filterPosterizeSteps = config.post.filterPosterizeSteps;
    }
    if (
      config.post.cellShading !== undefined &&
      config.post.cellShading !== DEFAULT_POST.cellShading
    ) {
      post.cellShading = config.post.cellShading;
    }
    if (
      config.post.ssaoEnabled !== undefined &&
      config.post.ssaoEnabled !== DEFAULT_POST.ssaoEnabled
    ) {
      post.ssaoEnabled = config.post.ssaoEnabled;
    }
    if (
      config.post.ssaoStrength !== undefined &&
      config.post.ssaoStrength !== DEFAULT_POST.ssaoStrength
    ) {
      post.ssaoStrength = config.post.ssaoStrength;
    }
    if (
      config.post.ssaoRadius !== undefined &&
      config.post.ssaoRadius !== DEFAULT_POST.ssaoRadius
    ) {
      post.ssaoRadius = config.post.ssaoRadius;
    }
    if (config.post.ssaoBias !== undefined && config.post.ssaoBias !== DEFAULT_POST.ssaoBias) {
      post.ssaoBias = config.post.ssaoBias;
    }
    if (config.post.ssaoPower !== undefined && config.post.ssaoPower !== DEFAULT_POST.ssaoPower) {
      post.ssaoPower = config.post.ssaoPower;
    }
    if (
      config.post.ssaoBlurRadius !== undefined &&
      config.post.ssaoBlurRadius !== DEFAULT_POST.ssaoBlurRadius
    ) {
      post.ssaoBlurRadius = config.post.ssaoBlurRadius;
    }
    if (
      config.post.ssaoBlurDepthThreshold !== undefined &&
      config.post.ssaoBlurDepthThreshold !== DEFAULT_POST.ssaoBlurDepthThreshold
    ) {
      post.ssaoBlurDepthThreshold = config.post.ssaoBlurDepthThreshold;
    }
    if (
      config.post.bloomEnabled !== undefined &&
      config.post.bloomEnabled !== DEFAULT_POST.bloomEnabled
    ) {
      post.bloomEnabled = config.post.bloomEnabled;
    }
    if (
      config.post.bloomThreshold !== undefined &&
      config.post.bloomThreshold !== DEFAULT_POST.bloomThreshold
    ) {
      post.bloomThreshold = config.post.bloomThreshold;
    }
    if (config.post.bloomKnee !== undefined && config.post.bloomKnee !== DEFAULT_POST.bloomKnee) {
      post.bloomKnee = config.post.bloomKnee;
    }
    if (
      config.post.bloomIntensity !== undefined &&
      config.post.bloomIntensity !== DEFAULT_POST.bloomIntensity
    ) {
      post.bloomIntensity = config.post.bloomIntensity;
    }
    if (
      config.post.bloomScatter !== undefined &&
      config.post.bloomScatter !== DEFAULT_POST.bloomScatter
    ) {
      post.bloomScatter = config.post.bloomScatter;
    }
    if (hasOwnKeys(post)) {
      payload.post = post;
    }
  }

  return payload;
}

function normalizeShadowConfig(config: Shadow3dConfig): Shadow3dConfig {
  const payload: Shadow3dConfig = {};

  if (
    config.tileResolution !== undefined &&
    config.tileResolution !== DEFAULT_SHADOW.tileResolution
  ) {
    payload.tileResolution = config.tileResolution;
  }
  if (config.atlasTilesW !== undefined && config.atlasTilesW !== DEFAULT_SHADOW.atlasTilesW) {
    payload.atlasTilesW = config.atlasTilesW;
  }
  if (config.atlasTilesH !== undefined && config.atlasTilesH !== DEFAULT_SHADOW.atlasTilesH) {
    payload.atlasTilesH = config.atlasTilesH;
  }
  if (config.atlasLayers !== undefined && config.atlasLayers !== DEFAULT_SHADOW.atlasLayers) {
    payload.atlasLayers = config.atlasLayers;
  }
  if (
    config.virtualGridSize !== undefined &&
    config.virtualGridSize !== DEFAULT_SHADOW.virtualGridSize
  ) {
    payload.virtualGridSize = config.virtualGridSize;
  }
  if (config.smoothing !== undefined && config.smoothing !== DEFAULT_SHADOW.smoothing) {
    payload.smoothing = config.smoothing;
  }
  if (config.normalBias !== undefined && config.normalBias !== DEFAULT_SHADOW.normalBias) {
    payload.normalBias = config.normalBias;
  }

  return payload;
}

/**
 * Applies world-scoped lifecycle/configuration intents.
 *
 * Covered intents:
 * - environment configuration
 * - shadow configuration
 * - notification dispatch
 */
export const WorldLifecycleSystem: System = (world, context) => {
  const intents = world.intentStore.takeMany(WORLD_LIFECYCLE_INTENT_TYPES);

  for (let i = 0; i < intents.length; i++) {
    const intent = intents[i];
    if (!intent) continue;

    if (intent.type === 'configure-environment') {
      const payload = normalizeEnvironmentConfig(intent.config as EnvironmentConfig);
      if (!hasOwnKeys(payload)) {
        continue;
      }
      enqueueCommand(context.worldId, 'cmd-environment-upsert', {
        environmentId: context.worldId,
        config: payload
      });
      // Keep current realm->target bindings synchronized with this environment.
      for (const binding of world.targetLayerBindings.values()) {
        binding.environmentId = context.worldId;
        if (world.coreRealmId === undefined) continue;
        enqueueCommand(context.worldId, 'cmd-target-layer-upsert', {
          realmId: world.coreRealmId,
          targetId: binding.targetId,
          layout: binding.layout,
          cameraId: binding.cameraId,
          enabledCameraIds: binding.cameraId !== undefined ? [binding.cameraId] : [],
          environmentId: context.worldId
        });
      }
    } else if (intent.type === 'configure-shadows') {
      const payload = normalizeShadowConfig(intent.config as Shadow3dConfig);
      if (!hasOwnKeys(payload)) {
        continue;
      }
      let windowId = world.primaryWindowId;
      if (windowId === undefined) {
        for (const boundWindowId of world.targetWindowBindings.values()) {
          windowId = boundWindowId;
          break;
        }
      }
      if (windowId === undefined) continue;
      enqueueCommand(context.worldId, 'cmd-shadow3d-configure', {
        windowId,
        config: payload
      });
    } else if (intent.type === 'send-notification') {
      enqueueCommand(context.worldId, 'cmd-notification-send', {
        level: intent.level,
        title: intent.title,
        body: intent.message
      });
    }
  }
};
