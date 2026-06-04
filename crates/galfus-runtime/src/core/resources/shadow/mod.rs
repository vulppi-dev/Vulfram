use crate::core::resources::{StorageBufferPool, UniformBufferPool};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4Swizzles};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
use wgpu::Queue;
use wgpu::{Device, TextureFormat, TextureUsages};

mod atlas;
pub mod cmd;
pub use atlas::{ShadowAtlasDesc, ShadowAtlasHandle, ShadowAtlasRelocation, ShadowAtlasSystem};
pub use cmd::*;

/// Configuration for the Shadow Manager
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Shadow3dConfig {
    pub tile_resolution: u32,
    pub atlas_tiles_w: u32,
    pub atlas_tiles_h: u32,
    pub atlas_layers: u32,
    pub virtual_grid_size: u32,
    pub smoothing: u32,
    pub normal_bias: f32,
}

impl Default for Shadow3dConfig {
    fn default() -> Self {
        Self {
            tile_resolution: 2048,
            atlas_tiles_w: 16,
            atlas_tiles_h: 16,
            atlas_layers: 2,
            virtual_grid_size: 1,
            smoothing: 2,
            normal_bias: 0.03,
        }
    }
}

/// Uniform data for shadow parameters in the shader
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ShadowParams {
    pub virtual_grid_size: f32,
    pub pcf_range: i32,
    pub table_capacity: u32,
    pub point_vp_base: u32,
    pub bias_min: f32,
    pub bias_slope: f32,
    pub point_bias_min: f32,
    pub point_bias_slope: f32,
    pub normal_bias: f32,
    pub _padding: [f32; 2],
}

/// Unique identifier for a virtual shadow page
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShadowPageKey {
    pub realm_tag: u32,
    pub light_id: u32,
    pub face: u32,
    pub x: u32,
    pub y: u32,
}

/// State of an allocated shadow page
#[derive(Debug, Clone)]
pub struct ShadowPageRecord {
    pub atlas_handle: ShadowAtlasHandle,
    pub last_frame_used: u64,
    pub is_dirty: bool,
}

/// Entry in the GPU page table for Virtual Shadow Maps
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowPageEntry {
    /// [scale.x, scale.y, offset.x, offset.y]
    pub scale_offset: glam::Vec4,
    /// Index into the texture array
    pub layer_index: u32,
    /// Padding for 16-byte alignment
    pub _padding: [u32; 3],
}

impl Default for ShadowPageEntry {
    fn default() -> Self {
        Self {
            scale_offset: glam::Vec4::ZERO,
            layer_index: u32::MAX,
            _padding: [0; 3],
        }
    }
}

/// Manages Virtual Shadow Maps paging and atlas allocation
pub struct ShadowManager3d {
    pub atlas: ShadowAtlasSystem,
    pub page_table: StorageBufferPool<ShadowPageEntry>,
    pub point_light_vp: StorageBufferPool<glam::Mat4>,
    pub params_pool: UniformBufferPool<ShadowParams>,
    pub table_capacity: u32,
    pub is_dirty: bool,

    // Virtual to Physical mapping
    pub cache: HashMap<ShadowPageKey, ShadowPageRecord>,

    pub config: Shadow3dConfig,
}

impl ShadowManager3d {
    const MAX_IDLE_FRAMES: u64 = 600;
    fn build_params(&self, point_vp_base: u32) -> ShadowParams {
        ShadowParams {
            virtual_grid_size: self.config.virtual_grid_size as f32,
            pcf_range: self.config.smoothing as i32,
            table_capacity: self.table_capacity,
            point_vp_base,
            bias_min: 0.00001,
            bias_slope: 0.0001,
            point_bias_min: 0.001,
            point_bias_slope: 0.003,
            normal_bias: self.config.normal_bias,
            _padding: [0.0; 2],
        }
    }

    pub fn set_realm_point_vp_base(&mut self, point_vp_base: u32) {
        self.params_pool.write(0, &self.build_params(point_vp_base));
    }

