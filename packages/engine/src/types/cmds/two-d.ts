import type { Mat4 as mat4, Vec2 as vec2 } from '../../math';

export interface CmdCamera2dCreateArgs {
  realmId: number;
  cameraId: number;
  label?: string;
  transform: mat4;
  nearFar: vec2;
  orthoScale?: number;
  layerMask?: number;
  order?: number;
}

export interface CmdCamera2dUpdateArgs {
  realmId: number;
  cameraId: number;
  label?: string;
  transform?: mat4;
  nearFar?: vec2;
  orthoScale?: number;
  layerMask?: number;
  order?: number;
}

export type CmdCamera2dUpsertArgs = CmdCamera2dCreateArgs | CmdCamera2dUpdateArgs;

export interface CmdCamera2dDisposeArgs {
  realmId: number;
  cameraId: number;
}

export interface CmdSprite2dCreateArgs {
  realmId: number;
  spriteId: number;
  label?: string;
  transform: mat4;
  geometryId: number;
  materialId?: number;
  layer?: number;
}

export interface CmdSprite2dUpdateArgs {
  realmId: number;
  spriteId: number;
  label?: string;
  transform?: mat4;
  geometryId?: number;
  materialId?: number;
  layer?: number;
}

export type CmdSprite2dUpsertArgs = CmdSprite2dCreateArgs | CmdSprite2dUpdateArgs;

export interface CmdSprite2dDisposeArgs {
  realmId: number;
  spriteId: number;
}

export interface CmdShape2dCreateArgs {
  realmId: number;
  shapeId: number;
  label?: string;
  transform: mat4;
  geometryId: number;
  materialId?: number;
  layer?: number;
}

export interface CmdShape2dUpdateArgs {
  realmId: number;
  shapeId: number;
  label?: string;
  transform?: mat4;
  geometryId?: number;
  materialId?: number;
  layer?: number;
}

export type CmdShape2dUpsertArgs = CmdShape2dCreateArgs | CmdShape2dUpdateArgs;

export interface CmdShape2dDisposeArgs {
  realmId: number;
  shapeId: number;
}

export interface CmdRealm2dShadowConfigUpdateArgs {
  realmId: number;
  shadowContactOffset?: number;
  shadowDebugLightIndex?: number;
  shadowDebugMode?: number;
  ambient?: number;
  lightRadius?: number;
  maxShadowUpdatesPerFrame?: number;
  angularResolution?: number;
  mapResolution?: number;
}

export interface CmdResultTwoDUpsert {
  success: boolean;
  message: string;
}

export interface CmdResultTwoDDispose {
  success: boolean;
  message: string;
}
