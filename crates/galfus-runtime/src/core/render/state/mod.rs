pub mod binding;
pub mod collector;
pub mod init;
pub mod library;
pub mod lifecycle;
pub mod light;
pub mod prepare;
pub mod scene;
pub mod skinning;

use crate::core::render::cache::RenderCache;
use crate::core::render::gizmos::GizmoSystem;
use crate::core::resources::VertexAllocatorSystem;
use crate::core::resources::shadow::ShadowManager3d;
use crate::core::resources::shadow2d::ShadowManager2d;
use crate::core::resources::{
    Camera2dRecord, CameraNode, EnvironmentConfig, ForwardAtlasEntry, GeometryPrimitiveType,
    LightRecord, MaterialDefinitionRecord, MaterialInstanceRecord, ModelRecord,
    Realm2dShadowConfig, ShaderMaterialRecord, Shape2dRecord, Sprite2dRecord, TargetTextureBinding,
    TextureRecord,
};

pub use self::binding::BindingSystem;
pub use self::collector::{DrawCollector, DrawItem};
pub use self::library::ResourceLibrary;
#[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
pub use self::library::SamplerSet;
pub use self::light::{FrustumPlane, LightCullingSystem};
pub use self::scene::RenderScene;
pub use self::skinning::SkinningSystem;
pub type RealmEntities = galfus_realm_3d::RealmEntities<CameraNode, ModelRecord, LightRecord>;
pub type UniversalGeometryRecord =
    galfus_realm_3d::GeometryRecord<(GeometryPrimitiveType, Vec<u8>)>;
pub type Realm3dState = galfus_realm_3d::Realm3dState<
    CameraNode,
    ModelRecord,
    LightRecord,
    ShaderMaterialRecord,
    UniversalGeometryRecord,
    EnvironmentConfig,
>;
pub type Realm2dState = galfus_realm_2d::Realm2dState<
    Camera2dRecord,
    Sprite2dRecord,
    Shape2dRecord,
    ShaderMaterialRecord,
>;

