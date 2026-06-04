use galfus_core::core::cmd::EngineEvent;
use galfus_core::core::cmd::{
    CmdCamera3dUpsertArgs, CmdEnvironmentUpsertArgs, CmdLight3dUpsertArgs,
    CmdMaterialDefinitionUpsertArgs, CmdMaterialUpsertArgs, CmdModel3dUpsertArgs, EngineCmd,
};
use galfus_core::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdEnvironmentCreateArgs, CmdLightCreateArgs,
    CmdMaterialCreateArgs, CmdMaterialDefinitionCreateArgs, CmdModelCreateArgs, CmdModelUpdateArgs,
    CmdPrimitiveGeometryCreateArgs, EnvironmentConfig, LightKind, MaterialKind, MaterialOptions,
    MaterialRealmKind, MaterialShaderType, PostProcessConfig, PrimitiveShape,
};
use galfus_core::core::target::{
    CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs, DimensionValue, TargetKind, TargetLayerLayout,
};
use glam::{Mat4, Quat, Vec2, Vec3, Vec4};
use std::collections::HashMap;

use crate::demo::DemoContext;
use crate::demo::DemoRunOptions;
use crate::demo::io::send_commands;
use crate::demo::scenarios::run_with_window_loop;

const FRAME_MS: u32 = 16;

const WINDOW_TARGET_ID: u64 = 1;

const GEOMETRY_CUBE_ID: u32 = 1;
const GEOMETRY_FLOOR_ID: u32 = 11;
const MATERIAL_STANDARD_ID: u32 = 2;
const MATERIAL_PBR_ID: u32 = 30;
const MATERIAL_CUSTOM_SIMPLE_ID: u32 = 4;
const MATERIAL_FLOOR_ID: u32 = 12;
const MATERIAL_DEF_CUSTOM_SIMPLE_ID: u32 = 100;
const CAMERA_ID: u32 = 5;
const LIGHT_ID: u32 = 6;
const LIGHT_FILL_ID: u32 = 7;
const ENVIRONMENT_ID: u32 = 7;
const MODEL_CUBE_A_ID: u32 = 8;
const MODEL_CUBE_B_ID: u32 = 9;
const MODEL_CUBE_C_ID: u32 = 10;
const MODEL_FLOOR_ID: u32 = 13;

pub fn run(ctx: DemoContext, options: DemoRunOptions) -> bool {
    let realm_3d = ctx.realm_id;

    let mut setup = vec![EngineCmd::CmdTargetUpsert(CmdTargetUpsertArgs {
        target_id: WINDOW_TARGET_ID,
        kind: TargetKind::Window,
        window_id: Some(ctx.window_id),
        size: None,
        format_policy: None,
        alpha_policy: None,
        msaa_samples: None,
    })];

    setup.extend(build_realm3d_scene(realm_3d));
    setup.push(bind_layer(
        realm_3d,
        WINDOW_TARGET_ID,
        full_layout(0),
        vec![CAMERA_ID],
        Some(ENVIRONMENT_ID),
    ));

    let _ = send_commands(setup);

    run_with_window_loop(
        ctx.window_id,
        FRAME_MS,
        options.timeout,
        |time_seconds| {
            let updates = build_rotating_cube_updates(realm_3d, time_seconds);
            let _ = send_commands(updates);
        },
        print_runtime_logs,
    )
}

