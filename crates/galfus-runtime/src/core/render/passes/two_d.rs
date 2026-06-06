use crate::core::render::RenderState;
use crate::core::render::cache::PipelineKey;
use crate::core::render::state::{
    TwoDBatchKey, TwoDBatchRange, TwoDItemKind, TwoDOccluderEdge, TwoDOccluderSilhouette,
    TwoDOccluderSourceKind, TwoDPreparedCamera, TwoDPreparedItem, TwoDPreparedOccluder,
};
use std::hash::{Hash, Hasher};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TwoDCameraRaw {
    view_projection: glam::Mat4,
    model_matrix: glam::Mat4,
    tint: glam::Vec4,
    model_position: glam::Vec4,
    light_offset_count: glam::UVec4,
    shadow_params: glam::Vec4,
    shadow_controls: glam::Vec4,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TwoDFrameSemanticMeta {
    resolution: glam::Vec2,
    inv_resolution: glam::Vec2,
    frame_index: u32,
    flags: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TwoDLightRaw {
    position: glam::Vec4,
    color: glam::Vec4,
    intensity_range: glam::Vec2,
    light_radius: f32,
    _padding0: f32,
    kind_flags: glam::UVec2,
    shadow_layer_mask: u32,
    shadow_index: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Shadow2dSegmentRaw {
    a: glam::Vec2,
    b: glam::Vec2,
}

#[derive(Clone)]
struct TwoDOccluderRaw {
    silhouette: TwoDOccluderSilhouette,
    shadow_layer_mask: u32,
    shadow_height: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Shadow2dSampleRaw {
    blocker_distance: f32,
    blocker_left: f32,
    blocker_right: f32,
    flags: f32,
    penumbra_left: f32,
    penumbra_right: f32,
    support_left_distance: f32,
    support_right_distance: f32,
    occluder_v0: glam::Vec2,
    occluder_v1: glam::Vec2,
    occluder_v2: glam::Vec2,
    occluder_v3: glam::Vec2,
}

const TWO_D_MAX_LIGHTS_PER_CAMERA: usize = 64;
const TWO_D_MAX_OCCLUDERS_PER_CAMERA: usize = 256;
const TWO_D_SHADOW_MASK_SIZE: u32 = 256;
const TWO_D_SHADOW_SAMPLES_PER_DIRECTION: usize = 4;
const TWO_D_TAU: f32 = std::f32::consts::TAU;

#[derive(Debug, Clone, Copy)]
struct TwoDOccluderAngularVertex {
    point: glam::Vec2,
    angle: f32,
    distance: f32,
}

#[derive(Debug, Clone, Copy)]
struct TwoDOccluderLightCone {
    support_points: [glam::Vec2; 2],
    blocker_interval: glam::Vec2,
    penumbra_interval: glam::Vec2,
}

#[derive(Clone, Copy)]
struct TwoDOccluderConeCandidate<'a> {
    occluder: &'a TwoDOccluderRaw,
    cone: TwoDOccluderLightCone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TwoDDrawBatch {
    key: TwoDBatchKey,
    start: u32,
    count: u32,
}

fn material_allows_2d(record: &crate::core::resources::ShaderMaterialRecord) -> bool {
    matches!(
        record.realm_kind,
        crate::core::resources::MaterialRealmKind::TwoD
    )
}

fn material_uses_compiled_2d_shader(record: &crate::core::resources::ShaderMaterialRecord) -> bool {
    material_allows_2d(record)
        && record.compile_error.is_none()
        && record.compiled_shader_source.is_some()
}

fn resolve_2d_draw_batches<FMat, FGeom>(
    ranges: &[TwoDBatchRange],
    mut material_exists: FMat,
    mut geometry_exists: FGeom,
) -> Vec<TwoDDrawBatch>
where
    FMat: FnMut(u32) -> bool,
    FGeom: FnMut(u32) -> bool,
{
    let mut batches = Vec::with_capacity(ranges.len());
    for range in ranges {
        if range.count == 0 {
            continue;
        }
        if !material_exists(range.key.material_id) {
            continue;
        }
        if !geometry_exists(range.key.geometry_id) {
            continue;
        }
        batches.push(TwoDDrawBatch {
            key: range.key,
            start: range.start,
            count: range.count,
        });
    }
    batches
}

fn material_tint_for_batch(
    scene: &crate::core::render::state::RenderScene,
    material_id: u32,
) -> glam::Vec4 {
    let Some(material) = scene.materials.get(&material_id) else {
        return glam::Vec4::ONE;
    };
    if let Some(input_tint) = material.inputs.first().copied()
        && input_tint.w > 0.0
    {
        return input_tint;
    }
    glam::Vec4::ONE
}

fn align_up(value: u64, alignment: u64) -> u64 {
    if alignment <= 1 {
        return value;
    }
    value.div_ceil(alignment) * alignment
}

fn collect_visible_2d_lights(
    render_state: &RenderState,
    camera: &crate::core::render::state::TwoDPreparedCamera,
    shadow_config: crate::core::resources::Realm2dShadowConfig,
) -> Vec<TwoDLightRaw> {
    let mut visible_lights = Vec::with_capacity(TWO_D_MAX_LIGHTS_PER_CAMERA);
    let camera_position = camera.transform.w_axis.truncate();
    let mut light_ids: Vec<u32> = render_state.two_d_source.lights.keys().copied().collect();
    light_ids.sort_unstable();
    for light_id in light_ids {
        let Some(light) = render_state.two_d_source.lights.get(&light_id) else {
            continue;
        };
        if !light.active || (light.layer_mask & camera.layer_mask) == 0 {
            continue;
        }
        let light_kind = light.data.kind_flags.x;
        if light_kind == crate::core::resources::LightKind::Point.to_u32()
            || light_kind == crate::core::resources::LightKind::Spot.to_u32()
        {
            let range = light.data.intensity_range.y.max(0.0001);
            let delta = light.data.position.truncate() - camera_position;
            if delta.length_squared() > (range * range * 4.0) {
                continue;
            }
        }
        visible_lights.push(TwoDLightRaw {
            position: light.data.position,
            color: light.data.color,
            intensity_range: light.data.intensity_range,
            light_radius: shadow_config.light_radius,
            _padding0: 0.0,
            kind_flags: light.data.kind_flags,
            shadow_layer_mask: light.shadow_layer_mask,
            shadow_index: 0,
        });
        if visible_lights.len() >= TWO_D_MAX_LIGHTS_PER_CAMERA {
            break;
        }
    }
    visible_lights
}

fn collect_shadow_occluders(
    render_state: &RenderState,
    camera: &crate::core::render::state::TwoDPreparedCamera,
) -> Vec<TwoDOccluderRaw> {
    let mut occluders = Vec::with_capacity(TWO_D_MAX_OCCLUDERS_PER_CAMERA);
    for occluder in &render_state.two_d_prepared.occluders {
        if occluder.shadow_height <= 0.0 {
            continue;
        }
        if !layer_visible_in_camera(occluder.layer, camera.layer_mask) {
            continue;
        }
        occluders.push(TwoDOccluderRaw {
            silhouette: occluder.silhouette.clone(),
            shadow_layer_mask: occluder.shadow_layer_mask,
            shadow_height: occluder.shadow_height,
        });
        if occluders.len() >= TWO_D_MAX_OCCLUDERS_PER_CAMERA {
            break;
        }
    }
    occluders
}

fn write_shadow_samples_for_light(
    resources: &mut crate::core::render::state::TwoDPassResources,
    queue: &wgpu::Queue,
    angular_resolution: u32,
    layer: u32,
    light: &TwoDLightRaw,
    occluders: &[&TwoDOccluderRaw],
) {
    let samples = rasterize_shadow_samples_for_light(angular_resolution, light, occluders);
    let layer_offset = layer as u64
        * angular_resolution.max(1) as u64
        * TWO_D_SHADOW_SAMPLES_PER_DIRECTION as u64
        * std::mem::size_of::<Shadow2dSampleRaw>() as u64;
    queue.write_buffer(
        &resources.shadow_sample_buffer,
        layer_offset,
        bytemuck::cast_slice(&samples),
    );
}

fn build_quad_occluder_silhouette(transform: glam::Mat4) -> Option<TwoDOccluderSilhouette> {
    let origin = glam::Vec2::new(transform.w_axis.x, transform.w_axis.y);
    let axis_x_world = glam::Vec2::new(transform.x_axis.x, transform.x_axis.y);
    let axis_y_world = glam::Vec2::new(transform.y_axis.x, transform.y_axis.y);
    let axis_x_len = axis_x_world.length();
    let axis_y_len = axis_y_world.length();
    if axis_x_len <= 1e-5 || axis_y_len <= 1e-5 {
        return None;
    }
    let axis_x = axis_x_world / axis_x_len;
    let axis_y = axis_y_world / axis_y_len;
    let half_x = axis_x * (axis_x_len * 0.5);
    let half_y = axis_y * (axis_y_len * 0.5);
    let p0 = origin - half_x - half_y;
    let p1 = origin + half_x - half_y;
    let p2 = origin + half_x + half_y;
    let p3 = origin - half_x + half_y;
    Some(TwoDOccluderSilhouette {
        vertices: [p0, p1, p2, p3],
        edges: [
            TwoDOccluderEdge { a: p0, b: p1 },
            TwoDOccluderEdge { a: p1, b: p2 },
            TwoDOccluderEdge { a: p2, b: p3 },
            TwoDOccluderEdge { a: p3, b: p0 },
        ],
    })
}

fn unwrap_angle_near(angle: f32, reference: f32) -> f32 {
    let mut unwrapped = angle;
    while unwrapped - reference > std::f32::consts::PI {
        unwrapped -= TWO_D_TAU;
    }
    while unwrapped - reference < -std::f32::consts::PI {
        unwrapped += TWO_D_TAU;
    }
    unwrapped
}

fn build_occluder_light_cone(
    silhouette: &TwoDOccluderSilhouette,
    light_pos: glam::Vec2,
    light_radius: f32,
) -> Option<TwoDOccluderLightCone> {
    let centroid = (silhouette.vertices[0]
        + silhouette.vertices[1]
        + silhouette.vertices[2]
        + silhouette.vertices[3])
        * 0.25;
    let centroid_delta = centroid - light_pos;
    if centroid_delta.length_squared() <= 1e-8 {
        return None;
    }
    let reference_angle = centroid_delta.y.atan2(centroid_delta.x);
    let mut angular_vertices = [TwoDOccluderAngularVertex {
        point: glam::Vec2::ZERO,
        angle: 0.0,
        distance: 0.0,
    }; 4];
    for (vertex, point) in angular_vertices
        .iter_mut()
        .zip(silhouette.vertices.iter().copied())
    {
        let delta = point - light_pos;
        let distance = delta.length();
        if distance <= 1e-5 {
            return None;
        }
        *vertex = TwoDOccluderAngularVertex {
            point,
            angle: unwrap_angle_near(delta.y.atan2(delta.x), reference_angle),
            distance,
        };
    }
    angular_vertices.sort_by(|a, b| a.angle.total_cmp(&b.angle));
    let left = angular_vertices[0];
    let right = angular_vertices[angular_vertices.len() - 1];
    let blocker_span = right.angle - left.angle;
    if blocker_span <= 1e-5 || blocker_span >= std::f32::consts::PI - 1e-4 {
        return None;
    }

    let source_radius = light_radius.max(0.0);
    if left.distance <= source_radius + 1e-5 || right.distance <= source_radius + 1e-5 {
        return None;
    }
    let left_offset = (source_radius / left.distance).clamp(0.0, 0.9999).asin();
    let right_offset = (source_radius / right.distance).clamp(0.0, 0.9999).asin();
    let penumbra_interval = glam::Vec2::new(left.angle - left_offset, right.angle + right_offset);

    Some(TwoDOccluderLightCone {
        support_points: [left.point, right.point],
        blocker_interval: glam::Vec2::new(left.angle, right.angle),
        penumbra_interval,
    })
}

fn cross2(a: glam::Vec2, b: glam::Vec2) -> f32 {
    a.x * b.y - a.y * b.x
}

fn ray_segment_hit_distance(
    ray_origin: glam::Vec2,
    ray_dir: glam::Vec2,
    segment: Shadow2dSegmentRaw,
) -> f32 {
    let v1 = ray_origin - segment.a;
    let v2 = segment.b - segment.a;
    let den = cross2(ray_dir, v2);
    if den.abs() <= 1e-6 {
        return -1.0;
    }
    let t = cross2(v2, v1) / den;
    let q = cross2(ray_dir, v1) / den;
    if t >= 0.0 && (0.0..=1.0).contains(&q) {
        return t;
    }
    -1.0
}

fn angle_in_interval(angle: f32, interval: glam::Vec2) -> bool {
    angle >= interval.x && angle <= interval.y
}

fn empty_shadow_sample() -> Shadow2dSampleRaw {
    Shadow2dSampleRaw {
        blocker_distance: 1.0,
        blocker_left: 0.0,
        blocker_right: 0.0,
        flags: 0.0,
        penumbra_left: 0.0,
        penumbra_right: 0.0,
        support_left_distance: 1.0,
        support_right_distance: 1.0,
        occluder_v0: glam::Vec2::ZERO,
        occluder_v1: glam::Vec2::ZERO,
        occluder_v2: glam::Vec2::ZERO,
        occluder_v3: glam::Vec2::ZERO,
    }
}

fn ray_hit_distance_for_occluder(
    light_pos: glam::Vec2,
    ray_dir: glam::Vec2,
    occluder: &TwoDOccluderRaw,
) -> Option<f32> {
    let mut min_t = f32::INFINITY;
    for edge in &occluder.silhouette.edges {
        let t = ray_segment_hit_distance(
            light_pos,
            ray_dir,
            Shadow2dSegmentRaw {
                a: edge.a,
                b: edge.b,
            },
        );
        if t >= 0.0 && t < min_t {
            min_t = t;
        }
    }
    (min_t.is_finite()).then_some(min_t)
}

fn rasterize_shadow_samples_for_light(
    angular_resolution: u32,
    light: &TwoDLightRaw,
    occluders: &[&TwoDOccluderRaw],
) -> Vec<Shadow2dSampleRaw> {
    let resolution = angular_resolution.max(1) as usize;
    let mut samples = vec![empty_shadow_sample(); resolution * TWO_D_SHADOW_SAMPLES_PER_DIRECTION];
    let light_pos = glam::Vec2::new(light.position.x, light.position.y);
    let light_range = light.intensity_range.y.max(0.0001);
    let light_radius = light.light_radius.max(0.0);
    let mut cones = Vec::with_capacity(occluders.len());
    for occluder in occluders {
        if let Some(cone) = build_occluder_light_cone(&occluder.silhouette, light_pos, light_radius)
        {
            cones.push(TwoDOccluderConeCandidate { occluder, cone });
        }
    }
    for x in 0..resolution {
        let u = (x as f32 + 0.5) / angular_resolution.max(1) as f32;
        let angle = u * TWO_D_TAU;
        let reference = angle;
        let ray_dir = glam::Vec2::new(angle.cos(), angle.sin());
        let mut candidates: Vec<(f32, f32, TwoDOccluderLightCone, &TwoDOccluderRaw)> =
            Vec::with_capacity(TWO_D_SHADOW_SAMPLES_PER_DIRECTION);
        for candidate in &cones {
            let cone = candidate.cone;
            let blocker_interval = glam::Vec2::new(
                unwrap_angle_near(cone.blocker_interval.x, reference),
                unwrap_angle_near(cone.blocker_interval.y, reference),
            );
            if !angle_in_interval(reference, blocker_interval) {
                continue;
            }
            let Some(ray_hit_depth) =
                ray_hit_distance_for_occluder(light_pos, ray_dir, candidate.occluder)
            else {
                continue;
            };
            let precedence_depth = ray_hit_depth + candidate.occluder.shadow_height * 1e-4;
            candidates.push((precedence_depth, ray_hit_depth, cone, candidate.occluder));
        }
        candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (slot, (_, blocker_depth, cone, occluder)) in candidates
            .into_iter()
            .take(TWO_D_SHADOW_SAMPLES_PER_DIRECTION)
            .enumerate()
        {
            let sample = &mut samples[x * TWO_D_SHADOW_SAMPLES_PER_DIRECTION + slot];
            sample.flags = 1.0;
            sample.blocker_distance = (blocker_depth / light_range).clamp(0.0, 1.0);
            sample.blocker_left = unwrap_angle_near(cone.blocker_interval.x, reference);
            sample.blocker_right = unwrap_angle_near(cone.blocker_interval.y, reference);
            sample.penumbra_left = unwrap_angle_near(cone.penumbra_interval.x, reference);
            sample.penumbra_right = unwrap_angle_near(cone.penumbra_interval.y, reference);
            sample.support_left_distance =
                (cone.support_points[0].distance(light_pos) / light_range).clamp(0.0, 1.0);
            sample.support_right_distance =
                (cone.support_points[1].distance(light_pos) / light_range).clamp(0.0, 1.0);
            sample.occluder_v0 = occluder.silhouette.vertices[0];
            sample.occluder_v1 = occluder.silhouette.vertices[1];
            sample.occluder_v2 = occluder.silhouette.vertices[2];
            sample.occluder_v3 = occluder.silhouette.vertices[3];
        }
    }
    samples
}

fn hash_2d_shadow_scene(
    render_state: &RenderState,
    cameras: &[crate::core::render::state::TwoDPreparedCamera],
    target_size: glam::UVec2,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    target_size.x.hash(&mut hasher);
    target_size.y.hash(&mut hasher);
    let shadow_cfg = render_state.two_d_source.shadow_config;
    shadow_cfg.shadow_contact_offset.to_bits().hash(&mut hasher);
    shadow_cfg.shadow_debug_light_index.hash(&mut hasher);
    shadow_cfg.shadow_debug_mode.hash(&mut hasher);
    shadow_cfg.ambient.to_bits().hash(&mut hasher);
    shadow_cfg.light_radius.to_bits().hash(&mut hasher);
    shadow_cfg.max_shadow_updates_per_frame.hash(&mut hasher);
    shadow_cfg.angular_resolution.hash(&mut hasher);
    shadow_cfg.map_resolution.hash(&mut hasher);
    cameras.len().hash(&mut hasher);

    for camera in cameras {
        camera.camera_id.hash(&mut hasher);
        bytemuck::bytes_of(&camera.transform).hash(&mut hasher);
        bytemuck::bytes_of(&camera.near_far).hash(&mut hasher);
        camera.ortho_scale.to_bits().hash(&mut hasher);
        camera.layer_mask.hash(&mut hasher);
        camera.order.hash(&mut hasher);

        let visible_lights = collect_visible_2d_lights(render_state, camera, shadow_cfg);
        visible_lights.len().hash(&mut hasher);
        for light in &visible_lights {
            bytemuck::bytes_of(light).hash(&mut hasher);
        }

        let occluders = collect_shadow_occluders(render_state, camera);
        occluders.len().hash(&mut hasher);
        for occluder in &occluders {
            occluder.shadow_layer_mask.hash(&mut hasher);
            occluder.shadow_height.to_bits().hash(&mut hasher);
            for segment in &occluder.silhouette.edges {
                segment.a.x.to_bits().hash(&mut hasher);
                segment.a.y.to_bits().hash(&mut hasher);
                segment.b.x.to_bits().hash(&mut hasher);
                segment.b.y.to_bits().hash(&mut hasher);
            }
        }
    }

    hasher.finish()
}

fn hash_2d_shadow_light_layer(
    camera_vp: glam::Mat4,
    light: &TwoDLightRaw,
    occluders: &[TwoDOccluderRaw],
    shadow_mask_size: glam::UVec2,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytemuck::bytes_of(&camera_vp).hash(&mut hasher);
    bytemuck::bytes_of(light).hash(&mut hasher);
    shadow_mask_size.x.hash(&mut hasher);
    shadow_mask_size.y.hash(&mut hasher);
    occluders.len().hash(&mut hasher);
    for occluder in occluders {
        occluder.shadow_layer_mask.hash(&mut hasher);
        occluder.shadow_height.to_bits().hash(&mut hasher);
        for segment in &occluder.silhouette.edges {
            segment.a.x.to_bits().hash(&mut hasher);
            segment.a.y.to_bits().hash(&mut hasher);
            segment.b.x.to_bits().hash(&mut hasher);
            segment.b.y.to_bits().hash(&mut hasher);
        }
    }
    hasher.finish()
}

fn ensure_two_d_pass_resources(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    required_slots: usize,
) {
    let desired_shadow_resolution = render_state
        .shadow_2d
        .as_ref()
        .map(|manager| manager.config.angular_resolution.clamp(128, 4096))
        .unwrap_or(TWO_D_SHADOW_MASK_SIZE);
    let min_alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
    let stride = align_up(std::mem::size_of::<TwoDCameraRaw>() as u64, min_alignment);
    let initial_slots = required_slots.max(1);
    let initial_light_slots = TWO_D_MAX_LIGHTS_PER_CAMERA.max(1);
    let resources = render_state.two_d_pass_resources.get_or_insert_with(|| {
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("2D Global BGL"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                                TwoDCameraRaw,
                            >(
                            )
                                as u64),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
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
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let library = render_state.library.as_ref().expect("library must exist");
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("2D Material Pipeline Layout"),
            bind_group_layouts: &[
                &global_bind_group_layout,
                &library.layout_object_3d_material,
                &library.layout_frame_semantics,
            ],
            ..Default::default()
        });
        let camera_dynamic_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Camera Dynamic Buffer"),
            size: stride * initial_slots as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let light_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Light Storage Buffer"),
            size: (std::mem::size_of::<TwoDLightRaw>() * initial_light_slots) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shadow_sample_capacity = desired_shadow_resolution as usize
            * TWO_D_MAX_LIGHTS_PER_CAMERA
            * TWO_D_SHADOW_SAMPLES_PER_DIRECTION;
        let shadow_sample_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Shadow Sample Buffer"),
            size: (std::mem::size_of::<Shadow2dSampleRaw>() * shadow_sample_capacity) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let fallback_depth = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("2D Fallback Depth Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let fallback_depth_view =
            fallback_depth.create_view(&wgpu::TextureViewDescriptor::default());
        let global_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("2D Global BG"),
                layout: &global_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &camera_dynamic_buffer,
                            offset: 0,
                            size: std::num::NonZeroU64::new(
                                std::mem::size_of::<TwoDCameraRaw>() as u64
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.point_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.point_repeat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_repeat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: light_storage_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: shadow_sample_buffer.as_entire_binding(),
                    },
                ],
            });
        crate::core::render::state::TwoDPassResources {
            global_bind_group_layout,
            pipeline_layout,
            camera_dynamic_buffer,
            light_storage_buffer,
            shadow_sample_buffer,
            global_bind_group,
            camera_dynamic_stride: stride,
            camera_dynamic_capacity_slots: initial_slots,
            light_capacity_slots: initial_light_slots,
            fallback_depth_view,
            shadow_mask_size: glam::UVec2::new(desired_shadow_resolution, 1),
            shadow_sample_capacity,
        }
    });
    if resources.shadow_mask_size.x != desired_shadow_resolution
        || resources.shadow_mask_size.y != 1
    {
        let shadow_sample_capacity = desired_shadow_resolution as usize
            * TWO_D_MAX_LIGHTS_PER_CAMERA
            * TWO_D_SHADOW_SAMPLES_PER_DIRECTION;
        let shadow_sample_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Shadow Sample Buffer"),
            size: (std::mem::size_of::<Shadow2dSampleRaw>() * shadow_sample_capacity) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let library = render_state.library.as_ref().expect("library must exist");
        let new_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("2D Global BG"),
                layout: &resources.global_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &resources.camera_dynamic_buffer,
                            offset: 0,
                            size: std::num::NonZeroU64::new(
                                std::mem::size_of::<TwoDCameraRaw>() as u64
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.point_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.point_repeat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_repeat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: resources.light_storage_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: shadow_sample_buffer.as_entire_binding(),
                    },
                ],
            });
        resources.shadow_sample_buffer = shadow_sample_buffer;
        resources.shadow_mask_size = glam::UVec2::new(desired_shadow_resolution, 1);
        resources.shadow_sample_capacity = shadow_sample_capacity;
        resources.global_bind_group = new_bind_group;
    }
    if resources.camera_dynamic_capacity_slots < required_slots {
        let mut new_camera_slots = resources.camera_dynamic_capacity_slots.max(1);
        while new_camera_slots < required_slots {
            new_camera_slots *= 2;
        }
        let new_camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Camera Dynamic Buffer"),
            size: resources.camera_dynamic_stride * new_camera_slots as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let new_light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("2D Light Storage Buffer"),
            size: (std::mem::size_of::<TwoDLightRaw>() * resources.light_capacity_slots.max(1))
                as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let library = render_state.library.as_ref().expect("library must exist");
        let new_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("2D Global BG"),
                layout: &resources.global_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &new_camera_buffer,
                            offset: 0,
                            size: std::num::NonZeroU64::new(
                                std::mem::size_of::<TwoDCameraRaw>() as u64
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.point_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.point_repeat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_repeat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: new_light_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: resources.shadow_sample_buffer.as_entire_binding(),
                    },
                ],
            });
        resources.camera_dynamic_buffer = new_camera_buffer;
        resources.light_storage_buffer = new_light_buffer;
        resources.global_bind_group = new_bind_group;
        resources.camera_dynamic_capacity_slots = new_camera_slots;
    }
}