#[derive(Debug, Default, Clone)]
pub struct TwoDSourceState {
    pub cameras: std::collections::HashMap<u32, Camera2dRecord>,
    pub sprites: std::collections::HashMap<u32, Sprite2dRecord>,
    pub shapes: std::collections::HashMap<u32, Shape2dRecord>,
    pub shadow_config: Realm2dShadowConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TwoDItemKind {
    Sprite,
    Shape,
}

#[derive(Debug, Clone)]
pub struct TwoDPreparedCamera {
    pub camera_id: u32,
    pub transform: glam::Mat4,
    pub near_far: glam::Vec2,
    pub ortho_scale: f32,
    pub layer_mask: u32,
    pub order: i32,
}

#[derive(Debug, Clone)]
pub struct TwoDPreparedItem {
    pub item_id: u32,
    pub kind: TwoDItemKind,
    pub transform: glam::Mat4,
    pub geometry_id: u32,
    pub material_id: Option<u32>,
    pub layer: i32,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
    pub occluder_only: bool,
    pub shadow_height: f32,
    pub shadow_layer_mask: u32,
}

#[derive(Debug, Default, Clone)]
pub struct TwoDPreparedState {
    pub cameras: Vec<TwoDPreparedCamera>,
    pub items: Vec<TwoDPreparedItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TwoDBatchKey {
    pub layer: i32,
    pub material_id: u32,
    pub geometry_id: u32,
    pub kind: TwoDItemKind,
}

#[derive(Debug, Clone)]
pub struct TwoDBatchRange {
    pub key: TwoDBatchKey,
    pub start: u32,
    pub count: u32,
}

#[derive(Debug, Default, Clone)]
pub struct TwoDBatchedState {
    pub items: Vec<TwoDPreparedItem>,
    pub ranges: Vec<TwoDBatchRange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SampledTargetBindKey {
    pub target_view_ptr: usize,
    pub outline_view_ptr: usize,
    pub ssao_view_ptr: usize,
    pub bloom_view_ptr: usize,
    pub uniform_buffer_ptr: usize,
}

pub struct TwoDPassResources {
    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline_layout: wgpu::PipelineLayout,
    pub camera_dynamic_buffer: wgpu::Buffer,
    pub light_storage_buffer: wgpu::Buffer,
    pub global_bind_group: wgpu::BindGroup,
    pub camera_dynamic_stride: u64,
    pub camera_dynamic_capacity_slots: usize,
    pub light_capacity_slots: usize,
    pub fallback_depth_view: wgpu::TextureView,
    pub shadow_mask_texture: wgpu::Texture,
    pub shadow_mask_view: wgpu::TextureView,
    pub shadow_mask_size: glam::UVec2,
    pub shadow_compute_bind_group_layout: wgpu::BindGroupLayout,
    pub shadow_compute_pipeline: wgpu::ComputePipeline,
}

#[derive(Debug, Default)]
pub struct RenderResourceState {
    pub textures: std::collections::HashMap<u32, TextureRecord>,
    pub forward_atlas_entries: std::collections::HashMap<u32, ForwardAtlasEntry>,
    pub target_texture_binds: std::collections::HashMap<u32, TargetTextureBinding>,
}

#[derive(Debug, Default)]
pub struct SceneRuntimeState {
    pub realm3d: Realm3dState,
    pub realm2d: Realm2dState,
    pub realm2d_shadow_configs:
        std::collections::HashMap<crate::core::realm::RealmId, Realm2dShadowConfig>,
    pub render_resources: RenderResourceState,
    pub material_definitions: std::collections::HashMap<u32, MaterialDefinitionRecord>,
    pub material_instances: std::collections::HashMap<u32, MaterialInstanceRecord>,
    pub material_program_cache:
        std::collections::HashMap<u64, galfus_render::CompiledMaterialShader>,
    pub material_program_cache_last_used_frame: std::collections::HashMap<u64, u64>,
}

#[derive(Debug, Default)]
pub struct RenderCatalogState {
    pub render_graphs_3d:
        std::collections::HashMap<u32, crate::core::render::graph::RenderGraphRecord>,
    pub render_graphs_2d:
        std::collections::HashMap<u32, crate::core::render::graph::RenderGraphRecord>,
    pub render_graph_plan_cache_3d:
        std::collections::HashMap<u64, crate::core::render::graph::RenderGraphState>,
    pub render_graph_plan_cache_2d:
        std::collections::HashMap<u64, crate::core::render::graph::RenderGraphState>,
    pub render_graph_compile_cache_hits: u64,
    pub render_graph_compile_cache_misses: u64,
}

pub struct RenderState {
    pub scene: RenderScene,
    pub detached_cameras: std::collections::HashMap<u32, crate::core::resources::CameraRecord>,
    pub camera_order: Vec<u32>,
    pub camera_uniform_slots: std::collections::HashMap<u32, u32>,
    pub material_uniform_slots: std::collections::HashMap<u32, u32>,
    pub target_texture_binds:
        std::collections::HashMap<u32, crate::core::resources::TargetTextureBinding>,
    pub external_textures: std::collections::HashMap<u32, wgpu::TextureView>,
    pub external_texture_sources: std::collections::HashMap<u32, usize>,
    pub bindings: Option<BindingSystem>,
    pub library: Option<ResourceLibrary>,
    pub vertex: Option<VertexAllocatorSystem>,
    pub light_system: Option<LightCullingSystem>,
    pub gizmos: GizmoSystem,
    pub shadow_3d: Option<ShadowManager3d>,
    pub shadow_2d: Option<ShadowManager2d>,
    pub cache: RenderCache,
    pub material_shader_modules: std::collections::HashMap<u64, wgpu::ShaderModule>,
    pub custom_screen_param_buffer: Option<wgpu::Buffer>,
    pub custom_screen_semantics_buffer: Option<wgpu::Buffer>,
    pub forward_semantics_buffer: Option<wgpu::Buffer>,
    pub post_uniform_buffer: Option<wgpu::Buffer>,
    pub compose_uniform_buffer: Option<wgpu::Buffer>,
    pub ssao_uniform_buffer: Option<wgpu::Buffer>,
    pub ssao_blur_uniform_buffer: Option<wgpu::Buffer>,
    pub bloom_uniform_buffer: Option<wgpu::Buffer>,
    pub skybox_uniform_buffer: Option<wgpu::Buffer>,
    pub environment: EnvironmentConfig,
    pub environment_is_configured: bool,
    pub camera_environment_overrides: std::collections::HashMap<u32, EnvironmentConfig>,
    pub compose_bind_cache: std::collections::HashMap<SampledTargetBindKey, wgpu::BindGroup>,
    pub post_bind_cache: std::collections::HashMap<SampledTargetBindKey, wgpu::BindGroup>,
    pub two_d_pass_resources: Option<TwoDPassResources>,
    pub compose_bind_cache_hits: u32,
    pub compose_bind_cache_misses: u32,
    pub post_bind_cache_hits: u32,
    pub post_bind_cache_misses: u32,
    pub compose_bind_cache_evictions: u32,
    pub post_bind_cache_evictions: u32,
    pub material_shader_module_evictions: u32,
    pub textures_sync_hash: u64,
    pub atlas_sync_hash: u64,
    pub target_binds_sync_hash: u64,
    pub light_prepare_sorted_ids: Vec<u32>,
    pub light_prepare_lights: Vec<crate::core::resources::LightComponent>,
    pub light_prepare_frustums: Vec<FrustumPlane>,
    pub rgba16f_msaa_supported_mask: u8,
    pub skinning: SkinningSystem,
    pub two_d_source: TwoDSourceState,
    pub two_d_prepared: TwoDPreparedState,
    pub two_d_batched: TwoDBatchedState,