    #[cfg(any(not(target_arch = "wasm32"), target_arch = "wasm32"))]
    pub fn new(device: &Device, queue: &Queue, table_capacity: u32) -> Self {
        let config = Shadow3dConfig::default();
        let atlas_desc = ShadowAtlasDesc {
            label: Some("Shadow Atlas"),
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            tile_px: config.tile_resolution,
            tiles_w: config.atlas_tiles_w,
            tiles_h: config.atlas_tiles_h,
            layers: config.atlas_layers,
        };

        let atlas = ShadowAtlasSystem::new(device, atlas_desc);
        let alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let storage_alignment = 0; // Tight packing: no dynamic offsets for storage buffers.

        let page_table =
            StorageBufferPool::new(device, queue, Some(table_capacity), storage_alignment);
        let point_light_vp = StorageBufferPool::new(device, queue, Some(4096), storage_alignment);
        let mut params_pool = UniformBufferPool::new(device, queue, Some(1), alignment);

        params_pool.write(
            0,
            &ShadowParams {
                virtual_grid_size: config.virtual_grid_size as f32,
                pcf_range: config.smoothing as i32,
                table_capacity,
                point_vp_base: 0,
                bias_min: 0.00001, // Very small default for Reverse Z + Float32
                bias_slope: 0.0001,
                point_bias_min: 0.001,
                point_bias_slope: 0.003,
                normal_bias: config.normal_bias,
                _padding: [0.0; 2],
            },
        );

        Self {
            atlas,
            page_table,
            point_light_vp,
            params_pool,
            table_capacity,
            cache: HashMap::new(),
            config,
            is_dirty: true,
        }
    }

    pub fn configure(&mut self, device: &Device, config: Shadow3dConfig) {
        let needs_atlas_rebuild = config.tile_resolution != self.config.tile_resolution
            || config.atlas_tiles_w != self.config.atlas_tiles_w
            || config.atlas_tiles_h != self.config.atlas_tiles_h
            || config.atlas_layers != self.config.atlas_layers;
        let grid_changed = config.virtual_grid_size != self.config.virtual_grid_size;
        let params_changed = grid_changed
            || config.smoothing != self.config.smoothing
            || config.normal_bias != self.config.normal_bias;

        self.config = config;

        self.params_pool.write(0, &self.build_params(0));

        if needs_atlas_rebuild {
            let atlas_desc = ShadowAtlasDesc {
                label: Some("Shadow Atlas"),
                format: TextureFormat::Depth32Float,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                tile_px: config.tile_resolution,
                tiles_w: config.atlas_tiles_w,
                tiles_h: config.atlas_tiles_h,
                layers: config.atlas_layers,
            };
            self.atlas = ShadowAtlasSystem::new(device, atlas_desc);
            self.cache.clear();
            self.is_dirty = true;
        } else if grid_changed {
            // Virtual grid change invalidates page keys.
            self.cache.clear();
            self.is_dirty = true;
        } else if params_changed {
            // Params-only changes still need a refresh of cached pages.
            self.is_dirty = true;
        }
    }

    /// Identifies which virtual pages are required for a given light and camera.

    /// Returns a list of (x, y) coordinates in the virtual grid.

    pub fn identify_required_pages(
        &self,
        light_view_proj: Mat4,
        camera_inv_view_proj: Mat4,
    ) -> Vec<(u32, u32)> {
        // 1. Get camera frustum corners in world space (NDC cube -> World).
        // WGPU uses depth range [0, 1]. In Reverse Z, near is 1.0 and far is 0.0.
        let ndc_corners = [
            glam::vec3(-1.0, -1.0, 1.0), // Near bottom left
            glam::vec3(1.0, -1.0, 1.0),  // Near bottom right
            glam::vec3(-1.0, 1.0, 1.0),  // Near top left
            glam::vec3(1.0, 1.0, 1.0),   // Near top right
            glam::vec3(-1.0, -1.0, 0.0), // Far bottom left
            glam::vec3(1.0, -1.0, 0.0),  // Far bottom right
            glam::vec3(-1.0, 1.0, 0.0),  // Far top left
            glam::vec3(1.0, 1.0, 0.0),   // Far top right
        ];

        let mut world_corners = [glam::Vec3::ZERO; 8];

        for i in 0..8 {
            let world_pos = camera_inv_view_proj * ndc_corners[i].extend(1.0);

            world_corners[i] = world_pos.xyz() / world_pos.w;
        }

        // 2. Transform world corners to light NDC space

        let mut min_ndc = glam::vec2(1.0, 1.0);

        let mut max_ndc = glam::vec2(-1.0, -1.0);

        for corner in world_corners {
            let light_ndc = light_view_proj * corner.extend(1.0);

            let ndc = light_ndc.xy() / light_ndc.w;

            min_ndc = min_ndc.min(ndc);

            max_ndc = max_ndc.max(ndc);
        }

        // 3. Clamp to light viewport [-1, 1]

        let min_x = min_ndc.x.max(-1.0);

        let max_x = max_ndc.x.min(1.0);

        let min_y = min_ndc.y.max(-1.0);

        let max_y = max_ndc.y.min(1.0);

        if min_x > max_x || min_y > max_y {
            return Vec::new();
        }

        // 4. Convert NDC to virtual grid coordinates

        // NDC X: -1.0 is left (0), 1.0 is right (s)

        // NDC Y: 1.0 is top (0), -1.0 is bottom (s)

        let s = self.config.virtual_grid_size as f32;

        let grid_min_x = (((min_x + 1.0) * 0.5) * s).floor() as u32;

        let grid_max_x = (((max_x + 1.0) * 0.5) * s).ceil() as u32;

        let grid_min_y = (((1.0 - max_y) * 0.5) * s).floor() as u32;

        let grid_max_y = (((1.0 - min_y) * 0.5) * s).ceil() as u32;

        let mut required = Vec::new();

        for y in grid_min_y..grid_max_y.min(self.config.virtual_grid_size) {
            for x in grid_min_x..grid_max_x.min(self.config.virtual_grid_size) {
                required.push((x, y));
            }
        }

        required
    }

