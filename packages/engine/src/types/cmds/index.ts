import * as Audio from './audio';
import * as Cam from './camera';
import * as Env from './environment';
import * as Geo from './geometry';
import * as Giz from './gizmo';
import * as Lite from './light';
import * as Mat from './material';
import * as Mod from './model';
import * as Realm from './realm';
import * as RenderGraph from './render-graph';
import * as Sys from './system';
import * as Target from './target';
import * as Tex from './texture';
import * as TwoD from './two-d';
import * as Win from './window';

export * from './audio';
export * from './camera';
export * from './environment';
export * from './geometry';
export * from './gizmo';
export * from './light';
export * from './material';
export * from './model';
export * from './realm';
export * from './render-graph';
export * from './resources';
export * from './shadow';
export * from './system';
export * from './target';
export * from './texture';
export * from './two-d';
export * from './window';

export type EngineCmd =
  | { type: 'cmd-notification-send'; content: Sys.CmdNotificationSendArgs }
  | { type: 'cmd-system-diagnostics-set'; content: Sys.CmdSystemDiagnosticsSetArgs }
  | { type: 'cmd-system-log-level-set'; content: Sys.CmdSystemLogLevelSetArgs }
  | { type: 'cmd-system-log-level-get'; content: Sys.CmdSystemLogLevelGetArgs }
  | { type: 'cmd-system-build-version-get'; content: Sys.CmdSystemBuildVersionGetArgs }
  | { type: 'cmd-window-create'; content: Win.CmdWindowCreateArgs }
  | { type: 'cmd-window-close'; content: Win.CmdWindowCloseArgs }
  | { type: 'cmd-window-measurement'; content: Win.CmdWindowMeasurementArgs }
  | { type: 'cmd-window-cursor'; content: Win.CmdWindowCursorArgs }
  | { type: 'cmd-window-state'; content: Win.CmdWindowStateArgs }
  | { type: 'cmd-upload-buffer-discard-all'; content: Sys.CmdUploadBufferDiscardAllArgs }
  | { type: 'cmd-camera3d-upsert'; content: Cam.CmdCamera3dUpsertArgs }
  | { type: 'cmd-camera2d-upsert'; content: TwoD.CmdCamera2dUpsertArgs }
  | { type: 'cmd-camera3d-dispose'; content: Cam.CmdCamera3dDisposeArgs }
  | { type: 'cmd-camera2d-dispose'; content: TwoD.CmdCamera2dDisposeArgs }
  | { type: 'cmd-model3d-upsert'; content: Mod.CmdModel3dUpsertArgs }
  | { type: 'cmd-sprite2d-upsert'; content: TwoD.CmdSprite2dUpsertArgs }
  | { type: 'cmd-shape2d-upsert'; content: TwoD.CmdShape2dUpsertArgs }
  | { type: 'cmd-realm2d-shadow-config-update'; content: TwoD.CmdRealm2dShadowConfigUpdateArgs }
  | { type: 'cmd-pose-update'; content: Mod.CmdPoseUpdateArgs }
  | { type: 'cmd-model3d-dispose'; content: Mod.CmdModel3dDisposeArgs }
  | { type: 'cmd-sprite2d-dispose'; content: TwoD.CmdSprite2dDisposeArgs }
  | { type: 'cmd-shape2d-dispose'; content: TwoD.CmdShape2dDisposeArgs }
  | { type: 'cmd-light3d-upsert'; content: Lite.CmdLight3dUpsertArgs }
  | { type: 'cmd-light3d-dispose'; content: Lite.CmdLight3dDisposeArgs }
  | { type: 'cmd-material-upsert'; content: Mat.CmdMaterialUpsertArgs }
  | { type: 'cmd-material-dispose'; content: Mat.CmdMaterialDisposeArgs }
  | { type: 'cmd-material-definition-upsert'; content: Mat.CmdMaterialDefinitionUpsertArgs }
  | { type: 'cmd-material-definition-dispose'; content: Mat.CmdMaterialDefinitionDisposeArgs }
  | { type: 'cmd-material-instance-upsert'; content: Mat.CmdMaterialInstanceUpsertArgs }
  | { type: 'cmd-material-instance-dispose'; content: Mat.CmdMaterialInstanceDisposeArgs }
  | { type: 'cmd-material-definition-get'; content: Mat.CmdMaterialDefinitionGetArgs }
  | { type: 'cmd-material-definition-list'; content: Mat.CmdMaterialDefinitionListArgs }
  | { type: 'cmd-material-instance-get'; content: Mat.CmdMaterialInstanceGetArgs }
  | { type: 'cmd-material-instance-list'; content: Mat.CmdMaterialInstanceListArgs }
  | { type: 'cmd-texture-create-from-buffer'; content: Tex.CmdTextureCreateFromBufferArgs }
  | { type: 'cmd-texture-create-solid-color'; content: Tex.CmdTextureCreateSolidColorArgs }
  | { type: 'cmd-texture-upsert'; content: Tex.CmdTextureUpsertArgs }
  | { type: 'cmd-texture-dispose'; content: Tex.CmdTextureDisposeArgs }
  | { type: 'cmd-texture-bind-target'; content: Tex.CmdTextureBindTargetArgs }
  | { type: 'cmd-texture-get'; content: Tex.CmdTextureGetArgs }
  | { type: 'cmd-audio-listener-upsert'; content: Audio.CmdAudioListenerUpsertArgs }
  | { type: 'cmd-audio-listener-dispose'; content: Audio.CmdAudioListenerDisposeArgs }
  | { type: 'cmd-audio-listener-get'; content: Audio.CmdAudioListenerGetArgs }
  | { type: 'cmd-audio-resource-upsert'; content: Audio.CmdAudioResourceUpsertArgs }
  | { type: 'cmd-audio-resource-get'; content: Audio.CmdAudioResourceGetArgs }
  | { type: 'cmd-audio-resource-list'; content: Audio.CmdAudioResourceListArgs }
  | { type: 'cmd-audio-source-upsert'; content: Audio.CmdAudioSourceUpsertArgs }
  | { type: 'cmd-audio-source-get'; content: Audio.CmdAudioSourceGetArgs }
  | { type: 'cmd-audio-source-list'; content: Audio.CmdAudioSourceListArgs }
  | { type: 'cmd-audio-source-transport'; content: Audio.CmdAudioSourceTransportArgs }
  | { type: 'cmd-audio-state-get'; content: Audio.CmdAudioStateGetArgs }
  | { type: 'cmd-audio-source-dispose'; content: Audio.CmdAudioSourceDisposeArgs }
  | { type: 'cmd-audio-resource-dispose'; content: Audio.CmdAudioResourceDisposeArgs }
  | { type: 'cmd-geometry-upsert'; content: Geo.CmdGeometryUpsertArgs }
  | { type: 'cmd-geometry-dispose'; content: Geo.CmdGeometryDisposeArgs }
  | { type: 'cmd-geometry-get'; content: Geo.CmdGeometryGetArgs }
  | { type: 'cmd-primitive-geometry-create'; content: Geo.CmdPrimitiveGeometryCreateArgs }
  | { type: 'cmd-environment-upsert'; content: Env.CmdEnvironmentUpsertArgs }
  | { type: 'cmd-environment-dispose'; content: Env.CmdEnvironmentDisposeArgs }
  | { type: 'cmd-environment-get'; content: Env.CmdEnvironmentGetArgs }
  | { type: 'cmd-environment-list'; content: Env.CmdEnvironmentListArgs }
  | { type: 'cmd-shadow3d-configure'; content: import('./shadow').CmdShadow3dConfigureArgs }
  | { type: 'cmd-realm-create'; content: Realm.CmdRealmCreateArgs }
  | { type: 'cmd-realm-dispose'; content: Realm.CmdRealmDisposeArgs }
  | { type: 'cmd-realm-get'; content: Realm.CmdRealmGetArgs }
  | { type: 'cmd-realm-list'; content: Realm.CmdRealmListArgs }
  | { type: 'cmd-render-graph-upsert'; content: RenderGraph.CmdRenderGraphUpsertArgs }
  | { type: 'cmd-render-graph-dispose'; content: RenderGraph.CmdRenderGraphDisposeArgs }
  | { type: 'cmd-render-graph-list'; content: RenderGraph.CmdRenderGraphListArgs }
  | { type: 'cmd-realm-render-graph-bind'; content: RenderGraph.CmdRealmRenderGraphBindArgs }
  | { type: 'cmd-target-upsert'; content: Target.CmdTargetUpsertArgs }
  | { type: 'cmd-target-get'; content: Target.CmdTargetGetArgs }
  | { type: 'cmd-target-list'; content: Target.CmdTargetListArgs }
  | { type: 'cmd-target-measurement'; content: Target.CmdTargetMeasurementArgs }
  | { type: 'cmd-target-dispose'; content: Target.CmdTargetDisposeArgs }
  | { type: 'cmd-target-layer-upsert'; content: Target.CmdTargetLayerUpsertArgs }
  | { type: 'cmd-target-layer-dispose'; content: Target.CmdTargetLayerDisposeArgs }
  | { type: 'cmd-target-layer-get'; content: Target.CmdTargetLayerGetArgs }
  | { type: 'cmd-target-layer-list'; content: Target.CmdTargetLayerListArgs }
  | { type: 'cmd-model-get'; content: Mod.CmdModelGetArgs }
  | { type: 'cmd-model-list'; content: Mod.CmdModelListArgs }
  | { type: 'cmd-material-get'; content: Mat.CmdMaterialGetArgs }
  | { type: 'cmd-material-list'; content: Mat.CmdMaterialListArgs }
  | { type: 'cmd-texture-list'; content: Tex.CmdTextureListArgs }
  | { type: 'cmd-geometry-list'; content: Geo.CmdGeometryListArgs }
  | { type: 'cmd-light-get'; content: Lite.CmdLightGetArgs }
  | { type: 'cmd-light-list'; content: Lite.CmdLightListArgs }
  | { type: 'cmd-camera-get'; content: Cam.CmdCameraGetArgs }
  | { type: 'cmd-camera-list'; content: Cam.CmdCameraListArgs }
  | { type: 'cmd-gizmo-draw-line'; content: Giz.CmdGizmoDrawLineArgs }
  | { type: 'cmd-gizmo-draw-aabb'; content: Giz.CmdGizmoDrawAabbArgs }
  | { type: 'cmd-gizmo-draw-polyline'; content: Giz.CmdGizmoDrawPolylineArgs };

