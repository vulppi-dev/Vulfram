use crate::core::render::RenderState;
use crate::core::render::cache::PipelineKey;
use crate::core::resources::geometry::Frustum;
use crate::core::resources::{CameraComponent, VertexStream};
use glam::Vec4Swizzles;

pub fn pass_shadow_3d_update(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
) {
    pass_shadow_update_impl(render_state, device, queue, encoder, frame_index);
}

pub fn pass_shadow_2d_update(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
    target_size: glam::UVec2,
) {
    crate::core::render::passes::two_d::pass_2d_shadow_masks_update(
        render_state,
        device,
        queue,
        encoder,
        frame_index,
        target_size,
    );
}

fn pass_shadow_update_impl(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    frame_index: u64,
) {
    let shadow_manager = match render_state.shadow_3d.as_mut() {
        Some(s) => s,
        None => return,
    };
    shadow_manager.set_realm_point_vp_base(0);

    // If the manager is dirty, it means something in the scene changed (light or model).
    // We must mark all currently cached pages as dirty so they get re-rendered if used.
    if shadow_manager.is_dirty {
        for record in shadow_manager.cache.values_mut() {
            record.is_dirty = true;
        }
    }

    let library = match render_state.library.as_ref() {
        Some(l) => l,
        None => return,
    };

    let bindings = match render_state.bindings.as_mut() {
        Some(b) => b,
        None => return,
    };

    let vertex_sys = match render_state.vertex.as_mut() {
        Some(v) => v,
        None => return,
    };

    let cache = &mut render_state.cache;

    // 1. Identify which pages need update for each light
    let primary_camera_inv = render_state
        .scene
        .cameras
        .values()
        .next()
        .map(|camera| camera.data.view_projection.inverse());
    if primary_camera_inv.is_none() {
        return;
    }
    let camera_inv_view_proj = primary_camera_inv.unwrap_or(glam::Mat4::IDENTITY);

    // Collect pages (re-render on dirty scenes)
    let mut pages_to_render = Vec::new();

    let mut light_ids: Vec<u32> = render_state.scene.lights.keys().copied().collect();
    light_ids.sort_unstable();

    let mut shadow_counter = 0u32;
    for light_id in light_ids {
        let light_record = match render_state.scene.lights.get(&light_id) {
            Some(record) => record,
            None => continue,
        };

        if !light_record.active || !light_record.cast_shadow {
            continue;
        }

        let shadow_light_id = shadow_counter;
        shadow_counter += 1;

        let mut light_views = Vec::new();
        let mut light_projs = Vec::new();

        match light_record.data.kind_flags.x {
            1 => {
                // Point Light (6 faces)
                let pos = light_record.data.position.xyz();
                let range = light_record.data.intensity_range.y;

                let near = 0.1; // Smaller near plane to match state.rs and avoid clipping
                let far = range;

                // Reverse Z: far and near are swapped in the projection call
                let projection =
                    glam::Mat4::perspective_rh(std::f32::consts::FRAC_PI_2, 1.0, far, near);

                // Cubemap face directions and up vectors (standard cubemap convention)
                let targets = [
                    (pos + glam::Vec3::X, glam::Vec3::NEG_Y),     // +X: 0
                    (pos + glam::Vec3::NEG_X, glam::Vec3::NEG_Y), // -X: 1
                    (pos + glam::Vec3::Y, glam::Vec3::Z),         // +Y: 2
                    (pos + glam::Vec3::NEG_Y, glam::Vec3::NEG_Z), // -Y: 3
                    (pos + glam::Vec3::Z, glam::Vec3::NEG_Y),     // +Z: 4
                    (pos + glam::Vec3::NEG_Z, glam::Vec3::NEG_Y), // -Z: 5
                ];

                for (target, up) in targets {
                    light_views.push(glam::Mat4::look_at_rh(pos, target, up));
                    light_projs.push(projection);
                }
            }
            _ => {
                // Directional or Spot
                light_views.push(light_record.data.view);
                light_projs.push(light_record.data.projection);
            }
        }

        for (face_index, (light_view, light_proj)) in light_views
            .into_iter()
            .zip(light_projs.into_iter())
            .enumerate()
        {
            let light_view_proj = light_proj * light_view;
            let vp_face = if light_record.data.kind_flags.x == 1 {
                face_index as u32
            } else {
                0
            };
            let vp_index = shadow_light_id * 6 + vp_face;
            let realm_tag = 0;
            shadow_manager
                .point_light_vp
                .write(vp_index, &light_view_proj);

            // For point lights (kind=1), always render all pages for all 6 faces
            // For other lights, identify pages based on camera frustum
            let required = if light_record.data.kind_flags.x == 1 {
                // Point light: render all virtual pages for this face
                let grid_size = shadow_manager.config.virtual_grid_size;
                let mut all_pages = Vec::new();
                for y in 0..grid_size {
                    for x in 0..grid_size {
                        all_pages.push((x, y));
                    }
                }
                all_pages
            } else {
                // Directional/Spot: only render visible pages
                shadow_manager.identify_required_pages(light_view_proj, camera_inv_view_proj)
            };

            for (x, y) in required {
                if let Some(handle) = shadow_manager.request_page(
                    realm_tag,
                    shadow_light_id,
                    face_index as u32,
                    x,
                    y,
                    frame_index,
                ) {
                    let key = crate::core::resources::shadow::ShadowPageKey {
                        realm_tag,
                        light_id: shadow_light_id,
                        face: face_index as u32,
                        x,
                        y,
                    };
                    if let Some(record) = shadow_manager.cache.get_mut(&key) {
                        if record.is_dirty {
                            pages_to_render.push((
                                key,
                                handle,
                                light_view,
                                light_proj,
                                light_record.data.kind_flags.x == 1,
                            ));
                            record.is_dirty = false;
                        }
                    }
                }
            }
        }
    }

    shadow_manager.prune_inactive_dense_lights(0, shadow_counter);

    if pages_to_render.is_empty() {
        return;
    }

    struct PageRender {
        layer: u32,
        shadow_cam_id: u32,
        view_projection: glam::Mat4,
        transform: (f32, f32, f32, f32, u32),
        is_point: bool,
    }

    let info = shadow_manager.atlas.info();
    let mut render_pages = Vec::new();

    for (key, handle, light_view, light_proj, is_point) in pages_to_render {
        let page_vp = shadow_manager.get_page_view_projection(light_view, light_proj, key.x, key.y);

        let shadow_camera_data = CameraComponent {
            view_projection: page_vp,
            ..Default::default()
        };

        // Use a very low ID for shadow cameras to fit in the initial pool capacity (128)
        let shadow_cam_id = 10 + key.light_id * 6 + key.face;
        bindings
            .shadow_camera_pool
            .write(shadow_cam_id, &shadow_camera_data);

        let transform = match shadow_manager.atlas.get_uv_transform(handle) {
            Some(transform) => transform,
            None => continue,
        };

        render_pages.push(PageRender {
            layer: transform.4,
            shadow_cam_id,
            view_projection: page_vp,
            transform,
            is_point,
        });
    }

    render_pages.sort_by_key(|page| page.layer);

    let mut i = 0;
    let collector = &mut render_state.collector;
    collector.shadow_instance_data.clear();
    let mut shadow_instance_cursor = 0u32;

    while i < render_pages.len() {
        let layer = render_pages[i].layer;
        let atlas_layer_view = shadow_manager
            .atlas
            .layer_view(layer)
            .expect("Atlas layer view missing");

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(&format!("Shadow Page Update Pass - Layer {}", layer)),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &atlas_layer_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        vertex_sys.begin_pass();

        let shadow_pipeline_key = PipelineKey {
            shader_id: crate::core::render::cache::ShaderId::Shadow as u64,
            color_format: wgpu::TextureFormat::Rgba8UnormSrgb, // Dummy, not used
            color_target_count: 1,
            depth_format: Some(wgpu::TextureFormat::Depth32Float),
            sample_count: 1,
            topology: wgpu::PrimitiveTopology::TriangleList,
            polygon_mode: wgpu::PolygonMode::Fill,
            cull_mode: Some(wgpu::Face::Back),
            front_face: wgpu::FrontFace::Ccw,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Greater, // Reverse Z
            blend: None,
        };

        let shadow_pipeline = cache.get_or_create(shadow_pipeline_key, frame_index, || {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Shadow Pipeline"),
                layout: Some(&library.shadow_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &library.shadow_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[
                        // 0: Position
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::Position.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 0,
                            }],
                        },
                        // 1: Normal
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::Normal.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 1,
                            }],
                        },
                        // 2: Tangent
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::Tangent.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 2,
                            }],
                        },
                        // 3: Color0
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::Color0.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 3,
                            }],
                        },
                        // 4: UV0
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::UV0.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 4,
                            }],
                        },
                        // 5: UV1
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::UV1.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 5,
                            }],
                        },
                        // 6: Joints
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::Joints.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Uint16x4,
                                offset: 0,
                                shader_location: 6,
                            }],
                        },
                        // 7: Weights
                        wgpu::VertexBufferLayout {
                            array_stride: VertexStream::Weights.stride_bytes(),
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 7,
                            }],
                        },
                    ],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: None, // Depth only
                primitive: wgpu::PrimitiveState {
                    topology: shadow_pipeline_key.topology,
                    front_face: shadow_pipeline_key.front_face,
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Greater, // Reverse Z
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState {
                        constant: -2, // Reverse Z: negative bias to push occluder farther (closer to 0.0)
                        slope_scale: -2.0,
                        clamp: 0.0,
                    },
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        });

        rpass.set_pipeline(shadow_pipeline);

        while i < render_pages.len() && render_pages[i].layer == layer {
            let page = &render_pages[i];
            let atlas_w = (info.tiles_w * info.pitch_px) as f32;
            let atlas_h = (info.tiles_h * info.pitch_px) as f32;

            // transform: (scale_x, scale_y, bias_x, bias_y, layer)
            let (sx, sy, bx, by, _) = page.transform;

            let vx = bx * atlas_w;
            let vy = by * atlas_h;
            let vw = sx * atlas_w;
            let vh = sy * atlas_h;

            rpass.set_viewport(vx, vy, vw, vh, 0.0, 1.0);
            rpass.set_scissor_rect(vx as u32, vy as u32, vw as u32, vh as u32);

            if let Some(shared_group) = bindings.shadow_shared_group.as_ref() {
                let camera_offset =
                    bindings.shadow_camera_pool.get_offset(page.shadow_cam_id) as u32;
                rpass.set_bind_group(0, shared_group, &[camera_offset, 0]);
            }

            let frustum = Frustum::from_view_projection(page.view_projection);

            if let Some(model_bind_group) = bindings.shadow_model_bind_group.as_ref() {
                rpass.set_bind_group(1, model_bind_group, &[]);
            }

            for (_model_id, model_record) in &render_state.scene.models {
                if !model_record.active || !model_record.cast_shadow {
                    continue;
                }

                if !page.is_point
                    && let Some(aabb) = vertex_sys.aabb(model_record.geometry_id)
                {
                    let world_aabb = aabb.transform(&model_record.data.transform);
                    if !frustum.intersects_aabb(world_aabb.min, world_aabb.max) {
                        continue;
                    }
                }

                if let Ok(Some(index_info)) = vertex_sys.index_info(model_record.geometry_id) {
                    if vertex_sys
                        .bind(&mut rpass, model_record.geometry_id)
                        .is_ok()
                    {
                        let inst_idx = shadow_instance_cursor;
                        collector.shadow_instance_data.push(model_record.data);
                        shadow_instance_cursor += 1;

                        rpass.draw_indexed(0..index_info.count, 0, inst_idx..(inst_idx + 1));
                    }
                }
            }

            i += 1;
        }
    }

    if !collector.shadow_instance_data.is_empty() {
        bindings
            .shadow_instance_pool
            .write_slice(0, &collector.shadow_instance_data);
    }

    shadow_manager.clear_dirty();
}
