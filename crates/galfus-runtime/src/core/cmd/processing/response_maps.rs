use super::super::*;

pub(super) fn response_is_success(response: &CommandResponse) -> bool {
    match response {
        CommandResponse::NotificationSend(result) => result.success,
        CommandResponse::SystemDiagnosticsSet(result) => result.success,
        CommandResponse::SystemLogLevelSet(result) => result.success,
        CommandResponse::SystemLogLevelGet(result) => result.success,
        CommandResponse::SystemBuildVersionGet(result) => result.success,
        CommandResponse::WindowCreate(result) => result.success,
        CommandResponse::WindowClose(_) => true,
        CommandResponse::WindowMeasurement(result) => result.success,
        CommandResponse::WindowCursor(result) => result.success,
        CommandResponse::WindowState(result) => result.success,
        CommandResponse::UploadBufferDiscardAll(result) => result.success,
        CommandResponse::Camera3dUpsert(result) => result.success,
        CommandResponse::Camera3dDispose(result) => result.success,
        CommandResponse::Model3dUpsert(result) => result.success,
        CommandResponse::PoseUpdate(result) => result.success,
        CommandResponse::Model3dDispose(result) => result.success,
        CommandResponse::Light3dUpsert(result) => result.success,
        CommandResponse::Light3dDispose(result) => result.success,
        CommandResponse::MaterialUpsert(result) => result.success,
        CommandResponse::MaterialDispose(result) => result.success,
        CommandResponse::MaterialDefinitionUpsert(result) => result.success,
        CommandResponse::MaterialDefinitionDispose(result) => result.success,
        CommandResponse::MaterialInstanceUpsert(result) => result.success,
        CommandResponse::MaterialInstanceDispose(result) => result.success,
        CommandResponse::TextureCreateFromBuffer(result) => result.success,
        CommandResponse::TextureCreateSolidColor(result) => result.success,
        CommandResponse::TextureDispose(result) => result.success,
        CommandResponse::TextureBindTarget(result) => result.success,
        CommandResponse::AudioListenerUpsert(result) => result.success,
        CommandResponse::AudioListenerDispose(result) => result.success,
        CommandResponse::AudioResourceUpsert(result) => result.success,
        CommandResponse::AudioSourceUpsert(result) => result.success,
        CommandResponse::AudioSourceTransport(result) => result.success,
        CommandResponse::AudioStateGet(result) => result.success,
        CommandResponse::AudioSourceDispose(result) => result.success,
        CommandResponse::AudioResourceDispose(result) => result.success,
        CommandResponse::GeometryUpsert(result) => result.success,
        CommandResponse::GeometryDispose(result) => result.success,
        CommandResponse::PrimitiveGeometryCreate(result) => result.success,
        CommandResponse::EnvironmentUpsert(result) => result.success,
        CommandResponse::EnvironmentDispose(result) => result.success,
        CommandResponse::Shadow3dConfigure(result) => result.success,
        CommandResponse::RealmCreate(result) => result.success,
        CommandResponse::RealmDispose(result) => result.success,
        CommandResponse::RenderGraphUpsert(result) => result.success,
        CommandResponse::RenderGraphDispose(result) => result.success,
        CommandResponse::RenderGraphList(result) => result.success,
        CommandResponse::RealmRenderGraphBind(result) => result.success,
        CommandResponse::TargetUpsert(result) => result.success,
        CommandResponse::TargetMeasurement(result) => result.success,
        CommandResponse::TargetDispose(result) => result.success,
        CommandResponse::TargetLayerUpsert(result) => result.success,
        CommandResponse::TargetLayerDispose(result) => result.success,
        CommandResponse::ModelList(result) => result.success,
        CommandResponse::MaterialList(result) => result.success,
        CommandResponse::TextureList(result) => result.success,
        CommandResponse::GeometryList(result) => result.success,
        CommandResponse::LightList(result) => result.success,
        CommandResponse::CameraList(result) => result.success,
        CommandResponse::GizmoDrawLine(result) => result.status == 0,
        CommandResponse::GizmoDrawAabb(result) => result.status == 0,
        CommandResponse::GizmoDrawPolyline(result) => result.status == 0,
        _ => true,
    }
}

