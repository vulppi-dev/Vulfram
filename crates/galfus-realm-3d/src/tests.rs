use crate::{
    CameraProjectionPlan, EntityRecordUpdatePlan, ForwardAtlasEntryMeta, LightRecordMeta,
    MaterialRecordMeta, MaterialRecordUpdatePlan, ModelRecordMeta, SyncPlan,
    TargetTextureBindingMeta, TextureRecordMeta, apply_sync_plan_map, graph_is_compatible,
    hash_forward_atlas_entries, hash_target_texture_binds, hash_texture_records,
    plan_camera_projection_update, plan_forward_atlas_sync, plan_geometry_registry_sync,
    plan_light_record_update, plan_material_record_update, plan_model_record_update,
    plan_target_texture_bind_sync, plan_texture_record_sync, supports_render_pass,
};
use galfus_realm_core::{
    RENDER_PASS_COMPOSE, RENDER_PASS_CUSTOM_POST_FORWARD, RENDER_PASS_CUSTOM_PRE_FORWARD,
    RENDER_PASS_FORWARD, RENDER_PASS_SHADOW_3D,
};

#[test]
fn threed_realm_accepts_full_pipeline_passes() {
    assert!(supports_render_pass(RENDER_PASS_SHADOW_3D));
    assert!(supports_render_pass(RENDER_PASS_CUSTOM_PRE_FORWARD));
    assert!(supports_render_pass(RENDER_PASS_CUSTOM_POST_FORWARD));
    assert!(!supports_render_pass("unknown"));
    assert!(graph_is_compatible([
        RENDER_PASS_SHADOW_3D,
        RENDER_PASS_CUSTOM_PRE_FORWARD,
        RENDER_PASS_FORWARD,
        RENDER_PASS_CUSTOM_POST_FORWARD,
        RENDER_PASS_COMPOSE,
    ]));
    assert!(!graph_is_compatible([RENDER_PASS_SHADOW_3D, "unknown"]));
}

#[test]
fn texture_hash_changes_with_metadata() {
    let a = vec![TextureRecordMeta {
        id: 1,
        label: Some("albedo".into()),
        width: 64,
        height: 64,
        depth_or_array_layers: 1,
        format: "rgba16float".into(),
    }];
    let b = vec![TextureRecordMeta {
        width: 128,
        ..a[0].clone()
    }];
    assert_ne!(hash_texture_records(&a), hash_texture_records(&b));
}

#[test]
fn atlas_hash_changes_with_uv_scale_bias() {
    let a = vec![ForwardAtlasEntryMeta {
        id: 1,
        label: Some("atlas".into()),
        layer: 0,
        uv_scale_bias: [1.0, 1.0, 0.0, 0.0].into(),
    }];
    let b = vec![ForwardAtlasEntryMeta {
        uv_scale_bias: [0.5, 1.0, 0.0, 0.0].into(),
        ..a[0].clone()
    }];
    assert_ne!(
        hash_forward_atlas_entries(&a),
        hash_forward_atlas_entries(&b)
    );
}

#[test]
fn target_bind_hash_changes_with_target_id() {
    let a = vec![TargetTextureBindingMeta {
        texture_id: 5,
        target_id: galfus_realm_core::TargetId(10),
        label: Some("bind".into()),
    }];
    let b = vec![TargetTextureBindingMeta {
        target_id: galfus_realm_core::TargetId(11),
        ..a[0].clone()
    }];
    assert_ne!(hash_target_texture_binds(&a), hash_target_texture_binds(&b));
}

#[test]
fn texture_sync_plan_marks_stale_and_changed_records() {
    let current = vec![
        TextureRecordMeta {
            id: 1,
            label: Some("a".into()),
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
            format: "rgba16float".into(),
        },
        TextureRecordMeta {
            id: 2,
            label: Some("b".into()),
            width: 32,
            height: 32,
            depth_or_array_layers: 1,
            format: "rgba16float".into(),
        },
    ];
    let next = vec![
        TextureRecordMeta {
            id: 1,
            label: Some("a2".into()),
            width: 64,
            height: 64,
            depth_or_array_layers: 1,
            format: "rgba16float".into(),
        },
        TextureRecordMeta {
            id: 3,
            label: Some("c".into()),
            width: 16,
            height: 16,
            depth_or_array_layers: 1,
            format: "rgba16float".into(),
        },
    ];

    assert_eq!(
        plan_texture_record_sync(&current, &next),
        SyncPlan {
            stale_ids: vec![2],
            replace_ids: vec![1, 3],
        }
    );
}

#[test]
fn atlas_sync_plan_marks_changed_entries() {
    let current = vec![ForwardAtlasEntryMeta {
        id: 1,
        label: Some("atlas".into()),
        layer: 0,
        uv_scale_bias: [1.0, 1.0, 0.0, 0.0].into(),
    }];
    let next = vec![ForwardAtlasEntryMeta {
        id: 1,
        label: Some("atlas".into()),
        layer: 1,
        uv_scale_bias: [1.0, 1.0, 0.0, 0.0].into(),
    }];

    assert_eq!(
        plan_forward_atlas_sync(&current, &next),
        SyncPlan {
            stale_ids: vec![],
            replace_ids: vec![1],
        }
    );
}