    /// Per-frame collector for draw calls, reused to avoid allocations.
    pub collector: DrawCollector,
}

impl RenderState {
    pub const MSAA_MASK_1: u8 = 1 << 0;
    pub const MSAA_MASK_2: u8 = 1 << 1;
    pub const MSAA_MASK_4: u8 = 1 << 2;
    pub const MSAA_MASK_8: u8 = 1 << 3;
    pub const MSAA_MASK_16: u8 = 1 << 4;
    pub const MSAA_MASK_DEFAULT_SAFE: u8 = Self::MSAA_MASK_1 | Self::MSAA_MASK_4;

    pub fn msaa_sample_count_for_format(
        &self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> u32 {
        self.msaa_sample_count_for_environment(&self.environment, device, format)
    }

    pub fn msaa_sample_count_for_environment(
        &self,
        environment: &EnvironmentConfig,
        _device: &wgpu::Device,
        _format: wgpu::TextureFormat,
    ) -> u32 {
        let requested = if environment.msaa.enabled && environment.msaa.sample_count >= 2 {
            environment.msaa.sample_count
        } else {
            1
        };

        if requested <= 1 {
            return 1;
        }

        let supported = if self.rgba16f_msaa_supported_mask == 0 {
            Self::MSAA_MASK_DEFAULT_SAFE
        } else {
            self.rgba16f_msaa_supported_mask
        };

        let mut best = 1_u32;
        if (supported & Self::MSAA_MASK_2) != 0 && requested >= 2 {
            best = 2;
        }
        if (supported & Self::MSAA_MASK_4) != 0 && requested >= 4 {
            best = 4;
        }
        if (supported & Self::MSAA_MASK_8) != 0 && requested >= 8 {
            best = 8;
        }
        if (supported & Self::MSAA_MASK_16) != 0 && requested >= 16 {
            best = 16;
        }
        best
    }

    pub fn environment_for_camera(&self, camera_id: u32) -> &EnvironmentConfig {
        self.camera_environment_overrides
            .get(&camera_id)
            .unwrap_or(&self.environment)
    }

    pub fn camera_uniform_slot(&self, camera_id: u32) -> Option<u32> {
        self.camera_uniform_slots.get(&camera_id).copied()
    }

    pub fn camera_record(&self, camera_id: u32) -> Option<&crate::core::resources::CameraRecord> {
        self.scene
            .cameras
            .get(&camera_id)
            .or_else(|| self.detached_cameras.get(&camera_id))
    }

    pub fn estimated_gpu_bytes(&self) -> u64 {
        let render_targets = self
            .scene
            .cameras
            .values()
            .chain(self.detached_cameras.values())
            .map(Self::camera_record_gpu_bytes)
            .sum::<u64>();
        let scene_textures = self
            .scene
            .textures
            .values()
            .map(|record| {
                galfus_render::estimate_texture_bytes(record._texture.size(), record.format, 1)
            })
            .sum::<u64>();
        let post_buffers = [
            self.post_uniform_buffer.as_ref(),
            self.compose_uniform_buffer.as_ref(),
            self.ssao_uniform_buffer.as_ref(),
            self.ssao_blur_uniform_buffer.as_ref(),
            self.bloom_uniform_buffer.as_ref(),
            self.skybox_uniform_buffer.as_ref(),
        ]
        .into_iter()
        .flatten()
        .map(wgpu::Buffer::size)
        .sum::<u64>();
        render_targets
            .saturating_add(scene_textures)
            .saturating_add(post_buffers)
            .saturating_add(
                self.bindings
                    .as_ref()
                    .map(Self::binding_gpu_bytes)
                    .unwrap_or(0),
            )
            .saturating_add(
                self.light_system
                    .as_ref()
                    .map(Self::light_culling_gpu_bytes)
                    .unwrap_or(0),
            )
            .saturating_add(
                self.shadow_3d
                    .as_ref()
                    .map(Self::shadow_manager_gpu_bytes)
                    .unwrap_or(0),
            )
            .saturating_add(
                self.vertex
                    .as_ref()
                    .map(crate::core::resources::VertexAllocatorSystem::estimated_gpu_bytes)
                    .unwrap_or(0),
            )
            .saturating_add(self.gizmos.estimated_gpu_bytes())
    }

    fn camera_record_gpu_bytes(record: &crate::core::resources::CameraRecord) -> u64 {
        [
            record.render_target.as_ref(),
            record.emissive_target.as_ref(),
            record.post_target.as_ref(),
            record.outline_target.as_ref(),
            record.ssao_target.as_ref(),
            record.ssao_blur_target.as_ref(),
            record.bloom_target.as_ref(),
            record.forward_depth_target.as_ref(),
            record.forward_msaa_target.as_ref(),
            record.forward_emissive_msaa_target.as_ref(),
            record.history0_target.as_ref(),
            record.history1_target.as_ref(),
        ]
        .into_iter()
        .flatten()
        .map(galfus_render::RenderTarget::estimated_bytes)
        .sum::<u64>()
            + record
                .bloom_chain
                .iter()
                .flatten()
                .map(galfus_render::RenderTarget::estimated_bytes)
                .sum::<u64>()
    }

    fn binding_gpu_bytes(bindings: &BindingSystem) -> u64 {
        bindings
            .frame_pool
            .allocated_bytes()
            .saturating_add(bindings.camera_pool.allocated_bytes())
            .saturating_add(bindings.shadow_camera_pool.allocated_bytes())
            .saturating_add(bindings.model_pool.allocated_bytes())
            .saturating_add(bindings.instance_pool.allocated_bytes())
            .saturating_add(bindings.outline_instance_pool.allocated_bytes())
            .saturating_add(bindings.shadow_instance_pool.allocated_bytes())
            .saturating_add(bindings.material_3d_pool.allocated_bytes())
            .saturating_add(bindings.material_3d_inputs.allocated_bytes())
            .saturating_add(bindings.bones_pool.allocated_bytes())
    }

    fn light_culling_gpu_bytes(light_system: &LightCullingSystem) -> u64 {
        light_system
            .lights
            .allocated_bytes()
            .saturating_add(light_system.visible_indices.allocated_bytes())
            .saturating_add(light_system.visible_counts.allocated_bytes())
            .saturating_add(light_system.camera_frustums.allocated_bytes())
            .saturating_add(light_system.light_params.allocated_bytes())
            .saturating_add(
                light_system
                    .params_buffer
                    .as_ref()
                    .map(wgpu::Buffer::size)
                    .unwrap_or(0),
            )
    }

    fn shadow_manager_gpu_bytes(
        shadow_manager: &crate::core::resources::shadow::ShadowManager3d,
    ) -> u64 {
        shadow_manager
            .atlas
            .estimated_bytes()
            .saturating_add(shadow_manager.page_table.allocated_bytes())
            .saturating_add(shadow_manager.point_light_vp.allocated_bytes())
            .saturating_add(shadow_manager.params_pool.allocated_bytes())
    }

    pub fn sync_camera_targets_and_projection(
        &mut self,
        device: &wgpu::Device,
        surface_size: glam::UVec2,
        camera_target_sizes: Option<&std::collections::HashMap<u32, glam::UVec2>>,
    ) -> bool {
        let mut any_camera_dirty = false;
        for (camera_id, record) in self.scene.cameras.iter_mut() {
            let requested_size = camera_target_sizes
                .and_then(|sizes| sizes.get(camera_id).copied())
                .unwrap_or_else(|| record.effective_target_size(surface_size));
            let projection_size =
                glam::UVec2::new(requested_size.x.max(1), requested_size.y.max(1));
            let target_width = projection_size.x;
            let target_height = projection_size.y;

            crate::core::resources::ensure_render_target(
                device,
                &mut record.render_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.emissive_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.post_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.outline_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba8Unorm,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.ssao_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.ssao_blur_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            crate::core::resources::ensure_render_target(
                device,
                &mut record.bloom_target,
                target_width,
                target_height,
                wgpu::TextureFormat::Rgba16Float,
            );
            for (level, target) in record.bloom_chain.iter_mut().enumerate() {
                let level_width = crate::core::render::bloom_chain_size(target_width, level);
                let level_height = crate::core::render::bloom_chain_size(target_height, level);
                crate::core::resources::ensure_render_target(
                    device,
                    target,
                    level_width,
                    level_height,
                    wgpu::TextureFormat::Rgba16Float,
                );
            }

            if record.last_projection_size != projection_size {
                record.data.update(
                    None,
                    None,
                    None,
                    None,
                    (target_width, target_height),
                    record.ortho_scale,
                );
                record.last_projection_size = projection_size;
                record.mark_dirty();
                any_camera_dirty = true;
            }
        }

        any_camera_dirty
    }

    #[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
    pub fn on_resize(&mut self, _device: &wgpu::Device, _width: u32, _height: u32) {
        for record in self.scene.cameras.values_mut() {
            record.forward_depth_target = None;
            record.forward_msaa_target = None;
            record.forward_emissive_msaa_target = None;
            record.history0_target = None;
            record.history1_target = None;
            record.history_valid = false;
            record.history_idle_frames = 0;
        }
    }
}
