use serde::{Deserialize, Serialize};

use crate::core::input::events::{KeyboardEvent, PointerEvent};
use crate::core::system::SystemEvent;
use crate::core::window::WindowEvent;
use galfus_input::GamepadEvent;

pub use crate::core::audio;
pub use crate::core::buffers as buf;
pub use crate::core::realm::cmd as realm;
pub use crate::core::render::gizmos as gizmo;
pub use crate::core::resources as res;
pub use crate::core::system as sys;
pub use crate::core::target::cmd as target;
pub use crate::core::window as win;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdCamera3dUpsertArgs {
    Create(res::CmdCameraCreateArgs),
    Update(res::CmdCameraUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdModel3dUpsertArgs {
    Create(res::CmdModelCreateArgs),
    Update(res::CmdModelUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdLight3dUpsertArgs {
    Create(res::CmdLightCreateArgs),
    Update(res::CmdLightUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdMaterialUpsertArgs {
    Create(res::CmdMaterialCreateArgs),
    Update(res::CmdMaterialUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdMaterialDefinitionUpsertArgs {
    Create(res::CmdMaterialDefinitionCreateArgs),
    Update(res::CmdMaterialDefinitionUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdMaterialInstanceUpsertArgs {
    Create(res::CmdMaterialInstanceCreateArgs),
    Update(res::CmdMaterialInstanceUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdGeometryUpsertArgs {
    Create(res::CmdGeometryCreateArgs),
    Update(res::CmdGeometryUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdEnvironmentUpsertArgs {
    Create(res::CmdEnvironmentCreateArgs),
    Update(res::CmdEnvironmentUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdAudioListenerUpsertArgs {
    Create(audio::CmdAudioListenerCreateArgs),
    Update(audio::CmdAudioListenerUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdAudioSourceUpsertArgs {
    Create(audio::CmdAudioSourceCreateArgs),
    Update(audio::CmdAudioSourceUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdTextureUpsertArgs {
    FromBuffer(res::CmdTextureCreateFromBufferArgs),
    SolidColor(res::CmdTextureCreateSolidColorArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdCamera2dUpsertArgs {
    Create(res::CmdCamera2dCreateArgs),
    Update(res::CmdCamera2dUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdSprite2dUpsertArgs {
    Create(res::CmdSprite2dCreateArgs),
    Update(res::CmdSprite2dUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdShape2dUpsertArgs {
    Create(res::CmdShape2dCreateArgs),
    Update(res::CmdShape2dUpdateArgs),
}

pub type CmdResultSimple = galfus_protocol::CmdResultSimple;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineCmd {
    CmdNotificationSend(sys::CmdNotificationSendArgs),
    CmdSystemDiagnosticsSet(sys::CmdSystemDiagnosticsSetArgs),
    CmdSystemLogLevelSet(sys::CmdSystemLogLevelSetArgs),
    CmdSystemLogLevelGet(sys::CmdSystemLogLevelGetArgs),
    CmdSystemBuildVersionGet(sys::CmdSystemBuildVersionGetArgs),
    CmdWindowCreate(win::CmdWindowCreateArgs),
    CmdWindowClose(win::CmdWindowCloseArgs),
    CmdWindowMeasurement(win::CmdWindowMeasurementArgs),
    CmdWindowCursor(win::CmdWindowCursorArgs),
    CmdWindowState(win::CmdWindowStateArgs),
    CmdUploadBufferDiscardAll(buf::CmdUploadBufferDiscardAllArgs),
    CmdCamera3dUpsert(CmdCamera3dUpsertArgs),
    CmdCamera2dUpsert(CmdCamera2dUpsertArgs),
    CmdCamera3dDispose(res::CmdCamera3dDisposeArgs),
    CmdCamera2dDispose(res::CmdCamera2dDisposeArgs),
    CmdModel3dUpsert(CmdModel3dUpsertArgs),
    CmdSprite2dUpsert(CmdSprite2dUpsertArgs),
    CmdShape2dUpsert(CmdShape2dUpsertArgs),
    CmdRealm2dShadowConfigUpdate(res::CmdRealm2dShadowConfigUpdateArgs),
    CmdPoseUpdate(res::CmdPoseUpdateArgs),
    CmdModel3dDispose(res::CmdModel3dDisposeArgs),
    CmdSprite2dDispose(res::CmdSprite2dDisposeArgs),
    CmdShape2dDispose(res::CmdShape2dDisposeArgs),
    CmdLight3dUpsert(CmdLight3dUpsertArgs),
    CmdLight3dDispose(res::CmdLight3dDisposeArgs),
    CmdMaterialUpsert(CmdMaterialUpsertArgs),
    CmdMaterialDispose(res::CmdMaterialDisposeArgs),
    CmdMaterialDefinitionUpsert(CmdMaterialDefinitionUpsertArgs),
    CmdMaterialDefinitionDispose(res::CmdMaterialDefinitionDisposeArgs),
    CmdMaterialInstanceUpsert(CmdMaterialInstanceUpsertArgs),
    CmdMaterialInstanceDispose(res::CmdMaterialInstanceDisposeArgs),
    CmdMaterialDefinitionGet(res::CmdResourceGetArgs),
    CmdMaterialDefinitionList(res::CmdResourceListArgs),
    CmdMaterialInstanceGet(res::CmdMaterialInstanceGetArgs),
    CmdMaterialInstanceList(res::CmdMaterialInstanceListArgs),
    CmdTextureCreateFromBuffer(res::CmdTextureCreateFromBufferArgs),
    CmdTextureCreateSolidColor(res::CmdTextureCreateSolidColorArgs),
    CmdTextureUpsert(CmdTextureUpsertArgs),
    CmdTextureDispose(res::CmdTextureDisposeArgs),
    CmdTextureBindTarget(res::CmdTextureBindTargetArgs),
    CmdTextureGet(res::CmdResourceGetArgs),
    CmdAudioListenerUpsert(CmdAudioListenerUpsertArgs),
    CmdAudioListenerDispose(audio::CmdAudioListenerDisposeArgs),
    CmdAudioListenerGet(audio::CmdAudioListenerGetArgs),
    CmdAudioResourceUpsert(audio::CmdAudioResourceUpsertArgs),
    CmdAudioResourceGet(audio::CmdAudioResourceGetArgs),
    CmdAudioResourceList(audio::CmdAudioResourceListArgs),
    CmdAudioSourceUpsert(CmdAudioSourceUpsertArgs),
    CmdAudioSourceGet(audio::CmdAudioSourceGetArgs),
    CmdAudioSourceList(audio::CmdAudioSourceListArgs),
    CmdAudioSourceTransport(audio::CmdAudioSourceTransportArgs),
    CmdAudioStateGet(audio::CmdAudioStateGetArgs),
    CmdAudioSourceDispose(audio::CmdAudioSourceDisposeArgs),
    CmdAudioResourceDispose(audio::CmdAudioResourceDisposeArgs),
    CmdGeometryUpsert(CmdGeometryUpsertArgs),
    CmdGeometryDispose(res::CmdGeometryDisposeArgs),
    CmdGeometryGet(res::CmdResourceGetArgs),
    CmdPrimitiveGeometryCreate(res::CmdPrimitiveGeometryCreateArgs),
    CmdEnvironmentUpsert(CmdEnvironmentUpsertArgs),
    CmdEnvironmentDispose(res::CmdEnvironmentDisposeArgs),
    CmdEnvironmentGet(res::CmdResourceGetArgs),
    CmdEnvironmentList(res::CmdResourceListArgs),
    CmdShadow3dConfigure(res::shadow::CmdShadow3dConfigureArgs),
    CmdRealmCreate(realm::CmdRealmCreateArgs),
    CmdRealmDispose(realm::CmdRealmDisposeArgs),
    CmdRealmGet(realm::CmdRealmGetArgs),
    CmdRealmList(realm::CmdRealmListArgs),
    CmdRenderGraphUpsert(realm::CmdRenderGraphUpsertArgs),
    CmdRenderGraphDispose(realm::CmdRenderGraphDisposeArgs),
    CmdRenderGraphList(realm::CmdRenderGraphListArgs),
    CmdRealmRenderGraphBind(realm::CmdRealmRenderGraphBindArgs),
    CmdTargetUpsert(target::CmdTargetUpsertArgs),
    CmdTargetGet(target::CmdTargetGetArgs),
    CmdTargetList(target::CmdTargetListArgs),
    CmdTargetMeasurement(target::CmdTargetMeasurementArgs),
    CmdTargetDispose(target::CmdTargetDisposeArgs),
    CmdTargetLayerUpsert(target::CmdTargetLayerUpsertArgs),
    CmdTargetLayerDispose(target::CmdTargetLayerDisposeArgs),
    CmdTargetLayerGet(target::CmdTargetLayerGetArgs),
    CmdTargetLayerList(target::CmdTargetLayerListArgs),
    CmdModelGet(res::CmdResourceGetArgs),
    CmdModelList(res::CmdModelListArgs),
    CmdMaterialGet(res::CmdMaterialGetArgs),
    CmdMaterialList(res::CmdMaterialListArgs),
    CmdTextureList(res::CmdTextureListArgs),
    CmdGeometryList(res::CmdGeometryListArgs),
    CmdLightGet(res::CmdResourceGetArgs),
    CmdLightList(res::CmdLightListArgs),
    CmdCameraGet(res::CmdResourceGetArgs),
    CmdCameraList(res::CmdCameraListArgs),
    CmdGizmoDrawLine(gizmo::CmdGizmoDrawLineArgs),
    CmdGizmoDrawAabb(gizmo::CmdGizmoDrawAabbArgs),
    CmdGizmoDrawPolyline(gizmo::CmdGizmoDrawPolylineArgs),
}

/// Spontaneous engine events (input, window changes, system events)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineEvent {
    Window(WindowEvent),
    Pointer(PointerEvent),
    Keyboard(KeyboardEvent),
    Gamepad(GamepadEvent),
    System(SystemEvent),
    Log(galfus_log::LogEvent),
}

/// Command responses (answers to commands sent by user)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum CommandResponse {
    NotificationSend(sys::CmdResultNotificationSend),
    SystemDiagnosticsSet(sys::CmdResultSystemDiagnosticsSet),
    SystemLogLevelSet(sys::CmdResultSystemLogLevelSet),
    SystemLogLevelGet(sys::CmdResultSystemLogLevelGet),
    SystemBuildVersionGet(sys::CmdResultSystemBuildVersionGet),
    WindowCreate(win::CmdResultWindowCreate),
    WindowClose(win::CmdResultWindowClose),
    WindowMeasurement(win::CmdResultWindowMeasurement),
    WindowCursor(win::CmdResultWindowCursor),
    WindowState(win::CmdResultWindowState),
    UploadBufferDiscardAll(buf::CmdResultUploadBufferDiscardAll),
    Camera3dUpsert(CmdResultSimple),
    Camera2dUpsert(res::CmdResultTwoDUpsert),
    Camera3dDispose(res::CmdResultCameraDispose),
    Camera2dDispose(res::CmdResultTwoDDispose),
    Model3dUpsert(CmdResultSimple),
    Sprite2dUpsert(res::CmdResultTwoDUpsert),
    Shape2dUpsert(res::CmdResultTwoDUpsert),
    Realm2dShadowConfigUpdate(res::CmdResultTwoDUpsert),
    PoseUpdate(res::CmdResultPoseUpdate),
    Model3dDispose(res::CmdResultModelDispose),
    Sprite2dDispose(res::CmdResultTwoDDispose),
    Shape2dDispose(res::CmdResultTwoDDispose),
    Light3dUpsert(CmdResultSimple),
    Light3dDispose(res::CmdResultLightDispose),
    MaterialUpsert(CmdResultSimple),
    MaterialDispose(res::CmdResultMaterialDispose),
    MaterialDefinitionUpsert(res::CmdResultMaterialDefinition),
    MaterialDefinitionDispose(res::CmdResultMaterialDefinition),
    MaterialDefinitionGet(res::CmdResultResourceGet),
    MaterialDefinitionList(res::CmdResultResourceList),
    MaterialInstanceUpsert(res::CmdResultMaterialInstance),
    MaterialInstanceDispose(res::CmdResultMaterialInstance),
    MaterialInstanceGet(res::CmdResultResourceGet),
    MaterialInstanceList(res::CmdResultResourceList),
    TextureCreateFromBuffer(res::CmdResultTextureCreateFromBuffer),
    TextureCreateSolidColor(res::CmdResultTextureCreateSolidColor),
    TextureUpsert(CmdResultSimple),
    TextureDispose(res::CmdResultTextureDispose),
    TextureBindTarget(res::CmdResultTextureBindTarget),
    TextureGet(res::CmdResultResourceGet),
    AudioListenerUpsert(CmdResultSimple),
    AudioListenerDispose(audio::CmdResultAudioListenerDispose),
    AudioListenerGet(audio::CmdResultAudioListenerGet),
    AudioResourceUpsert(audio::CmdResultAudioResourceUpsert),
    AudioResourceGet(audio::CmdResultAudioResourceGet),
    AudioResourceList(audio::CmdResultAudioResourceList),
    AudioSourceUpsert(CmdResultSimple),
    AudioSourceGet(audio::CmdResultAudioSourceGet),
    AudioSourceList(audio::CmdResultAudioSourceList),
    AudioSourceTransport(audio::CmdResultAudioSourceTransport),
    AudioStateGet(audio::CmdResultAudioStateGet),
    AudioSourceDispose(audio::CmdResultAudioSourceDispose),
    AudioResourceDispose(audio::CmdResultAudioResourceDispose),
    GeometryUpsert(CmdResultSimple),
    GeometryDispose(res::CmdResultGeometryDispose),
    GeometryGet(res::CmdResultResourceGet),
    PrimitiveGeometryCreate(res::CmdResultPrimitiveGeometryCreate),
    EnvironmentUpsert(CmdResultSimple),
    EnvironmentDispose(res::CmdResultEnvironment),
    EnvironmentGet(res::CmdResultResourceGet),
    EnvironmentList(res::CmdResultResourceList),
    Shadow3dConfigure(res::shadow::CmdResultShadow3dConfigure),
    RealmCreate(realm::CmdResultRealmCreate),
    RealmDispose(realm::CmdResultRealmDispose),
    RealmGet(realm::CmdResultRealmGet),
    RealmList(realm::CmdResultRealmList),
    RenderGraphUpsert(realm::CmdResultRenderGraphUpsert),
    RenderGraphDispose(realm::CmdResultRenderGraphDispose),
    RenderGraphList(realm::CmdResultRenderGraphList),
    RealmRenderGraphBind(realm::CmdResultRealmRenderGraphBind),
    TargetUpsert(target::CmdResultTargetUpsert),
    TargetGet(target::CmdResultTargetGet),
    TargetList(target::CmdResultTargetList),
    TargetMeasurement(target::CmdResultTargetMeasurement),
    TargetDispose(target::CmdResultTargetDispose),
    TargetLayerUpsert(target::CmdResultTargetLayerUpsert),
    TargetLayerDispose(target::CmdResultTargetLayerDispose),
    TargetLayerGet(target::CmdResultTargetLayerGet),
    TargetLayerList(target::CmdResultTargetLayerList),
    ModelGet(res::CmdResultResourceGet),
    ModelList(res::CmdResultModelList),
    MaterialGet(res::CmdResultResourceGet),
    MaterialList(res::CmdResultMaterialList),
    TextureList(res::CmdResultTextureList),
    GeometryList(res::CmdResultGeometryList),
    LightGet(res::CmdResultResourceGet),
    LightList(res::CmdResultLightList),
    CameraGet(res::CmdResultResourceGet),
    CameraList(res::CmdResultCameraList),
    GizmoDrawLine(gizmo::CmdResultGizmoDraw),
    GizmoDrawAabb(gizmo::CmdResultGizmoDraw),
    GizmoDrawPolyline(gizmo::CmdResultGizmoDraw),
}

pub type EngineCmdEnvelope = galfus_protocol::CommandEnvelope<EngineCmd>;

pub type CommandResponseEnvelope = galfus_protocol::ResponseEnvelope<CommandResponse>;

pub type EngineBatchCmds = Vec<EngineCmdEnvelope>;

mod processing;

pub(crate) use processing::{deferred_command_key, engine_process_batch};