pub fn pass_2d_prepare(render_state: &mut RenderState) {
    let prepared = &mut render_state.two_d_prepared;
    prepared.cameras.clear();
    prepared.items.clear();
    prepared.occluders.clear();

    prepared.cameras.extend(
        render_state
            .two_d_source
            .cameras
            .iter()
            .map(|(camera_id, record)| TwoDPreparedCamera {
                camera_id: *camera_id,
                transform: record.transform,
                near_far: record.near_far,
                ortho_scale: record.ortho_scale,
                layer_mask: record.layer_mask,
                order: record.order,
            }),
    );
    prepared.items.extend(
        render_state
            .two_d_source
            .sprites
            .iter()
            .map(|(item_id, record)| TwoDPreparedItem {
                item_id: *item_id,
                kind: TwoDItemKind::Sprite,
                transform: record.transform,
                geometry_id: record.geometry_id,
                material_id: record.material_id,
                layer: record.layer,
                cast_shadow: record.cast_shadow,
                receive_shadow: record.receive_shadow,
                occluder_only: record.occluder_only,
                shadow_height: record.shadow_height,
                shadow_layer_mask: record.shadow_layer_mask,
            }),
    );
    prepared.occluders.extend(
        render_state
            .two_d_source
            .sprites
            .iter()
            .filter(|(_, record)| record.cast_shadow && record.shadow_height > 0.0)
            .filter_map(|(occluder_id, record)| {
                let silhouette = build_quad_occluder_silhouette(record.transform)?;
                Some(TwoDPreparedOccluder {
                    occluder_id: *occluder_id,
                    source_kind: TwoDOccluderSourceKind::Sprite,
                    transform: record.transform,
                    silhouette,
                    layer: record.layer,
                    shadow_height: record.shadow_height,
                    shadow_layer_mask: record.shadow_layer_mask,
                })
            }),
    );
    prepared.items.extend(
        render_state
            .two_d_source
            .shapes
            .iter()
            .map(|(item_id, record)| TwoDPreparedItem {
                item_id: *item_id,
                kind: TwoDItemKind::Shape,
                transform: record.transform,
                geometry_id: record.geometry_id,
                material_id: record.material_id,
                layer: record.layer,
                cast_shadow: record.cast_shadow,
                receive_shadow: record.receive_shadow,
                occluder_only: record.occluder_only,
                shadow_height: record.shadow_height,
                shadow_layer_mask: record.shadow_layer_mask,
            }),
    );
    prepared.occluders.extend(
        render_state
            .two_d_source
            .shapes
            .iter()
            .filter(|(_, record)| record.cast_shadow && record.shadow_height > 0.0)
            .filter_map(|(occluder_id, record)| {
                let silhouette = build_quad_occluder_silhouette(record.transform)?;
                Some(TwoDPreparedOccluder {
                    occluder_id: *occluder_id,
                    source_kind: TwoDOccluderSourceKind::Shape,
                    transform: record.transform,
                    silhouette,
                    layer: record.layer,
                    shadow_height: record.shadow_height,
                    shadow_layer_mask: record.shadow_layer_mask,
                })
            }),
    );

    prepared
        .cameras
        .sort_unstable_by_key(|camera| (camera.order, camera.camera_id));
    prepared.items.sort_unstable_by_key(|item| {
        (
            item.layer,
            match item.kind {
                TwoDItemKind::Sprite => 0_u8,
                TwoDItemKind::Shape => 1_u8,
            },
            item.item_id,
        )
    });
    prepared.occluders.sort_unstable_by_key(|occluder| {
        (
            occluder.layer,
            match occluder.source_kind {
                TwoDOccluderSourceKind::Sprite => 0_u8,
                TwoDOccluderSourceKind::Shape => 1_u8,
            },
            occluder.occluder_id,
        )
    });
}

