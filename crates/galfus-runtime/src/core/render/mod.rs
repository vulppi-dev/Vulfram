pub mod cache;
#[cfg(not(target_arch = "wasm32"))]
mod debug_capture;
mod frame_helpers;
pub mod gizmos;
pub mod graph;
mod graph_execute;
mod passes;
mod realm_graph;
pub mod runtime;
mod scene_sync;
pub mod state;
use crate::core::profiling::gpu::apply_gpu_timing_report;
use crate::core::realm::{FrameReport, apply_target_graph_stats};
use crate::core::resources::RenderTarget;
use crate::core::state::EngineState;
use crate::core::target::TargetKind;
use frame_helpers::{
    apply_realm_environment_bindings, apply_target_size_requests, build_target_surface_map,
    collect_window_camera_target_sizes, refresh_window_target_textures, should_render_realm,
};
use graph_execute::execute_graph_to_view;
use realm_graph::{
    collect_present_sizes, collect_surface_views, ensure_surface_target, map_realms_to_windows,
    resolve_realm_surface,
};
pub use runtime::RenderManager;
use scene_sync::{sync_scene_from_realm_and_universal_resources, sync_window_geometry_registry};
pub use state::{
    Realm3dState, RealmEntities, RenderCatalogState, RenderResourceState, RenderState,
    SceneRuntimeState, UniversalGeometryRecord,
};
use std::collections::HashSet;
pub fn bloom_chain_size(base: u32, level: usize) -> u32 {
    passes::bloom_chain_size(base, level)
}

#[cfg(target_arch = "wasm32")]
use js_sys::Date;

#[cfg(target_arch = "wasm32")]
fn now_ns() -> u64 {
    (Date::now() * 1_000_000.0) as u64
}

#[cfg(not(target_arch = "wasm32"))]
fn pass_capture_path(base_path: &str, suffix: &str) -> String {
    let path = std::path::Path::new(base_path);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("capture");
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("png");
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new(""));
    let file_name = format!("{stem}-{suffix}.{ext}");
    if parent.as_os_str().is_empty() {
        file_name
    } else {
        parent.join(file_name).to_string_lossy().to_string()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn capture_source_view(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    frame_index: u64,
    base_path: &str,
    capture_size: glam::UVec2,
    source_view: &wgpu::TextureView,
    source_size: glam::UVec2,
    suffix: &str,
) {
    let capture_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("DebugCapture.PassTarget"),
        size: wgpu::Extent3d {
            width: capture_size.x,
            height: capture_size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let capture_view = capture_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut capture_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("DebugCapture.PassEncoder"),
    });
    passes::pass_compose_surface(
        render_state,
        device,
        queue,
        &mut capture_encoder,
        &capture_view,
        wgpu::TextureFormat::Bgra8Unorm,
        capture_size,
        source_view,
        source_size,
        frame_index,
    );
    queue.submit(Some(capture_encoder.finish()));
    let path = pass_capture_path(base_path, suffix);
    let _ = debug_capture::capture_texture_png(
        device,
        queue,
        &capture_texture,
        capture_size,
        wgpu::TextureFormat::Bgra8Unorm,
        &path,
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn capture_pass_outputs(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    capture_passes: bool,
    downscale_factor: f32,
    base_path: &str,
    frame_index: u64,
    compose_target: &RenderTarget,
) {
    if !capture_passes {
        return;
    }
    let resolve_size = |width: u32, height: u32| {
        let factor = downscale_factor.clamp(0.05, 1.0);
        if (factor - 1.0).abs() < f32::EPSILON {
            return glam::UVec2::new(width.max(1), height.max(1));
        }
        glam::UVec2::new(
            ((width as f32) * factor).round().max(1.0) as u32,
            ((height as f32) * factor).round().max(1.0) as u32,
        )
    };

    let mut jobs: Vec<(wgpu::TextureView, glam::UVec2, String)> = Vec::new();
    let compose_size = compose_target.texture.size();
    jobs.push((
        compose_target.view.clone(),
        glam::UVec2::new(compose_size.width, compose_size.height),
        "pass-compose".to_string(),
    ));

    for camera_id in &render_state.camera_order {
        let Some(camera) = render_state.camera_record(*camera_id) else {
            continue;
        };
        if let Some(target) = camera.render_target.as_ref() {
            let size = target.texture.size();
            jobs.push((
                target.view.clone(),
                glam::UVec2::new(size.width, size.height),
                format!("cam-{camera_id}-pass-forward"),
            ));
        }
        if let Some(target) = camera.post_target.as_ref() {
            let size = target.texture.size();
            jobs.push((
                target.view.clone(),
                glam::UVec2::new(size.width, size.height),
                format!("cam-{camera_id}-pass-post"),
            ));
        }
    }
    for (source_view, source_size, suffix) in jobs {
        capture_source_view(
            render_state,
            device,
            queue,
            frame_index,
            base_path,
            resolve_size(source_size.x, source_size.y),
            &source_view,
            source_size,
            &suffix,
        );
    }
}

fn resolve_realm_render_graph<'a>(
    universal: &'a crate::core::realm::UniversalState,
    realm_id: crate::core::realm::RealmId,
) -> Option<&'a crate::core::render::graph::RenderGraphState> {
    let realm = universal.composition.realms.entries.get(&realm_id)?;
    let realm_kind = realm.value.kind;
    let render_graph_id =
        galfus_render::resolve_render_graph_id(realm.value.render_graph_id, realm_kind);
    let registry = match realm_kind {
        crate::core::realm::RealmKind::ThreeD => &universal.render_catalog.render_graphs_3d,
        crate::core::realm::RealmKind::TwoD => &universal.render_catalog.render_graphs_2d,
    };
    if let Some(graph) = registry.get(&render_graph_id) {
        return Some(&graph.state);
    }
    None
}

