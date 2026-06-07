use super::*;
use crate::core::cmd::EngineEvent;
use crate::core::realm::RealmId;
use crate::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdEnvironmentCreateArgs, CmdMaterialCreateArgs,
    CmdMaterialDefinitionCreateArgs, CmdMaterialDefinitionDisposeArgs,
    CmdMaterialDefinitionUpdateArgs, CmdMaterialInstanceCreateArgs, CmdMaterialInstanceDisposeArgs,
    CmdMaterialInstanceUpdateArgs, EnvironmentConfig, MATERIAL_DEFINITION_STANDARD_2D_ID,
    MATERIAL_DEFINITION_STANDARD_ID, MaterialKind, MaterialOptions, MaterialRealmKind,
    ShaderMaterialPreset, engine_cmd_camera_create, engine_cmd_environment_create,
    engine_cmd_material_create, engine_cmd_material_definition_create,
    engine_cmd_material_definition_dispose, engine_cmd_material_definition_update,
    engine_cmd_material_instance_create, engine_cmd_material_instance_dispose,
    engine_cmd_material_instance_update,
};
use crate::core::system::SystemEvent;
use crate::core::test_support::test_engine;
use glam::{Mat4, Vec2};

#[test]
fn camera_get_returns_not_found_when_scope_cannot_be_resolved() {
    let mut engine = test_engine();
    let result = engine_cmd_camera_get(
        &mut engine,
        &CmdResourceGetArgs {
            id: 1,
            scope: QueryScopeArgs::default(),
        },
    );

    assert!(!result.success);
    assert_eq!(result.kind, "camera");
    assert_eq!(result.message, "Realm scope not resolved");
}

#[test]
fn camera_get_returns_entity_when_realm_scope_exists() {
    let mut engine = test_engine();
    let realm_id = RealmId(7);
    engine
        .universal_state
        .targets
        .host_realm_index
        .insert(99, realm_id);
    let created = engine_cmd_camera_create(
        &mut engine,
        &CmdCameraCreateArgs {
            realm_id: 7,
            camera_id: 10,
            label: Some("cam-10".into()),
            transform: Mat4::IDENTITY,
            kind: CameraKind::Perspective,
            flags: 0,
            near_far: Vec2::new(0.1, 1000.0),
            layer_mask: u32::MAX,
            order: 0,
            view_position: None,
            ortho_scale: 10.0,
        },
    );
    assert!(created.success, "{}", created.message);

    let result = engine_cmd_camera_get(
        &mut engine,
        &CmdResourceGetArgs {
            id: 10,
            scope: QueryScopeArgs {
                window_id: Some(99),
                ..Default::default()
            },
        },
    );

    assert!(result.success);
    assert_eq!(result.kind, "camera");
    assert_eq!(result.id, Some(10));
    assert_eq!(result.realm_id, Some(7));
}

#[test]
fn environment_get_and_list_support_ids_filter() {
    let mut engine = test_engine();

    assert!(
        engine_cmd_environment_create(
            &mut engine,
            &CmdEnvironmentCreateArgs {
                environment_id: 11,
                config: EnvironmentConfig::default(),
            },
        )
        .success
    );
    assert!(
        engine_cmd_environment_create(
            &mut engine,
            &CmdEnvironmentCreateArgs {
                environment_id: 12,
                config: EnvironmentConfig::default(),
            },
        )
        .success
    );

    let get = engine_cmd_environment_get(
        &mut engine,
        &CmdResourceGetArgs {
            id: 11,
            scope: QueryScopeArgs::default(),
        },
    );
    assert!(get.success);
    assert_eq!(get.kind, "environment");
    assert_eq!(get.id, Some(11));

    let listed = engine_cmd_environment_list(
        &mut engine,
        &CmdResourceListArgs {
            scope: QueryScopeArgs {
                ids: Some(vec![12]),
                ..Default::default()
            },
        },
    );
    assert!(listed.success);
    assert_eq!(listed.items.len(), 1);
    assert_eq!(listed.items[0].id, 12);
}