pub fn pass_2d_batch(render_state: &mut RenderState) {
    let batched = &mut render_state.two_d_batched;
    batched.items.clear();
    batched.ranges.clear();

    batched
        .items
        .extend(render_state.two_d_prepared.items.iter().cloned());
    batched.items.sort_unstable_by_key(|item| {
        (
            TwoDBatchKey {
                layer: item.layer,
                material_id: item
                    .material_id
                    .unwrap_or(crate::core::resources::MATERIAL_FALLBACK_ID),
                geometry_id: item.geometry_id,
                kind: item.kind,
            },
            item.item_id,
        )
    });

    let mut i = 0usize;
    while i < batched.items.len() {
        let first = &batched.items[i];
        let key = TwoDBatchKey {
            layer: first.layer,
            material_id: first
                .material_id
                .unwrap_or(crate::core::resources::MATERIAL_FALLBACK_ID),
            geometry_id: first.geometry_id,
            kind: first.kind,
        };
        let start = i;
        i += 1;
        while i < batched.items.len() {
            let item = &batched.items[i];
            let next_key = TwoDBatchKey {
                layer: item.layer,
                material_id: item
                    .material_id
                    .unwrap_or(crate::core::resources::MATERIAL_FALLBACK_ID),
                geometry_id: item.geometry_id,
                kind: item.kind,
            };
            if next_key != key {
                break;
            }
            i += 1;
        }
        batched.ranges.push(TwoDBatchRange {
            key,
            start: start as u32,
            count: (i - start) as u32,
        });
    }
}

