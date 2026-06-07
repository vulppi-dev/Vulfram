#[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
use super::RenderScene;
use super::RenderState;
#[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
use crate::core::render::cache::RenderCache;
#[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
use crate::core::render::gizmos::GizmoSystem;
#[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
use crate::core::render::state::collector::DrawCollector;
use crate::core::resources::{MATERIAL_FALLBACK_ID, ShaderMaterialRecord};
#[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
use std::collections::HashMap;
use std::collections::HashSet;

impl RenderState {
    const VERTEX_COMPACT_FRAME_INTERVAL: u64 = 120;
    const VERTEX_COMPACT_THRESHOLD: f32 = 0.25;
    const VERTEX_COMPACT_SLACK_RATIO: f32 = 0.3;
    const VERTEX_COMPACT_MIN_DEAD_BYTES: u64 = 256 * 1024;
    const COMPOSE_BIND_CACHE_HARD_MAX: usize = 512;
    const POST_BIND_CACHE_HARD_MAX: usize = 512;

    /// Create a new RenderState with empty systems
    #[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
    pub fn new(_surface_format: wgpu::TextureFormat) -> Self {
        let mut materials = HashMap::new();
        materials.insert(
            MATERIAL_FALLBACK_ID,
            ShaderMaterialRecord::new_standard(Some("Fallback Material".into())),
        );

        Self {
            scene: RenderScene {
                cameras: HashMap::new(),
                models: HashMap::new(),
                lights: HashMap::new(),
                materials,
                textures: HashMap::new(),
                forward_atlas_entries: HashMap::new(),
            },
            detached_cameras: HashMap::new(),
            camera_order: Vec::new(),
            camera_uniform_slots: HashMap::new(),
            material_uniform_slots: HashMap::new(),
            target_texture_binds: HashMap::new(),
            external_textures: HashMap::new(),
            external_texture_sources: HashMap::new(),
            bindings: None,
            library: None,
            vertex: None,
            light_system: None,
            gizmos: GizmoSystem::new(),
            shadow_3d: None,
            shadow_2d: None,
            cache: RenderCache::new(),
            material_shader_modules: HashMap::new(),
            custom_screen_param_buffer: None,
            custom_screen_semantics_buffer: None,
            forward_semantics_buffer: None,
            post_uniform_buffer: None,
            compose_uniform_buffer: None,
            ssao_uniform_buffer: None,
            ssao_blur_uniform_buffer: None,
            bloom_uniform_buffer: None,
            skybox_uniform_buffer: None,
            collector: DrawCollector::default(),
            skinning: crate::core::render::state::SkinningSystem::default(),
            two_d_source: crate::core::render::state::TwoDSourceState::default(),
            two_d_prepared: crate::core::render::state::TwoDPreparedState::default(),
            two_d_batched: crate::core::render::state::TwoDBatchedState::default(),
            environment: crate::core::resources::EnvironmentConfig::default(),
            environment_is_configured: false,
            camera_environment_overrides: HashMap::new(),
            compose_bind_cache: HashMap::new(),
            post_bind_cache: HashMap::new(),
            two_d_pass_resources: None,
            compose_bind_cache_hits: 0,
            compose_bind_cache_misses: 0,
            post_bind_cache_hits: 0,
            post_bind_cache_misses: 0,
            compose_bind_cache_evictions: 0,
            post_bind_cache_evictions: 0,
            material_shader_module_evictions: 0,
            textures_sync_hash: 0,
            atlas_sync_hash: 0,
            target_binds_sync_hash: 0,
            light_prepare_sorted_ids: Vec::new(),
            light_prepare_lights: Vec::new(),
            light_prepare_frustums: Vec::new(),
            rgba16f_msaa_supported_mask: RenderState::MSAA_MASK_DEFAULT_SAFE,
        }
    }

    /// Explicitly drop all render state resources
    pub fn drop_all(&mut self) {
        self.scene.cameras.clear();
        self.detached_cameras.clear();
        self.camera_order.clear();
        self.camera_uniform_slots.clear();
        self.material_uniform_slots.clear();
        self.scene.models.clear();
        self.scene.lights.clear();
        self.scene.materials.clear();
        self.scene.materials.insert(
            MATERIAL_FALLBACK_ID,
            ShaderMaterialRecord::new_standard(Some("Fallback Material".into())),
        );
        self.scene.textures.clear();
        self.scene.forward_atlas_entries.clear();
        self.target_texture_binds.clear();
        self.external_textures.clear();
        self.external_texture_sources.clear();
        self.bindings = None;
        self.library = None;
        self.vertex = None;
        self.light_system = None;
        self.shadow_3d = None;
        self.shadow_2d = None;
        self.cache.clear();
        self.material_shader_modules.clear();
        self.custom_screen_param_buffer = None;
        self.custom_screen_semantics_buffer = None;
        self.forward_semantics_buffer = None;
        self.post_uniform_buffer = None;
        self.compose_uniform_buffer = None;
        self.ssao_uniform_buffer = None;
        self.ssao_blur_uniform_buffer = None;
        self.bloom_uniform_buffer = None;
        self.skybox_uniform_buffer = None;
        self.skinning.clear();
        self.two_d_source.cameras.clear();
        self.two_d_source.sprites.clear();
        self.two_d_source.shapes.clear();
        self.two_d_prepared.cameras.clear();
        self.two_d_prepared.items.clear();
        self.two_d_batched.items.clear();
        self.two_d_batched.ranges.clear();
        self.environment = crate::core::resources::EnvironmentConfig::default();
        self.environment_is_configured = false;
        self.camera_environment_overrides.clear();
        self.compose_bind_cache.clear();
        self.post_bind_cache.clear();
        self.two_d_pass_resources = None;
        self.compose_bind_cache_hits = 0;
        self.compose_bind_cache_misses = 0;
        self.post_bind_cache_hits = 0;
        self.post_bind_cache_misses = 0;
        self.compose_bind_cache_evictions = 0;
        self.post_bind_cache_evictions = 0;
        self.material_shader_module_evictions = 0;
        self.textures_sync_hash = 0;
        self.atlas_sync_hash = 0;
        self.target_binds_sync_hash = 0;
        self.light_prepare_sorted_ids.clear();
        self.light_prepare_lights.clear();
        self.light_prepare_frustums.clear();
    }

