use std::collections::HashMap;

use galfus_core::core;
use galfus_core::core::GalfusResult;
use galfus_core::core::cmd::EngineEvent;
use galfus_core::core::cmd::{
    CmdCamera2dUpsertArgs, CmdMaterialUpsertArgs, CmdShape2dUpsertArgs, CmdSprite2dUpsertArgs,
    EngineCmd,
};
use galfus_core::core::realm::cmd::{CmdRealmCreateArgs, RealmKindDto};
use galfus_core::core::resources::{
    CmdCamera2dCreateArgs, CmdCamera2dUpdateArgs, CmdMaterialCreateArgs,
    CmdPrimitiveGeometryCreateArgs, CmdShape2dCreateArgs, CmdShape2dUpdateArgs,
    CmdSprite2dCreateArgs, CmdSprite2dUpdateArgs, MaterialKind, MaterialOptions, MaterialRealmKind,
    PrimitiveShape,
};
use galfus_core::core::system::LogLevel;
use galfus_core::core::target::{
    CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs, DimensionValue, TargetKind, TargetLayerLayout,
};
use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::demo::DemoContext;
use crate::demo::DemoRunOptions;
use crate::demo::io::{receive_responses, send_commands};
use crate::demo::scenarios::run_with_window_loop;

const FRAME_MS: u32 = 16;
const WINDOW_TARGET_ID: u64 = 1;

const GEOMETRY_QUAD_A_ID: u32 = 301;
const GEOMETRY_QUAD_B_ID: u32 = 302;
const MATERIAL_RED_ID: u32 = 303;
const MATERIAL_BLUE_ID: u32 = 304;
const CAMERA_2D_ID: u32 = 305;
const SPRITE_A_ID: u32 = 306;
const SPRITE_B_ID: u32 = 307;
const SHAPE_A_ID: u32 = 308;

pub fn run(ctx: DemoContext, options: DemoRunOptions) -> bool {
    let Some(realm_2d) = create_twod_realm() else {
        return false;
    };

    let mut setup = vec![EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
        target_id: WINDOW_TARGET_ID,
        kind: TargetKind::Window,
        window_id: Some(ctx.window_id),
        size: None,
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    })];
    setup.extend(build_scene(realm_2d));
    setup.push(bind_layer(
        realm_2d,
        WINDOW_TARGET_ID,
        full_layout(0),
        vec![CAMERA_2D_ID],
    ));

    let _ = send_commands(setup);

    run_with_window_loop(
        ctx.window_id,
        FRAME_MS,
        options.timeout,
        |t| {
            let updates = build_animated_updates(realm_2d, t);
            let _ = send_commands(updates);
        },
        print_runtime_logs,
    )
}

fn create_twod_realm() -> Option<u32> {
    let _ = send_commands(vec![EngineCmd::CmdRealmCreate(CmdRealmCreateArgs {
        kind: RealmKindDto::TwoD,
        importance: None,
        cache_policy: None,
        flags: None,
    })]);
    let mut total_ms = 0u64;
    for _ in 0..8 {
        assert_eq!(
            core::galfus_tick(total_ms as i64, FRAME_MS),
            GalfusResult::Success
        );
        total_ms = total_ms.saturating_add(FRAME_MS as u64);
        for response in receive_responses() {
            if let galfus_core::core::cmd::CommandResponse::RealmCreate(result) = response.response
                && result.success
            {
                return result.realm_id;
            }
        }
    }
    None
}

fn build_scene(realm_id: u32) -> Vec<EngineCmd> {
    vec![
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: GEOMETRY_QUAD_A_ID,
            label: Some("demo3-quad-a".into()),
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: GEOMETRY_QUAD_B_ID,
            label: Some("demo3-quad-b".into()),
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_RED_ID,
            label: Some("demo3-mat-red".into()),
            slug: "standard-2d".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::TwoD,
            options: Some(MaterialOptions::Schema(HashMap::from([(
                "baseColor".to_string(),
                Vec4::new(0.95, 0.25, 0.35, 1.0),
            )]))),
        })),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_BLUE_ID,
            label: Some("demo3-mat-blue".into()),
            slug: "standard-2d".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::TwoD,
            options: Some(MaterialOptions::Schema(HashMap::from([(
                "baseColor".to_string(),
                Vec4::new(0.25, 0.45, 0.95, 1.0),
            )]))),
        })),
        EngineCmd::CmdCamera2dUpsert(CmdCamera2dUpsertArgs::Create(CmdCamera2dCreateArgs {
            realm_id,
            camera_id: CAMERA_2D_ID,
            label: Some("demo3-camera-2d".into()),
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 5.0)),
            near_far: Vec2::new(0.01, 100.0),
            ortho_scale: 2.5,
            layer_mask: u32::MAX,
            order: 0,
        })),
        EngineCmd::CmdSprite2dUpsert(CmdSprite2dUpsertArgs::Create(CmdSprite2dCreateArgs {
            realm_id,
            sprite_id: SPRITE_A_ID,
            label: Some("demo3-sprite-a".into()),
            transform: Mat4::from_translation(Vec3::new(-0.8, 0.0, 0.0))
                * Mat4::from_scale(Vec3::new(0.9, 0.9, 1.0)),
            geometry_id: GEOMETRY_QUAD_A_ID,
            material_id: Some(MATERIAL_RED_ID),
            layer: 1,
            cast_shadow: true,
            receive_shadow: true,
            occluder_only: false,
            shadow_height: 1.0,
            shadow_layer_mask: u32::MAX,
        })),
        EngineCmd::CmdSprite2dUpsert(CmdSprite2dUpsertArgs::Create(CmdSprite2dCreateArgs {
            realm_id,
            sprite_id: SPRITE_B_ID,
            label: Some("demo3-sprite-b".into()),
            transform: Mat4::from_translation(Vec3::new(0.9, -0.5, 0.0))
                * Mat4::from_scale(Vec3::new(0.7, 0.7, 1.0)),
            geometry_id: GEOMETRY_QUAD_B_ID,
            material_id: Some(MATERIAL_BLUE_ID),
            layer: 2,
            cast_shadow: true,
            receive_shadow: true,
            occluder_only: false,
            shadow_height: 1.0,
            shadow_layer_mask: u32::MAX,
        })),
        EngineCmd::CmdShape2dUpsert(CmdShape2dUpsertArgs::Create(CmdShape2dCreateArgs {
            realm_id,
            shape_id: SHAPE_A_ID,
            label: Some("demo3-shape-a".into()),
            transform: Mat4::from_translation(Vec3::new(0.0, 0.35, 0.0))
                * Mat4::from_scale(Vec3::new(0.5, 1.4, 1.0)),
            geometry_id: GEOMETRY_QUAD_A_ID,
            material_id: Some(MATERIAL_RED_ID),
            layer: 0,
            cast_shadow: true,
            receive_shadow: true,
            occluder_only: false,
            shadow_height: 1.0,
            shadow_layer_mask: u32::MAX,
        })),
    ]
}