pub fn pass_2d_draw(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    target_view: &wgpu::TextureView,
    target_format: wgpu::TextureFormat,
    target_size: glam::UVec2,
    frame_index: u64,
) {
    let cameras = if render_state.two_d_prepared.cameras.is_empty() {
        vec![crate::core::render::state::TwoDPreparedCamera {
            camera_id: 0,
            transform: glam::Mat4::IDENTITY,
            near_far: glam::Vec2::new(0.0, 1.0),
            ortho_scale: 1.0,
            layer_mask: u32::MAX,
            order: 0,
        }]
    } else {
        render_state.two_d_prepared.cameras.clone()
    };
    let required_slots = (cameras.len() * (1 + render_state.two_d_batched.items.len())).max(1);
    ensure_two_d_pass_resources(render_state, device, queue, required_slots);
    let (
        pipeline_layout,
        camera_dynamic_buffer,
        light_storage_buffer,
        global_bind_group,
        camera_dynamic_stride,
        fallback_depth_view,
        shadow_mask_size,
    ) = {
        let resources = render_state
            .two_d_pass_resources
            .as_ref()
            .expect("2D pass resources must be initialized");
        (
            resources.pipeline_layout.clone(),
            resources.camera_dynamic_buffer.clone(),
            resources.light_storage_buffer.clone(),
            resources.global_bind_group.clone(),
            resources.camera_dynamic_stride,
            resources.fallback_depth_view.clone(),
            resources.shadow_mask_size,
        )
    };
    let library = render_state.library.as_ref().expect("library must exist");
    let meta = TwoDFrameSemanticMeta {
        resolution: glam::Vec2::new(target_size.x.max(1) as f32, target_size.y.max(1) as f32),
        inv_resolution: glam::Vec2::new(
            1.0 / target_size.x.max(1) as f32,
            1.0 / target_size.y.max(1) as f32,
        ),
        frame_index: frame_index as u32,
        flags: 1,
    };
    let meta_bytes = bytemuck::bytes_of(&meta);
    let needs_realloc = render_state
        .forward_semantics_buffer
        .as_ref()
        .map(|buffer| buffer.size() < meta_bytes.len() as u64)
        .unwrap_or(true);
    if needs_realloc {
        render_state.forward_semantics_buffer =
            Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("2D Frame Semantics Buffer"),
                size: meta_bytes.len() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
    }
    let Some(frame_semantics_buffer) = render_state.forward_semantics_buffer.as_ref() else {
        return;
    };
    queue.write_buffer(frame_semantics_buffer, 0, meta_bytes);
    let frame_semantics_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("2D Frame Semantics BG"),
        layout: &library.layout_frame_semantics,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&library.fallback_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&fallback_depth_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::TextureView(&library.fallback_view),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&library.fallback_view),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::Sampler(&library.samplers.point_clamp),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: frame_semantics_buffer.as_entire_binding(),
            },
        ],
    });

    let draw_batches = {
        let scene = &render_state.scene;
        match render_state.vertex.as_mut() {
            Some(vertex_sys) => resolve_2d_draw_batches(
                &render_state.two_d_batched.ranges,
                |material_id| {
                    if material_id == crate::core::resources::MATERIAL_FALLBACK_ID {
                        return true;
                    }
                    scene
                        .materials
                        .get(&material_id)
                        .map(material_allows_2d)
                        .unwrap_or(false)
                },
                |geometry_id| matches!(vertex_sys.index_info(geometry_id), Ok(Some(index_info)) if index_info.count > 0),
            ),
            None => Vec::new(),
        }
    };
    let camera_visible_lights: Vec<Vec<TwoDLightRaw>> = cameras
        .iter()
        .map(|camera| {
            collect_visible_2d_lights(
                render_state,
                camera,
                render_state.two_d_source.shadow_config,
            )
        })
        .collect();

    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("2D Draw Pass"),
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
    let mut current_pipeline_key: Option<PipelineKey> = None;

    // Pipeline/material binding is introduced in the next phase; for now we resolve valid batches
    // and consume the batched state deterministically inside the render pass.
    if let Some(vertex_sys) = render_state.vertex.as_mut() {
        vertex_sys.begin_pass();
        let mut camera_slot_index: usize = 0;
        for (camera_index, camera) in cameras.iter().enumerate() {
            let camera_vp = build_2d_view_projection(Some(camera), target_size);
            let mut visible_lights = camera_visible_lights[camera_index].clone();
            for (light_idx, light) in visible_lights.iter_mut().enumerate() {
                light.shadow_index = light_idx as u32;
            }
            if !visible_lights.is_empty() {
                queue.write_buffer(
                    &light_storage_buffer,
                    0,
                    bytemuck::cast_slice(&visible_lights),
                );
            }
            // Reserve one slot per camera to keep deterministic offset mapping and sizing.
            camera_slot_index = camera_slot_index.saturating_add(1);
            for batch in &draw_batches {
                vertex_sys.begin_pass();
                if !layer_visible_in_camera(batch.key.layer, camera.layer_mask) {
                    continue;
                }
                let Ok(Some(index_info)) = vertex_sys.index_info(batch.key.geometry_id) else {
                    continue;
                };
                if vertex_sys.bind(&mut pass, batch.key.geometry_id).is_err() {
                    continue;
                }
                let material = render_state
                    .scene
                    .materials
                    .get(&batch.key.material_id)
                    .or_else(|| {
                        render_state
                            .scene
                            .materials
                            .get(&crate::core::resources::MATERIAL_FALLBACK_ID)
                    });
                let surface_type = material
                    .map(|record| record.surface_type)
                    .unwrap_or(crate::core::resources::SurfaceType::Opaque);
                let topology = material
                    .map(|record| record.topology)
                    .unwrap_or(crate::core::resources::PrimitiveTopology::TriangleList);
                let polygon_mode = material
                    .map(|record| record.polygon_mode)
                    .unwrap_or(crate::core::resources::PolygonMode::Fill);
                let render_side = material
                    .map(|record| record.render_side)
                    .unwrap_or(crate::core::resources::RenderSide::Front);
                let blend = match surface_type {
                    crate::core::resources::SurfaceType::Transparent => {
                        Some(wgpu::BlendState::ALPHA_BLENDING)
                    }
                    crate::core::resources::SurfaceType::Opaque
                    | crate::core::resources::SurfaceType::Masked => None,
                };
                let cull_mode = match render_side {
                    crate::core::resources::RenderSide::Front => Some(wgpu::Face::Back),
                    crate::core::resources::RenderSide::Back => Some(wgpu::Face::Front),
                    crate::core::resources::RenderSide::DoubleSide => None,
                };
                let Some(record) = material else {
                    continue;
                };
                if !material_uses_compiled_2d_shader(record) {
                    continue;
                }
                let Some(source) = record.compiled_shader_source.as_ref() else {
                    continue;
                };
                let shader_id = if record.compiled_shader_hash == 0 {
                    1
                } else {
                    record.compiled_shader_hash
                };
                if !render_state
                    .material_shader_modules
                    .contains_key(&shader_id)
                {
                    render_state.material_shader_modules.insert(
                        shader_id,
                        device.create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("2D Material Shader"),
                            source: wgpu::ShaderSource::Wgsl(source.clone().into()),
                        }),
                    );
                }
                let Some(shader_module) = render_state.material_shader_modules.get(&shader_id)
                else {
                    continue;
                };
                let pipeline_key = PipelineKey {
                    shader_id,
                    color_format: target_format,
                    color_target_count: 1,
                    depth_format: None,
                    sample_count: 1,
                    topology: to_wgpu_topology(topology),
                    polygon_mode: to_wgpu_polygon_mode(polygon_mode),
                    cull_mode,
                    front_face: wgpu::FrontFace::Ccw,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::Always,
                    blend,
                };
                if current_pipeline_key != Some(pipeline_key) {
                    let pipeline =
                        render_state
                            .cache
                            .get_or_create(pipeline_key, frame_index, || {
                                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                                    label: Some("2D Material Pipeline"),
                                    layout: Some(&pipeline_layout),
                                    vertex: wgpu::VertexState {
                                        module: shader_module,
                                        entry_point: Some("vs_main"),
                                        compilation_options:
                                            wgpu::PipelineCompilationOptions::default(),
                                        buffers: &[
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::Position
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[wgpu::VertexAttribute {
                                                    format: wgpu::VertexFormat::Float32x3,
                                                    offset: 0,
                                                    shader_location: 0,
                                                }],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::Normal
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::Tangent
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::Color0
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[wgpu::VertexAttribute {
                                                    format: wgpu::VertexFormat::Float32x4,
                                                    offset: 0,
                                                    shader_location: 3,
                                                }],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::UV0
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[wgpu::VertexAttribute {
                                                    format: wgpu::VertexFormat::Float32x2,
                                                    offset: 0,
                                                    shader_location: 4,
                                                }],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::UV1
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::Joints
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[],
                                            },
                                            wgpu::VertexBufferLayout {
                                                array_stride:
                                                    crate::core::resources::VertexStream::Weights
                                                        .stride_bytes(),
                                                step_mode: wgpu::VertexStepMode::Vertex,
                                                attributes: &[],
                                            },
                                        ],
                                    },
                                    fragment: Some(wgpu::FragmentState {
                                        module: shader_module,
                                        entry_point: Some("fs_main"),
                                        compilation_options:
                                            wgpu::PipelineCompilationOptions::default(),
                                        targets: &[Some(wgpu::ColorTargetState {
                                            format: target_format,
                                            blend,
                                            write_mask: wgpu::ColorWrites::ALL,
                                        })],
                                    }),
                                    primitive: wgpu::PrimitiveState {
                                        topology: to_wgpu_topology(topology),
                                        strip_index_format: None,
                                        front_face: wgpu::FrontFace::Ccw,
                                        cull_mode,
                                        unclipped_depth: false,
                                        polygon_mode: to_wgpu_polygon_mode(polygon_mode),
                                        conservative: false,
                                    },
                                    depth_stencil: None,
                                    multisample: wgpu::MultisampleState::default(),
                                    multiview_mask: None,
                                    cache: None,
                                })
                            });
                    pass.set_pipeline(pipeline);
                    current_pipeline_key = Some(pipeline_key);
                }
                let marker = format!(
                    "2d.camera={} layer={} material={} geometry={} kind={:?} start={} count={}",
                    camera.camera_id,
                    batch.key.layer,
                    batch.key.material_id,
                    batch.key.geometry_id,
                    batch.key.kind,
                    batch.start,
                    batch.count,
                );
                pass.insert_debug_marker(&marker);
                pass.set_bind_group(0, &global_bind_group, &[0]);
                pass.set_bind_group(2, &frame_semantics_bind_group, &[]);
                let Some(material) = material else {
                    continue;
                };
                let Some(group) = material.bind_group.as_ref() else {
                    continue;
                };
                let Some(material_slot) = render_state
                    .material_uniform_slots
                    .get(&batch.key.material_id)
                    .copied()
                else {
                    continue;
                };
                let Some(bindings) = render_state.bindings.as_ref() else {
                    continue;
                };
                let material_offset = bindings.material_3d_pool.get_offset(material_slot) as u32;
                pass.set_bind_group(1, group, &[material_offset]);

                let start = batch.start as usize;
                let end = start.saturating_add(batch.count as usize);
                let Some(items) = render_state.two_d_batched.items.get(start..end) else {
                    continue;
                };
                if items.is_empty() {
                    continue;
                }
                let material_tint =
                    material_tint_for_batch(&render_state.scene, batch.key.material_id);
                for item in items {
                    if item.occluder_only {
                        continue;
                    }
                    let camera_raw = TwoDCameraRaw {
                        view_projection: camera_vp,
                        model_matrix: item.transform,
                        tint: material_tint,
                        model_position: item.transform.w_axis,
                        light_offset_count: glam::UVec4::new(
                            0,
                            visible_lights.len() as u32,
                            item.shadow_layer_mask,
                            0,
                        ),
                        shadow_params: glam::Vec4::new(
                            1.0,
                            if item.receive_shadow { 1.0 } else { 0.0 },
                            render_state.two_d_source.shadow_config.ambient,
                            shadow_mask_size.x.max(1) as f32,
                        ),
                        shadow_controls: glam::Vec4::new(
                            render_state
                                .two_d_source
                                .shadow_config
                                .shadow_contact_offset,
                            0.0,
                            render_state
                                .two_d_source
                                .shadow_config
                                .shadow_debug_light_index as f32,
                            render_state.two_d_source.shadow_config.shadow_debug_mode as f32,
                        ),
                    };
                    let offset = (camera_slot_index as u64) * camera_dynamic_stride;
                    queue.write_buffer(
                        &camera_dynamic_buffer,
                        offset,
                        bytemuck::bytes_of(&camera_raw),
                    );
                    pass.set_bind_group(0, &global_bind_group, &[offset as u32]);
                    camera_slot_index = camera_slot_index.saturating_add(1);
                    pass.draw_indexed(0..index_info.count, 0, 0..1);
                }
            }
        }
    } else {
        for batch in &draw_batches {
            let marker = format!(
                "2d.batch layer={} material={} geometry={} kind={:?} start={} count={}",
                batch.key.layer,
                batch.key.material_id,
                batch.key.geometry_id,
                batch.key.kind,
                batch.start,
                batch.count,
            );
            pass.insert_debug_marker(&marker);
        }
    }
}

