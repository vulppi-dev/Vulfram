use super::super::*;
use crate::core::state::EngineState;
use galfus_runtime::DeferredCommandKey;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

#[derive(PartialEq, Eq)]
pub(super) enum DeferredFailureKind {
    Transient,
    Permanent,
}
pub(super) fn command_signature(cmd: &EngineCmd) -> u64 {
    let mut hasher = DefaultHasher::new();
    match rmp_serde::to_vec_named(cmd) {
        Ok(bytes) => hasher.write(&bytes),
        Err(_) => {
            hasher.write_u8(command_type_for_cmd(cmd).len() as u8);
            hasher.write(command_type_for_cmd(cmd).as_bytes());
        }
    }
    hasher.finish()
}

pub(crate) fn deferred_command_key(command_id: u64, cmd: &EngineCmd) -> DeferredCommandKey {
    DeferredCommandKey {
        command_id,
        command_signature: command_signature(cmd),
    }
}

pub(super) fn command_type_for_cmd(cmd: &EngineCmd) -> &'static str {
    match cmd {
        EngineCmd::CmdNotificationSend(_) => "notification-send",
        EngineCmd::CmdSystemDiagnosticsSet(_) => "system-diagnostics-set",
        EngineCmd::CmdSystemLogLevelSet(_) => "system-log-level-set",
        EngineCmd::CmdSystemLogLevelGet(_) => "system-log-level-get",
        EngineCmd::CmdSystemBuildVersionGet(_) => "system-build-version-get",
        EngineCmd::CmdWindowCreate(_) => "window-create",
        EngineCmd::CmdWindowClose(_) => "window-close",
        EngineCmd::CmdWindowMeasurement(_) => "window-measurement",
        EngineCmd::CmdWindowCursor(_) => "window-cursor",
        EngineCmd::CmdWindowState(_) => "window-state",
        EngineCmd::CmdUploadBufferDiscardAll(_) => "upload-buffer-discard-all",
        EngineCmd::CmdCamera3dUpsert(_) => "camera3d-upsert",
        EngineCmd::CmdCamera2dUpsert(_) => "camera2d-upsert",
        EngineCmd::CmdCamera3dDispose(_) => "camera3d-dispose",
        EngineCmd::CmdCamera2dDispose(_) => "camera2d-dispose",
        EngineCmd::CmdModel3dUpsert(_) => "model3d-upsert",
        EngineCmd::CmdSprite2dUpsert(_) => "sprite2d-upsert",
        EngineCmd::CmdShape2dUpsert(_) => "shape2d-upsert",
        EngineCmd::CmdRealm2dShadowConfigUpdate(_) => "realm2d-shadow-config-update",
        EngineCmd::CmdPoseUpdate(_) => "pose-update",
        EngineCmd::CmdModel3dDispose(_) => "model3d-dispose",
        EngineCmd::CmdSprite2dDispose(_) => "sprite2d-dispose",
        EngineCmd::CmdShape2dDispose(_) => "shape2d-dispose",
        EngineCmd::CmdLight3dUpsert(_) => "light3d-upsert",
        EngineCmd::CmdLight3dDispose(_) => "light3d-dispose",
        EngineCmd::CmdMaterialUpsert(_) => "material-upsert",
        EngineCmd::CmdMaterialDispose(_) => "material-dispose",
        EngineCmd::CmdMaterialDefinitionUpsert(_) => "material-definition-upsert",
        EngineCmd::CmdMaterialDefinitionDispose(_) => "material-definition-dispose",
        EngineCmd::CmdMaterialInstanceUpsert(_) => "material-instance-upsert",
        EngineCmd::CmdMaterialInstanceDispose(_) => "material-instance-dispose",
        EngineCmd::CmdTextureCreateFromBuffer(_) => "texture-create-from-buffer",
        EngineCmd::CmdTextureCreateSolidColor(_) => "texture-create-solid-color",
        EngineCmd::CmdTextureDispose(_) => "texture-dispose",
        EngineCmd::CmdTextureBindTarget(_) => "texture-bind-target",
        EngineCmd::CmdAudioListenerUpsert(_) => "audio-listener-upsert",
        EngineCmd::CmdAudioListenerDispose(_) => "audio-listener-dispose",
        EngineCmd::CmdAudioResourceUpsert(_) => "audio-resource-upsert",
        EngineCmd::CmdAudioSourceUpsert(_) => "audio-source-upsert",
        EngineCmd::CmdAudioSourceTransport(_) => "audio-source-transport",
        EngineCmd::CmdAudioStateGet(_) => "audio-state-get",
        EngineCmd::CmdAudioSourceDispose(_) => "audio-source-dispose",
        EngineCmd::CmdAudioResourceDispose(_) => "audio-resource-dispose",
        EngineCmd::CmdGeometryUpsert(_) => "geometry-upsert",
        EngineCmd::CmdGeometryDispose(_) => "geometry-dispose",
        EngineCmd::CmdPrimitiveGeometryCreate(_) => "primitive-geometry-create",
        EngineCmd::CmdEnvironmentUpsert(_) => "environment-upsert",
        EngineCmd::CmdEnvironmentDispose(_) => "environment-dispose",
        EngineCmd::CmdShadow3dConfigure(_) => "shadow3d-configure",
        EngineCmd::CmdRealmCreate(_) => "realm-create",
        EngineCmd::CmdRealmDispose(_) => "realm-dispose",
        EngineCmd::CmdRenderGraphUpsert(_) => "render-graph-upsert",
        EngineCmd::CmdRenderGraphDispose(_) => "render-graph-dispose",
        EngineCmd::CmdRenderGraphList(_) => "render-graph-list",
        EngineCmd::CmdRealmRenderGraphBind(_) => "realm-render-graph-bind",
        EngineCmd::CmdTargetUpsert(_) => "target-upsert",
        EngineCmd::CmdTargetMeasurement(_) => "target-measurement",
        EngineCmd::CmdTargetDispose(_) => "target-dispose",
        EngineCmd::CmdTargetLayerUpsert(_) => "target-layer-upsert",
        EngineCmd::CmdTargetLayerDispose(_) => "target-layer-dispose",
        EngineCmd::CmdModelList(_) => "model-list",
        EngineCmd::CmdMaterialList(_) => "material-list",
        EngineCmd::CmdTextureList(_) => "texture-list",
        EngineCmd::CmdGeometryList(_) => "geometry-list",
        EngineCmd::CmdLightList(_) => "light-list",
        EngineCmd::CmdCameraList(_) => "camera-list",
        EngineCmd::CmdGizmoDrawLine(_) => "gizmo-draw-line",
        EngineCmd::CmdGizmoDrawAabb(_) => "gizmo-draw-aabb",
        EngineCmd::CmdGizmoDrawPolyline(_) => "gizmo-draw-polyline",
        _ => "query-or-extended",
    }
}

