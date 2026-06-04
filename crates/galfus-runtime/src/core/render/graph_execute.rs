use crate::core::realm::RealmId;
use crate::core::render::graph::RenderGraphPlan;
use crate::core::render::passes;
use galfus_realm_core::{
    RENDER_PASS_BATCH, RENDER_PASS_BLOOM, RENDER_PASS_COMPOSE, RENDER_PASS_CUSTOM_POST_FORWARD,
    RENDER_PASS_CUSTOM_PRE_FORWARD, RENDER_PASS_FORWARD, RENDER_PASS_LIGHT_CULL,
    RENDER_PASS_OUTLINE, RENDER_PASS_POST, RENDER_PASS_PREPARE, RENDER_PASS_SHADOW_2D,
    RENDER_PASS_SHADOW_3D, RENDER_PASS_SKYBOX, RENDER_PASS_SSAO, RENDER_PASS_SSAO_BLUR, RealmKind,
};
use std::hash::{DefaultHasher, Hash, Hasher};

use super::RenderState;
use super::frame_helpers::write_gpu_timestamp;
use bytemuck::{Pod, Zeroable};

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

#[cfg(target_arch = "wasm32")]
fn now_ns() -> u64 {
    (Date::now() * 1_000_000.0) as u64
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CustomFrameSemanticMeta {
    resolution: glam::Vec2,
    inv_resolution: glam::Vec2,
    frame_index: u32,
    flags: u32,
}

const SEM_SCENE_COLOR: u32 = 1 << 0;
const SEM_SCENE_DEPTH: u32 = 1 << 1;
const SEM_HISTORY0: u32 = 1 << 2;
const SEM_HISTORY1: u32 = 1 << 3;
const HISTORY_IDLE_RELEASE_FRAMES: u32 = 120;

fn shader_requests_history(node: &crate::core::render::graph::RenderGraphNode) -> bool {
    let Some(shader) = node.shader.as_ref() else {
        return false;
    };
    shader
        .capabilities
        .semantics
        .iter()
        .any(|value| value == "history0" || value == "history1")
}

fn encode_custom_param_bytes(
    schema: &std::collections::HashMap<String, String>,
    values: &std::collections::HashMap<String, crate::core::render::graph::RenderGraphValue>,
) -> Result<Vec<u8>, String> {
    let mut fields: Vec<(&str, &str)> = schema
        .iter()
        .map(|(name, ty)| (name.as_str(), ty.as_str()))
        .collect();
    fields.sort_by(|a, b| a.0.cmp(b.0));
    let mut bytes = Vec::with_capacity(fields.len() * 4 + 16);
    for (name, ty) in fields {
        let value = values
            .get(name)
            .ok_or_else(|| format!("Missing value for shader param '{}'", name))?;
        match (ty, value) {
            ("f32", crate::core::render::graph::RenderGraphValue::Float(v)) => {
                bytes.extend_from_slice(&(*v as f32).to_ne_bytes());
            }
            ("f32", crate::core::render::graph::RenderGraphValue::Int(v)) => {
                bytes.extend_from_slice(&(*v as f32).to_ne_bytes());
            }
            ("i32", crate::core::render::graph::RenderGraphValue::Int(v)) => {
                bytes.extend_from_slice(&(*v as i32).to_ne_bytes());
            }
            ("bool", crate::core::render::graph::RenderGraphValue::Bool(v)) => {
                let raw: u32 = if *v { 1 } else { 0 };
                bytes.extend_from_slice(&raw.to_ne_bytes());
            }
            _ => {
                return Err(format!(
                    "Unsupported shader param '{}' with declared type '{}'",
                    name, ty
                ));
            }
        }
    }
    while bytes.len() % 16 != 0 {
        bytes.push(0);
    }
    Ok(bytes)
}

fn ensure_custom_history_targets(
    record: &mut crate::core::resources::CameraRecord,
    device: &wgpu::Device,
    width: u32,
    height: u32,
) {
    crate::core::resources::ensure_render_target(
        device,
        &mut record.history0_target,
        width,
        height,
        wgpu::TextureFormat::Rgba16Float,
    );
    crate::core::resources::ensure_render_target(
        device,
        &mut record.history1_target,
        width,
        height,
        wgpu::TextureFormat::Rgba16Float,
    );
}

fn custom_semantics_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Custom Screen Semantics BGL"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Depth,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn execute_custom_screen_pass(
    node: &crate::core::render::graph::RenderGraphNode,
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    scene_color_view: &wgpu::TextureView,
    scene_depth_view: &wgpu::TextureView,
    history0_view: &wgpu::TextureView,
    history1_view: &wgpu::TextureView,
    linear_sampler: &wgpu::Sampler,
    point_sampler: &wgpu::Sampler,
    semantics_meta: CustomFrameSemanticMeta,
    frame_index: u64,
) -> bool {
    let Some(shader) = node.shader.as_ref() else {
        return false;
    };
    if shader.shader_type != crate::core::render::graph::RenderGraphShaderType::Screen {
        return false;
    }

    let generated_wgsl = match crate::core::render::graph::validate_shader_spec(
        shader,
        &node.inputs,
        &node.outputs,
        &node.params,
    ) {
        Ok(source) => source,
        Err(_) => return false,
    };
    let mut hasher = DefaultHasher::new();
    generated_wgsl.hash(&mut hasher);
    let shader_hash = hasher.finish();
    let shader_id = (1u64 << 63) | shader_hash;
    let has_params = !shader.params.is_empty();

    let param_bytes = if has_params {
        match encode_custom_param_bytes(&shader.params, &node.params) {
            Ok(bytes) => Some(bytes),
            Err(_) => return false,
        }
    } else {
        None
    };

    let key = crate::core::render::cache::PipelineKey {
        shader_id,
        color_format: target_format,
        color_target_count: 1,
        depth_format: None,
        sample_count: 1,
        topology: wgpu::PrimitiveTopology::TriangleList,
        polygon_mode: wgpu::PolygonMode::Fill,
        cull_mode: None,
        front_face: wgpu::FrontFace::Ccw,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::Always,
        blend: None,
    };
    let pipeline = render_state.cache.get_or_create(key, frame_index, || {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Custom Screen Shader"),
            source: wgpu::ShaderSource::Wgsl(generated_wgsl.clone().into()),
        });
        let semantics_layout = custom_semantics_layout(device);
        let bind_group_layout = if has_params {
            Some(
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Custom Screen Params BGL"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                }),
            )
        } else {
            None
        };
        let bind_group_layouts = match bind_group_layout.as_ref() {
            Some(layout) => vec![layout, &semantics_layout],
            None => vec![&semantics_layout],
        };
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Custom Screen Pipeline Layout"),
            bind_group_layouts: &bind_group_layouts,
            ..Default::default()
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Custom Screen Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    });

    let param_bind_group = if let Some(bytes) = param_bytes.as_ref() {
        let needs_realloc = render_state
            .custom_screen_param_buffer
            .as_ref()
            .map(|buffer| buffer.size() < bytes.len() as u64)
            .unwrap_or(true);
        if needs_realloc {
            render_state.custom_screen_param_buffer =
                Some(device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Custom Screen Params Buffer"),
                    size: bytes.len() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                }));
        }
        let Some(buffer) = render_state.custom_screen_param_buffer.as_ref() else {
            return false;
        };
        queue.write_buffer(buffer, 0, bytes);
        let bgl = pipeline.get_bind_group_layout(0);
        Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Custom Screen Params BG"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        }))
    } else {
        None
    };
    let semantics_bytes = bytemuck::bytes_of(&semantics_meta);
    let semantics_needs_realloc = render_state
        .custom_screen_semantics_buffer
        .as_ref()
        .map(|buffer| buffer.size() < semantics_bytes.len() as u64)
        .unwrap_or(true);
    if semantics_needs_realloc {
        render_state.custom_screen_semantics_buffer =
            Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Custom Screen Semantics Meta Buffer"),
                size: semantics_bytes.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
    }
    let Some(semantics_buffer) = render_state.custom_screen_semantics_buffer.as_ref() else {
        return false;
    };
    queue.write_buffer(semantics_buffer, 0, semantics_bytes);
    let semantics_layout_index = if has_params { 1 } else { 0 };
    let semantics_bgl = pipeline.get_bind_group_layout(semantics_layout_index);
    let semantics_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Custom Screen Semantics BG"),
        layout: &semantics_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(scene_color_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(scene_depth_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(history0_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(history1_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(linear_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(point_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: semantics_buffer.as_entire_binding(),
            },
        ],
    });

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Custom Screen Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target_view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });
    pass.set_pipeline(pipeline);
    if let Some(bind_group) = param_bind_group.as_ref() {
        pass.set_bind_group(0, bind_group, &[]);
    }
    pass.set_bind_group(semantics_layout_index, &semantics_bind_group, &[]);
    pass.draw(0..3, 0..1);
    true
}

