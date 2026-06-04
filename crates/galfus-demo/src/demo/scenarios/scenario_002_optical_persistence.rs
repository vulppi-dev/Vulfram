use galfus_core::core::cmd::EngineEvent;
use galfus_core::core::cmd::{
    CmdCamera3dUpsertArgs, CmdEnvironmentUpsertArgs, CmdLight3dUpsertArgs,
    CmdMaterialDefinitionUpsertArgs, CmdMaterialUpsertArgs, CmdModel3dUpsertArgs, EngineCmd,
};
use galfus_core::core::resources::{
    CameraKind, CmdCameraCreateArgs, CmdEnvironmentCreateArgs, CmdLightCreateArgs,
    CmdMaterialCreateArgs, CmdMaterialDefinitionCreateArgs, CmdModelCreateArgs, CmdModelUpdateArgs,
    CmdPrimitiveGeometryCreateArgs, EnvironmentConfig, LightKind, MaterialKind, MaterialOptions,
    MaterialRealmKind, MaterialShaderCapabilities, MaterialShaderType, PostProcessConfig,
    PrimitiveShape,
};
use galfus_core::core::target::{
    CmdTargetLayerUpsertArgs, CmdTargetUpsertArgs, DimensionValue, TargetKind, TargetLayerLayout,
};
use glam::{Mat4, Vec2, Vec3, Vec4};
use std::collections::HashMap;

use crate::demo::DemoContext;
use crate::demo::DemoRunOptions;
use crate::demo::io::send_commands;
use crate::demo::scenarios::run_with_window_loop;

const FRAME_MS: u32 = 16;

const WINDOW_TARGET_ID: u64 = 1;
const GEOMETRY_CUBE_ID: u32 = 201;
const GEOMETRY_FLOOR_ID: u32 = 202;
const MATERIAL_DEF_GHOST_ID: u32 = 210;
const MATERIAL_DEF_FRESNEL_ID: u32 = 211;
const MATERIAL_GHOST_ID: u32 = 212;
const MATERIAL_FRESNEL_ID: u32 = 213;
const MATERIAL_FLOOR_ID: u32 = 214;
const CAMERA_ID: u32 = 220;
const LIGHT_ID: u32 = 221;
const ENVIRONMENT_ID: u32 = 222;
const MODEL_GHOST_CUBE_ID: u32 = 230;
const MODEL_FRESNEL_CUBE_ID: u32 = 231;
const MODEL_FLOOR_ID: u32 = 232;

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
    setup.extend(build_scene(realm_3d));
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
        |t| {
            let updates = vec![
                EngineCmd::CmdModel3dUpsert(CmdModel3dUpsertArgs::Update(CmdModelUpdateArgs {
                    realm_id: realm_3d,
                    model_id: MODEL_GHOST_CUBE_ID,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(
                        Mat4::from_translation(Vec3::new(
                            -2.0 + (t * 2.4).sin() * 1.8,
                            0.8 + (t * 3.0).sin() * 0.28,
                            0.0,
                        )) * Mat4::from_rotation_y(t * 2.8)
                            * Mat4::from_rotation_x(t * 1.7),
                    ),
                    layer_mask: None,
                    active: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                })),
                EngineCmd::CmdModel3dUpsert(CmdModel3dUpsertArgs::Update(CmdModelUpdateArgs {
                    realm_id: realm_3d,
                    model_id: MODEL_FRESNEL_CUBE_ID,
                    label: None,
                    geometry_id: None,
                    material_id: None,
                    transform: Some(
                        Mat4::from_translation(Vec3::new(
                            2.0 + (t * 2.0).sin() * 1.2,
                            0.8 + (t * 2.6).sin() * 0.2,
                            0.0,
                        )) * Mat4::from_rotation_y(-(t * 2.2))
                            * Mat4::from_rotation_x(t * 1.3),
                    ),
                    layer_mask: None,
                    active: None,
                    cast_shadow: None,
                    receive_shadow: None,
                    cast_outline: None,
                    outline_color: None,
                })),
            ];
            let _ = send_commands(updates);
        },
        print_runtime_logs,
    )
}