#[test]
fn material_definition_and_instance_get_and_list_work() {
    let mut engine = test_engine();

    let definition_result = engine_cmd_material_definition_create(
        &mut engine,
        &CmdMaterialDefinitionCreateArgs {
            definition_id: 901,
            slug: "test-def-901".into(),
            label: Some("Test Definition".into()),
            realm_kind: MaterialRealmKind::ThreeD,
            preset: Some(ShaderMaterialPreset::Standard),
            shader_type: None,
            shader_source: None,
            shader_params_schema: None,
            capabilities: None,
        },
    );
    assert!(definition_result.success, "{}", definition_result.message);

    let instance_result = engine_cmd_material_instance_create(
        &mut engine,
        &CmdMaterialInstanceCreateArgs {
            material_id: 902,
            slug: "test-def-901".into(),
            label: Some("Test Instance".into()),
            options: None,
        },
    );
    assert!(instance_result.success, "{}", instance_result.message);

    let definition_get = engine_cmd_material_definition_get(
        &mut engine,
        &CmdResourceGetArgs {
            id: 901,
            scope: QueryScopeArgs::default(),
        },
    );
    assert!(definition_get.success);
    assert_eq!(definition_get.label.as_deref(), Some("Test Definition"));

    let instance_get = engine_cmd_material_instance_get(
        &mut engine,
        &CmdMaterialInstanceGetArgs {
            id: 902,
            scope: QueryScopeArgs::default(),
            realm_kind: None,
        },
    );
    assert!(instance_get.success);
    assert_eq!(instance_get.label.as_deref(), Some("Test Instance"));

    let definition_list = engine_cmd_material_definition_list(
        &mut engine,
        &CmdResourceListArgs {
            scope: QueryScopeArgs {
                ids: Some(vec![901]),
                ..Default::default()
            },
        },
    );
    assert!(definition_list.success);
    assert_eq!(definition_list.items.len(), 1);
    assert_eq!(definition_list.items[0].id, 901);

    let instance_list = engine_cmd_material_instance_list(
        &mut engine,
        &CmdMaterialInstanceListArgs {
            scope: QueryScopeArgs {
                ids: Some(vec![902]),
                ..Default::default()
            },
            realm_kind: None,
        },
    );
    assert!(instance_list.success);
    assert_eq!(instance_list.items.len(), 1);
    assert_eq!(instance_list.items[0].id, 902);
}

#[test]
fn material_get_filters_by_realm_kind() {
    let mut engine = test_engine();

    let create_result = engine_cmd_material_create(
        &mut engine,
        &CmdMaterialCreateArgs {
            material_id: 1001,
            label: Some("mat-3d-only".into()),
            slug: "standard".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: None,
        },
    );
    assert!(create_result.success, "{}", create_result.message);

    let get_ok = engine_cmd_material_get(
        &mut engine,
        &CmdMaterialGetArgs {
            id: 1001,
            scope: QueryScopeArgs::default(),
            realm_kind: Some(MaterialRealmKind::ThreeD),
        },
    );
    assert!(get_ok.success, "{}", get_ok.message);

    let get_mismatch = engine_cmd_material_get(
        &mut engine,
        &CmdMaterialGetArgs {
            id: 1001,
            scope: QueryScopeArgs::default(),
            realm_kind: Some(MaterialRealmKind::TwoD),
        },
    );
    assert!(!get_mismatch.success);
    assert_eq!(get_mismatch.message, "Material realm kind mismatch");
}