    /// Calculates the View-Projection matrix for a specific virtual page.

    /// This "zooms in" the light's base projection to the specific page area.

    pub fn get_page_view_projection(
        &self,

        light_view: Mat4,

        light_proj: Mat4,

        x: u32,

        y: u32,
    ) -> Mat4 {
        // Calculate the range in NDC space [-1, 1] for this page

        let s = self.config.virtual_grid_size as f32;

        let x_min = -1.0 + (x as f32 * 2.0 / s);

        let x_max = -1.0 + ((x + 1) as f32 * 2.0 / s);

        // y=0 is top NDC (1.0)

        let y_max = 1.0 - (y as f32 * 2.0 / s);

        let y_min = 1.0 - ((y + 1) as f32 * 2.0 / s);

        // Create a scale and bias matrix to transform the base projection

        let scale_x = 2.0 / (x_max - x_min);

        let scale_y = 2.0 / (y_max - y_min);

        let offset_x = -(x_max + x_min) / (x_max - x_min);

        let offset_y = -(y_max + y_min) / (y_max - y_min);

        let custom_proj = Mat4::from_cols(
            glam::vec4(scale_x, 0.0, 0.0, 0.0),
            glam::vec4(0.0, scale_y, 0.0, 0.0),
            glam::vec4(0.0, 0.0, 1.0, 0.0),
            glam::vec4(offset_x, offset_y, 0.0, 1.0),
        );

        custom_proj * light_proj * light_view
    }

    /// Requests a tile for a specific virtual page.

    /// If the page is already cached, returns its handle.
    /// If not, tries to allocate a new one.
    pub fn request_page(
        &mut self,
        realm_tag: u32,
        light_id: u32,
        face: u32,
        x: u32,
        y: u32,
        frame_index: u64,
    ) -> Option<ShadowAtlasHandle> {
        let key = ShadowPageKey {
            realm_tag,
            light_id,
            face,
            x,
            y,
        };

        if let Some(record) = self.cache.get_mut(&key) {
            record.last_frame_used = frame_index;
            return Some(record.atlas_handle);
        }

        // New page needed
        if let Some((handle, relocations)) = self.atlas.alloc(1, 1) {
            // Handle relocations if repack happened
            for relocation in relocations {
                self.update_cache_after_relocation(relocation);
            }

            self.cache.insert(
                key,
                ShadowPageRecord {
                    atlas_handle: handle,
                    last_frame_used: frame_index,
                    is_dirty: true,
                },
            );

            return Some(handle);
        }

        None
    }

    fn update_cache_after_relocation(&mut self, relocation: ShadowAtlasRelocation) {
        for record in self.cache.values_mut() {
            if record.atlas_handle == relocation.handle {
                record.is_dirty = true; // Must re-render since it moved
            }
        }
    }

    fn compute_table_index(table_capacity: u32, grid: u32, key: ShadowPageKey) -> Option<u32> {
        let light_base = key.light_id.checked_mul(6)?.checked_add(key.face)?;
        let page_base = light_base.checked_mul(grid.checked_mul(grid)?)?;
        let page_offset = key.y.checked_mul(grid)?.checked_add(key.x)?;
        let linear = page_base.checked_add(page_offset)?;
        if linear < table_capacity {
            Some(linear)
        } else {
            None
        }
    }

