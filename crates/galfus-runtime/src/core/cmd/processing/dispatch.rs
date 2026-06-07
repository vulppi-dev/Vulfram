use super::super::*;
use crate::core::platforms::PlatformProxy;
use crate::core::state::EngineState;

fn mark_windows_dirty(engine: &mut EngineState) {
    for window_state in engine.window.states.values_mut() {
        window_state.is_dirty = true;
    }
}

fn emit_resource_mutation(
    engine: &mut EngineState,
    kind: &str,
    id: u64,
    action: &str,
    realm_id: Option<u32>,
    window_id: Option<u32>,
) {
    engine.revision = engine.revision.saturating_add(1);
    engine
        .runtime
        .push_event(EngineEvent::System(SystemEvent::ResourceMutation {
            kind: kind.to_string(),
            id,
            action: action.to_string(),
            realm_id,
            window_id,
            revision: engine.revision,
        }));
}

pub(super) fn dispatch_command(
    engine: &mut EngineState,
    platform: &mut dyn PlatformProxy,
    pack: EngineCmdEnvelope,
) {
    match pack.cmd {
        EngineCmd::CmdNotificationSend(args) => {
            let result =
                sys::engine_cmd_notification_send(engine, platform.event_loop_proxy(), &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::NotificationSend(result),
            });
        }
        EngineCmd::CmdSystemDiagnosticsSet(args) => {
            let result = sys::engine_cmd_system_diagnostics_set(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::SystemDiagnosticsSet(result),
            });
        }
        EngineCmd::CmdSystemLogLevelSet(args) => {
            let result = sys::engine_cmd_system_log_level_set(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::SystemLogLevelSet(result),
            });
        }
        EngineCmd::CmdSystemLogLevelGet(args) => {
            let result = sys::engine_cmd_system_log_level_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::SystemLogLevelGet(result),
            });
        }
        EngineCmd::CmdSystemBuildVersionGet(args) => {
            let result = sys::engine_cmd_system_build_version_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::SystemBuildVersionGet(result),
            });
        }
        EngineCmd::CmdWindowCreate(args) => {
            match platform.handle_window_create(engine, pack.id, &args) {
                Ok(()) => {}
                Err(result) => {
                    engine.runtime.push_response(CommandResponseEnvelope {
                        id: pack.id,
                        response: CommandResponse::WindowCreate(result),
                    });
                }
            }
        }
        EngineCmd::CmdWindowClose(args) => {
            let result = win::engine_cmd_window_close(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::WindowClose(result),
            });
        }
        EngineCmd::CmdWindowMeasurement(args) => {
            let result = win::engine_cmd_window_measurement(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::WindowMeasurement(result),
            });
        }
        EngineCmd::CmdWindowCursor(args) => {
            let result = win::engine_cmd_window_cursor(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::WindowCursor(result),
            });
        }
        EngineCmd::CmdWindowState(args) => {
            let result = win::engine_cmd_window_state(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::WindowState(result),
            });
        }
        EngineCmd::CmdUploadBufferDiscardAll(args) => {
            let result = buf::engine_cmd_upload_buffer_discard_all(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::UploadBufferDiscardAll(result),
            });
        }
        EngineCmd::CmdCamera3dUpsert(args) => {
            let (_id, _realm_id, _action) = match &args {
                CmdCamera3dUpsertArgs::Create(create_args) => (
                    create_args.camera_id as u64,
                    Some(create_args.realm_id),
                    "created",
                ),
                CmdCamera3dUpsertArgs::Update(update_args) => (
                    update_args.camera_id as u64,
                    Some(update_args.realm_id),
                    "updated",
                ),
            };
            let result = match args {
                CmdCamera3dUpsertArgs::Create(create_args) => {
                    let create_result = res::engine_cmd_camera_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdCamera3dUpsertArgs::Update(update_args) => {
                    let update_result = res::engine_cmd_camera_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            let success = result.success;
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Camera3dUpsert(result),
            });
            if success {
                emit_resource_mutation(engine, "camera", _id, _action, _realm_id, None);
            }
        }
        EngineCmd::CmdCamera2dUpsert(args) => {
            let result = match args {
                CmdCamera2dUpsertArgs::Create(create_args) => res::engine_cmd_camera2d_upsert(
                    engine,
                    res::CmdCamera2dUpsertArgs::Create(create_args),
                ),
                CmdCamera2dUpsertArgs::Update(update_args) => res::engine_cmd_camera2d_upsert(
                    engine,
                    res::CmdCamera2dUpsertArgs::Update(update_args),
                ),
            };
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Camera2dUpsert(result),
            });
        }
        EngineCmd::CmdCamera3dDispose(args) => {
            let result = res::engine_cmd_camera_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Camera3dDispose(result),
            });
        }
        EngineCmd::CmdCamera2dDispose(args) => {
            let result = res::engine_cmd_camera2d_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Camera2dDispose(result),
            });
        }
        EngineCmd::CmdModel3dUpsert(args) => {
            let result = match args {
                CmdModel3dUpsertArgs::Create(create_args) => {
                    let create_result = res::engine_cmd_model_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdModel3dUpsertArgs::Update(update_args) => {
                    let update_result = res::engine_cmd_model_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Model3dUpsert(result),
            });
        }
        EngineCmd::CmdSprite2dUpsert(args) => {
            let result = match args {
                CmdSprite2dUpsertArgs::Create(create_args) => res::engine_cmd_sprite2d_upsert(
                    engine,
                    res::CmdSprite2dUpsertArgs::Create(create_args),
                ),
                CmdSprite2dUpsertArgs::Update(update_args) => res::engine_cmd_sprite2d_upsert(
                    engine,
                    res::CmdSprite2dUpsertArgs::Update(update_args),
                ),
            };
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Sprite2dUpsert(result),
            });
        }
        EngineCmd::CmdShape2dUpsert(args) => {
            let result = match args {
                CmdShape2dUpsertArgs::Create(create_args) => res::engine_cmd_shape2d_upsert(
                    engine,
                    res::CmdShape2dUpsertArgs::Create(create_args),
                ),
                CmdShape2dUpsertArgs::Update(update_args) => res::engine_cmd_shape2d_upsert(
                    engine,
                    res::CmdShape2dUpsertArgs::Update(update_args),
                ),
            };
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Shape2dUpsert(result),
            });
        }
        EngineCmd::CmdRealm2dShadowConfigUpdate(args) => {
            let result = res::engine_cmd_realm2d_shadow_config_update(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Realm2dShadowConfigUpdate(result),
            });
        }
        EngineCmd::CmdPoseUpdate(args) => {
            let result = res::engine_cmd_pose_update(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::PoseUpdate(result),
            });
        }
        EngineCmd::CmdModel3dDispose(args) => {
            let result = res::engine_cmd_model_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Model3dDispose(result),
            });
        }
        EngineCmd::CmdSprite2dDispose(args) => {
            let result = res::engine_cmd_sprite2d_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Sprite2dDispose(result),
            });
        }
        EngineCmd::CmdShape2dDispose(args) => {
            let result = res::engine_cmd_shape2d_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Shape2dDispose(result),
            });
        }
        EngineCmd::CmdLight3dUpsert(args) => {
            let result = match args {
                CmdLight3dUpsertArgs::Create(create_args) => {
                    let create_result = res::engine_cmd_light_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdLight3dUpsertArgs::Update(update_args) => {
                    let update_result = res::engine_cmd_light_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Light3dUpsert(result),
            });
        }
        EngineCmd::CmdLight3dDispose(args) => {
            let result = res::engine_cmd_light_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Light3dDispose(result),
            });
        }
        EngineCmd::CmdMaterialUpsert(args) => {
            let result = match args {
                CmdMaterialUpsertArgs::Create(create_args) => {
                    let create_result = res::engine_cmd_material_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdMaterialUpsertArgs::Update(update_args) => {
                    let update_result = res::engine_cmd_material_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::MaterialUpsert(result),
            });
        }
        EngineCmd::CmdMaterialDispose(args) => {
            let result = res::engine_cmd_material_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::MaterialDispose(result),
            });
        }
        EngineCmd::CmdMaterialDefinitionUpsert(args) => {
            let result = match args {
                CmdMaterialDefinitionUpsertArgs::Create(create_args) => {
                    res::engine_cmd_material_definition_create(engine, &create_args)
                }
                CmdMaterialDefinitionUpsertArgs::Update(update_args) => {
                    res::engine_cmd_material_definition_update(engine, &update_args)
                }
            };
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::MaterialDefinitionUpsert(result),
            });
        }
        EngineCmd::CmdMaterialDefinitionDispose(args) => {
            let result = res::engine_cmd_material_definition_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::MaterialDefinitionDispose(result),
            });
        }
        EngineCmd::CmdMaterialInstanceUpsert(args) => {
            let result = match args {
                CmdMaterialInstanceUpsertArgs::Create(create_args) => {
                    res::engine_cmd_material_instance_create(engine, &create_args)
                }
                CmdMaterialInstanceUpsertArgs::Update(update_args) => {
                    res::engine_cmd_material_instance_update(engine, &update_args)
                }
            };
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::MaterialInstanceUpsert(result),
            });
        }
        EngineCmd::CmdMaterialInstanceDispose(args) => {
            let result = res::engine_cmd_material_instance_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::MaterialInstanceDispose(result),
            });
        }
        EngineCmd::CmdTextureCreateFromBuffer(args) => {
            let result = res::engine_cmd_texture_create_from_buffer(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TextureCreateFromBuffer(result),
            });
        }
        EngineCmd::CmdTextureCreateSolidColor(args) => {
            let result = res::engine_cmd_texture_create_solid_color(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TextureCreateSolidColor(result),
            });
        }
        EngineCmd::CmdTextureUpsert(args) => {
            let (texture_id, result) = match args {
                CmdTextureUpsertArgs::FromBuffer(create_args) => {
                    let texture_id = create_args.texture_id;
                    let result = res::engine_cmd_texture_create_from_buffer(engine, &create_args);
                    (
                        texture_id,
                        CmdResultSimple {
                            success: result.success,
                            message: result.message,
                        },
                    )
                }
                CmdTextureUpsertArgs::SolidColor(create_args) => {
                    let texture_id = create_args.texture_id;
                    let result = res::engine_cmd_texture_create_solid_color(engine, &create_args);
                    (
                        texture_id,
                        CmdResultSimple {
                            success: result.success,
                            message: result.message,
                        },
                    )
                }
            };
            let success = result.success;
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TextureUpsert(result),
            });
            if success {
                emit_resource_mutation(
                    engine,
                    "texture",
                    texture_id as u64,
                    "upserted",
                    None,
                    None,
                );
            }
        }
        EngineCmd::CmdTextureDispose(args) => {
            let result = res::engine_cmd_texture_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TextureDispose(result),
            });
        }
        EngineCmd::CmdTextureBindTarget(args) => {
            let result = res::engine_cmd_texture_bind_target(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TextureBindTarget(result),
            });
        }
        EngineCmd::CmdAudioListenerUpsert(args) => {
            let result = match args {
                CmdAudioListenerUpsertArgs::Create(create_args) => {
                    let create_result =
                        audio::engine_cmd_audio_listener_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdAudioListenerUpsertArgs::Update(update_args) => {
                    let update_result =
                        audio::engine_cmd_audio_listener_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioListenerUpsert(result),
            });
        }
        EngineCmd::CmdAudioListenerDispose(args) => {
            let result = audio::engine_cmd_audio_listener_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioListenerDispose(result),
            });
        }
        EngineCmd::CmdAudioResourceUpsert(args) => {
            let result = audio::engine_cmd_audio_resource_upsert(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioResourceUpsert(result),
            });
        }
        EngineCmd::CmdAudioSourceUpsert(args) => {
            let result = match args {
                CmdAudioSourceUpsertArgs::Create(create_args) => {
                    let create_result = audio::engine_cmd_audio_source_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdAudioSourceUpsertArgs::Update(update_args) => {
                    let update_result = audio::engine_cmd_audio_source_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioSourceUpsert(result),
            });
        }
        EngineCmd::CmdAudioSourceTransport(args) => {
            let result = audio::engine_cmd_audio_source_transport(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioSourceTransport(result),
            });
        }
        EngineCmd::CmdAudioStateGet(args) => {
            let result = audio::engine_cmd_audio_state_get(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioStateGet(result),
            });
        }
        EngineCmd::CmdAudioSourceDispose(args) => {
            let result = audio::engine_cmd_audio_source_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioSourceDispose(result),
            });
        }
        EngineCmd::CmdAudioResourceDispose(args) => {
            let result = audio::engine_cmd_audio_resource_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::AudioResourceDispose(result),
            });
        }
        EngineCmd::CmdGeometryUpsert(args) => {
            let result = match args {
                CmdGeometryUpsertArgs::Create(create_args) => {
                    let create_result = res::engine_cmd_geometry_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdGeometryUpsertArgs::Update(update_args) => {
                    let update_result = res::engine_cmd_geometry_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::GeometryUpsert(result),
            });
        }
        EngineCmd::CmdGeometryDispose(args) => {
            let result = res::engine_cmd_geometry_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::GeometryDispose(result),
            });
        }
        EngineCmd::CmdPrimitiveGeometryCreate(args) => {
            let result = res::engine_cmd_primitive_geometry_create(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::PrimitiveGeometryCreate(result),
            });
        }
        EngineCmd::CmdEnvironmentUpsert(args) => {
            let result = match args {
                CmdEnvironmentUpsertArgs::Create(create_args) => {
                    let create_result = res::engine_cmd_environment_create(engine, &create_args);
                    CmdResultSimple {
                        success: create_result.success,
                        message: create_result.message,
                    }
                }
                CmdEnvironmentUpsertArgs::Update(update_args) => {
                    let update_result = res::engine_cmd_environment_update(engine, &update_args);
                    CmdResultSimple {
                        success: update_result.success,
                        message: update_result.message,
                    }
                }
            };
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::EnvironmentUpsert(result),
            });
        }
        EngineCmd::CmdEnvironmentDispose(args) => {
            let result = res::engine_cmd_environment_dispose(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::EnvironmentDispose(result),
            });
        }
        EngineCmd::CmdShadow3dConfigure(args) => {
            let result = res::shadow::engine_cmd_shadow3d_configure(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::Shadow3dConfigure(result),
            });
        }
        EngineCmd::CmdRealmCreate(args) => {
            let result = realm::engine_cmd_realm_create(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
            }
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::RealmCreate(result),
            });
        }
        EngineCmd::CmdRealmDispose(args) => {
            let result = realm::engine_cmd_realm_dispose(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
            }
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::RealmDispose(result),
            });
        }
        EngineCmd::CmdRenderGraphUpsert(args) => {
            let result = realm::engine_cmd_render_graph_upsert(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
            }
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::RenderGraphUpsert(result),
            });
        }
        EngineCmd::CmdRenderGraphDispose(args) => {
            let result = realm::engine_cmd_render_graph_dispose(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
            }
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::RenderGraphDispose(result),
            });
        }
        EngineCmd::CmdRenderGraphList(args) => {
            let result = realm::engine_cmd_render_graph_list(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::RenderGraphList(result),
            });
        }
        EngineCmd::CmdRealmRenderGraphBind(args) => {
            let result = realm::engine_cmd_realm_render_graph_bind(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
            }
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::RealmRenderGraphBind(result),
            });
        }
        EngineCmd::CmdTargetUpsert(args) => {
            let result = target::engine_cmd_target_upsert(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
                emit_resource_mutation(
                    engine,
                    "target",
                    args.target_id,
                    "upserted",
                    None,
                    args.window_id,
                );
            }
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TargetUpsert(result),
            });
        }
        EngineCmd::CmdTargetMeasurement(args) => {
            let result = target::engine_cmd_target_measurement(engine, &args);
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TargetMeasurement(result),
            });
        }
        EngineCmd::CmdTargetDispose(args) => {
            let result = target::engine_cmd_target_dispose(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
                emit_resource_mutation(engine, "target", args.target_id, "disposed", None, None);
            }
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TargetDispose(result),
            });
        }
        EngineCmd::CmdTargetLayerUpsert(args) => {
            let result = target::engine_cmd_target_layer_upsert(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
                emit_resource_mutation(
                    engine,
                    "target-layer",
                    args.target_id,
                    "upserted",
                    Some(args.realm_id),
                    None,
                );
            }
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TargetLayerUpsert(result),
            });
        }
        EngineCmd::CmdTargetLayerDispose(args) => {
            let result = target::engine_cmd_target_layer_dispose(engine, &args);
            if result.success {
                mark_windows_dirty(engine);
                emit_resource_mutation(
                    engine,
                    "target-layer",
                    args.target_id,
                    "disposed",
                    Some(args.realm_id),
                    None,
                );
            }
            engine.runtime.push_response(CommandResponseEnvelope {
                id: pack.id,
                response: CommandResponse::TargetLayerDispose(result),
            });
        }
        cmd @ (EngineCmd::CmdModelList(_)
        | EngineCmd::CmdModelGet(_)
        | EngineCmd::CmdMaterialGet(_)
        | EngineCmd::CmdMaterialList(_)
        | EngineCmd::CmdTextureGet(_)
        | EngineCmd::CmdTextureList(_)
        | EngineCmd::CmdGeometryGet(_)
        | EngineCmd::CmdGeometryList(_)
        | EngineCmd::CmdLightGet(_)
        | EngineCmd::CmdLightList(_)
        | EngineCmd::CmdCameraGet(_)
        | EngineCmd::CmdCameraList(_)
        | EngineCmd::CmdEnvironmentGet(_)
        | EngineCmd::CmdEnvironmentList(_)
        | EngineCmd::CmdMaterialDefinitionGet(_)
        | EngineCmd::CmdMaterialDefinitionList(_)
        | EngineCmd::CmdMaterialInstanceGet(_)
        | EngineCmd::CmdMaterialInstanceList(_)
        | EngineCmd::CmdAudioListenerGet(_)
        | EngineCmd::CmdAudioSourceGet(_)
        | EngineCmd::CmdAudioSourceList(_)
        | EngineCmd::CmdAudioResourceGet(_)
        | EngineCmd::CmdAudioResourceList(_)
        | EngineCmd::CmdRealmGet(_)
        | EngineCmd::CmdRealmList(_)
        | EngineCmd::CmdTargetGet(_)
        | EngineCmd::CmdTargetList(_)
        | EngineCmd::CmdTargetLayerGet(_)
        | EngineCmd::CmdTargetLayerList(_)
        | EngineCmd::CmdGizmoDrawLine(_)
        | EngineCmd::CmdGizmoDrawAabb(_)
        | EngineCmd::CmdGizmoDrawPolyline(_)) => {
            super::dispatch_tail::dispatch_ui_and_misc(engine, pack.id, cmd);
        }
    }
}