#[test]
fn target_bind_sync_plan_marks_removed_and_changed_binds() {
    let current = vec![
        TargetTextureBindingMeta {
            texture_id: 5,
            target_id: galfus_realm_core::TargetId(10),
            label: Some("bind".into()),
        },
        TargetTextureBindingMeta {
            texture_id: 7,
            target_id: galfus_realm_core::TargetId(20),
            label: Some("old".into()),
        },
    ];
    let next = vec![TargetTextureBindingMeta {
        texture_id: 5,
        target_id: galfus_realm_core::TargetId(11),
        label: Some("bind".into()),
    }];

    assert_eq!(
        plan_target_texture_bind_sync(&current, &next),
        SyncPlan {
            stale_ids: vec![7],
            replace_ids: vec![5],
        }
    );
}

#[test]
fn geometry_registry_sync_plan_marks_missing_and_stale_ids() {
    assert_eq!(
        plan_geometry_registry_sync(&[1, 2, 3], &[2, 3, 4]),
        SyncPlan {
            stale_ids: vec![1],
            replace_ids: vec![4],
        }
    );
}

#[test]
fn apply_sync_plan_map_removes_stale_and_replaces_changed_entries() {
    let mut current = std::collections::HashMap::from([(1_u32, "old-a"), (2_u32, "old-b")]);
    let next = std::collections::HashMap::from([(1_u32, "new-a"), (3_u32, "new-c")]);
    let plan = SyncPlan {
        stale_ids: vec![2],
        replace_ids: vec![1, 3],
    };

    apply_sync_plan_map(&mut current, &next, &plan);

    assert_eq!(current.len(), 2);
    assert_eq!(current.get(&1), Some(&"new-a"));
    assert_eq!(current.get(&3), Some(&"new-c"));
    assert!(!current.contains_key(&2));
}

#[test]
fn camera_projection_plan_detects_preserve_vs_reset() {
    assert_eq!(
        plan_camera_projection_update(
            [1, 0],
            [0.1, 100.0].into(),
            1.0,
            [1, 0],
            [0.1, 100.0].into(),
            1.0,
            [1280, 720],
        ),
        CameraProjectionPlan {
            preserve_runtime_projection: true,
            reset_projection_size: false,
        }
    );
    assert_eq!(
        plan_camera_projection_update(
            [1, 0],
            [0.1, 100.0].into(),
            1.0,
            [2, 0],
            [0.1, 100.0].into(),
            1.0,
            [1280, 720],
        ),
        CameraProjectionPlan {
            preserve_runtime_projection: false,
            reset_projection_size: true,
        }
    );
}

#[test]
fn entity_and_material_change_detectors_compare_semantic_fields() {
    let model = ModelRecordMeta {
        transform: [0.0; 16],
        translation: [0.0; 4].into(),
        rotation: [0.0; 4].into(),
        scale: [1.0, 1.0, 1.0, 0.0].into(),
        flags: [0; 4],
        outline_color: [1.0, 0.0, 0.0, 1.0].into(),
        geometry_id: 1,
        material_id: Some(2),
        active: true,
        layer_mask: 3,
        cast_shadow: true,
        receive_shadow: true,
        cast_outline: false,
    };
    let changed_model = ModelRecordMeta {
        geometry_id: 9,
        ..model.clone()
    };
    assert_eq!(
        plan_model_record_update(&model, &model),
        EntityRecordUpdatePlan { mark_dirty: false }
    );
    assert_eq!(
        plan_model_record_update(&model, &changed_model),
        EntityRecordUpdatePlan { mark_dirty: true }
    );

    let light = LightRecordMeta {
        position: [0.0; 4].into(),
        direction: [0.0; 4].into(),
        color: [1.0; 4].into(),
        ground_color: [0.0; 4].into(),
        view: [0.0; 16],
        projection: [0.0; 16],
        view_projection: [0.0; 16],
        intensity_range: [1.0, 10.0].into(),
        spot_inner_outer: [0.0, 0.0].into(),
        kind_flags: [0, 0],
        active: true,
        layer_mask: 1,
        shadow_layer_mask: u32::MAX,
        cast_shadow: false,
    };
    let changed_light = LightRecordMeta {
        cast_shadow: true,
        ..light.clone()
    };
    assert_eq!(
        plan_light_record_update(&light, &light),
        EntityRecordUpdatePlan { mark_dirty: false }
    );
    assert_eq!(
        plan_light_record_update(&light, &changed_light),
        EntityRecordUpdatePlan { mark_dirty: true }
    );

    let material = MaterialRecordMeta {
        label: Some("mat".into()),
        data_bytes: vec![1, 2, 3],
        inputs_bytes: vec![4, 5, 6],
        texture_ids: vec![1, 2],
        surface_type: 0,
        topology: 2,
        polygon_mode: 0,
    };
    let changed_material = MaterialRecordMeta {
        texture_ids: vec![1, 3],
        ..material.clone()
    };
    assert_eq!(
        plan_material_record_update(&material, &material),
        MaterialRecordUpdatePlan {
            mark_dirty: false,
            reset_bind_group: false,
        }
    );
    assert_eq!(
        plan_material_record_update(&material, &changed_material),
        MaterialRecordUpdatePlan {
            mark_dirty: true,
            reset_bind_group: true,
        }
    );
}