pub(super) fn emit_deferred_event(
    engine: &mut EngineState,
    command_id: u64,
    command_type: &str,
    attempts: u32,
    reason: String,
) {
    engine
        .runtime
        .push_event(EngineEvent::System(SystemEvent::CommandDeferred {
            command_id,
            command_type: command_type.to_string(),
            attempts,
            reason,
        }));
}

pub(super) fn emit_applied_event(
    engine: &mut EngineState,
    command_id: u64,
    command_type: &str,
    attempts: u32,
) {
    engine
        .runtime
        .push_event(EngineEvent::System(SystemEvent::CommandApplied {
            command_id,
            command_type: command_type.to_string(),
            attempts,
        }));
}

pub(super) fn emit_dropped_event(
    engine: &mut EngineState,
    command_id: u64,
    command_type: &str,
    attempts: u32,
    reason: String,
) {
    engine
        .runtime
        .push_event(EngineEvent::System(SystemEvent::CommandDropped {
            command_id,
            command_type: command_type.to_string(),
            attempts,
            reason,
        }));
}

fn command_has_pending_dependencies(engine: &EngineState, cmd: &EngineCmd) -> bool {
    match cmd {
        // Query/measurement commands must be non-blocking and should not be deferred.
        EngineCmd::CmdWindowMeasurement(_) => false,
        EngineCmd::CmdTargetMeasurement(_) => false,
        EngineCmd::CmdShadow3dConfigure(args) => {
            engine.device.is_none()
                || engine
                    .render
                    .get(&args.window_id)
                    .and_then(|state| state.shadow_3d.as_ref())
                    .is_none()
        }
        EngineCmd::CmdTextureCreateSolidColor(_) => {
            engine.device.is_none() || engine.queue.is_none()
        }
        EngineCmd::CmdTextureCreateFromBuffer(args) => {
            !engine.buffers.uploads.contains_key(&args.buffer_id)
        }
        EngineCmd::CmdPoseUpdate(args) => {
            let realm_id = crate::core::realm::RealmId(args.realm_id);
            let has_realm = engine
                .universal_state
                .scene
                .realm3d
                .entities
                .contains_key(&realm_id);
            let has_model = engine
                .universal_state
                .scene
                .realm3d
                .entities
                .get(&realm_id)
                .and_then(|entities| entities.models.get(&args.model_id))
                .is_some();
            let has_buffer = engine
                .buffers
                .uploads
                .contains_key(&args.matrices_buffer_id);
            let has_render = if let Some(window_id) = args.window_id {
                engine.render.states.contains_key(&window_id)
            } else {
                !engine.render.states.is_empty()
            };
            !has_realm || !has_model || !has_buffer || !has_render
        }
        EngineCmd::CmdCamera3dUpsert(args) => match args {
            CmdCamera3dUpsertArgs::Create(_) => false,
            CmdCamera3dUpsertArgs::Update(update) => {
                let realm_id = crate::core::realm::RealmId(update.realm_id);
                engine
                    .universal_state
                    .scene
                    .realm3d
                    .entities
                    .get(&realm_id)
                    .and_then(|entities| entities.cameras.get(&update.camera_id))
                    .is_none()
            }
        },
        EngineCmd::CmdCamera2dUpsert(args) => match args {
            CmdCamera2dUpsertArgs::Create(_) => false,
            CmdCamera2dUpsertArgs::Update(update) => {
                let realm_id = crate::core::realm::RealmId(update.realm_id);
                engine
                    .universal_state
                    .scene
                    .realm2d
                    .entities
                    .get(&realm_id)
                    .and_then(|entities| entities.cameras.get(&update.camera_id))
                    .is_none()
            }
        },
        EngineCmd::CmdModel3dUpsert(args) => match args {
            CmdModel3dUpsertArgs::Create(_) => false,
            CmdModel3dUpsertArgs::Update(update) => {
                let realm_id = crate::core::realm::RealmId(update.realm_id);
                engine
                    .universal_state
                    .scene
                    .realm3d
                    .entities
                    .get(&realm_id)
                    .and_then(|entities| entities.models.get(&update.model_id))
                    .is_none()
            }
        },
        EngineCmd::CmdSprite2dUpsert(args) => match args {
            CmdSprite2dUpsertArgs::Create(_) => false,
            CmdSprite2dUpsertArgs::Update(update) => {
                let realm_id = crate::core::realm::RealmId(update.realm_id);
                engine
                    .universal_state
                    .scene
                    .realm2d
                    .entities
                    .get(&realm_id)
                    .and_then(|entities| entities.sprites.get(&update.sprite_id))
                    .is_none()
            }
        },
        EngineCmd::CmdShape2dUpsert(args) => match args {
            CmdShape2dUpsertArgs::Create(_) => false,
            CmdShape2dUpsertArgs::Update(update) => {
                let realm_id = crate::core::realm::RealmId(update.realm_id);
                engine
                    .universal_state
                    .scene
                    .realm2d
                    .entities
                    .get(&realm_id)
                    .and_then(|entities| entities.shapes.get(&update.shape_id))
                    .is_none()
            }
        },
        EngineCmd::CmdLight3dUpsert(args) => match args {
            CmdLight3dUpsertArgs::Create(_) => false,
            CmdLight3dUpsertArgs::Update(update) => {
                let realm_id = crate::core::realm::RealmId(update.realm_id);
                engine
                    .universal_state
                    .scene
                    .realm3d
                    .entities
                    .get(&realm_id)
                    .and_then(|entities| entities.lights.get(&update.light_id))
                    .is_none()
            }
        },
        EngineCmd::CmdMaterialUpsert(args) => match args {
            CmdMaterialUpsertArgs::Create(_) => false,
            CmdMaterialUpsertArgs::Update(update) => {
                let realm3d = &engine.universal_state.scene.realm3d;
                !realm3d.materials.contains_key(&update.material_id)
            }
        },
        EngineCmd::CmdMaterialDefinitionUpsert(args) => match args {
            CmdMaterialDefinitionUpsertArgs::Create(_) => false,
            CmdMaterialDefinitionUpsertArgs::Update(update) => !engine
                .universal_state
                .scene
                .material_definitions
                .contains_key(&update.definition_id),
        },
        EngineCmd::CmdMaterialDefinitionDispose(args) => !engine
            .universal_state
            .scene
            .material_definitions
            .contains_key(&args.definition_id),
        EngineCmd::CmdMaterialInstanceUpsert(args) => match args {
            CmdMaterialInstanceUpsertArgs::Create(create) => !engine
                .universal_state
                .scene
                .material_definitions
                .values()
                .any(|definition| definition.slug == create.slug),
            CmdMaterialInstanceUpsertArgs::Update(update) => !engine
                .universal_state
                .scene
                .material_instances
                .contains_key(&update.material_id),
        },
        EngineCmd::CmdMaterialInstanceDispose(args) => !engine
            .universal_state
            .scene
            .material_instances
            .contains_key(&args.material_id),
        EngineCmd::CmdGeometryUpsert(args) => match args {
            CmdGeometryUpsertArgs::Create(_) => false,
            CmdGeometryUpsertArgs::Update(update) => !engine
                .universal_state
                .scene
                .realm3d
                .geometries
                .contains_key(&update.geometry_id),
        },
        EngineCmd::CmdEnvironmentUpsert(args) => match args {
            CmdEnvironmentUpsertArgs::Create(_) => false,
            CmdEnvironmentUpsertArgs::Update(update) => !engine
                .universal_state
                .scene
                .realm3d
                .environment_profiles
                .contains_key(&update.environment_id),
        },
        EngineCmd::CmdTargetUpsert(args) => {
            matches!(args.kind, crate::core::target::TargetKind::Window)
                && args
                    .window_id
                    .is_some_and(|window_id| !engine.window.states.contains_key(&window_id))
        }
        EngineCmd::CmdTargetLayerUpsert(args) => {
            let realm_id = crate::core::realm::RealmId(args.realm_id);
            !engine
                .universal_state
                .composition
                .realms
                .entries
                .contains_key(&realm_id)
                || !engine
                    .universal_state
                    .targets
                    .targets
                    .entries
                    .contains_key(&crate::core::target::TargetId(args.target_id))
        }
        _ => false,
    }
}

pub(super) fn classify_failed_response(
    engine: &EngineState,
    cmd: &EngineCmd,
    response: &CommandResponse,
) -> Option<(DeferredFailureKind, String)> {
    if super::response_maps::response_is_success(response) {
        return None;
    }

    let message = super::response_maps::response_message(response)
        .unwrap_or_else(|| "command failed".to_string());
    let kind = if command_has_pending_dependencies(engine, cmd) {
        DeferredFailureKind::Transient
    } else {
        DeferredFailureKind::Permanent
    };
    Some((kind, message))
}

pub(super) use galfus_runtime::should_drop_deferred;