#[test]
fn material_instance_get_and_list_filter_by_realm_kind() {
    let mut engine = test_engine();

    let definition_result = engine_cmd_material_definition_create(
        &mut engine,
        &CmdMaterialDefinitionCreateArgs {
            definition_id: 2001,
            slug: "test-def-2001".into(),
            label: Some("Test Definition 2001".into()),
            realm_kind: MaterialRealmKind::ThreeD,
            preset: Some(ShaderMaterialPreset::Standard),
            shader_type: None,
            shader_source: None,
            shader_params_schema: None,
            capabilities: None,
        },
    );
    assert!(definition_result.success, "{}", definition_result.message);

    let instance_result = engine_cmd_material_instance_create(
        &mut engine,
        &CmdMaterialInstanceCreateArgs {
            material_id: 2002,
            slug: "test-def-2001".into(),
            label: Some("Instance 2002".into()),
            options: None,
        },
    );
    assert!(instance_result.success, "{}", instance_result.message);

    {
        let material = engine
            .universal_state
            .scene
            .realm3d
            .materials
            .get_mut(&2002)
            .expect("material instance should create backing material");
        material.realm_kind = MaterialRealmKind::TwoD;
    }

    let get_mismatch = engine_cmd_material_instance_get(
        &mut engine,
        &CmdMaterialInstanceGetArgs {
            id: 2002,
            scope: QueryScopeArgs::default(),
            realm_kind: Some(MaterialRealmKind::ThreeD),
        },
    );
    assert!(!get_mismatch.success);
    assert_eq!(
        get_mismatch.message,
        "Material instance realm kind mismatch"
    );

    let get_ok = engine_cmd_material_instance_get(
        &mut engine,
        &CmdMaterialInstanceGetArgs {
            id: 2002,
            scope: QueryScopeArgs::default(),
            realm_kind: Some(MaterialRealmKind::TwoD),
        },
    );
    assert!(get_ok.success, "{}", get_ok.message);

    let list_filtered = engine_cmd_material_instance_list(
        &mut engine,
        &CmdMaterialInstanceListArgs {
            scope: QueryScopeArgs::default(),
            realm_kind: Some(MaterialRealmKind::ThreeD),
        },
    );
    assert!(list_filtered.success);
    assert!(list_filtered.items.iter().all(|entry| entry.id != 2002));

    let list_ok = engine_cmd_material_instance_list(
        &mut engine,
        &CmdMaterialInstanceListArgs {
            scope: QueryScopeArgs::default(),
            realm_kind: Some(MaterialRealmKind::TwoD),
        },
    );
    assert!(list_ok.success);
    assert!(list_ok.items.iter().any(|entry| entry.id == 2002));
}

#[test]
fn builtin_material_definitions_are_immutable() {
    let mut engine = test_engine();

    let result = engine_cmd_material_definition_update(
        &mut engine,
        &CmdMaterialDefinitionUpdateArgs {
            definition_id: MATERIAL_DEFINITION_STANDARD_ID,
            slug: None,
            label: Some("mutated".into()),
            realm_kind: None,
            preset: None,
            shader_type: None,
            shader_source: None,
            shader_params_schema: None,
            capabilities: None,
        },
    );

    assert!(!result.success);
    assert_eq!(result.message, "Builtin material definitions are immutable");
}

#[test]
fn material_create_rejects_realm_kind_mismatch_with_definition() {
    let mut engine = test_engine();

    let create = engine_cmd_material_create(
        &mut engine,
        &CmdMaterialCreateArgs {
            material_id: 32001,
            label: Some("bad-realm".into()),
            slug: "standard".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::TwoD,
            options: None,
        },
    );

    assert!(!create.success);
    assert!(create.message.contains("Material realm kind mismatch"));
}

#[test]
fn disposing_definition_rebinds_instances_to_fallback_and_emits_event() {
    let mut engine = test_engine();

    let create_definition = engine_cmd_material_definition_create(
        &mut engine,
        &CmdMaterialDefinitionCreateArgs {
            definition_id: 33001,
            slug: "custom-33001".into(),
            label: Some("Custom 33001".into()),
            realm_kind: MaterialRealmKind::ThreeD,
            preset: Some(ShaderMaterialPreset::Standard),
            shader_type: None,
            shader_source: None,
            shader_params_schema: None,
            capabilities: None,
        },
    );
    assert!(create_definition.success, "{}", create_definition.message);

    let create_instance = engine_cmd_material_instance_create(
        &mut engine,
        &CmdMaterialInstanceCreateArgs {
            material_id: 33002,
            slug: "custom-33001".into(),
            label: Some("Instance 33002".into()),
            options: None,
        },
    );
    assert!(create_instance.success, "{}", create_instance.message);

    let dispose = engine_cmd_material_definition_dispose(
        &mut engine,
        &CmdMaterialDefinitionDisposeArgs {
            definition_id: 33001,
        },
    );
    assert!(dispose.success, "{}", dispose.message);

    let instance = engine
        .universal_state
        .scene
        .material_instances
        .get(&33002)
        .expect("instance must remain after fallback");
    assert_eq!(instance.definition_id, MATERIAL_DEFINITION_STANDARD_ID);

    let material = engine
        .universal_state
        .scene
        .realm3d
        .materials
        .get(&33002)
        .expect("material must remain after fallback");
    assert_eq!(material.realm_kind, MaterialRealmKind::ThreeD);

    let had_event = engine.runtime.events().iter().any(|event| {
        matches!(
            event,
            EngineEvent::System(SystemEvent::MaterialInstanceFallbackApplied {
                material_id,
                previous_definition_id,
                fallback_definition_id,
                ..
            }) if *material_id == 33002
                && *previous_definition_id == 33001
                && *fallback_definition_id == MATERIAL_DEFINITION_STANDARD_ID
        )
    });
    assert!(had_event);

    let dispose_builtin_2d = engine_cmd_material_definition_dispose(
        &mut engine,
        &CmdMaterialDefinitionDisposeArgs {
            definition_id: MATERIAL_DEFINITION_STANDARD_2D_ID,
        },
    );
    assert!(!dispose_builtin_2d.success);
}