fn build_scene(realm_id: u32) -> Vec<EngineCmd> {
    vec![
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: GEOMETRY_CUBE_ID,
            label: Some("demo2-cube".into()),
            shape: PrimitiveShape::Cube,
            options: None,
        }),
        EngineCmd::CmdPrimitiveGeometryCreate(CmdPrimitiveGeometryCreateArgs {
            geometry_id: GEOMETRY_FLOOR_ID,
            label: Some("demo2-floor".into()),
            shape: PrimitiveShape::Plane,
            options: None,
        }),
        EngineCmd::CmdMaterialDefinitionUpsert(CmdMaterialDefinitionUpsertArgs::Create(
            CmdMaterialDefinitionCreateArgs {
                definition_id: MATERIAL_DEF_GHOST_ID,
                slug: "demo2-ghost".into(),
                label: Some("demo2-ghost-definition".into()),
                realm_kind: MaterialRealmKind::ThreeD,
                preset: None,
                shader_type: Some(MaterialShaderType::Model),
                shader_source: Some(
                    r#"
fn project_world_to_screen_uv(world_position: vec3<f32>) -> vec2<f32> {
  let clip = camera.view_projection * vec4<f32>(world_position, 1.0);
  let inv_w = select(0.0, 1.0 / clip.w, abs(clip.w) > 1e-6);
  let ndc = clip.xy * inv_w;
  return vec2<f32>(ndc.x * 0.5 + 0.5, -ndc.y * 0.5 + 0.5);
}

fn vertex(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.world_position = input.position;
  out.world_normal = input.normal;
  out.uv = input.uv;
  out.clip_position = vec4<f32>(0.0);
  return out;
}

fn fragment(input: FragmentInput) -> FragmentOutput {
  var out: FragmentOutput;
  let base = vec3<f32>(0.08, 0.48, 0.88);
  let screen_uv = project_world_to_screen_uv(input.world_position);
  let ghost_uv = screen_uv + vec2<f32>(0.012, 0.0);
  let history = sample_history0(ghost_uv).rgb;
  let trail_decay = 0.965;
  let history_weight = 0.62;
  let persisted = history * trail_decay;
  let ghost_tint = vec3<f32>(0.18, 0.9, 1.0);
  let ghost = persisted * ghost_tint * history_weight;
  let composed = max(base, base + ghost);

  out.color = vec4<f32>(composed, 1.0);
  out.emissive = vec4<f32>(ghost_tint * 0.08, 1.0);
  return out;
}
"#
                    .to_string(),
                ),
                shader_params_schema: None,
                capabilities: Some(MaterialShaderCapabilities {
                    semantics: vec!["history0".to_string()],
                }),
            },
        )),
        EngineCmd::CmdMaterialDefinitionUpsert(CmdMaterialDefinitionUpsertArgs::Create(
            CmdMaterialDefinitionCreateArgs {
                definition_id: MATERIAL_DEF_FRESNEL_ID,
                slug: "demo2-fresnel".into(),
                label: Some("demo2-fresnel-definition".into()),
                realm_kind: MaterialRealmKind::ThreeD,
                preset: None,
                shader_type: Some(MaterialShaderType::Model),
                shader_source: Some(
                    r#"
fn vertex(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.world_position = input.position;
  out.world_normal = input.normal;
  out.uv = input.uv;
  out.clip_position = vec4<f32>(0.0);
  return out;
}

fn fragment(input: FragmentInput) -> FragmentOutput {
  var out: FragmentOutput;
  let t = frame.time;
  let base = vec3<f32>(0.95, 0.28, 0.18);
  let view_dir = normalize(camera.position.xyz - input.world_position);
  let fresnel = pow(1.0 - max(dot(normalize(input.world_normal), view_dir), 0.0), 2.4);
  let pulse = 0.5 + 0.5 * sin(t * 5.0);
  let rim = fresnel * (0.35 + 0.45 * pulse);
  let color = mix(base, vec3<f32>(1.0, 0.92, 0.82), rim);
  out.color = vec4<f32>(color, 1.0);
  out.emissive = vec4<f32>(vec3<f32>(1.0, 0.45, 0.28) * (0.08 + 0.55 * rim), 1.0);
  return out;
}
"#
                    .to_string(),
                ),
                shader_params_schema: None,
                capabilities: None,
            },
        )),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_GHOST_ID,
            label: Some("demo2-mat-ghost".into()),
            slug: "demo2-ghost".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: Some(MaterialOptions::Schema(HashMap::from([(
                "baseColor".to_string(),
                Vec4::new(0.2, 0.7, 1.0, 1.0),
            )]))),
        })),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_FRESNEL_ID,
            label: Some("demo2-mat-fresnel".into()),
            slug: "demo2-fresnel".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: Some(MaterialOptions::Schema(HashMap::from([(
                "baseColor".to_string(),
                Vec4::new(1.0, 0.3, 0.2, 1.0),
            )]))),
        })),
        EngineCmd::CmdMaterialUpsert(CmdMaterialUpsertArgs::Create(CmdMaterialCreateArgs {
            material_id: MATERIAL_FLOOR_ID,
            label: Some("demo2-mat-floor".into()),
            slug: "standard".into(),
            kind: MaterialKind::Shader,
            realm_kind: MaterialRealmKind::ThreeD,
            options: Some(MaterialOptions::Schema(HashMap::from([
                ("baseColor".to_string(), Vec4::new(0.08, 0.08, 0.10, 1.0)),
                ("specColor".to_string(), Vec4::new(0.02, 0.02, 0.02, 1.0)),
                ("specPower".to_string(), Vec4::new(6.0, 0.0, 0.0, 0.0)),
            ]))),
        })),
        EngineCmd::CmdCamera3dUpsert(CmdCamera3dUpsertArgs::Create(CmdCameraCreateArgs {
            realm_id,
            camera_id: CAMERA_ID,
            label: Some("demo2-camera".into()),
            transform: Mat4::look_at_rh(
                Vec3::new(0.0, 2.3, 7.5),
                Vec3::new(0.0, 0.7, 0.0),
                Vec3::Y,
            )
            .inverse(),
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
            label: Some("demo2-light".into()),
            kind: Some(LightKind::Point),
            position: Some(Vec4::new(3.0, 4.0, 4.0, 1.0)),
            direction: None,
            color: Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
            ground_color: None,
            intensity: Some(3.5),
            range: Some(24.0),
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
                        outline_enabled: false,
                        bloom_enabled: false,
                        ssao_enabled: false,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )),
        EngineCmd::CmdModel3dUpsert(CmdModel3dUpsertArgs::Create(CmdModelCreateArgs {
            realm_id,
            model_id: MODEL_GHOST_CUBE_ID,
            label: Some("demo2-ghost-cube-model".into()),
            geometry_id: GEOMETRY_CUBE_ID,
            material_id: Some(MATERIAL_GHOST_ID),
            transform: Mat4::from_translation(Vec3::new(-2.0, 0.8, 0.0)),
            layer_mask: 1,
            active: true,
            cast_shadow: true,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
        EngineCmd::CmdModel3dUpsert(CmdModel3dUpsertArgs::Create(CmdModelCreateArgs {
            realm_id,
            model_id: MODEL_FRESNEL_CUBE_ID,
            label: Some("demo2-fresnel-cube-model".into()),
            geometry_id: GEOMETRY_CUBE_ID,
            material_id: Some(MATERIAL_FRESNEL_ID),
            transform: Mat4::from_translation(Vec3::new(2.0, 0.8, 0.0)),
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
            label: Some("demo2-floor-model".into()),
            geometry_id: GEOMETRY_FLOOR_ID,
            material_id: Some(MATERIAL_FLOOR_ID),
            transform: Mat4::from_scale(Vec3::new(12.0, 1.0, 12.0))
                * Mat4::from_translation(Vec3::new(0.0, -0.01, 0.0)),
            layer_mask: 1,
            active: true,
            cast_shadow: false,
            receive_shadow: true,
            cast_outline: false,
            outline_color: Vec4::ZERO,
        })),
    ]
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

fn print_runtime_logs(events: Vec<EngineEvent>) {
    for event in events {
        if let EngineEvent::Log(log) = event {
            println!("[runtime/{:?}][{}] {}", log.level, log.tag, log.message);
        }
    }
}