pub fn ensure_runtime_render_defaults(universal: &mut crate::core::realm::UniversalState) {
    let fallback_3d = crate::core::render::graph::fallback_graph();
    let hash_3d = crate::core::render::graph::render_graph_desc_hash(&fallback_3d);
    let state_3d = universal
        .render_catalog
        .render_graph_plan_cache_3d
        .entry(hash_3d)
        .or_default()
        .clone();
    universal
        .render_catalog
        .render_graphs_3d
        .entry(crate::core::render::graph::DEFAULT_3D_RENDER_GRAPH_ID)
        .or_insert(crate::core::render::graph::RenderGraphRecord {
            state: state_3d,
            desc_hash: hash_3d,
        });

    let fallback_2d = crate::core::render::graph::realm2d_fallback_graph();
    let hash_2d = crate::core::render::graph::render_graph_desc_hash(&fallback_2d);
    let state_2d = universal
        .render_catalog
        .render_graph_plan_cache_2d
        .entry(hash_2d)
        .or_insert_with(crate::core::render::graph::RenderGraphState::new_realm2d)
        .clone();
    universal
        .render_catalog
        .render_graphs_2d
        .entry(crate::core::render::graph::DEFAULT_2D_RENDER_GRAPH_ID)
        .or_insert(crate::core::render::graph::RenderGraphRecord {
            state: state_2d,
            desc_hash: hash_2d,
        });
}