#[test]
fn material_create_accepts_schema_options_path() {
    let mut engine = test_engine();

    let create = engine_cmd_material_create(
        &mut engine,
        &CmdMaterialCreateArgs {
            material_id: 34001,
            label: Some("schema-mat".into()),
            slug: "standard".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: Some(MaterialOptions::Schema(std::collections::HashMap::from([
                ("baseColor".to_string(), glam::Vec4::new(0.2, 0.3, 0.4, 1.0)),
                ("input6".to_string(), glam::Vec4::new(6.0, 0.0, 0.0, 0.0)),
            ]))),
        },
    );
    assert!(create.success, "{}", create.message);

    let material = engine
        .universal_state
        .scene
        .realm3d
        .materials
        .get(&34001)
        .expect("material should exist");
    assert_eq!(material.inputs[0], glam::Vec4::new(0.2, 0.3, 0.4, 1.0));
    assert_eq!(material.inputs[6], glam::Vec4::new(6.0, 0.0, 0.0, 0.0));
}

#[test]
fn material_program_cache_reports_evictions() {
    let mut engine = test_engine();

    for i in 0..1100_u32 {
        engine.runtime.begin_tick(i as u64, 16);
        engine.runtime.advance_frame();

        let result = engine_cmd_material_definition_create(
            &mut engine,
            &CmdMaterialDefinitionCreateArgs {
                definition_id: 40000 + i,
                slug: format!("cache-evict-{i}"),
                label: None,
                realm_kind: MaterialRealmKind::ThreeD,
                preset: Some(ShaderMaterialPreset::Standard),
                shader_type: None,
                shader_source: None,
                shader_params_schema: Some(std::collections::HashMap::from([(
                    format!("param_{i}"),
                    "vec4<f32>".to_string(),
                )])),
                capabilities: None,
            },
        );
        assert!(result.success, "{}", result.message);
    }

    assert!(engine.profiling.render.material_program_cache_evictions > 0);
}

#[test]
fn twod_material_definition_and_instance_full_lifecycle_works() {
    let mut engine = test_engine();

    let create_definition = engine_cmd_material_definition_create(
        &mut engine,
        &CmdMaterialDefinitionCreateArgs {
            definition_id: 51001,
            slug: "custom-2d-51001".into(),
            label: Some("Custom 2D Definition".into()),
            realm_kind: MaterialRealmKind::TwoD,
            preset: Some(ShaderMaterialPreset::Standard),
            shader_type: None,
            shader_source: None,
            shader_params_schema: None,
            capabilities: None,
        },
    );
    assert!(create_definition.success, "{}", create_definition.message);

    let update_definition = engine_cmd_material_definition_update(
        &mut engine,
        &CmdMaterialDefinitionUpdateArgs {
            definition_id: 51001,
            slug: Some("custom-2d-51001-v2".into()),
            label: Some("Custom 2D Definition V2".into()),
            realm_kind: Some(MaterialRealmKind::TwoD),
            preset: Some(ShaderMaterialPreset::Standard),
            shader_type: None,
            shader_source: None,
            shader_params_schema: None,
            capabilities: None,
        },
    );
    assert!(update_definition.success, "{}", update_definition.message);

    let create_instance = engine_cmd_material_instance_create(
        &mut engine,
        &CmdMaterialInstanceCreateArgs {
            material_id: 51002,
            slug: "custom-2d-51001-v2".into(),
            label: Some("Custom 2D Instance".into()),
            options: None,
        },
    );
    assert!(create_instance.success, "{}", create_instance.message);

    let update_instance = engine_cmd_material_instance_update(
        &mut engine,
        &CmdMaterialInstanceUpdateArgs {
            material_id: 51002,
            slug: Some("standard-2d".into()),
            label: Some("Instance moved to builtin 2D".into()),
            options: None,
        },
    );
    assert!(update_instance.success, "{}", update_instance.message);

    let dispose_instance = engine_cmd_material_instance_dispose(
        &mut engine,
        &CmdMaterialInstanceDisposeArgs { material_id: 51002 },
    );
    assert!(dispose_instance.success, "{}", dispose_instance.message);
    assert!(
        !engine
            .universal_state
            .scene
            .material_instances
            .contains_key(&51002)
    );

    let dispose_definition = engine_cmd_material_definition_dispose(
        &mut engine,
        &CmdMaterialDefinitionDisposeArgs {
            definition_id: 51001,
        },
    );
    assert!(dispose_definition.success, "{}", dispose_definition.message);
}

