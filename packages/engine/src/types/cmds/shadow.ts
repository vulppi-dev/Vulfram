/** Shadow rendering configuration. */
export interface Shadow3dConfig {
  tileResolution?: number;
  atlasTilesW?: number;
  atlasTilesH?: number;
  atlasLayers?: number;
  virtualGridSize?: number;
  smoothing?: number;
  normalBias?: number;
}

/** Command payload for shadow configuration. */
export interface CmdShadow3dConfigureArgs {
  windowId: number;
  config: Shadow3dConfig;
}

/** Result payload for shadow configuration. */
export interface CmdResultShadow3dConfigure {
  success: boolean;
  message: string;
}