    pub fn begin_frame(&mut self, frame_index: u64) {
        if let Some(vertex) = self.vertex.as_mut() {
            vertex.begin_frame(frame_index);
            if frame_index % Self::VERTEX_COMPACT_FRAME_INTERVAL == 0 {
                vertex.maybe_compact_all(
                    frame_index,
                    Self::VERTEX_COMPACT_THRESHOLD,
                    Self::VERTEX_COMPACT_SLACK_RATIO,
                    Self::VERTEX_COMPACT_MIN_DEAD_BYTES,
                );
            }
        }
        if let Some(bindings) = self.bindings.as_mut() {
            bindings.frame_pool.begin_frame(frame_index);
            bindings.camera_pool.begin_frame(frame_index);
            bindings.model_pool.begin_frame(frame_index);
            bindings.instance_pool.begin_frame(frame_index);
            bindings.outline_instance_pool.begin_frame(frame_index);
            bindings.shadow_instance_pool.begin_frame(frame_index);
            bindings.material_3d_pool.begin_frame(frame_index);
            bindings.material_3d_inputs.begin_frame(frame_index);
            bindings.bones_pool.begin_frame(frame_index);
        }
        if let Some(light_system) = self.light_system.as_mut() {
            light_system.lights.begin_frame(frame_index);
            light_system.visible_indices.begin_frame(frame_index);
            light_system.visible_counts.begin_frame(frame_index);
            light_system.camera_frustums.begin_frame(frame_index);
            light_system.light_params.begin_frame(frame_index);
        }
        if let Some(shadow) = self.shadow_3d.as_mut() {
            shadow.begin_frame(frame_index);
        }
        if let Some(shadow) = self.shadow_2d.as_mut() {
            shadow.begin_frame(frame_index);
        }
        self.gizmos.clear();
        self.compose_bind_cache_hits = 0;
        self.compose_bind_cache_misses = 0;
        self.post_bind_cache_hits = 0;
        self.post_bind_cache_misses = 0;
        self.compose_bind_cache_evictions = 0;
        self.post_bind_cache_evictions = 0;
        self.material_shader_module_evictions = 0;
        self.cache.reset_frame_stats();
        self.cache.gc(frame_index);
        self.two_d_prepared.cameras.clear();
        self.two_d_prepared.items.clear();
        self.two_d_batched.items.clear();
        self.two_d_batched.ranges.clear();
        let active_shader_ids: HashSet<u64> = self
            .scene
            .materials
            .values()
            .filter_map(|record| {
                if record.compiled_shader_source.is_some() {
                    Some(if record.compiled_shader_hash == 0 {
                        1
                    } else {
                        record.compiled_shader_hash
                    })
                } else {
                    None
                }
            })
            .collect();
        let before_shader_modules = self.material_shader_modules.len();
        self.material_shader_modules
            .retain(|shader_id, _| active_shader_ids.contains(shader_id));
        self.material_shader_module_evictions =
            self.material_shader_module_evictions.saturating_add(
                before_shader_modules.saturating_sub(self.material_shader_modules.len()) as u32,
            );
        self.trim_bind_caches_hard_limit();
    }

    fn trim_bind_caches_hard_limit(&mut self) {
        if self.compose_bind_cache.len() > Self::COMPOSE_BIND_CACHE_HARD_MAX {
            let overflow = self
                .compose_bind_cache
                .len()
                .saturating_sub(Self::COMPOSE_BIND_CACHE_HARD_MAX);
            let keys_to_remove: Vec<_> = self
                .compose_bind_cache
                .keys()
                .copied()
                .take(overflow)
                .collect();
            for key in keys_to_remove {
                self.compose_bind_cache.remove(&key);
            }
            self.compose_bind_cache_evictions = self
                .compose_bind_cache_evictions
                .saturating_add(overflow as u32);
        }
        if self.post_bind_cache.len() > Self::POST_BIND_CACHE_HARD_MAX {
            let overflow = self
                .post_bind_cache
                .len()
                .saturating_sub(Self::POST_BIND_CACHE_HARD_MAX);
            let keys_to_remove: Vec<_> = self
                .post_bind_cache
                .keys()
                .copied()
                .take(overflow)
                .collect();
            for key in keys_to_remove {
                self.post_bind_cache.remove(&key);
            }
            self.post_bind_cache_evictions = self
                .post_bind_cache_evictions
                .saturating_add(overflow as u32);
        }
    }
}