#[test]
fn twod_definition_dispose_rebinds_instance_to_twod_fallback() {
    let mut engine = test_engine();

    let create_definition = engine_cmd_material_definition_create(
        &mut engine,
        &CmdMaterialDefinitionCreateArgs {
            definition_id: 52001,
            slug: "custom-2d-52001".into(),
            label: Some("Custom 2D Definition".into()),
            realm_kind: MaterialRealmKind::TwoD,
            preset: Some(ShaderMaterialPreset::Standard),
            shader_type: None,
            shader_source: None,
            shader_params_schema: None,
            capabilities: None,
        },
    );
    assert!(create_definition.success, "{}", create_definition.message);

    let create_instance = engine_cmd_material_instance_create(
        &mut engine,
        &CmdMaterialInstanceCreateArgs {
            material_id: 52002,
            slug: "custom-2d-52001".into(),
            label: Some("Custom 2D Instance".into()),
            options: None,
        },
    );
    assert!(create_instance.success, "{}", create_instance.message);

    let dispose_definition = engine_cmd_material_definition_dispose(
        &mut engine,
        &CmdMaterialDefinitionDisposeArgs {
            definition_id: 52001,
        },
    );
    assert!(dispose_definition.success, "{}", dispose_definition.message);

    let instance = engine
        .universal_state
        .scene
        .material_instances
        .get(&52002)
        .expect("2D instance must remain after fallback");
    assert_eq!(instance.definition_id, MATERIAL_DEFINITION_STANDARD_2D_ID);

    let material = engine
        .universal_state
        .scene
        .realm3d
        .materials
        .get(&52002)
        .expect("2D material must remain after fallback");
    assert_eq!(material.realm_kind, MaterialRealmKind::TwoD);

    let had_event = engine.runtime.events().iter().any(|event| {
        matches!(
            event,
            EngineEvent::System(SystemEvent::MaterialInstanceFallbackApplied {
                material_id,
                previous_definition_id,
                fallback_definition_id,
                ..
            }) if *material_id == 52002
                && *previous_definition_id == 52001
                && *fallback_definition_id == MATERIAL_DEFINITION_STANDARD_2D_ID
        )
    });
    assert!(had_event);
}

#[test]
fn twod_realm_exclusive_contracts_are_enforced_for_materials() {
    let mut engine = test_engine();

    let create_pbr_2d = engine_cmd_material_definition_create(
        &mut engine,
        &CmdMaterialDefinitionCreateArgs {
            definition_id: 53001,
            slug: "bad-pbr-2d".into(),
            label: Some("Bad PBR 2D".into()),
            realm_kind: MaterialRealmKind::TwoD,
            preset: Some(ShaderMaterialPreset::Pbr),
            shader_type: None,
            shader_source: None,
            shader_params_schema: None,
            capabilities: None,
        },
    );
    assert!(!create_pbr_2d.success);
    assert!(
        create_pbr_2d
            .message
            .contains("PBR preset is only supported for ThreeD realm")
    );

    let create_2d_with_3d_realm = engine_cmd_material_create(
        &mut engine,
        &CmdMaterialCreateArgs {
            material_id: 53002,
            label: Some("bad-realm-mismatch".into()),
            slug: "standard-2d".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: None,
        },
    );
    assert!(!create_2d_with_3d_realm.success);
    assert!(
        create_2d_with_3d_realm
            .message
            .contains("Material realm kind mismatch")
    );
}