pub(super) fn response_message(response: &CommandResponse) -> Option<String> {
    match response {
        CommandResponse::NotificationSend(_) => None,
        CommandResponse::SystemDiagnosticsSet(result) => Some(result.message.clone()),
        CommandResponse::SystemLogLevelSet(result) => Some(result.message.clone()),
        CommandResponse::SystemLogLevelGet(result) => Some(result.message.clone()),
        CommandResponse::SystemBuildVersionGet(result) => Some(result.message.clone()),
        CommandResponse::WindowCreate(result) => Some(result.message.clone()),
        CommandResponse::WindowClose(_) => None,
        CommandResponse::WindowMeasurement(result) => Some(result.message.clone()),
        CommandResponse::WindowCursor(result) => Some(result.message.clone()),
        CommandResponse::WindowState(result) => Some(result.message.clone()),
        CommandResponse::UploadBufferDiscardAll(result) => Some(result.message.clone()),
        CommandResponse::Camera3dUpsert(result) => Some(result.message.clone()),
        CommandResponse::Camera3dDispose(result) => Some(result.message.clone()),
        CommandResponse::Model3dUpsert(result) => Some(result.message.clone()),
        CommandResponse::PoseUpdate(result) => Some(result.message.clone()),
        CommandResponse::Model3dDispose(result) => Some(result.message.clone()),
        CommandResponse::Light3dUpsert(result) => Some(result.message.clone()),
        CommandResponse::Light3dDispose(result) => Some(result.message.clone()),
        CommandResponse::MaterialUpsert(result) => Some(result.message.clone()),
        CommandResponse::MaterialDispose(result) => Some(result.message.clone()),
        CommandResponse::MaterialDefinitionUpsert(result) => Some(result.message.clone()),
        CommandResponse::MaterialDefinitionDispose(result) => Some(result.message.clone()),
        CommandResponse::MaterialInstanceUpsert(result) => Some(result.message.clone()),
        CommandResponse::MaterialInstanceDispose(result) => Some(result.message.clone()),
        CommandResponse::TextureCreateFromBuffer(result) => Some(result.message.clone()),
        CommandResponse::TextureCreateSolidColor(result) => Some(result.message.clone()),
        CommandResponse::TextureDispose(result) => Some(result.message.clone()),
        CommandResponse::TextureBindTarget(result) => Some(result.message.clone()),
        CommandResponse::AudioListenerUpsert(result) => Some(result.message.clone()),
        CommandResponse::AudioListenerDispose(result) => Some(result.message.clone()),
        CommandResponse::AudioResourceUpsert(result) => Some(result.message.clone()),
        CommandResponse::AudioSourceUpsert(result) => Some(result.message.clone()),
        CommandResponse::AudioSourceTransport(result) => Some(result.message.clone()),
        CommandResponse::AudioStateGet(result) => Some(result.message.clone()),
        CommandResponse::AudioSourceDispose(result) => Some(result.message.clone()),
        CommandResponse::AudioResourceDispose(result) => Some(result.message.clone()),
        CommandResponse::GeometryUpsert(result) => Some(result.message.clone()),
        CommandResponse::GeometryDispose(result) => Some(result.message.clone()),
        CommandResponse::PrimitiveGeometryCreate(result) => Some(result.message.clone()),
        CommandResponse::EnvironmentUpsert(result) => Some(result.message.clone()),
        CommandResponse::EnvironmentDispose(result) => Some(result.message.clone()),
        CommandResponse::Shadow3dConfigure(result) => Some(result.message.clone()),
        CommandResponse::RealmCreate(result) => Some(result.message.clone()),
        CommandResponse::RealmDispose(result) => Some(result.message.clone()),
        CommandResponse::RenderGraphUpsert(result) => Some(result.message.clone()),
        CommandResponse::RenderGraphDispose(result) => Some(result.message.clone()),
        CommandResponse::RenderGraphList(result) => Some(result.message.clone()),
        CommandResponse::RealmRenderGraphBind(result) => Some(result.message.clone()),
        CommandResponse::TargetUpsert(result) => Some(result.message.clone()),
        CommandResponse::TargetMeasurement(result) => Some(result.message.clone()),
        CommandResponse::TargetDispose(result) => Some(result.message.clone()),
        CommandResponse::TargetLayerUpsert(result) => Some(result.message.clone()),
        CommandResponse::TargetLayerDispose(result) => Some(result.message.clone()),
        CommandResponse::ModelList(result) => Some(result.message.clone()),
        CommandResponse::MaterialList(result) => Some(result.message.clone()),
        CommandResponse::TextureList(result) => Some(result.message.clone()),
        CommandResponse::GeometryList(result) => Some(result.message.clone()),
        CommandResponse::LightList(result) => Some(result.message.clone()),
        CommandResponse::CameraList(result) => Some(result.message.clone()),
        CommandResponse::GizmoDrawLine(_) => None,
        CommandResponse::GizmoDrawAabb(_) => None,
        CommandResponse::GizmoDrawPolyline(_) => None,
        _ => None,
    }
}

