use std::collections::HashMap;

use galfus_core::core;
use galfus_core::core::GalfusResult;
use galfus_core::core::cmd::EngineEvent;
use galfus_core::core::cmd::{
    CmdCamera2dUpsertArgs, CmdLight3dUpsertArgs, CmdMaterialUpsertArgs, CmdShape2dUpsertArgs,
    CmdSprite2dUpsertArgs, EngineCmd,
};
use galfus_core::core::realm::cmd::{CmdRealmCreateArgs, CmdRealmDisposeArgs, RealmKindDto};
use galfus_core::core::resources::{
    CmdCamera2dCreateArgs, CmdCamera2dUpdateArgs, CmdLightCreateArgs, CmdLightUpdateArgs,
    CmdMaterialCreateArgs, CmdPrimitiveGeometryCreateArgs, CmdRealm2dShadowConfigUpdateArgs,
    CmdShape2dCreateArgs, CmdShape2dUpdateArgs, CmdSprite2dCreateArgs, CmdSprite2dUpdateArgs,
    LightKind, MaterialKind, MaterialOptions, MaterialRealmKind, PrimitiveShape,
};
use galfus_core::core::system::CmdSystemLogLevelSetArgs;
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

const GEOMETRY_QUAD_ID: u32 = 401;
const MATERIAL_RECEIVER_ID: u32 = 402;
const MATERIAL_CASTER_ID: u32 = 403;
const MATERIAL_FLOOR_ID: u32 = 404;
const CAMERA_2D_ID: u32 = 405;
const RECEIVER_A_ID: u32 = 406;
const RECEIVER_B_ID: u32 = 407;
const CASTER_A_ID: u32 = 408;
const CASTER_B_ID: u32 = 409;
const FLOOR_ID: u32 = 410;
const LIGHT_A_ID: u32 = 411;
const LIGHT_B_ID: u32 = 412;
const BACKDROP_ID: u32 = 413;
pub fn run(ctx: DemoContext, options: DemoRunOptions) -> bool {
    let _ = send_commands(vec![EngineCmd::CmdSystemLogLevelSet(
        CmdSystemLogLevelSetArgs {
            level: LogLevel::Debug,
        },
    )]);
    let _ = send_commands(vec![EngineCmd::CmdRealmDispose(CmdRealmDisposeArgs {
        realm_id: ctx.realm_id,
    })]);
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
    let _ = send_commands(build_shadow_runtime_updates(realm_2d));

    run_with_window_loop(
        ctx.window_id,
        FRAME_MS,
        options.timeout,
        |t| {
            let updates = build_animated_updates(realm_2d, t);
            let _ = send_commands(updates);
        },
        move |events| print_runtime_logs(events),
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
            geometry_id: GEOMETRY_QUAD_ID,
            label: Some("demo4-quad".into()),
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_RECEIVER_ID,
            label: Some("demo4-mat-receiver".into()),
            slug: "standard-2d".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::TwoD,
            options: Some(MaterialOptions::Schema(HashMap::from([(
                "baseColor".to_string(),
                Vec4::new(0.22, 0.45, 0.96, 1.0),
            )]))),
        })),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_CASTER_ID,
            label: Some("demo4-mat-caster".into()),
            slug: "standard-2d".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::TwoD,
            options: Some(MaterialOptions::Schema(HashMap::from([(
                "baseColor".to_string(),
                Vec4::new(0.02, 0.02, 0.02, 1.0),
            )]))),
        })),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_FLOOR_ID,
            label: Some("demo4-mat-floor".into()),
            slug: "standard-2d".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::TwoD,
            options: Some(MaterialOptions::Schema(HashMap::from([(
                "baseColor".to_string(),
                Vec4::new(0.16, 0.16, 0.2, 1.0),
            )]))),
        })),
        EngineCmd::CmdCamera2dUpsert(CmdCamera2dUpsertArgs::Create(CmdCamera2dCreateArgs {
            realm_id,
            camera_id: CAMERA_2D_ID,
            label: Some("demo4-camera-2d".into()),
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 5.0)),
            near_far: Vec2::new(0.01, 100.0),
            ortho_scale: 2.8,
            layer_mask: u32::MAX,
            order: 0,
        })),
        EngineCmd::CmdRealm2dShadowConfigUpdate(CmdRealm2dShadowConfigUpdateArgs {
            realm_id,
            shadow_contact_offset: Some(0.0),
            shadow_debug_light_index: Some(-1),
            shadow_debug_mode: Some(0),
            ambient: Some(0.0),
            light_radius: Some(0.7),
            max_shadow_updates_per_frame: Some(64),
            angular_resolution: Some(1024),
            map_resolution: Some(512),
        }),
        EngineCmd::CmdShape2dUpsert(CmdShape2dUpsertArgs::Create(CmdShape2dCreateArgs {
            realm_id,
            shape_id: BACKDROP_ID,
            label: Some("demo4-backdrop".into()),
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, -0.2))
                * Mat4::from_scale(Vec3::new(7.0, 4.5, 1.0)),
            geometry_id: GEOMETRY_QUAD_ID,
            material_id: Some(MATERIAL_FLOOR_ID),
            layer: 0,
            cast_shadow: false,
            receive_shadow: true,
            occluder_only: false,
            shadow_height: 0.0,
            shadow_layer_mask: u32::MAX,
        })),
        EngineCmd::CmdShape2dUpsert(CmdShape2dUpsertArgs::Create(CmdShape2dCreateArgs {
            realm_id,
            shape_id: FLOOR_ID,
            label: Some("demo4-floor".into()),
            transform: Mat4::from_translation(Vec3::new(0.0, -1.45, 0.0))
                * Mat4::from_scale(Vec3::new(4.5, 0.5, 1.0)),
            geometry_id: GEOMETRY_QUAD_ID,
            material_id: Some(MATERIAL_FLOOR_ID),
            layer: 0,
            cast_shadow: false,
            receive_shadow: true,
            occluder_only: false,
            shadow_height: 0.0,
            shadow_layer_mask: u32::MAX,
        })),
        EngineCmd::CmdSprite2dUpsert(CmdSprite2dUpsertArgs::Create(CmdSprite2dCreateArgs {
            realm_id,
            sprite_id: RECEIVER_A_ID,
            label: Some("demo4-receiver-a".into()),
            transform: Mat4::from_translation(Vec3::new(-1.0, -0.1, 0.0))
                * Mat4::from_scale(Vec3::new(0.9, 0.9, 1.0)),
            geometry_id: GEOMETRY_QUAD_ID,
            material_id: Some(MATERIAL_RECEIVER_ID),
            layer: 1,
            cast_shadow: false,
            receive_shadow: true,
            occluder_only: false,
            shadow_height: 0.0,
            shadow_layer_mask: u32::MAX,
        })),
        EngineCmd::CmdSprite2dUpsert(CmdSprite2dUpsertArgs::Create(CmdSprite2dCreateArgs {
            realm_id,
            sprite_id: RECEIVER_B_ID,
            label: Some("demo4-receiver-b".into()),
            transform: Mat4::from_translation(Vec3::new(1.1, 0.25, 0.0))
                * Mat4::from_scale(Vec3::new(0.8, 0.8, 1.0)),
            geometry_id: GEOMETRY_QUAD_ID,
            material_id: Some(MATERIAL_RECEIVER_ID),
            layer: 1,
            cast_shadow: false,
            receive_shadow: true,
            occluder_only: false,
            shadow_height: 0.0,
            shadow_layer_mask: u32::MAX,
        })),
        EngineCmd::CmdShape2dUpsert(CmdShape2dUpsertArgs::Create(CmdShape2dCreateArgs {
            realm_id,
            shape_id: CASTER_A_ID,
            label: Some("demo4-caster-a".into()),
            transform: Mat4::from_translation(Vec3::new(-0.1, 0.0, 0.0))
                * Mat4::from_scale(Vec3::new(0.45, 1.6, 1.0)),
            geometry_id: GEOMETRY_QUAD_ID,
            material_id: Some(MATERIAL_CASTER_ID),
            layer: 1,
            cast_shadow: true,
            receive_shadow: false,
            occluder_only: false,
            shadow_height: 8.0,
            shadow_layer_mask: u32::MAX,
        })),
        EngineCmd::CmdShape2dUpsert(CmdShape2dUpsertArgs::Create(CmdShape2dCreateArgs {
            realm_id,
            shape_id: CASTER_B_ID,
            label: Some("demo4-caster-b".into()),
            transform: Mat4::from_translation(Vec3::new(0.55, -0.45, 0.0))
                * Mat4::from_scale(Vec3::new(0.5, 0.5, 1.0)),
            geometry_id: GEOMETRY_QUAD_ID,
            material_id: Some(MATERIAL_CASTER_ID),
            layer: 1,
            cast_shadow: true,
            receive_shadow: false,
            occluder_only: false,
            shadow_height: 6.0,
            shadow_layer_mask: u32::MAX,
        })),
        EngineCmd::CmdLight3dUpsert(CmdLight3dUpsertArgs::Create(CmdLightCreateArgs {
            realm_id,
            light_id: LIGHT_A_ID,
            label: Some("demo4-light-a".into()),
            kind: Some(LightKind::Point),
            position: Some(Vec4::new(-1.8, 1.3, 2.4, 1.0)),
            direction: None,
            color: Some(Vec4::new(0.25, 0.95, 1.0, 1.0)),
            ground_color: None,
            intensity: Some(4.2),
            range: Some(7.0),
            spot_inner_outer: None,
            layer_mask: u32::MAX,
            shadow_layer_mask: None,
            active: true,
            cast_shadow: true,
        })),
        EngineCmd::CmdLight3dUpsert(CmdLight3dUpsertArgs::Create(CmdLightCreateArgs {
            realm_id,
            light_id: LIGHT_B_ID,
            label: Some("demo4-light-b".into()),
            kind: Some(LightKind::Point),
            position: Some(Vec4::new(1.8, 1.0, 2.2, 1.0)),
            direction: None,
            color: Some(Vec4::new(1.0, 0.45, 0.25, 1.0)),
            ground_color: None,
            intensity: Some(2.6),
            range: Some(6.8),
            spot_inner_outer: None,
            layer_mask: u32::MAX,
            shadow_layer_mask: None,
            active: true,
            cast_shadow: true,
        })),
    ]
}