pub(super) fn execute_graph_to_view(
    plan: &RenderGraphPlan,
    render_state: &mut RenderState,
    _realm_id: RealmId,
    realm_kind: RealmKind,
    _targets: &crate::core::target::TargetTable,
    _target_layers: &crate::core::target::TargetLayerTable,
    _surfaces: &crate::core::realm::SurfaceTable,
    _target_surface_map: &std::collections::HashMap<
        crate::core::target::TargetId,
        crate::core::realm::SurfaceId,
    >,
    _surface_targets: &std::collections::HashMap<
        crate::core::realm::SurfaceId,
        crate::core::resources::RenderTarget,
    >,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    target_size: glam::UVec2,
    frame_index: u64,
    _time_seconds: f64,
    _window_id: u32,
    _window_focused: bool,
    gpu_profiler: Option<&crate::core::profiling::gpu::GpuProfiler>,
    gpu_base: Option<u32>,
    shadow_cpu_ns_accum: &mut u64,
    log_events: &mut Vec<galfus_log::LogEvent>,
) -> bool {
    let mut gpu_written = false;
    let mut skybox_done = false;
    let pass_order: Vec<&str> = plan
        .order
        .iter()
        .map(|&node_idx| plan.nodes[node_idx].pass_id.as_str())
        .collect();
    galfus_log::galfus_log_debug!(
        log_events,
        "rendergraph.passes",
        "realm={} passes={:?}",
        _realm_id.0,
        pass_order
    );
    let mut history_requested = false;
    for &node_idx in &plan.order {
        if shader_requests_history(&plan.nodes[node_idx]) {
            history_requested = true;
            break;
        }
    }
    if !history_requested {
        history_requested = render_state.scene.materials.values().any(|record| {
            record
                .shader_capabilities
                .iter()
                .any(|value| value == "history0" || value == "history1")
        });
    }
    let active_camera_id = render_state.camera_order.first().copied();
    if history_requested
        && let Some(camera_id) = active_camera_id
        && let Some(record) = render_state.scene.cameras.get_mut(&camera_id)
    {
        record.history_idle_frames = 0;
        let had_history_targets =
            record.history0_target.is_some() && record.history1_target.is_some();
        let source_target = record
            .post_target
            .as_ref()
            .or(record.render_target.as_ref());
        if let Some(source) = source_target {
            let size = source.texture.size();
            ensure_custom_history_targets(record, device, size.width.max(1), size.height.max(1));
            let has_history_targets =
                record.history0_target.is_some() && record.history1_target.is_some();
            if !had_history_targets && has_history_targets {
                galfus_log::galfus_log_debug!(
                    log_events,
                    "history.allocate",
                    "camera={} size={}x{}",
                    camera_id,
                    size.width,
                    size.height
                );
            }
        }
    }
    let Some(library) = render_state.library.as_ref() else {
        return gpu_written;
    };
    let fallback_view = library.fallback_view.clone();
    let fallback_depth_view =
        library
            ._fallback_shadow_texture
            .create_view(&wgpu::TextureViewDescriptor {
                label: Some("Custom Screen Fallback Depth View"),
                format: Some(wgpu::TextureFormat::Depth32Float),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::DepthOnly,
                base_mip_level: 0,
                mip_level_count: Some(1),
                base_array_layer: 0,
                array_layer_count: Some(1),
                usage: None,
            });
    let linear_sampler = library.samplers.linear_clamp.clone();
    let point_sampler = library.samplers.point_clamp.clone();
    let (scene_color_view, scene_depth_view, history0_view, history1_view, semantics_meta) =
        if let Some(camera_id) = active_camera_id {
            if let Some(record) = render_state.camera_record(camera_id) {
                let source_target = record
                    .post_target
                    .as_ref()
                    .or(record.render_target.as_ref());
                let scene_color = source_target
                    .map(|target| &target.view)
                    .unwrap_or(&fallback_view);
                let scene_depth = record
                    .forward_depth_target
                    .as_ref()
                    .map(|target| &target.view)
                    .unwrap_or(&fallback_depth_view);
                let history0 = record
                    .history0_target
                    .as_ref()
                    .map(|target| &target.view)
                    .unwrap_or(&fallback_view);
                let history1 = record
                    .history1_target
                    .as_ref()
                    .map(|target| &target.view)
                    .unwrap_or(&fallback_view);
                let size = source_target.map(|target| target.texture.size());
                let (w, h) = size
                    .map(|value| (value.width.max(1) as f32, value.height.max(1) as f32))
                    .unwrap_or((1.0, 1.0));
                let mut flags = SEM_SCENE_DEPTH;
                if source_target.is_some() {
                    flags |= SEM_SCENE_COLOR;
                }
                if record.history0_target.is_some() && record.history_valid {
                    flags |= SEM_HISTORY0;
                }
                if record.history1_target.is_some() && record.history_valid {
                    flags |= SEM_HISTORY1;
                }
                (
                    scene_color.clone(),
                    scene_depth.clone(),
                    history0.clone(),
                    history1.clone(),
                    CustomFrameSemanticMeta {
                        resolution: glam::Vec2::new(w, h),
                        inv_resolution: glam::Vec2::new(1.0 / w, 1.0 / h),
                        frame_index: frame_index as u32,
                        flags,
                    },
                )
            } else {
                (
                    fallback_view.clone(),
                    fallback_depth_view.clone(),
                    fallback_view.clone(),
                    fallback_view.clone(),
                    CustomFrameSemanticMeta {
                        resolution: glam::Vec2::ONE,
                        inv_resolution: glam::Vec2::ONE,
                        frame_index: frame_index as u32,
                        flags: SEM_SCENE_DEPTH,
                    },
                )
            }
        } else {
            (
                fallback_view.clone(),
                fallback_depth_view.clone(),
                fallback_view.clone(),
                fallback_view.clone(),
                CustomFrameSemanticMeta {
                    resolution: glam::Vec2::ONE,
                    inv_resolution: glam::Vec2::ONE,
                    frame_index: frame_index as u32,
                    flags: SEM_SCENE_DEPTH,
                },
            )
        };

    for &node_idx in &plan.order {
        let node = &plan.nodes[node_idx];
        galfus_log::galfus_log_debug!(
            log_events,
            "rendergraph.pass",
            "realm={} pass={} node={:?}",
            _realm_id.0,
            node.pass_id,
            node.node_id
        );
        match node.pass_id.as_str() {
            RENDER_PASS_SHADOW_3D | RENDER_PASS_SHADOW_2D => {
                #[cfg(not(target_arch = "wasm32"))]
                let shadow_start = std::time::Instant::now();
                #[cfg(target_arch = "wasm32")]
                let shadow_start = now_ns();
                match realm_kind {
                    RealmKind::ThreeD => {
                        passes::pass_shadow_3d_update(
                            render_state,
                            device,
                            queue,
                            encoder,
                            frame_index,
                        );
                    }
                    RealmKind::TwoD => {
                        passes::pass_shadow_2d_update(
                            render_state,
                            device,
                            queue,
                            encoder,
                            frame_index,
                            target_size,
                        );
                    }
                }
                if matches!(realm_kind, RealmKind::ThreeD)
                    && let Some(shadow) = &mut render_state.shadow_3d
                {
                    shadow.sync_table_for_realm(0);
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let shadow_ns = shadow_start.elapsed().as_nanos() as u64;
                    *shadow_cpu_ns_accum = shadow_cpu_ns_accum.saturating_add(shadow_ns);
                    if matches!(realm_kind, RealmKind::TwoD)
                        && let Some(shadow2d) = render_state.shadow_2d.as_mut()
                    {
                        let shadow_ms = shadow_ns as f32 / 1_000_000.0;
                        galfus_log::galfus_log_debug!(
                            log_events,
                            "render.shadow2d",
                            "realm={} shadow2d_ms={:.3} updated_layers={} angular_res={} budget=disabled",
                            _realm_id.0,
                            shadow_ms as f64,
                            shadow2d.last_updated_layers,
                            shadow2d.config.angular_resolution
                        );
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    *shadow_cpu_ns_accum =
                        shadow_cpu_ns_accum.saturating_add(now_ns().saturating_sub(shadow_start));
                }
            }
            RENDER_PASS_SKYBOX => {
                skybox_done =
                    passes::pass_skybox(render_state, device, queue, encoder, frame_index);
            }
            RENDER_PASS_LIGHT_CULL => {
                if let Some(base) = gpu_base {
                    write_gpu_timestamp(encoder, gpu_profiler, base, &mut gpu_written);
                }
                passes::pass_light_cull(render_state, device, encoder, frame_index);
                if let Some(base) = gpu_base {
                    write_gpu_timestamp(encoder, gpu_profiler, base + 1, &mut gpu_written);
                }
            }
            RENDER_PASS_FORWARD => match realm_kind {
                RealmKind::ThreeD => {
                    if let Some(base) = gpu_base {
                        write_gpu_timestamp(encoder, gpu_profiler, base + 2, &mut gpu_written);
                    }
                    passes::pass_forward(
                        render_state,
                        device,
                        queue,
                        encoder,
                        frame_index,
                        !skybox_done,
                        log_events,
                    );
                    if let Some(base) = gpu_base {
                        write_gpu_timestamp(encoder, gpu_profiler, base + 3, &mut gpu_written);
                    }
                }
                RealmKind::TwoD => {
                    passes::pass_2d_draw(
                        render_state,
                        device,
                        queue,
                        encoder,
                        target_view,
                        target_format,
                        target_size,
                        frame_index,
                    );
                }
            },
            RENDER_PASS_OUTLINE => {
                passes::pass_outline(render_state, device, queue, encoder, frame_index);
            }
            RENDER_PASS_SSAO => {
                passes::pass_ssao(render_state, device, queue, encoder, frame_index);
            }
            RENDER_PASS_SSAO_BLUR => {
                passes::pass_ssao_blur(render_state, device, queue, encoder, frame_index);
            }
            RENDER_PASS_BLOOM => {
                passes::pass_bloom(render_state, device, queue, encoder, frame_index);
            }
            RENDER_PASS_POST => {
                passes::pass_post(render_state, device, queue, encoder, frame_index);
            }
            RENDER_PASS_COMPOSE => match realm_kind {
                RealmKind::ThreeD => {
                    if let Some(base) = gpu_base {
                        write_gpu_timestamp(encoder, gpu_profiler, base + 4, &mut gpu_written);
                    }
                    passes::pass_compose_to_view(
                        render_state,
                        device,
                        queue,
                        encoder,
                        target_view,
                        target_format,
                        target_size.x,
                        target_size.y,
                        frame_index,
                    );
                    if let Some(base) = gpu_base {
                        write_gpu_timestamp(encoder, gpu_profiler, base + 5, &mut gpu_written);
                    }
                }
                RealmKind::TwoD => {
                    passes::pass_2d_compose(
                        render_state,
                        device,
                        queue,
                        encoder,
                        target_view,
                        target_format,
                        target_size,
                        frame_index,
                    );
                }
            },
            RENDER_PASS_PREPARE => {
                passes::pass_2d_prepare(render_state);
            }
            RENDER_PASS_BATCH => {
                passes::pass_2d_batch(render_state);
            }
            RENDER_PASS_CUSTOM_PRE_FORWARD | RENDER_PASS_CUSTOM_POST_FORWARD => {
                if execute_custom_screen_pass(
                    node,
                    render_state,
                    device,
                    queue,
                    encoder,
                    target_view,
                    target_format,
                    &scene_color_view,
                    &scene_depth_view,
                    &history0_view,
                    &history1_view,
                    &linear_sampler,
                    &point_sampler,
                    semantics_meta,
                    frame_index,
                ) {
                    gpu_written = true;
                }
            }
            _ => {
                if execute_custom_screen_pass(
                    node,
                    render_state,
                    device,
                    queue,
                    encoder,
                    target_view,
                    target_format,
                    &scene_color_view,
                    &scene_depth_view,
                    &history0_view,
                    &history1_view,
                    &linear_sampler,
                    &point_sampler,
                    semantics_meta,
                    frame_index,
                ) {
                    gpu_written = true;
                }
            }
        }
    }

    if history_requested
        && let Some(camera_id) = active_camera_id
        && let Some(record) = render_state.scene.cameras.get_mut(&camera_id)
    {
        let source_target = record
            .post_target
            .as_ref()
            .or(record.render_target.as_ref());
        if let Some(source) = source_target {
            let source_size = source.texture.size();
            if source.sample_count == 1 {
                if let Some(history0) = record.history0_target.as_ref() {
                    if history0.sample_count == 1 {
                        if record.history_valid
                            && let Some(history1) = record.history1_target.as_ref()
                        {
                            encoder.copy_texture_to_texture(
                                wgpu::TexelCopyTextureInfo {
                                    texture: &history0.texture,
                                    mip_level: 0,
                                    origin: wgpu::Origin3d::ZERO,
                                    aspect: wgpu::TextureAspect::All,
                                },
                                wgpu::TexelCopyTextureInfo {
                                    texture: &history1.texture,
                                    mip_level: 0,
                                    origin: wgpu::Origin3d::ZERO,
                                    aspect: wgpu::TextureAspect::All,
                                },
                                wgpu::Extent3d {
                                    width: source_size.width,
                                    height: source_size.height,
                                    depth_or_array_layers: 1,
                                },
                            );
                        }
                        encoder.copy_texture_to_texture(
                            wgpu::TexelCopyTextureInfo {
                                texture: &source.texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::TexelCopyTextureInfo {
                                texture: &history0.texture,
                                mip_level: 0,
                                origin: wgpu::Origin3d::ZERO,
                                aspect: wgpu::TextureAspect::All,
                            },
                            wgpu::Extent3d {
                                width: source_size.width,
                                height: source_size.height,
                                depth_or_array_layers: 1,
                            },
                        );
                        record.history_valid = true;
                        galfus_log::galfus_log_debug!(
                            log_events,
                            "history.copy",
                            "camera={} size={}x{} valid={}",
                            camera_id,
                            source_size.width,
                            source_size.height,
                            record.history_valid
                        );
                    }
                }
            }
        }
    }
    if !history_requested
        && let Some(camera_id) = active_camera_id
        && let Some(record) = render_state.scene.cameras.get_mut(&camera_id)
    {
        record.history_idle_frames = record.history_idle_frames.saturating_add(1);
        if record.history_idle_frames >= HISTORY_IDLE_RELEASE_FRAMES {
            record.history0_target = None;
            record.history1_target = None;
            record.history_valid = false;
            record.history_idle_frames = 0;
            galfus_log::galfus_log_debug!(
                log_events,
                "history.release",
                "camera={} reason=idle timeout_frames={}",
                camera_id,
                HISTORY_IDLE_RELEASE_FRAMES
            );
        }
    }

    gpu_written
}