pub(crate) fn pass_2d_shadow_masks_update(
    render_state: &mut RenderState,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    _encoder: &mut wgpu::CommandEncoder,
    _frame_index: u64,
    target_size: glam::UVec2,
) {
    if let Some(shadow_manager) = render_state.shadow_2d.as_mut() {
        let requested = render_state
            .two_d_source
            .shadow_config
            .angular_resolution
            .clamp(128, 4096);
        if shadow_manager.config.angular_resolution != requested {
            shadow_manager.config.angular_resolution = requested;
            shadow_manager.mark_dirty();
        }
    }
    let cameras = if render_state.two_d_prepared.cameras.is_empty() {
        vec![crate::core::render::state::TwoDPreparedCamera {
            camera_id: 0,
            transform: glam::Mat4::IDENTITY,
            near_far: glam::Vec2::new(0.0, 1.0),
            ortho_scale: 1.0,
            layer_mask: u32::MAX,
            order: 0,
        }]
    } else {
        render_state.two_d_prepared.cameras.clone()
    };
    let scene_hash = hash_2d_shadow_scene(render_state, &cameras, target_size);
    let required_slots = (cameras.len() * (1 + render_state.two_d_batched.items.len())).max(1);
    ensure_two_d_pass_resources(render_state, device, queue, required_slots);
    let shadow_mask_size = {
        let resources = render_state
            .two_d_pass_resources
            .as_ref()
            .expect("2D pass resources must be initialized");
        resources.shadow_mask_size
    };
    let previous_camera_hashes = render_state
        .shadow_2d
        .as_ref()
        .map(|m| m.camera_light_hashes.clone())
        .unwrap_or_default();
    let mut camera_light_hash_updates: Vec<(u32, Vec<u64>)> = Vec::new();
    let mut updates_this_frame = 0_u32;
    let mut update_cursor = render_state
        .shadow_2d
        .as_ref()
        .map(|m| m.update_cursor)
        .unwrap_or(0);
    for camera in &cameras {
        let camera_vp = build_2d_view_projection(Some(camera), target_size);
        let mut visible_lights = collect_visible_2d_lights(
            render_state,
            camera,
            render_state.two_d_source.shadow_config,
        );
        for (light_idx, light) in visible_lights.iter_mut().enumerate() {
            light.shadow_index = light_idx as u32;
        }
        let shadow_occluders = collect_shadow_occluders(render_state, camera);
        let mut camera_light_hashes = previous_camera_hashes
            .get(&camera.camera_id)
            .cloned()
            .unwrap_or_else(|| vec![0_u64; visible_lights.len()]);
        camera_light_hashes.resize(visible_lights.len(), 0);
        if visible_lights.is_empty() {
            camera_light_hash_updates.push((camera.camera_id, camera_light_hashes));
            continue;
        }
        let light_count = visible_lights.len();
        let start_idx = update_cursor % light_count;
        update_cursor = update_cursor.saturating_add(1);
        for offset in 0..light_count {
            let light = &visible_lights[(start_idx + offset) % light_count];
            let layer_hash =
                hash_2d_shadow_light_layer(camera_vp, light, &shadow_occluders, shadow_mask_size);
            let relevant_occluders: Vec<&TwoDOccluderRaw> = shadow_occluders
                .iter()
                .filter(|occ| (occ.shadow_layer_mask & light.shadow_layer_mask) != 0)
                .collect();
            let Some(resources) = render_state.two_d_pass_resources.as_mut() else {
                continue;
            };
            write_shadow_samples_for_light(
                resources,
                queue,
                shadow_mask_size.x,
                light.shadow_index,
                light,
                &relevant_occluders,
            );
            camera_light_hashes[light.shadow_index as usize] = layer_hash;
            updates_this_frame = updates_this_frame.saturating_add(1);
        }
        camera_light_hash_updates.push((camera.camera_id, camera_light_hashes));
    }
    if let Some(shadow_manager) = render_state.shadow_2d.as_mut() {
        for (camera_id, light_hashes) in camera_light_hash_updates {
            shadow_manager.set_camera_light_hashes(camera_id, light_hashes);
        }
        shadow_manager.last_updated_layers = updates_this_frame;
        shadow_manager.update_cursor = update_cursor;
        shadow_manager.mark_updated(scene_hash);
    }
}