fn build_animated_updates(realm_id: u32, time_seconds: f32) -> Vec<EngineCmd> {
    let x_a = (time_seconds * 1.2).sin() * 0.55;
    let y_b = (time_seconds * 1.8).cos() * 0.35;
    let rot = time_seconds * 1.7;
    vec![
        EngineCmd::CmdSprite2dUpsert(CmdSprite2dUpsertArgs::Update(CmdSprite2dUpdateArgs {
            realm_id,
            sprite_id: SPRITE_A_ID,
            label: None,
            transform: Some(
                Mat4::from_translation(Vec3::new(x_a, 0.0, 0.0))
                    * Mat4::from_rotation_z(rot)
                    * Mat4::from_scale(Vec3::new(0.9, 0.9, 1.0)),
            ),
            geometry_id: None,
            material_id: None,
            layer: None,
            cast_shadow: None,
            receive_shadow: None,
            occluder_only: None,
            shadow_height: None,
            shadow_layer_mask: None,
        })),
        EngineCmd::CmdSprite2dUpsert(CmdSprite2dUpsertArgs::Update(CmdSprite2dUpdateArgs {
            realm_id,
            sprite_id: SPRITE_B_ID,
            label: None,
            transform: Some(
                Mat4::from_translation(Vec3::new(0.9, y_b, 0.0))
                    * Mat4::from_scale(Vec3::new(0.7, 0.7, 1.0)),
            ),
            geometry_id: None,
            material_id: None,
            layer: None,
            cast_shadow: None,
            receive_shadow: None,
            occluder_only: None,
            shadow_height: None,
            shadow_layer_mask: None,
        })),
        EngineCmd::CmdShape2dUpsert(CmdShape2dUpsertArgs::Update(CmdShape2dUpdateArgs {
            realm_id,
            shape_id: SHAPE_A_ID,
            label: None,
            transform: Some(
                Mat4::from_translation(Vec3::new(0.0, 0.35, 0.0))
                    * Mat4::from_rotation_z(-rot * 0.7)
                    * Mat4::from_scale(Vec3::new(0.5, 1.4, 1.0)),
            ),
            geometry_id: None,
            material_id: None,
            layer: None,
            cast_shadow: None,
            receive_shadow: None,
            occluder_only: None,
            shadow_height: None,
            shadow_layer_mask: None,
        })),
        EngineCmd::CmdCamera2dUpsert(CmdCamera2dUpsertArgs::Update(CmdCamera2dUpdateArgs {
            realm_id,
            camera_id: CAMERA_2D_ID,
            label: None,
            transform: Some(Mat4::from_translation(Vec3::new(
                (time_seconds * 0.4).sin() * 0.1,
                0.0,
                5.0,
            ))),
            near_far: None,
            ortho_scale: None,
            layer_mask: None,
            order: None,
        })),
    ]
}

fn bind_layer(
    realm_id: u32,
    target_id: u64,
    layout: TargetLayerLayout,
    enabled_camera_ids: Vec<u32>,
) -> EngineCmd {
    EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
        realm_id,
        target_id,
        layout,
        enabled_camera_ids,
        environment_id: None,
    })
}

fn full_layout(z_index: i32) -> TargetLayerLayout {
    TargetLayerLayout {
        left: DimensionValue::Percent(0.0),
        top: DimensionValue::Percent(0.0),
        width: DimensionValue::Percent(100.0),
        height: DimensionValue::Percent(100.0),
        enabled: true,
        opacity: 1.0,
        z_index,
        blend_mode: 0,
        clip: None,
    }
}

fn print_runtime_logs(events: Vec<EngineEvent>) {
    for event in events {
        if let EngineEvent::Log(log) = event {
            if matches!(log.level, LogLevel::Trace | LogLevel::Debug) {
                continue;
            }
            println!("[runtime/{:?}][{}] {}", log.level, log.tag, log.message);
        }
    }
}