    fn table_index(&self, key: ShadowPageKey) -> Option<u32> {
        Self::compute_table_index(self.table_capacity, self.config.virtual_grid_size, key)
    }

    fn write_table_for_realm(&mut self, realm_tag: u32) {
        let mut entries = vec![ShadowPageEntry::default(); self.table_capacity as usize];

        for (key, record) in &self.cache {
            if key.realm_tag != realm_tag {
                continue;
            }
            let Some(id) = self.table_index(*key) else {
                continue;
            };

            if let Some(transform) = self.atlas.get_uv_transform(record.atlas_handle) {
                entries[id as usize] = ShadowPageEntry {
                    scale_offset: glam::Vec4::new(
                        transform.0,
                        transform.1,
                        transform.2,
                        transform.3,
                    ),
                    layer_index: transform.4,
                    _padding: [0; 3],
                };
            }
        }

        self.page_table.write_slice(0, &entries);
    }

    /// Synchronizes the GPU page table with only one realm namespace.
    pub fn sync_table_for_realm(&mut self, realm_tag: u32) {
        self.write_table_for_realm(realm_tag);
    }

    /// Synchronizes the GPU page table with all cached namespaces.
    pub fn sync_table(&mut self) {
        let mut entries = vec![ShadowPageEntry::default(); self.table_capacity as usize];
        for realm_tag in [0_u32, 1_u32] {
            let mut realm_entries = vec![ShadowPageEntry::default(); self.table_capacity as usize];
            for (key, record) in &self.cache {
                if key.realm_tag != realm_tag {
                    continue;
                }
                let Some(id) = self.table_index(*key) else {
                    continue;
                };
                if let Some(transform) = self.atlas.get_uv_transform(record.atlas_handle) {
                    realm_entries[id as usize] = ShadowPageEntry {
                        scale_offset: glam::Vec4::new(
                            transform.0,
                            transform.1,
                            transform.2,
                            transform.3,
                        ),
                        layer_index: transform.4,
                        _padding: [0; 3],
                    };
                }
            }
            for (idx, entry) in realm_entries.into_iter().enumerate() {
                if entry.layer_index != u32::MAX {
                    entries[idx] = entry;
                }
            }
        }
        self.page_table.write_slice(0, &entries);
    }

    pub fn begin_frame(&mut self, frame_index: u64) {
        self.evict_stale_pages(frame_index, Self::MAX_IDLE_FRAMES);
        self.page_table.begin_frame(frame_index);
        self.point_light_vp.begin_frame(frame_index);
        self.params_pool.begin_frame(frame_index);
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn prune_inactive_dense_lights(&mut self, realm_tag: u32, active_shadow_light_count: u32) {
        let mut to_remove: Vec<(ShadowPageKey, ShadowAtlasHandle)> = Vec::new();
        for (key, record) in &self.cache {
            if key.realm_tag == realm_tag && key.light_id >= active_shadow_light_count {
                to_remove.push((*key, record.atlas_handle));
            }
        }
        if to_remove.is_empty() {
            return;
        }
        for (key, handle) in to_remove {
            self.cache.remove(&key);
            self.atlas.free(handle);
        }
        self.is_dirty = true;
    }

    fn evict_stale_pages(&mut self, current_frame: u64, max_idle_frames: u64) {
        if max_idle_frames == 0 {
            return;
        }
        let mut to_remove: Vec<(ShadowPageKey, ShadowAtlasHandle)> = Vec::new();
        for (key, record) in &self.cache {
            let idle_frames = current_frame.saturating_sub(record.last_frame_used);
            if idle_frames > max_idle_frames {
                to_remove.push((*key, record.atlas_handle));
            }
        }
        if to_remove.is_empty() {
            return;
        }
        for (key, handle) in to_remove {
            self.cache.remove(&key);
            self.atlas.free(handle);
        }
        self.is_dirty = true;
    }

    pub fn clear_dirty(&mut self) {
        self.is_dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::{ShadowManager3d, ShadowPageKey};

    #[test]
    fn compute_table_index_is_linear_and_bounded() {
        let key = ShadowPageKey {
            realm_tag: 1,
            light_id: 0,
            face: 2,
            x: 1,
            y: 0,
        };
        assert_eq!(ShadowManager3d::compute_table_index(16, 2, key), Some(9));
        let overflow = ShadowPageKey { light_id: 2, ..key };
        assert_eq!(ShadowManager3d::compute_table_index(16, 2, overflow), None);
    }
}
