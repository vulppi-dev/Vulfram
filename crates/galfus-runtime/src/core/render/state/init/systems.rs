use super::super::{BindingSystem, LightCullingSystem, RenderState, SamplerSet};
use crate::core::resources::shadow::ShadowManager3d;
use crate::core::resources::shadow2d::ShadowManager2d;
use crate::core::resources::{
    StorageBufferPool, UniformBufferPool, VertexAllocatorConfig, VertexAllocatorSystem,
};
use std::collections::HashMap;

impl RenderState {
    pub(crate) fn init_core_systems(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.vertex = Some(VertexAllocatorSystem::new(
            device,
            queue,
            VertexAllocatorConfig::default(),
        ));

        let alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let storage_alignment = device.limits().min_storage_buffer_offset_alignment as u64;

        // Initialize bindings
        self.bindings = Some(BindingSystem {
            frame_pool: UniformBufferPool::new(device, queue, Some(1), alignment),
            camera_pool: UniformBufferPool::new(device, queue, Some(128), alignment),
            shadow_camera_pool: UniformBufferPool::new(device, queue, Some(256), alignment),
            model_pool: UniformBufferPool::new(device, queue, Some(2048), alignment),
            instance_pool: StorageBufferPool::new(device, queue, Some(16384), 0),
            outline_instance_pool: StorageBufferPool::new(device, queue, Some(16384), 0),
            shadow_instance_pool: StorageBufferPool::new(device, queue, Some(16384), 0),
            material_3d_pool: UniformBufferPool::new(device, queue, Some(256), alignment),
            material_3d_inputs: StorageBufferPool::new(device, queue, Some(256), 0),
            bones_pool: StorageBufferPool::new(device, queue, Some(256), 0),
            shared_group: None,
            shadow_shared_group: None,
            model_bind_group: None,
            outline_model_bind_group: None,
            shadow_model_bind_group: None,
            pool_versions: HashMap::new(),
            last_with_shadows: false,
        });

        self.light_system = Some(LightCullingSystem {
            lights: StorageBufferPool::new(device, queue, Some(32), storage_alignment),
            visible_indices: StorageBufferPool::new(device, queue, Some(128), storage_alignment),
            visible_counts: StorageBufferPool::new(device, queue, Some(8), storage_alignment),
            camera_frustums: StorageBufferPool::new(device, queue, Some(96), storage_alignment),
            light_params: UniformBufferPool::new(device, queue, Some(16), alignment),
            params_buffer: Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("LightCull Params"),
                size: std::mem::size_of::<u32>() as u64 * 4,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })),
            bind_group: None,
            light_count: 0,
            camera_count: 0,
            max_lights_per_camera: 0,
            queue: queue.clone(),
        });

        self.shadow_3d = Some(ShadowManager3d::new(device, queue, 2048));
        self.shadow_2d = Some(ShadowManager2d::new());
    }

    pub(crate) fn init_samplers(&mut self, device: &wgpu::Device) -> SamplerSet {
        galfus_render::create_standard_samplers(device)
    }
}