fn build_rotating_cube_updates(realm_id: u32, time_seconds: f32) -> Vec<EngineCmd> {
    let angle_a = time_seconds * 1.80;
    let angle_b = time_seconds * 2.50 + 0.60;
    let angle_c = time_seconds * 1.40 + 1.20;
    vec![
        EngineCmd::CmdModel3dUpsert(CmdModel3dUpsertArgs::Update(CmdModelUpdateArgs {
            realm_id,
            model_id: MODEL_CUBE_A_ID,
            label: None,
            geometry_id: None,
            material_id: None,
            transform: Some(
                Mat4::from_translation(Vec3::new(-2.0, 0.0, 0.0)) * Mat4::from_rotation_y(angle_a),
            ),
            layer_mask: None,
            active: None,
            cast_shadow: None,
            receive_shadow: None,
            cast_outline: None,
            outline_color: None,
        })),
        EngineCmd::CmdModel3dUpsert(CmdModel3dUpsertArgs::Update(CmdModelUpdateArgs {
            realm_id,
            model_id: MODEL_CUBE_B_ID,
            label: None,
            geometry_id: None,
            material_id: None,
            transform: Some(
                Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)) * Mat4::from_rotation_y(angle_b),
            ),
            layer_mask: None,
            active: None,
            cast_shadow: None,
            receive_shadow: None,
            cast_outline: None,
            outline_color: None,
        })),
        EngineCmd::CmdModel3dUpsert(CmdModel3dUpsertArgs::Update(CmdModelUpdateArgs {
            realm_id,
            model_id: MODEL_CUBE_C_ID,
            label: None,
            geometry_id: None,
            material_id: None,
            transform: Some(
                Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)) * Mat4::from_rotation_y(angle_c),
            ),
            layer_mask: None,
            active: None,
            cast_shadow: None,
            receive_shadow: None,
            cast_outline: None,
            outline_color: None,
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

fn bind_layer(
    realm_id: u32,
    target_id: u64,
    layout: TargetLayerLayout,
    enabled_camera_ids: Vec<u32>,
    environment_id: Option<u32>,
) -> EngineCmd {
    EngineCmd::CmdTargetLayerUpsert(CmdTargetLayerUpsertArgs {
        realm_id,
        target_id,
        layout,
        enabled_camera_ids,
        environment_id,
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

fn build_realm3d_scene(realm_id: u32) -> Vec<EngineCmd> {
    vec![
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: GEOMETRY_CUBE_ID,
            label: Some("demo-cube".into()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: GEOMETRY_FLOOR_ID,
            label: Some("demo-floor-plane".into()),
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_STANDARD_ID,
            label: Some("demo-mat-standard".into()),
            slug: "standard".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: Some(MaterialOptions::Schema(HashMap::from([(
                "baseColor".to_string(),
                Vec4::new(1.0, 0.2, 0.8, 1.0),
            )]))),
        })),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_PBR_ID,
            label: Some("demo-mat-pbr".into()),
            slug: "pbr".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: Some(MaterialOptions::Schema(HashMap::from([
                ("baseColor".to_string(), Vec4::new(1.0, 1.0, 0.2, 1.0)),
                ("metallic".to_string(), Vec4::new(0.55, 0.0, 0.0, 0.0)),
                ("roughness".to_string(), Vec4::new(0.35, 0.0, 0.0, 0.0)),
            ]))),
        })),
        EngineCmd::CmdMaterialDefinitionUpsert(CmdMaterialDefinitionUpsertArgs::Create(
            CmdMaterialDefinitionCreateArgs {
                definition_id: MATERIAL_DEF_CUSTOM_SIMPLE_ID,
                slug: "demo-custom-simple".into(),
                label: Some("demo-def-custom-simple".into()),
                realm_kind: MaterialRealmKind::ThreeD,
                preset: None,
                shader_type: Some(MaterialShaderType::Model),
                shader_source: Some(r#"
fn vertex(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.world_position = vec3<f32>(0.0);
  out.world_normal = vec3<f32>(0.0);
  out.uv = vec2<f32>(0.0);
  out.clip_position = vec4<f32>(0.0);
  return out;
}

fn fragment(input: FragmentInput) -> FragmentOutput {
  var out: FragmentOutput;
  let fresnel = pow(1.0 - max(dot(normalize(input.world_normal), vec3<f32>(0.0, 0.0, 1.0)), 0.0), 3.0);
  out.color = vec4<f32>(mix(vec3<f32>(0.2, 0.35, 0.95), vec3<f32>(0.8, 0.9, 1.0), fresnel), 1.0);
  out.emissive = vec4<f32>(0.0, 0.0, 0.0, 1.0);
  return out;
}
"#
                .to_string()),
                shader_params_schema: None,
                capabilities: None,
            },
        )),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_CUSTOM_SIMPLE_ID,
            label: Some("demo-mat-custom-simple".into()),
            slug: "demo-custom-simple".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: Some(MaterialOptions::Schema(HashMap::from([
                ("baseColor".to_string(), Vec4::new(0.25, 0.45, 0.98, 1.0)),
                ("emissiveColor".to_string(), Vec4::new(0.0, 0.0, 0.0, 0.0)),
                ("specColor".to_string(), Vec4::new(0.0, 1.0, 1.0, 1.0)),
                ("specPower".to_string(), Vec4::new(64.0, 0.0, 0.0, 0.0)),
            ]))),
        })),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_FLOOR_ID,
            label: Some("demo-mat-floor".into()),
            slug: "standard".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: Some(MaterialOptions::Schema(HashMap::from([
                ("baseColor".to_string(), Vec4::new(0.24, 0.24, 0.26, 1.0)),
                ("specColor".to_string(), Vec4::new(0.05, 0.05, 0.05, 1.0)),
                ("specPower".to_string(), Vec4::new(8.0, 0.0, 0.0, 0.0)),
            ]))),
        })),
        EngineCmd::CmdCamera3dUpsert(CmdCamera3dUpsertArgs::Create(CmdCameraCreateArgs {
            realm_id,
            camera_id: CAMERA_ID,
            label: Some("demo-camera".into()),
            transform: Mat4::look_at_rh(Vec3::new(0.0, 2.0, 7.0), Vec3::ZERO, Vec3::Y).inverse(),
            kind: CameraKind::Perspective,
            flags: 0,
            near_far: Vec2::new(0.1, 120.0),
            layer_mask: 1,
            order: 0,
            view_position: None,
            ortho_scale: 10.0,
        })),
        EngineCmd::CmdLight3dUpsert(CmdLight3dUpsertArgs::Create(CmdLightCreateArgs {
            realm_id,
            light_id: LIGHT_ID,
            label: Some("demo-light".into()),
            kind: Some(LightKind::Point),
            position: Some(Vec4::new(3.0, 5.0, 5.0, 1.0)),
            direction: None,
            color: Some(Vec4::new(0.0, 1.0, 1.0, 1.0)),
            ground_color: None,
            intensity: Some(4.0),
            range: Some(30.0),
            spot_inner_outer: None,
            layer_mask: 1,
            shadow_layer_mask: None,
            shadow_softness: None,
            active: true,
            cast_shadow: true,
        })),
        EngineCmd::CmdLight3dUpsert(CmdLight3dUpsertArgs::Create(CmdLightCreateArgs {
            realm_id,
            light_id: LIGHT_FILL_ID,
            label: Some("demo-fill-light".into()),
            kind: Some(LightKind::Point),
            position: Some(Vec4::new(-4.0, 3.0, -2.0, 1.0)),
            direction: None,
            color: Some(Vec4::new(1.0, 0.0, 1.0, 1.0)),
            ground_color: None,
            intensity: Some(2.4),
            range: Some(18.0),
            spot_inner_outer: None,
            layer_mask: 1,
            shadow_layer_mask: None,
            shadow_softness: None,
            active: true,
            cast_shadow: true,
        })),
        EngineCmd::CmdEnvironmentUpsert(CmdEnvironmentUpsertArgs::Create(
            CmdEnvironmentCreateArgs {
                environment_id: ENVIRONMENT_ID,
                config: EnvironmentConfig {
                    clear_color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                    post: PostProcessConfig {
                        outline_enabled: true,
                        outline_strength: 0.5,
                        outline_threshold: 0.25,
                        outline_width: 1.25,
                        bloom_enabled: true,
                        bloom_intensity: 1.0,
                        bloom_threshold: 0.05,
                        bloom_knee: 0.5,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )),
        EngineCmd::CmdModel3dUpsert(CmdModel3dUpsertArgs::Create(CmdModelCreateArgs {
            realm_id,
            model_id: MODEL_CUBE_A_ID,
            label: Some("demo-cube-standard".into()),
            geometry_id: GEOMETRY_CUBE_ID,
            material_id: Some(MATERIAL_STANDARD_ID),
            transform: Mat4::from_translation(Vec3::new(-2.0, 0.0, 0.0)),
            layer_mask: 1,
            active: true,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: true,
            outline_color: Vec4::new(1.0, 0.55, 0.0, 1.0),
        })),
        EngineCmd::CmdModel3dUpsert(CmdModel3dUpsertArgs::Create(CmdModelCreateArgs {
            realm_id,
            model_id: MODEL_CUBE_B_ID,
            label: Some("demo-cube-pbr".into()),
            geometry_id: GEOMETRY_CUBE_ID,
            material_id: Some(MATERIAL_PBR_ID),
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            layer_mask: 1,
            active: true,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
        EngineCmd::CmdModel3dUpsert(CmdModel3dUpsertArgs::Create(CmdModelCreateArgs {
            realm_id,
            model_id: MODEL_CUBE_C_ID,
            label: Some("demo-cube-custom-simple".into()),
            geometry_id: GEOMETRY_CUBE_ID,
            material_id: Some(MATERIAL_CUSTOM_SIMPLE_ID),
            transform: Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            layer_mask: 1,
            active: true,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
        EngineCmd::CmdModel3dUpsert(CmdModel3dUpsertArgs::Create(CmdModelCreateArgs {
            realm_id,
            model_id: MODEL_FLOOR_ID,
            label: Some("demo-floor".into()),
            geometry_id: GEOMETRY_FLOOR_ID,
            material_id: Some(MATERIAL_FLOOR_ID),
            transform: Mat4::from_scale_rotation_translation(
                Vec3::new(20.0, 20.0, 1.0),
                Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                Vec3::new(0.0, -1.0, 0.0),
            ),
            layer_mask: 1,
            active: true,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
    ]
}