fn to_wgpu_topology(
    topology: crate::core::resources::PrimitiveTopology,
) -> wgpu::PrimitiveTopology {
    match topology {
        crate::core::resources::PrimitiveTopology::PointList => wgpu::PrimitiveTopology::PointList,
        crate::core::resources::PrimitiveTopology::LineList => wgpu::PrimitiveTopology::LineList,
        crate::core::resources::PrimitiveTopology::TriangleList => {
            wgpu::PrimitiveTopology::TriangleList
        }
    }
}

fn to_wgpu_polygon_mode(mode: crate::core::resources::PolygonMode) -> wgpu::PolygonMode {
    match mode {
        crate::core::resources::PolygonMode::Fill => wgpu::PolygonMode::Fill,
        crate::core::resources::PolygonMode::Line => wgpu::PolygonMode::Line,
        crate::core::resources::PolygonMode::Point => wgpu::PolygonMode::Point,
    }
}

fn build_2d_view_projection(
    camera: Option<&crate::core::render::state::TwoDPreparedCamera>,
    target_size: glam::UVec2,
) -> glam::Mat4 {
    let width = target_size.x.max(1) as f32;
    let height = target_size.y.max(1) as f32;
    let aspect = width / height;
    match camera {
        Some(camera) => {
            let scale = camera.ortho_scale.max(1e-4);
            let half_h = scale;
            let half_w = half_h * aspect;
            let near = camera.near_far.x;
            let far = camera.near_far.y;
            let proj = glam::Mat4::orthographic_rh(-half_w, half_w, -half_h, half_h, near, far);
            let view = camera.transform.inverse();
            proj * view
        }
        None => glam::Mat4::orthographic_rh(-aspect, aspect, -1.0, 1.0, 0.0, 1.0),
    }
}

