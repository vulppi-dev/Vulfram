use glam::UVec2;

use crate::core::realm::{PresentState, RealmId, RealmKind, RealmState, SurfaceKind, SurfaceState};
use crate::core::render::RenderState;
use crate::core::render::ensure_runtime_render_defaults;
use crate::core::render::graph::DEFAULT_3D_RENDER_GRAPH_ID;
use crate::core::resources::RenderTarget;
use crate::core::state::EngineState;

pub struct WindowRealmBinding {
    pub realm_id: RealmId,
}

pub struct WindowRenderBootstrapArtifacts {
    pub config: wgpu::SurfaceConfiguration,
    pub render_state: RenderState,
    pub surface_target: Option<RenderTarget>,
}

pub fn register_window_realm(
    engine: &mut EngineState,
    window_id: u32,
    size: UVec2,
) -> WindowRealmBinding {
    ensure_runtime_render_defaults(&mut engine.universal_state);
    let surface_id = engine
        .universal_state
        .composition
        .surfaces
        .alloc(SurfaceState {
            kind: SurfaceKind::Onscreen,
            size,
            format_policy: None,
            alpha_policy: None,
            msaa_samples: None,
        });
    let realm_id = engine.universal_state.composition.realms.alloc(RealmState {
        kind: RealmKind::ThreeD,
        output_surface: Some(surface_id),
        render_graph_id: Some(DEFAULT_3D_RENDER_GRAPH_ID),
        importance: 1,
        cache_policy: 0,
        last_render_frame: 0,
    });
    engine
        .universal_state
        .scene
        .realm3d
        .entities
        .entry(realm_id)
        .or_default();
    let _present_id = engine
        .universal_state
        .composition
        .presents
        .alloc(PresentState {
            window_id,
            surface: surface_id,
        });

    WindowRealmBinding { realm_id }
}

pub fn build_window_render_state(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    surface_format: wgpu::TextureFormat,
    target_size: UVec2,
    rgba16f_msaa_supported_mask: u8,
) -> (RenderState, Option<RenderTarget>) {
    let mut render_state = RenderState::new(surface_format);
    render_state.rgba16f_msaa_supported_mask = rgba16f_msaa_supported_mask;
    render_state.init(device, queue, surface_format);
    render_state.on_resize(device, target_size.x, target_size.y);

    let mut surface_target = None;
    crate::core::resources::ensure_render_target(
        device,
        &mut surface_target,
        target_size.x,
        target_size.y,
        wgpu::TextureFormat::Rgba16Float,
    );

    (render_state, surface_target)
}

pub fn build_window_render_bootstrap_artifacts(
    surface: &wgpu::Surface<'static>,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    caps: &wgpu::SurfaceCapabilities,
    bootstrap_target: galfus_platform::PlatformRenderBootstrapTarget,
    rgba16f_msaa_supported_mask: u8,
) -> WindowRenderBootstrapArtifacts {
    let surface_plan = galfus_render::plan_surface_config(caps, bootstrap_target);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        width: surface_plan.width,
        height: surface_plan.height,
        present_mode: surface_plan.present_mode,
        format: surface_plan.format,
        alpha_mode: surface_plan.alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(device, &config);

    let (render_state, surface_target) = build_window_render_state(
        device,
        queue,
        surface_plan.format,
        bootstrap_target.size,
        rgba16f_msaa_supported_mask,
    );

    WindowRenderBootstrapArtifacts {
        config,
        render_state,
        surface_target,
    }
}

#[cfg(test)]
mod tests {
    use super::register_window_realm;
    use crate::core::realm::{RealmKind, SurfaceKind};
    use crate::core::render::graph::DEFAULT_3D_RENDER_GRAPH_ID;
    use crate::core::state::EngineState;
    use crate::core::target::{
        TargetId, TargetKind, TargetLayerLayout, TargetLayerState, TargetState,
    };
    use glam::UVec2;
    use std::collections::HashMap;

    #[test]
    fn register_window_realm_creates_3d_realm_with_surface_and_default_graph() {
        let mut engine = EngineState::new();
        let window_id = 11;
        let size = UVec2::new(1280, 720);

        let binding = register_window_realm(&mut engine, window_id, size);

        let realm_entry = engine
            .universal_state
            .composition
            .realms
            .entries
            .get(&binding.realm_id)
            .expect("window realm should exist");
        assert_eq!(realm_entry.value.kind, RealmKind::ThreeD);
        assert_eq!(
            realm_entry.value.render_graph_id,
            Some(DEFAULT_3D_RENDER_GRAPH_ID)
        );

        let surface_id = realm_entry
            .value
            .output_surface
            .expect("window realm should own an output surface");
        let surface_entry = engine
            .universal_state
            .composition
            .surfaces
            .entries
            .get(&surface_id)
            .expect("output surface should exist");
        assert_eq!(surface_entry.value.kind, SurfaceKind::Onscreen);
        assert_eq!(surface_entry.value.size, size);

        assert!(
            engine
                .universal_state
                .scene
                .realm3d
                .entities
                .contains_key(&binding.realm_id),
            "realm3d entities bucket should be initialized"
        );
        assert!(
            engine
                .universal_state
                .composition
                .presents
                .entries
                .values()
                .any(
                    |entry| entry.value.window_id == window_id && entry.value.surface == surface_id
                ),
            "present should bind the window to the realm output surface"
        );
    }