pub fn render_frames(engine_state: &mut EngineState) {
    engine_state.profiling.render.total_ns = 0;
    engine_state.profiling.render.shadow_ns = 0;
    engine_state.profiling.render.windows_ns = 0;
    engine_state.profiling.gpu.shadow_ns = 0;
    engine_state.profiling.gpu.light_cull_ns = 0;
    engine_state.profiling.gpu.forward_ns = 0;
    engine_state.profiling.gpu.compose_ns = 0;
    engine_state.profiling.gpu.total_ns = 0;

    let target_size_requests =
        std::mem::take(&mut engine_state.universal_state.targets.target_size_requests);
    apply_target_size_requests(engine_state, &target_size_requests);

    let (target_plan, target_diff) = {
        let target_dependencies = crate::core::target::collect_target_dependencies(
            &engine_state.universal_state.targets.target_layers.entries,
            &engine_state.universal_state.scene,
        );
        let cache = &mut engine_state.universal_state.targets.target_graph_cache;
        let diff = cache
            .update(
                &engine_state.universal_state.targets.targets.entries,
                &target_dependencies,
                &engine_state.universal_state.targets.target_layers.entries,
                &engine_state.universal_state.composition.realms,
            )
            .cloned();
        let plan = cache.last_plan.clone();
        (plan, diff)
    };
    crate::core::target::sync_auto_graph(engine_state);

    let device = match engine_state.device.as_ref() {
        Some(device) => device.clone(),
        None => return,
    };

    let queue = match engine_state.queue.as_ref() {
        Some(queue) => queue.clone(),
        None => return,
    };

    if let Some(gpu_profiler) = engine_state.gpu_profiler.as_mut() {
        gpu_profiler.ensure_capacity(&device, &queue, engine_state.window.states.len());
    }

    let time = engine_state.runtime.time_ms() as f32 / 1000.0;
    let delta_time = engine_state.runtime.delta_time_ms() as f32 / 1000.0;
    let frame_index = engine_state.runtime.frame_index() as u32;
    let frame_spec = crate::core::resources::FrameComponent::new(time, delta_time, frame_index);
    let mut gpu_written = false;
    #[cfg(not(target_arch = "wasm32"))]
    let debug_capture_path_template = engine_state.debug_capture.path_template.clone();
    #[cfg(not(target_arch = "wasm32"))]
    let debug_capture_passes_enabled = engine_state.debug_capture.capture_passes;
    #[cfg(not(target_arch = "wasm32"))]
    let debug_capture_downscale_factor = engine_state.debug_capture.downscale_factor;

    #[cfg(not(target_arch = "wasm32"))]
    let total_start = std::time::Instant::now();
    #[cfg(target_arch = "wasm32")]
    let total_start = now_ns();

    // 1. Render realms in target-scheduler order
    let mut windows_ns: u64 = 0;
    let mut shadow_ns: u64 = 0;
    let mut frame_report = FrameReport::default();
    apply_target_graph_stats(&mut frame_report, &target_plan, target_diff.as_ref());
    let realm_plan =
        crate::core::realm::RealmGraphPlanner.build_plan(&engine_state.universal_state);
    galfus_log::galfus_log_debug!(
        engine_state,
        "realmgraph",
        "realmgraph: realms={} cut_edges={} order={:?}",
        realm_plan.order.len(),
        realm_plan.cut_edges.len(),
        realm_plan.order
    );
    let window_sizes: std::collections::HashMap<u32, glam::UVec2> = engine_state
        .window
        .states
        .iter()
        .map(|(window_id, state)| (*window_id, state.inner_size))
        .collect();
    let target_invocations = crate::core::target::collect_render_invocations(
        &target_plan.order,
        &engine_state.universal_state.targets.targets.entries,
        &engine_state.universal_state.targets.target_layers.entries,
        &window_sizes,
        engine_state.runtime.frame_index(),
    );
    galfus_log::galfus_log_debug!(
        engine_state,
        "framegraph",
        "framegraph: targets={} invocations={} cut_edges={}",
        target_plan.order.len(),
        target_invocations.len(),
        target_plan.cut_edges.len()
    );
    frame_report.target_invocations = target_invocations
        .iter()
        .map(|invocation| crate::core::realm::TargetInvocationReport {
            realm_id: invocation.realm_id,
            target_id: invocation.target_id.0,
            rect_px: [
                invocation.resolved_rect_px.x,
                invocation.resolved_rect_px.y,
                invocation.resolved_rect_px.z,
                invocation.resolved_rect_px.w,
            ],
            render_size_px: [invocation.render_size_px.x, invocation.render_size_px.y],
            frame_id: invocation.frame_id,
        })
        .collect();
    for invocation in &target_invocations {
        galfus_log::galfus_log_debug!(
            engine_state,
            "framegraph.intent",
            "draw-intent realm={} target={} size={}x{} rect=[{},{},{},{}] frame={}",
            invocation.realm_id,
            invocation.target_id.0,
            invocation.render_size_px.x,
            invocation.render_size_px.y,
            invocation.resolved_rect_px.x,
            invocation.resolved_rect_px.y,
            invocation.resolved_rect_px.z,
            invocation.resolved_rect_px.w,
            invocation.frame_id
        );
    }
    frame_report.target_autolink_failures = engine_state
        .universal_state
        .targets
        .target_autolink_failures
        .clone();
    let realm_windows = map_realms_to_windows(&engine_state.universal_state);
    collect_present_sizes(
        &engine_state.universal_state,
        &engine_state.window.states,
        &mut engine_state.present_sizes_cache,
        &mut engine_state.present_sizes_hash,
    );
    let present_sizes = &engine_state.present_sizes_cache;
    let surface_views = collect_surface_views(
        &device,
        &engine_state.universal_state,
        &mut engine_state.surface_targets,
        present_sizes,
    );
    let target_surface_map = build_target_surface_map(
        &engine_state.universal_state.targets.targets,
        &engine_state.universal_state.targets.auto_links,
    );
    let blocked_target_ids: HashSet<_> = target_plan
        .cut_edges
        .iter()
        .map(|edge| edge.parent)
        .collect();
    refresh_window_target_textures(
        &mut engine_state.render.states,
        &engine_state
            .universal_state
            .scene
            .render_resources
            .target_texture_binds,
        &blocked_target_ids,
        &target_surface_map,
        &engine_state.surface_targets,
    );
    let mut updated_surfaces: HashSet<crate::core::realm::SurfaceId> = HashSet::new();
    let mut invocation_targets: std::collections::HashMap<(u64, u32), RenderTarget> =
        std::collections::HashMap::new();
    const MAX_REALM_ITERATIONS: u32 = 1;
    let mut iteration: u32 = 0;
    loop {
        frame_report.no_progress_realms.clear();
        let mut window_counter: u32 = 0;

        for invocation in &target_invocations {
            let realm_id = crate::core::realm::RealmId(invocation.realm_id);
            let target_id = invocation.target_id;
            let target_window_id = engine_state
                .universal_state
                .targets
                .targets
                .entries
                .get(&target_id)
                .and_then(|target| target.window_id);
            let Some(window_id) =
                target_window_id.or_else(|| realm_windows.get(&realm_id).copied())
            else {
                continue;
            };
            let Some(window_state) = engine_state.window.states.get(&window_id) else {
                continue;
            };
            let Some(render_state) = engine_state.render.get_mut(&window_id) else {
                continue;
            };
            let Some(layer_state) = engine_state
                .universal_state
                .targets
                .target_layers
                .entries
                .get(&invocation.layer_key)
            else {
                continue;
            };
            let layer_blend_mode = layer_state.layout.blend_mode;
            let layer_clip = layer_state.layout.clip;
            let layer_opacity = layer_state.layout.opacity;
            let present_surface_for_window = |window_id: u32| {
                engine_state
                    .universal_state
                    .composition
                    .presents
                    .entries
                    .values()
                    .find(|entry| entry.value.window_id == window_id)
                    .map(|entry| entry.value.surface)
            };
            let Some(surface_id) = target_surface_map
                .get(&target_id)
                .copied()
                .or_else(|| {
                    engine_state
                        .universal_state
                        .targets
                        .targets
                        .entries
                        .get(&target_id)
                        .and_then(|target| match (target.kind, target.window_id) {
                            (TargetKind::Window, Some(window_id)) => {
                                present_surface_for_window(window_id)
                            }
                            _ => None,
                        })
                })
                .or_else(|| resolve_realm_surface(&engine_state.universal_state, realm_id))
            else {
                continue;
            };
            let should_render = engine_state
                .universal_state
                .composition
                .realms
                .entries
                .get_mut(&realm_id)
                .map(|realm_entry| {
                    should_render_realm(realm_entry, engine_state.runtime.frame_index())
                })
                .unwrap_or(false);
            let realm_kind = engine_state
                .universal_state
                .composition
                .realms
                .entries
                .get(&realm_id)
                .map(|entry| entry.value.kind)
                .unwrap_or(crate::core::realm::RealmKind::ThreeD);
            if !should_render {
                FrameReport::push_unique(&mut frame_report.throttled_realms, realm_id.0);
                continue;
            }

            let target_size = surface_views
                .get(&surface_id)
                .map(|snapshot| snapshot.size)
                .or_else(|| {
                    engine_state
                        .universal_state
                        .composition
                        .surfaces
                        .entries
                        .get(&surface_id)
                        .map(|entry| entry.value.size)
                })
                .unwrap_or(window_state.inner_size);
            let target_format = engine_state
                .universal_state
                .composition
                .surfaces
                .entries
                .get(&surface_id)
                .and_then(|entry| entry.value.format_policy)
                .unwrap_or(wgpu::TextureFormat::Rgba16Float);

            let (target_view, target_format) = {
                let surface_target = ensure_surface_target(
                    &device,
                    &mut engine_state.surface_targets,
                    surface_id,
                    target_size,
                    target_format,
                );
                (surface_target.view.clone(), surface_target.format)
            };
            let invocation_size = invocation.render_size_px;
            let invocation_target = galfus_render::ensure_surface_target(
                &device,
                &mut invocation_targets,
                (target_id.0, invocation.realm_id),
                invocation_size,
                target_format,
            );
            let invocation_view = invocation_target.view.clone();

            #[cfg(not(target_arch = "wasm32"))]
            let window_start = std::time::Instant::now();
            #[cfg(target_arch = "wasm32")]
            let window_start = now_ns();

            sync_scene_from_realm_and_universal_resources(
                render_state,
                &engine_state.universal_state,
                realm_id,
            );
            sync_window_geometry_registry(
                render_state,
                &engine_state.universal_state.scene.realm3d.geometries,
            );
            let camera_target_sizes = collect_window_camera_target_sizes(
                &engine_state.universal_state,
                realm_id,
                window_id,
                window_state.inner_size,
            );
            if render_state.sync_camera_targets_and_projection(
                &device,
                window_state.inner_size,
                Some(&camera_target_sizes),
            ) {
                if let Some(shadow) = render_state.shadow_3d.as_mut() {
                    shadow.mark_dirty();
                }
            }
            render_state.prepare_render(&device, frame_spec, true);
            let active_camera_order = render_state.camera_order.clone();
            let enabled_camera_ids: std::collections::HashSet<u32> =
                layer_state.enabled_camera_ids.iter().copied().collect();
            render_state.camera_order = active_camera_order
                .into_iter()
                .filter(|camera_id| enabled_camera_ids.contains(camera_id))
                .collect();

            let mut graph_encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            let already_rendered_surface = updated_surfaces.contains(&surface_id);
            let clear_alpha = engine_state
                .universal_state
                .composition
                .realms
                .entries
                .get(&realm_id)
                .map(|entry| galfus_render::clear_alpha_for_realm_kind(entry.value.kind))
                .unwrap_or(1.0);
            {
                let _clear_pass = graph_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Realm Target Clear"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: if already_rendered_surface {
                                wgpu::LoadOp::Load
                            } else {
                                wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: clear_alpha,
                                })
                            },
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            }

            let gpu_base = engine_state.gpu_profiler.as_ref().and_then(|gpu_profiler| {
                let base = 2 + (window_counter as u32) * 6;
                if gpu_profiler.query_count() >= base + 6 {
                    Some(base)
                } else {
                    None
                }
            });
            window_counter = window_counter.saturating_add(1);

            let plan = match resolve_realm_render_graph(&engine_state.universal_state, realm_id) {
                Some(graph) => graph.plan().clone(),
                None => {
                    log::error!("Realm {} is missing a render graph", realm_id.0);
                    FrameReport::push_unique(&mut frame_report.no_progress_realms, realm_id.0);
                    continue;
                }
            };
            apply_realm_environment_bindings(
                render_state,
                &engine_state.universal_state,
                realm_id,
                window_id,
            );
            let universal = &mut engine_state.universal_state;
            let targets = &universal.targets.targets;
            let target_layers = &universal.targets.target_layers;
            let surfaces = &universal.composition.surfaces;
            #[cfg(not(target_arch = "wasm32"))]
            let window_focused = engine_state
                .window
                .cache
                .caches
                .get(&window_id)
                .map(|cache| cache.focused)
                .unwrap_or(true);
            #[cfg(target_arch = "wasm32")]
            let window_focused = true;

            let mut pass_logs: Vec<galfus_log::LogEvent> = Vec::new();
            gpu_written |= execute_graph_to_view(
                &plan,
                render_state,
                realm_id,
                realm_kind,
                targets,
                target_layers,
                surfaces,
                &target_surface_map,
                &engine_state.surface_targets,
                &device,
                &queue,
                &mut graph_encoder,
                &invocation_view,
                target_format,
                invocation_size,
                engine_state.runtime.frame_index(),
                time as f64,
                window_id,
                window_focused,
                engine_state.gpu_profiler.as_ref(),
                gpu_base,
                &mut shadow_ns,
                &mut pass_logs,
            );
            for log_event in pass_logs {
                engine_state
                    .runtime
                    .push_event(crate::core::cmd::EngineEvent::Log(log_event));
            }
            queue.submit(Some(graph_encoder.finish()));
            #[cfg(not(target_arch = "wasm32"))]
            device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: None,
                })
                .ok();

            let mut overlay_encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            let _ = (layer_blend_mode, layer_clip, layer_opacity);
            passes::pass_compose_surface(
                render_state,
                &device,
                &queue,
                &mut overlay_encoder,
                &target_view,
                target_format,
                target_size,
                &invocation_view,
                invocation_size,
                engine_state.runtime.frame_index(),
            );

            updated_surfaces.insert(surface_id);

            queue.submit(Some(overlay_encoder.finish()));
            #[cfg(not(target_arch = "wasm32"))]
            device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: None,
                })
                .ok();
            #[cfg(not(target_arch = "wasm32"))]
            {
                let base_path = debug_capture_path_template
                    .replace("{frame}", &engine_state.runtime.frame_index().to_string())
                    .replace("{window}", &window_id.to_string())
                    .replace("{surface}", &surface_id.0.to_string());
                capture_pass_outputs(
                    render_state,
                    &device,
                    &queue,
                    debug_capture_passes_enabled,
                    debug_capture_downscale_factor,
                    &base_path,
                    engine_state.runtime.frame_index(),
                    invocation_target,
                );
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                windows_ns = windows_ns.saturating_add(window_start.elapsed().as_nanos() as u64);
            }
            #[cfg(target_arch = "wasm32")]
            {
                windows_ns = windows_ns.saturating_add(now_ns().saturating_sub(window_start));
            }
        }

        for present in engine_state
            .universal_state
            .composition
            .presents
            .entries
            .values()
        {
            if !updated_surfaces.contains(&present.value.surface) {
                continue;
            }
            let window_id = present.value.window_id;
            let Some(surface_target) = engine_state.surface_targets.get(&present.value.surface)
            else {
                continue;
            };
            let (window_states, render_states) = (
                &mut engine_state.window.states,
                &mut engine_state.render.states,
            );
            let Some(window_state) = window_states.get_mut(&window_id) else {
                continue;
            };
            let Some(render_state) = render_states.get_mut(&window_id) else {
                continue;
            };

            let surface_texture = match window_state.surface.get_current_texture() {
                Ok(texture) => texture,
                Err(e) => {
                    log::error!("Failed to get surface texture: {:?}", e);
                    continue;
                }
            };

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            let surface_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            passes::pass_compose_surface(
                render_state,
                &device,
                &queue,
                &mut encoder,
                &surface_view,
                window_state.config.format,
                glam::UVec2::new(window_state.config.width, window_state.config.height),
                &surface_target.view,
                glam::UVec2::new(
                    surface_target.texture.size().width,
                    surface_target.texture.size().height,
                ),
                engine_state.runtime.frame_index(),
            );

            queue.submit(Some(encoder.finish()));
            #[cfg(not(target_arch = "wasm32"))]
            device
                .poll(wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: None,
                })
                .ok();
            #[cfg(not(target_arch = "wasm32"))]
            {
                if engine_state.debug_capture.should_capture() {
                    let capture_path = engine_state.debug_capture.resolve_path(
                        engine_state.runtime.frame_index(),
                        window_id,
                        present.value.surface.0,
                    );
                    let capture_size = engine_state.debug_capture.resolve_capture_size(
                        window_state.config.width,
                        window_state.config.height,
                    );
                    let capture_texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("DebugCapture.ComposeTarget"),
                        size: wgpu::Extent3d {
                            width: capture_size.x,
                            height: capture_size.y,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: window_state.config.format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::COPY_SRC,
                        view_formats: &[],
                    });
                    let capture_view =
                        capture_texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut capture_encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("DebugCapture.ComposeEncoder"),
                        });
                    passes::pass_compose_surface(
                        render_state,
                        &device,
                        &queue,
                        &mut capture_encoder,
                        &capture_view,
                        window_state.config.format,
                        capture_size,
                        &surface_target.view,
                        glam::UVec2::new(
                            surface_target.texture.size().width,
                            surface_target.texture.size().height,
                        ),
                        engine_state.runtime.frame_index(),
                    );
                    queue.submit(Some(capture_encoder.finish()));

                    let _ = debug_capture::capture_texture_png(
                        &device,
                        &queue,
                        &capture_texture,
                        capture_size,
                        window_state.config.format,
                        &capture_path,
                    );
                    if !engine_state.debug_capture.capture_every_frame {
                        engine_state.debug_capture.captured_once = true;
                    }
                }
            }
            surface_texture.present();
            #[cfg(not(target_arch = "wasm32"))]
            {
                let now = std::time::Instant::now();
                let delta_ns = window_state
                    .last_present_instant
                    .map(|prev| now.duration_since(prev).as_nanos() as u64)
                    .unwrap_or(0);
                window_state.last_present_instant = Some(now);
                window_state.last_frame_delta_ns = delta_ns;
                window_state.fps_instant = if delta_ns > 0 {
                    1_000_000_000.0 / delta_ns as f64
                } else {
                    0.0
                };
            }
            #[cfg(target_arch = "wasm32")]
            {
                let now = now_ns();
                let delta_ns = if window_state.last_present_ns > 0 {
                    now.saturating_sub(window_state.last_present_ns)
                } else {
                    0
                };
                window_state.last_present_ns = now;
                window_state.last_frame_delta_ns = delta_ns;
                window_state.fps_instant = if delta_ns > 0 {
                    1_000_000_000.0 / delta_ns as f64
                } else {
                    0.0
                };
            }
        }

        iteration = iteration.saturating_add(1);
        if frame_report.no_progress_realms.is_empty() || iteration >= MAX_REALM_ITERATIONS {
            break;
        }
    }

    engine_state.universal_state.composition.frame_report = frame_report;
    if gpu_written {
        if let Some(gpu_profiler) = engine_state.gpu_profiler.as_mut() {
            if gpu_profiler.query_count() > 0 {
                let mut resolve_encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("GpuProfiler Resolve Encoder"),
                    });
                resolve_encoder.resolve_query_set(
                    gpu_profiler.query_set(),
                    0..gpu_profiler.query_count(),
                    gpu_profiler.resolve_buffer(),
                    0,
                );
                resolve_encoder.copy_buffer_to_buffer(
                    gpu_profiler.resolve_buffer(),
                    0,
                    gpu_profiler.readback_buffer(),
                    0,
                    gpu_profiler.buffer_size(),
                );
                queue.submit(Some(resolve_encoder.finish()));
                if let Some(report) = gpu_profiler.readback_report(&device) {
                    apply_gpu_timing_report(&mut engine_state.profiling, report);
                }
            }
        }
    }
    engine_state.profiling.render.shadow_ns = shadow_ns;
    engine_state.profiling.render.windows_ns = windows_ns;
    #[cfg(not(target_arch = "wasm32"))]
    {
        engine_state.profiling.render.total_ns = total_start.elapsed().as_nanos() as u64;
    }
    #[cfg(target_arch = "wasm32")]
    {
        engine_state.profiling.render.total_ns = now_ns().saturating_sub(total_start);
    }
}