fn build_animated_updates(realm_id: u32, t: f32) -> Vec<EngineCmd> {
    let rot_a = t * 1.22;
    let rot_b = -t * 1.66;
    vec![
        EngineCmd::CmdShape2dUpsert(CmdShape2dUpsertArgs::Update(CmdShape2dUpdateArgs {
            realm_id,
            shape_id: CASTER_A_ID,
            label: None,
            transform: Some(
                Mat4::from_translation(Vec3::new(
                    (t * 0.8).cos() * 0.35,
                    (t * 1.2).sin() * 0.25,
                    0.0,
                )) * Mat4::from_rotation_z(rot_a)
                    * Mat4::from_scale(Vec3::new(0.45, 1.6, 1.0)),
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
            shape_id: CASTER_B_ID,
            label: None,
            transform: Some(
                Mat4::from_translation(Vec3::new(
                    0.7 + (t * 1.6).cos() * 0.25,
                    -0.45 + (t * 1.9).sin() * 0.2,
                    0.0,
                )) * Mat4::from_rotation_z(rot_b)
                    * Mat4::from_scale(Vec3::new(0.5, 0.5, 1.0)),
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
            sprite_id: RECEIVER_A_ID,
            label: None,
            transform: Some(
                Mat4::from_translation(Vec3::new(-1.0 + (t * 0.9).sin() * 0.2, -0.1, 0.0))
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
            sprite_id: RECEIVER_B_ID,
            label: None,
            transform: Some(
                Mat4::from_translation(Vec3::new(1.1 + (t * 1.0).cos() * 0.2, 0.25, 0.0))
                    * Mat4::from_scale(Vec3::new(0.8, 0.8, 1.0)),
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
                (t * 0.2).sin() * 0.08,
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

fn build_shadow_runtime_updates(realm_id: u32) -> Vec<EngineCmd> {
    vec![
        EngineCmd::CmdRealm2dShadowConfigUpdate(CmdRealm2dShadowConfigUpdateArgs {
            realm_id,
            shadow_contact_offset: Some(0.0),
            shadow_debug_light_index: Some(-1),
            shadow_debug_mode: Some(0),
            ambient: Some(0.0),
            light_radius: Some(0.7),
            max_shadow_updates_per_frame: Some(64),
            angular_resolution: Some(1024),
            map_resolution: Some(512),
        }),
        EngineCmd::CmdLight3dUpsert(CmdLight3dUpsertArgs::Update(CmdLightUpdateArgs {
            realm_id,
            light_id: LIGHT_A_ID,
            label: None,
            kind: None,
            position: None,
            direction: None,
            color: None,
            ground_color: None,
            intensity: None,
            range: None,
            spot_inner_outer: None,
            layer_mask: None,
            shadow_layer_mask: None,
            active: None,
            cast_shadow: None,
        })),
        EngineCmd::CmdLight3dUpsert(CmdLight3dUpsertArgs::Update(CmdLightUpdateArgs {
            realm_id,
            light_id: LIGHT_B_ID,
            label: None,
            kind: None,
            position: None,
            direction: None,
            color: None,
            ground_color: None,
            intensity: None,
            range: None,
            spot_inner_outer: None,
            layer_mask: None,
            shadow_layer_mask: None,
            active: None,
            cast_shadow: None,
        })),
    ]
}

fn print_runtime_logs(events: Vec<EngineEvent>) {
    for event in events {
        if let EngineEvent::Log(log) = event {
            println!("[runtime/{:?}][{}] {}", log.level, log.tag, log.message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_contains_two_point_lights_with_shadow_casting() {
        let scene = build_scene(777);
        let lights: Vec<_> = scene
            .iter()
            .filter_map(|cmd| match cmd {
                EngineCmd::CmdLight3dUpsert(CmdLight3dUpsertArgs::Create(args)) => Some(args),
                _ => None,
            })
            .collect();
        assert_eq!(lights.len(), 2);
        assert!(lights.iter().all(|light| light.cast_shadow));
        assert!(
            lights
                .iter()
                .all(|light| matches!(light.kind, Some(LightKind::Point)))
        );
    }

    #[test]
    fn scene_has_mixed_cast_and_receive_shadow_flags() {
        let scene = build_scene(777);
        let mut has_receiver = false;
        let mut has_caster = false;
        for cmd in scene {
            match cmd {
                EngineCmd::CmdSprite2dUpsert(CmdSprite2dUpsertArgs::Create(args)) => {
                    has_receiver |= !args.cast_shadow && args.receive_shadow;
                    has_caster |= args.cast_shadow && !args.receive_shadow;
                }
                EngineCmd::CmdShape2dUpsert(CmdShape2dUpsertArgs::Create(args)) => {
                    has_receiver |= !args.cast_shadow && args.receive_shadow;
                    has_caster |= args.cast_shadow && !args.receive_shadow;
                }
                _ => {}
            }
        }
        assert!(has_receiver);
        assert!(has_caster);
    }
}