export type CommandResponse =
  | { type: 'notification-send'; content: Sys.CmdResultNotificationSend }
  | { type: 'system-diagnostics-set'; content: Sys.CmdResultSystemDiagnosticsSet }
  | { type: 'system-log-level-set'; content: Sys.CmdResultSystemLogLevelSet }
  | { type: 'system-log-level-get'; content: Sys.CmdResultSystemLogLevelGet }
  | { type: 'system-build-version-get'; content: Sys.CmdResultSystemBuildVersionGet }
  | { type: 'window-create'; content: Win.CmdResultWindowCreate }
  | { type: 'window-close'; content: Win.CmdResultWindowClose }
  | { type: 'window-measurement'; content: Win.CmdResultWindowMeasurement }
  | { type: 'window-cursor'; content: Win.CmdResultWindowCursor }
  | { type: 'window-state'; content: Win.CmdResultWindowState }
  | { type: 'upload-buffer-discard-all'; content: Sys.CmdResultUploadBufferDiscardAll }
  | { type: 'camera3d-upsert'; content: Cam.CmdResultCameraUpsert }
  | { type: 'camera2d-upsert'; content: TwoD.CmdResultTwoDUpsert }
  | { type: 'camera3d-dispose'; content: Cam.CmdResultCameraDispose }
  | { type: 'camera2d-dispose'; content: TwoD.CmdResultTwoDDispose }
  | { type: 'model3d-upsert'; content: Mod.CmdResultModelUpsert }
  | { type: 'sprite2d-upsert'; content: TwoD.CmdResultTwoDUpsert }
  | { type: 'shape2d-upsert'; content: TwoD.CmdResultTwoDUpsert }
  | { type: 'realm2d-shadow-config-update'; content: TwoD.CmdResultTwoDUpsert }
  | { type: 'pose-update'; content: Mod.CmdResultPoseUpdate }
  | { type: 'model3d-dispose'; content: Mod.CmdResultModelDispose }
  | { type: 'sprite2d-dispose'; content: TwoD.CmdResultTwoDDispose }
  | { type: 'shape2d-dispose'; content: TwoD.CmdResultTwoDDispose }
  | { type: 'light3d-upsert'; content: Lite.CmdResultLightUpsert }
  | { type: 'light3d-dispose'; content: Lite.CmdResultLightDispose }
  | { type: 'material-upsert'; content: Mat.CmdResultMaterialUpsert }
  | { type: 'material-dispose'; content: Mat.CmdResultMaterialDispose }
  | { type: 'material-definition-upsert'; content: Mat.CmdResultMaterialDefinition }
  | { type: 'material-definition-dispose'; content: Mat.CmdResultMaterialDefinition }
  | { type: 'material-definition-get'; content: Mat.CmdResultMaterialDefinitionGet }
  | { type: 'material-definition-list'; content: Mat.CmdResultMaterialDefinitionList }
  | { type: 'material-instance-upsert'; content: Mat.CmdResultMaterialInstance }
  | { type: 'material-instance-dispose'; content: Mat.CmdResultMaterialInstance }
  | { type: 'material-instance-get'; content: Mat.CmdResultMaterialInstanceGet }
  | { type: 'material-instance-list'; content: Mat.CmdResultMaterialInstanceList }
  | { type: 'texture-create-from-buffer'; content: Tex.CmdResultTextureCreateFromBuffer }
  | { type: 'texture-create-solid-color'; content: Tex.CmdResultTextureCreateSolidColor }
  | { type: 'texture-upsert'; content: Tex.CmdResultTextureUpsert }
  | { type: 'texture-dispose'; content: Tex.CmdResultTextureDispose }
  | { type: 'texture-bind-target'; content: Tex.CmdResultTextureBindTarget }
  | { type: 'texture-get'; content: Tex.CmdResultTextureGet }
  | { type: 'audio-listener-upsert'; content: Audio.CmdResultAudioListenerUpsert }
  | { type: 'audio-listener-dispose'; content: Audio.CmdResultAudioListenerDispose }
  | { type: 'audio-listener-get'; content: Audio.CmdResultAudioListenerGet }
  | { type: 'audio-resource-upsert'; content: Audio.CmdResultAudioResourceUpsert }
  | { type: 'audio-resource-get'; content: Audio.CmdResultAudioResourceGet }
  | { type: 'audio-resource-list'; content: Audio.CmdResultAudioResourceList }
  | { type: 'audio-source-upsert'; content: Audio.CmdResultAudioSourceUpsert }
  | { type: 'audio-source-get'; content: Audio.CmdResultAudioSourceGet }
  | { type: 'audio-source-list'; content: Audio.CmdResultAudioSourceList }
  | { type: 'audio-source-transport'; content: Audio.CmdResultAudioSourceTransport }
  | { type: 'audio-state-get'; content: Audio.CmdResultAudioStateGet }
  | { type: 'audio-source-dispose'; content: Audio.CmdResultAudioSourceDispose }
  | { type: 'audio-resource-dispose'; content: Audio.CmdResultAudioResourceDispose }
  | { type: 'geometry-upsert'; content: Geo.CmdResultGeometryUpsert }
  | { type: 'geometry-dispose'; content: Geo.CmdResultGeometryDispose }
  | { type: 'geometry-get'; content: Geo.CmdResultGeometryGet }
  | { type: 'primitive-geometry-create'; content: Geo.CmdResultPrimitiveGeometryCreate }
  | { type: 'environment-upsert'; content: Env.CmdResultEnvironment }
  | { type: 'environment-dispose'; content: Env.CmdResultEnvironment }
  | { type: 'environment-get'; content: Env.CmdResultEnvironmentGet }
  | { type: 'environment-list'; content: Env.CmdResultEnvironmentList }
  | { type: 'shadow3d-configure'; content: import('./shadow').CmdResultShadow3dConfigure }
  | { type: 'realm-create'; content: Realm.CmdResultRealmCreate }
  | { type: 'realm-dispose'; content: Realm.CmdResultRealmDispose }
  | { type: 'realm-get'; content: Realm.CmdResultRealmGet }
  | { type: 'realm-list'; content: Realm.CmdResultRealmList }
  | { type: 'render-graph-upsert'; content: RenderGraph.CmdResultRenderGraphUpsert }
  | { type: 'render-graph-dispose'; content: RenderGraph.CmdResultRenderGraphDispose }
  | { type: 'render-graph-list'; content: RenderGraph.CmdResultRenderGraphList }
  | { type: 'realm-render-graph-bind'; content: RenderGraph.CmdResultRealmRenderGraphBind }
  | { type: 'target-upsert'; content: Target.CmdResultTargetUpsert }
  | { type: 'target-get'; content: Target.CmdResultTargetGet }
  | { type: 'target-list'; content: Target.CmdResultTargetList }
  | { type: 'target-measurement'; content: Target.CmdResultTargetMeasurement }
  | { type: 'target-dispose'; content: Target.CmdResultTargetDispose }
  | { type: 'target-layer-upsert'; content: Target.CmdResultTargetLayerUpsert }
  | { type: 'target-layer-dispose'; content: Target.CmdResultTargetLayerDispose }
  | { type: 'target-layer-get'; content: Target.CmdResultTargetLayerGet }
  | { type: 'target-layer-list'; content: Target.CmdResultTargetLayerList }
  | { type: 'model-get'; content: Mod.CmdResultModelGet }
  | { type: 'model-list'; content: Mod.CmdResultModelList }
  | { type: 'material-get'; content: Mat.CmdResultMaterialGet }
  | { type: 'material-list'; content: Mat.CmdResultMaterialList }
  | { type: 'texture-list'; content: Tex.CmdResultTextureList }
  | { type: 'geometry-list'; content: Geo.CmdResultGeometryList }
  | { type: 'light-get'; content: Lite.CmdResultLightGet }
  | { type: 'light-list'; content: Lite.CmdResultLightList }
  | { type: 'camera-get'; content: Cam.CmdResultCameraGet }
  | { type: 'camera-list'; content: Cam.CmdResultCameraList }
  | { type: 'gizmo-draw-line'; content: Giz.CmdResultGizmoDraw }
  | { type: 'gizmo-draw-aabb'; content: Giz.CmdResultGizmoDraw }
  | { type: 'gizmo-draw-polyline'; content: Giz.CmdResultGizmoDraw };

export interface EngineCmdEnvelope {
  id: number;
  type: EngineCmd['type'];
  content: EngineCmd['content'];
}

export interface CommandResponseEnvelope {
  id: number;
  type: CommandResponse['type'];
  content: CommandResponse['content'];
}

export type EngineBatchCmds = EngineCmdEnvelope[];
export type EngineBatchResponses = CommandResponseEnvelope[];