fn layer_visible_in_camera(layer: i32, layer_mask: u32) -> bool {
    if layer < 0 || layer > 31 {
        return false;
    }
    let bit = 1_u32 << (layer as u32);
    (layer_mask & bit) != 0
}

#[cfg(test)]
mod tests {
    use super::{
        build_occluder_light_cone, build_quad_occluder_silhouette, collect_visible_2d_lights,
        layer_visible_in_camera, material_allows_2d, material_tint_for_batch,
        material_uses_compiled_2d_shader, pass_2d_batch, pass_2d_prepare, resolve_2d_draw_batches,
    };
    use crate::core::render::RenderState;
    use crate::core::render::state::{
        TwoDBatchKey, TwoDBatchRange, TwoDItemKind, TwoDOccluderSourceKind,
    };
    use crate::core::resources::{Camera2dRecord, Shape2dRecord, Sprite2dRecord};

    fn assert_vec2_close(actual: glam::Vec2, expected: glam::Vec2) {
        let delta = (actual - expected).abs();
        assert!(
            delta.x <= 1e-4 && delta.y <= 1e-4,
            "{actual:?} != {expected:?}"
        );
    }

    #[test]
    fn prepare_2d_collects_and_sorts_items_deterministically() {
        let mut render_state = RenderState::new(wgpu::TextureFormat::Rgba16Float);
        render_state.two_d_source.cameras.insert(
            7,
            Camera2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                near_far: glam::Vec2::new(0.01, 10.0),
                ortho_scale: 2.0,
                layer_mask: 1,
                order: 2,
            },
        );
        render_state.two_d_source.cameras.insert(
            2,
            Camera2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                near_far: glam::Vec2::new(0.01, 10.0),
                ortho_scale: 1.0,
                layer_mask: 1,
                order: 1,
            },
        );
        render_state.two_d_source.sprites.insert(
            10,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 1,
                material_id: Some(100),
                layer: 4,
                cast_shadow: true,
                receive_shadow: true,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            },
        );
        render_state.two_d_source.shapes.insert(
            4,
            Shape2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 2,
                material_id: None,
                layer: 4,
                cast_shadow: true,
                receive_shadow: true,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            },
        );
        render_state.two_d_source.sprites.insert(
            3,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 3,
                material_id: None,
                layer: 1,
                cast_shadow: true,
                receive_shadow: true,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            },
        );

        pass_2d_prepare(&mut render_state);

        let camera_order: Vec<u32> = render_state
            .two_d_prepared
            .cameras
            .iter()
            .map(|camera| camera.camera_id)
            .collect();
        assert_eq!(camera_order, vec![2, 7]);

        let item_order: Vec<u32> = render_state
            .two_d_prepared
            .items
            .iter()
            .map(|item| item.item_id)
            .collect();
        assert_eq!(item_order, vec![3, 10, 4]);
    }

    #[test]
    fn batch_2d_groups_by_layer_material_geometry_and_kind() {
        let mut render_state = RenderState::new(wgpu::TextureFormat::Rgba16Float);
        render_state.two_d_source.sprites.insert(
            10,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 7,
                material_id: Some(11),
                layer: 1,
                cast_shadow: true,
                receive_shadow: true,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            },
        );
        render_state.two_d_source.sprites.insert(
            11,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 7,
                material_id: Some(11),
                layer: 1,
                cast_shadow: true,
                receive_shadow: true,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            },
        );
        render_state.two_d_source.shapes.insert(
            20,
            Shape2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 7,
                material_id: Some(11),
                layer: 1,
                cast_shadow: true,
                receive_shadow: true,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            },
        );
        render_state.two_d_source.sprites.insert(
            12,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 9,
                material_id: None,
                layer: 2,
                cast_shadow: true,
                receive_shadow: true,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            },
        );

        pass_2d_prepare(&mut render_state);
        pass_2d_batch(&mut render_state);

        assert_eq!(render_state.two_d_batched.ranges.len(), 3);
        assert_eq!(render_state.two_d_batched.ranges[0].count, 2);
        assert_eq!(render_state.two_d_batched.ranges[1].count, 1);
        assert_eq!(render_state.two_d_batched.ranges[2].count, 1);
        assert_eq!(render_state.two_d_batched.ranges[0].key.layer, 1);
        assert_eq!(render_state.two_d_batched.ranges[1].key.layer, 1);
        assert_eq!(render_state.two_d_batched.ranges[2].key.layer, 2);
    }

    #[test]
    fn batch_2d_keeps_deterministic_order_for_same_batch_key() {
        let mut render_state = RenderState::new(wgpu::TextureFormat::Rgba16Float);
        render_state.two_d_source.sprites.insert(
            300,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 7,
                material_id: Some(11),
                layer: 1,
                cast_shadow: true,
                receive_shadow: true,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            },
        );
        render_state.two_d_source.sprites.insert(
            100,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 7,
                material_id: Some(11),
                layer: 1,
                cast_shadow: true,
                receive_shadow: true,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            },
        );
        render_state.two_d_source.sprites.insert(
            200,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 7,
                material_id: Some(11),
                layer: 1,
                cast_shadow: true,
                receive_shadow: true,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            },
        );

        pass_2d_prepare(&mut render_state);
        pass_2d_batch(&mut render_state);

        let item_order: Vec<u32> = render_state
            .two_d_batched
            .items
            .iter()
            .map(|item| item.item_id)
            .collect();
        assert_eq!(item_order, vec![100, 200, 300]);
        assert_eq!(render_state.two_d_batched.ranges.len(), 1);
        assert_eq!(render_state.two_d_batched.ranges[0].count, 3);
    }

    #[test]
    fn resolve_draw_batches_filters_missing_material_or_geometry() {
        let ranges = vec![
            TwoDBatchRange {
                key: TwoDBatchKey {
                    layer: 0,
                    material_id: 10,
                    geometry_id: 20,
                    kind: TwoDItemKind::Sprite,
                },
                start: 0,
                count: 2,
            },
            TwoDBatchRange {
                key: TwoDBatchKey {
                    layer: 0,
                    material_id: 11,
                    geometry_id: 21,
                    kind: TwoDItemKind::Shape,
                },
                start: 2,
                count: 3,
            },
            TwoDBatchRange {
                key: TwoDBatchKey {
                    layer: 1,
                    material_id: 12,
                    geometry_id: 22,
                    kind: TwoDItemKind::Sprite,
                },
                start: 5,
                count: 0,
            },
        ];
        let resolved = resolve_2d_draw_batches(
            &ranges,
            |material_id| material_id == 10,
            |geometry_id| geometry_id == 20,
        );
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].start, 0);
        assert_eq!(resolved[0].count, 2);
        assert_eq!(resolved[0].key.material_id, 10);
        assert_eq!(resolved[0].key.geometry_id, 20);
    }

    #[test]
    fn material_tint_uses_first_material_input_or_white() {
        let mut scene = crate::core::render::state::RenderScene::default();
        let mut material = crate::core::resources::ShaderMaterialRecord::new_standard(None);
        material.inputs[0] = glam::Vec4::new(0.25, 0.5, 0.75, 1.0);
        scene.materials.insert(5, material);
        assert_eq!(
            material_tint_for_batch(&scene, 5),
            glam::Vec4::new(0.25, 0.5, 0.75, 1.0)
        );
        assert_eq!(material_tint_for_batch(&scene, 99), glam::Vec4::ONE);
    }

    #[test]
    fn material_tint_falls_back_to_white_when_alpha_is_zero() {
        let mut scene = crate::core::render::state::RenderScene::default();
        let mut material = crate::core::resources::ShaderMaterialRecord::new_standard(None);
        material.inputs[0] = glam::Vec4::new(1.0, 0.0, 0.0, 0.0);
        scene.materials.insert(15, material);
        assert_eq!(material_tint_for_batch(&scene, 15), glam::Vec4::ONE);
    }

    #[test]
    fn material_compiled_2d_shader_requires_realm_and_compiled_source() {
        let mut material = crate::core::resources::ShaderMaterialRecord::new_standard(None);
        assert!(!material_uses_compiled_2d_shader(&material));
        material.realm_kind = crate::core::resources::MaterialRealmKind::TwoD;
        assert!(material_uses_compiled_2d_shader(&material));
        material.compiled_shader_source = None;
        assert!(!material_uses_compiled_2d_shader(&material));
        material.compiled_shader_source = Some("@vertex fn vs_main(){}".to_string());
        material.compile_error = None;
        assert!(material_uses_compiled_2d_shader(&material));
        material.compile_error = Some("broken".to_string());
        assert!(!material_uses_compiled_2d_shader(&material));
    }

    #[test]
    fn material_realm_kind_controls_2d_eligibility() {
        let mut material = crate::core::resources::ShaderMaterialRecord::new_standard(None);
        material.realm_kind = crate::core::resources::MaterialRealmKind::ThreeD;
        assert!(!material_allows_2d(&material));
        material.realm_kind = crate::core::resources::MaterialRealmKind::TwoD;
        assert!(material_allows_2d(&material));
        material.realm_kind = crate::core::resources::MaterialRealmKind::TwoD;
        assert!(material_allows_2d(&material));
    }

    #[test]
    fn layer_visibility_respects_bit_mask_and_bounds() {
        let layer_mask = (1_u32 << 1) | (1_u32 << 4);
        assert!(layer_visible_in_camera(1, layer_mask));
        assert!(layer_visible_in_camera(4, layer_mask));
        assert!(!layer_visible_in_camera(0, layer_mask));
        assert!(!layer_visible_in_camera(-1, layer_mask));
        assert!(!layer_visible_in_camera(32, layer_mask));
    }

    #[test]
    fn prepare_2d_keeps_cast_and_receive_shadow_flags() {
        let mut render_state = RenderState::new(wgpu::TextureFormat::Rgba16Float);
        render_state.two_d_source.sprites.insert(
            1,
            Sprite2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 1,
                material_id: Some(1),
                layer: 0,
                cast_shadow: true,
                receive_shadow: false,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            },
        );
        render_state.two_d_source.shapes.insert(
            2,
            Shape2dRecord {
                label: None,
                transform: glam::Mat4::IDENTITY,
                geometry_id: 1,
                material_id: Some(1),
                layer: 0,
                cast_shadow: false,
                receive_shadow: true,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            },
        );
        pass_2d_prepare(&mut render_state);
        assert_eq!(render_state.two_d_prepared.items.len(), 2);
        assert_eq!(render_state.two_d_prepared.occluders.len(), 1);
        assert_eq!(render_state.two_d_prepared.items[0].item_id, 1);
        assert!(render_state.two_d_prepared.items[0].cast_shadow);
        assert!(!render_state.two_d_prepared.items[0].receive_shadow);
        assert_eq!(render_state.two_d_prepared.items[1].item_id, 2);
        assert!(!render_state.two_d_prepared.items[1].cast_shadow);
        assert!(render_state.two_d_prepared.items[1].receive_shadow);
        assert_eq!(render_state.two_d_prepared.occluders[0].occluder_id, 1);
        assert_eq!(
            render_state.two_d_prepared.occluders[0].source_kind,
            TwoDOccluderSourceKind::Sprite
        );
        assert_eq!(
            render_state.two_d_prepared.occluders[0].silhouette.vertices[0],
            glam::Vec2::new(-0.5, -0.5)
        );
        assert_eq!(
            render_state.two_d_prepared.occluders[0].silhouette.vertices[2],
            glam::Vec2::new(0.5, 0.5)
        );
    }

    #[test]
    fn quad_occluder_silhouette_uses_world_space_corners() {
        let transform = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::new(2.0, 4.0, 1.0),
            glam::Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            glam::Vec3::new(3.0, 5.0, 0.0),
        );
        let silhouette = build_quad_occluder_silhouette(transform).expect("valid quad silhouette");

        assert_vec2_close(silhouette.vertices[0], glam::Vec2::new(5.0, 4.0));
        assert_vec2_close(silhouette.vertices[1], glam::Vec2::new(5.0, 6.0));
        assert_vec2_close(silhouette.vertices[2], glam::Vec2::new(1.0, 6.0));
        assert_vec2_close(silhouette.vertices[3], glam::Vec2::new(1.0, 4.0));
        assert_vec2_close(silhouette.edges[0].a, silhouette.vertices[0]);
        assert_vec2_close(silhouette.edges[0].b, silhouette.vertices[1]);
    }

    #[test]
    fn occluder_light_cone_expands_penumbra_from_support_vertices() {
        let silhouette = build_quad_occluder_silhouette(glam::Mat4::from_translation(
            glam::Vec3::new(2.0, 0.0, 0.0),
        ))
        .expect("valid quad silhouette");
        let cone =
            build_occluder_light_cone(&silhouette, glam::Vec2::ZERO, 0.25).expect("valid cone");

        assert_vec2_close(cone.support_points[0], glam::Vec2::new(1.5, -0.5));
        assert_vec2_close(cone.support_points[1], glam::Vec2::new(1.5, 0.5));
        assert!(cone.penumbra_interval.x < cone.blocker_interval.x);
        assert!(cone.penumbra_interval.y > cone.blocker_interval.y);
    }

    #[test]
    fn occluder_light_cone_drops_umbra_when_source_is_too_wide() {
        let silhouette = build_quad_occluder_silhouette(glam::Mat4::from_translation(
            glam::Vec3::new(1.0, 0.0, 0.0),
        ))
        .expect("valid quad silhouette");

        assert!(build_occluder_light_cone(&silhouette, glam::Vec2::ZERO, 0.75).is_none());
    }

    #[test]
    fn visible_2d_lights_are_filtered_by_camera_proximity() {
        let mut render_state = RenderState::new(wgpu::TextureFormat::Rgba16Float);
        render_state.two_d_source.lights.insert(
            10,
            crate::core::resources::LightRecord::new(
                None,
                crate::core::resources::LightComponent::new(
                    glam::Vec4::new(100.0, 100.0, 0.0, 1.0),
                    glam::Vec4::new(0.0, -1.0, 0.0, 0.0),
                    glam::Vec4::ONE,
                    glam::Vec4::ZERO,
                    1.0,
                    1.0,
                    glam::Vec2::ZERO,
                    crate::core::resources::LightKind::Point,
                    true,
                ),
                true,
                u32::MAX,
                u32::MAX,
                true,
            ),
        );
        render_state.two_d_source.lights.insert(
            20,
            crate::core::resources::LightRecord::new(
                None,
                crate::core::resources::LightComponent::new(
                    glam::Vec4::new(0.0, 0.0, 0.0, 1.0),
                    glam::Vec4::new(0.0, -1.0, 0.0, 0.0),
                    glam::Vec4::ONE,
                    glam::Vec4::ZERO,
                    1.0,
                    10.0,
                    glam::Vec2::ZERO,
                    crate::core::resources::LightKind::Point,
                    true,
                ),
                true,
                u32::MAX,
                u32::MAX,
                true,
            ),
        );
        let camera = crate::core::render::state::TwoDPreparedCamera {
            camera_id: 1,
            transform: glam::Mat4::IDENTITY,
            near_far: glam::Vec2::new(0.01, 100.0),
            ortho_scale: 2.0,
            layer_mask: u32::MAX,
            order: 0,
        };
        let visible = collect_visible_2d_lights(
            &render_state,
            &camera,
            render_state.two_d_source.shadow_config,
        );
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].position, glam::Vec4::new(0.0, 0.0, 0.0, 1.0));
    }
}

pub fn pass_2d_compose(
    _render_state: &mut RenderState,
    _device: &wgpu::Device,
    _queue: &wgpu::Queue,
    _encoder: &mut wgpu::CommandEncoder,
    _target_view: &wgpu::TextureView,
    _target_format: wgpu::TextureFormat,
    _target_size: glam::UVec2,
    _frame_index: u64,
) {
    // 2D draw pass writes directly into the target view. For the default 2D graph,
    // compose is intentionally a no-op to avoid overwriting the frame with 3D compose.
}
