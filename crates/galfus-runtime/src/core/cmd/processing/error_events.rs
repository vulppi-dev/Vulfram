use super::super::*;
use crate::core::state::EngineState;

pub(super) fn maybe_emit_response_error_event(
    engine: &mut EngineState,
    command_id: u64,
    response: &CommandResponse,
) {
    macro_rules! failure_case {
        ($result:expr, $name:literal) => {{
            let result = $result;
            if !result.success {
                Some(($name, result.message.as_str()))
            } else {
                None
            }
        }};
    }

    let failure = match response {
        CommandResponse::UploadBufferDiscardAll(result) => {
            failure_case!(result, "upload-buffer-discard-all")
        }
        CommandResponse::SystemDiagnosticsSet(result) => {
            failure_case!(result, "system-diagnostics-set")
        }
        CommandResponse::Camera3dUpsert(result) => failure_case!(result, "camera3d-upsert"),
        CommandResponse::WindowCreate(result) => failure_case!(result, "window-create"),
        CommandResponse::WindowMeasurement(result) => failure_case!(result, "window-measurement"),
        CommandResponse::WindowCursor(result) => failure_case!(result, "window-cursor"),
        CommandResponse::WindowState(result) => failure_case!(result, "window-state"),
        CommandResponse::Camera3dDispose(result) => failure_case!(result, "camera3d-dispose"),
        CommandResponse::Model3dUpsert(result) => failure_case!(result, "model3d-upsert"),
        CommandResponse::PoseUpdate(result) => failure_case!(result, "pose-update"),
        CommandResponse::Model3dDispose(result) => failure_case!(result, "model3d-dispose"),
        CommandResponse::Light3dUpsert(result) => failure_case!(result, "light3d-upsert"),
        CommandResponse::Light3dDispose(result) => failure_case!(result, "light3d-dispose"),
        CommandResponse::MaterialUpsert(result) => failure_case!(result, "material-upsert"),
        CommandResponse::MaterialDispose(result) => failure_case!(result, "material-dispose"),
        CommandResponse::MaterialDefinitionUpsert(result) => {
            failure_case!(result, "material-definition-upsert")
        }
        CommandResponse::MaterialDefinitionDispose(result) => {
            failure_case!(result, "material-definition-dispose")
        }
        CommandResponse::MaterialInstanceUpsert(result) => {
            failure_case!(result, "material-instance-upsert")
        }
        CommandResponse::MaterialInstanceDispose(result) => {
            failure_case!(result, "material-instance-dispose")
        }
        CommandResponse::TextureCreateFromBuffer(result) => {
            failure_case!(result, "texture-create-from-buffer")
        }
        CommandResponse::TextureCreateSolidColor(result) => {
            failure_case!(result, "texture-create-solid-color")
        }
        CommandResponse::TextureDispose(result) => failure_case!(result, "texture-dispose"),
        CommandResponse::TextureBindTarget(result) => failure_case!(result, "texture-bind-target"),
        CommandResponse::AudioListenerUpsert(result) => {
            failure_case!(result, "audio-listener-upsert")
        }
        CommandResponse::AudioListenerDispose(result) => {
            failure_case!(result, "audio-listener-dispose")
        }
        CommandResponse::AudioResourceUpsert(result) => {
            failure_case!(result, "audio-resource-upsert")
        }
        CommandResponse::AudioSourceUpsert(result) => failure_case!(result, "audio-source-upsert"),
        CommandResponse::AudioSourceTransport(result) => {
            failure_case!(result, "audio-source-transport")
        }
        CommandResponse::AudioStateGet(result) => failure_case!(result, "audio-state-get"),
        CommandResponse::AudioSourceDispose(result) => {
            failure_case!(result, "audio-source-dispose")
        }
        CommandResponse::AudioResourceDispose(result) => {
            failure_case!(result, "audio-resource-dispose")
        }
        CommandResponse::GeometryUpsert(result) => failure_case!(result, "geometry-upsert"),
        CommandResponse::GeometryDispose(result) => failure_case!(result, "geometry-dispose"),
        CommandResponse::PrimitiveGeometryCreate(result) => {
            failure_case!(result, "primitive-geometry-create")
        }
        CommandResponse::EnvironmentUpsert(result) => failure_case!(result, "environment-upsert"),
        CommandResponse::EnvironmentDispose(result) => {
            failure_case!(result, "environment-dispose")
        }
        CommandResponse::Shadow3dConfigure(result) => failure_case!(result, "shadow3d-configure"),
        CommandResponse::RealmCreate(result) => failure_case!(result, "realm-create"),
        CommandResponse::RealmDispose(result) => failure_case!(result, "realm-dispose"),
        CommandResponse::RenderGraphUpsert(result) => {
            failure_case!(result, "render-graph-upsert")
        }
        CommandResponse::RenderGraphDispose(result) => {
            failure_case!(result, "render-graph-dispose")
        }
        CommandResponse::RenderGraphList(result) => {
            failure_case!(result, "render-graph-list")
        }
        CommandResponse::RealmRenderGraphBind(result) => {
            failure_case!(result, "realm-render-graph-bind")
        }
        CommandResponse::TargetUpsert(result) => failure_case!(result, "target-upsert"),
        CommandResponse::TargetMeasurement(result) => failure_case!(result, "target-measurement"),
        CommandResponse::TargetDispose(result) => failure_case!(result, "target-dispose"),
        CommandResponse::TargetLayerUpsert(result) => {
            failure_case!(result, "target-layer-upsert")
        }
        CommandResponse::TargetLayerDispose(result) => {
            failure_case!(result, "target-layer-dispose")
        }
        CommandResponse::ModelList(result) => failure_case!(result, "model-list"),
        CommandResponse::MaterialList(result) => failure_case!(result, "material-list"),
        CommandResponse::TextureList(result) => failure_case!(result, "texture-list"),
        CommandResponse::GeometryList(result) => failure_case!(result, "geometry-list"),
        CommandResponse::LightList(result) => failure_case!(result, "light-list"),
        CommandResponse::CameraList(result) => failure_case!(result, "camera-list"),
        _ => None,
    };

    let Some((command_type, message)) = failure else {
        return;
    };

    sys::push_error_event(
        engine,
        "command",
        message.to_string(),
        Some(command_id),
        Some(command_type.to_string()),
    );
}