    #[test]
    fn window_realm_layer_produces_framegraph_invocation_for_that_realm() {
        let mut engine = EngineState::new();
        let window_id = 21;
        let window_size = UVec2::new(1600, 900);
        let binding = register_window_realm(&mut engine, window_id, window_size);

        let target_id = TargetId(1);
        engine.universal_state.targets.targets.entries.insert(
            target_id,
            TargetState {
                kind: TargetKind::Window,
                window_id: Some(window_id),
                size: None,
                format_policy: None,
                alpha_policy: None,
                msaa_samples: None,
            },
        );
        engine.universal_state.targets.target_layers.entries.insert(
            (binding.realm_id.0, target_id),
            TargetLayerState {
                realm_id: binding.realm_id.0,
                target_id,
                layout: TargetLayerLayout::default(),
                enabled_camera_ids: Vec::new(),
                environment_id: None,
            },
        );

        let invocations = crate::core::target::collect_render_invocations(
            &[target_id],
            &engine.universal_state.targets.targets.entries,
            &engine.universal_state.targets.target_layers.entries,
            &HashMap::from([(window_id, window_size)]),
            7,
        );

        assert_eq!(invocations.len(), 1);
        let invocation = &invocations[0];
        assert_eq!(invocation.realm_id, binding.realm_id.0);
        assert_eq!(invocation.target_id, target_id);
        assert_eq!(invocation.render_size_px, window_size);
        assert_eq!(invocation.frame_id, 7);
    }

    #[test]
    fn realm3d_element_states_are_created_with_expected_values() {
        use crate::core::resources::{
            CameraKind, CmdCameraCreateArgs, CmdLightCreateArgs, CmdMaterialCreateArgs,
            CmdModelCreateArgs, CmdResultCameraCreate, CmdResultLightCreate,
            CmdResultMaterialCreate, CmdResultModelCreate, LightKind, MaterialKind,
            ShaderMaterialPreset,
        };
        use glam::{Mat4, Vec2, Vec4};

        let mut engine = EngineState::new();
        let binding = register_window_realm(&mut engine, 31, UVec2::new(1024, 768));
        let realm_id = binding.realm_id.0;

        let material_res: CmdResultMaterialCreate =
            crate::core::resources::engine_cmd_material_create(
                &mut engine,
                &CmdMaterialCreateArgs {
                    material_id: 2001,
                    label: Some("mat-standard".into()),
                    slug: "standard".into(),
                    kind: MaterialKind::Shader,
                    realm_kind: crate::core::resources::MaterialRealmKind::ThreeD,
                    options: None,
                },
            );
        assert!(material_res.success);

        let camera_res: CmdResultCameraCreate = crate::core::resources::engine_cmd_camera_create(
            &mut engine,
            &CmdCameraCreateArgs {
                realm_id,
                camera_id: 3001,
                label: Some("camera-main".into()),
                transform: Mat4::IDENTITY,
                kind: CameraKind::Perspective,
                flags: 0,
                near_far: Vec2::new(0.1, 100.0),
                layer_mask: 1,
                order: 0,
                view_position: None,
                ortho_scale: 10.0,
            },
        );
        assert!(camera_res.success);

        let light_res: CmdResultLightCreate = crate::core::resources::engine_cmd_light_create(
            &mut engine,
            &CmdLightCreateArgs {
                realm_id,
                light_id: 4001,
                label: Some("light-main".into()),
                kind: Some(LightKind::Point),
                position: Some(Vec4::new(1.0, 2.0, 3.0, 1.0)),
                direction: None,
                color: Some(Vec4::new(1.0, 1.0, 1.0, 1.0)),
                ground_color: None,
                intensity: Some(4.0),
                range: Some(12.0),
                spot_inner_outer: None,
                layer_mask: 1,
                shadow_layer_mask: None,
                cast_shadow: false,
                active: true,
            },
        );
        assert!(light_res.success);

        let model_res: CmdResultModelCreate = crate::core::resources::engine_cmd_model_create(
            &mut engine,
            &CmdModelCreateArgs {
                realm_id,
                model_id: 5001,
                label: Some("cube".into()),
                geometry_id: 1001,
                material_id: Some(2001),
                transform: Mat4::IDENTITY,
                layer_mask: 1,
                cast_shadow: false,
                receive_shadow: false,
                cast_outline: false,
                outline_color: Vec4::ZERO,
                active: true,
            },
        );
        assert!(model_res.success);

        let entities = engine
            .universal_state
            .scene
            .realm3d
            .entities
            .get(&binding.realm_id)
            .expect("realm entities should exist");
        assert!(entities.cameras.contains_key(&3001));
        assert!(entities.lights.contains_key(&4001));
        assert!(entities.models.contains_key(&5001));

        let material = engine
            .universal_state
            .scene
            .realm3d
            .materials
            .get(&2001)
            .expect("material should exist");
        assert_eq!(material.label.as_deref(), Some("mat-standard"));
        assert_eq!(material.preset, ShaderMaterialPreset::Standard);
    }
}