pub(super) fn response_with_message(response: CommandResponse, message: String) -> CommandResponse {
    match response {
        CommandResponse::SystemDiagnosticsSet(mut result) => {
            result.message = message;
            CommandResponse::SystemDiagnosticsSet(result)
        }
        CommandResponse::SystemLogLevelSet(mut result) => {
            result.message = message;
            CommandResponse::SystemLogLevelSet(result)
        }
        CommandResponse::SystemLogLevelGet(mut result) => {
            result.message = message;
            CommandResponse::SystemLogLevelGet(result)
        }
        CommandResponse::SystemBuildVersionGet(mut result) => {
            result.message = message;
            CommandResponse::SystemBuildVersionGet(result)
        }
        CommandResponse::WindowCreate(mut result) => {
            result.message = message;
            CommandResponse::WindowCreate(result)
        }
        CommandResponse::WindowMeasurement(mut result) => {
            result.message = message;
            CommandResponse::WindowMeasurement(result)
        }
        CommandResponse::WindowCursor(mut result) => {
            result.message = message;
            CommandResponse::WindowCursor(result)
        }
        CommandResponse::WindowState(mut result) => {
            result.message = message;
            CommandResponse::WindowState(result)
        }
        CommandResponse::UploadBufferDiscardAll(mut result) => {
            result.message = message;
            CommandResponse::UploadBufferDiscardAll(result)
        }
        CommandResponse::Camera3dUpsert(mut result) => {
            result.message = message;
            CommandResponse::Camera3dUpsert(result)
        }
        CommandResponse::Camera3dDispose(mut result) => {
            result.message = message;
            CommandResponse::Camera3dDispose(result)
        }
        CommandResponse::Model3dUpsert(mut result) => {
            result.message = message;
            CommandResponse::Model3dUpsert(result)
        }
        CommandResponse::PoseUpdate(mut result) => {
            result.message = message;
            CommandResponse::PoseUpdate(result)
        }
        CommandResponse::Model3dDispose(mut result) => {
            result.message = message;
            CommandResponse::Model3dDispose(result)
        }
        CommandResponse::Light3dUpsert(mut result) => {
            result.message = message;
            CommandResponse::Light3dUpsert(result)
        }
        CommandResponse::Light3dDispose(mut result) => {
            result.message = message;
            CommandResponse::Light3dDispose(result)
        }
        CommandResponse::MaterialUpsert(mut result) => {
            result.message = message;
            CommandResponse::MaterialUpsert(result)
        }
        CommandResponse::MaterialDispose(mut result) => {
            result.message = message;
            CommandResponse::MaterialDispose(result)
        }
        CommandResponse::MaterialDefinitionUpsert(mut result) => {
            result.message = message;
            CommandResponse::MaterialDefinitionUpsert(result)
        }
        CommandResponse::MaterialDefinitionDispose(mut result) => {
            result.message = message;
            CommandResponse::MaterialDefinitionDispose(result)
        }
        CommandResponse::MaterialInstanceUpsert(mut result) => {
            result.message = message;
            CommandResponse::MaterialInstanceUpsert(result)
        }
        CommandResponse::MaterialInstanceDispose(mut result) => {
            result.message = message;
            CommandResponse::MaterialInstanceDispose(result)
        }
        CommandResponse::TextureCreateFromBuffer(mut result) => {
            result.message = message;
            CommandResponse::TextureCreateFromBuffer(result)
        }
        CommandResponse::TextureCreateSolidColor(mut result) => {
            result.message = message;
            CommandResponse::TextureCreateSolidColor(result)
        }
        CommandResponse::TextureDispose(mut result) => {
            result.message = message;
            CommandResponse::TextureDispose(result)
        }
        CommandResponse::TextureBindTarget(mut result) => {
            result.message = message;
            CommandResponse::TextureBindTarget(result)
        }
        CommandResponse::AudioListenerUpsert(mut result) => {
            result.message = message;
            CommandResponse::AudioListenerUpsert(result)
        }
        CommandResponse::AudioListenerDispose(mut result) => {
            result.message = message;
            CommandResponse::AudioListenerDispose(result)
        }
        CommandResponse::AudioResourceUpsert(mut result) => {
            result.message = message;
            CommandResponse::AudioResourceUpsert(result)
        }
        CommandResponse::AudioSourceUpsert(mut result) => {
            result.message = message;
            CommandResponse::AudioSourceUpsert(result)
        }
        CommandResponse::AudioSourceTransport(mut result) => {
            result.message = message;
            CommandResponse::AudioSourceTransport(result)
        }
        CommandResponse::AudioStateGet(mut result) => {
            result.message = message;
            CommandResponse::AudioStateGet(result)
        }
        CommandResponse::AudioSourceDispose(mut result) => {
            result.message = message;
            CommandResponse::AudioSourceDispose(result)
        }
        CommandResponse::AudioResourceDispose(mut result) => {
            result.message = message;
            CommandResponse::AudioResourceDispose(result)
        }
        CommandResponse::GeometryUpsert(mut result) => {
            result.message = message;
            CommandResponse::GeometryUpsert(result)
        }
        CommandResponse::GeometryDispose(mut result) => {
            result.message = message;
            CommandResponse::GeometryDispose(result)
        }
        CommandResponse::PrimitiveGeometryCreate(mut result) => {
            result.message = message;
            CommandResponse::PrimitiveGeometryCreate(result)
        }
        CommandResponse::EnvironmentUpsert(mut result) => {
            result.message = message;
            CommandResponse::EnvironmentUpsert(result)
        }
        CommandResponse::EnvironmentDispose(mut result) => {
            result.message = message;
            CommandResponse::EnvironmentDispose(result)
        }
        CommandResponse::Shadow3dConfigure(mut result) => {
            result.message = message;
            CommandResponse::Shadow3dConfigure(result)
        }
        CommandResponse::RealmCreate(mut result) => {
            result.message = message;
            CommandResponse::RealmCreate(result)
        }
        CommandResponse::RealmDispose(mut result) => {
            result.message = message;
            CommandResponse::RealmDispose(result)
        }
        CommandResponse::TargetUpsert(mut result) => {
            result.message = message;
            CommandResponse::TargetUpsert(result)
        }
        CommandResponse::TargetMeasurement(mut result) => {
            result.message = message;
            CommandResponse::TargetMeasurement(result)
        }
        CommandResponse::TargetDispose(mut result) => {
            result.message = message;
            CommandResponse::TargetDispose(result)
        }
        CommandResponse::TargetLayerUpsert(mut result) => {
            result.message = message;
            CommandResponse::TargetLayerUpsert(result)
        }
        CommandResponse::TargetLayerDispose(mut result) => {
            result.message = message;
            CommandResponse::TargetLayerDispose(result)
        }
        CommandResponse::ModelList(mut result) => {
            result.message = message;
            CommandResponse::ModelList(result)
        }
        CommandResponse::MaterialList(mut result) => {
            result.message = message;
            CommandResponse::MaterialList(result)
        }
        CommandResponse::TextureList(mut result) => {
            result.message = message;
            CommandResponse::TextureList(result)
        }
        CommandResponse::GeometryList(mut result) => {
            result.message = message;
            CommandResponse::GeometryList(result)
        }
        CommandResponse::LightList(mut result) => {
            result.message = message;
            CommandResponse::LightList(result)
        }
        CommandResponse::CameraList(mut result) => {
            result.message = message;
            CommandResponse::CameraList(result)
        }
        other => other,
    }
}
