use super::super::RenderState;
use crate::core::resources::{CameraComponent, FrameComponent};

impl RenderState {
    pub(crate) fn update_bind_groups(&mut self, device: &wgpu::Device, with_shadows: bool) {
        let bindings = self.bindings.as_mut().unwrap();
        let library = self.library.as_ref().unwrap();
        let light_system = self.light_system.as_ref().unwrap();
        let shadow_manager = self.shadow_3d.as_ref().unwrap();

        let forward_atlas_view = &library.fallback_forward_atlas_view;

        let shadow_atlas_view = if with_shadows {
            shadow_manager.atlas.view()
        } else {
            &library.fallback_shadow_view
        };

        let make_shared_group = |camera_buffer: &wgpu::Buffer,
                                 shadow_view: &wgpu::TextureView,
                                 label: &str| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout: &library.layout_shared,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: bindings.frame_pool.buffer(),
                            offset: 0,
                            size: Some(
                                std::num::NonZeroU64::new(
                                    std::mem::size_of::<FrameComponent>() as u64
                                )
                                .unwrap(),
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: camera_buffer,
                            offset: 0,
                            size: Some(
                                std::num::NonZeroU64::new(
                                    std::mem::size_of::<CameraComponent>() as u64
                                )
                                .unwrap(),
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: light_system.light_params.buffer(),
                            offset: 0,
                            size: Some(
                                std::num::NonZeroU64::new(std::mem::size_of::<
                                    super::super::light::LightDrawParams,
                                >()
                                    as u64)
                                .unwrap(),
                            ),
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: light_system.lights.buffer(),
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: light_system.visible_indices.buffer(),
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: light_system.visible_counts.buffer(),
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: shadow_manager.params_pool.buffer(),
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: wgpu::BindingResource::TextureView(shadow_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: shadow_manager.page_table.buffer(),
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 9,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: shadow_manager.point_light_vp.buffer(),
                            offset: 0,
                            size: None,
                        }),
                    },
                    wgpu::BindGroupEntry {
                        binding: 10,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.point_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 11,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_clamp),
                    },
                    wgpu::BindGroupEntry {
                        binding: 12,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.point_repeat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 13,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.linear_repeat),
                    },
                    wgpu::BindGroupEntry {
                        binding: 14,
                        resource: wgpu::BindingResource::Sampler(&library.samplers.comparison),
                    },
                    wgpu::BindGroupEntry {
                        binding: 15,
                        resource: wgpu::BindingResource::TextureView(forward_atlas_view),
                    },
                ],
            })
        };

        if bindings.shared_group.is_none() {
            bindings.shared_group = Some(make_shared_group(
                bindings.camera_pool.buffer(),
                shadow_atlas_view,
                &format!("BindGroup Shared (Consolidated, shadows={with_shadows})"),
            ));
        }
        if bindings.shadow_shared_group.is_none() {
            bindings.shadow_shared_group = Some(make_shared_group(
                bindings.shadow_camera_pool.buffer(),
                &library.fallback_shadow_view,
                &format!("BindGroup Shadow Shared (Consolidated, shadows={with_shadows})"),
            ));
        }

        // 3. Create Model Bind Group (Group 1)
        if bindings.model_bind_group.is_none() {
            bindings.model_bind_group =
                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("BindGroup Object (Dynamic Instance Data)"),
                    layout: &library.layout_object,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: bindings.instance_pool.buffer(),
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: bindings.bones_pool.buffer(),
                                offset: 0,
                                size: None,
                            }),
                        },
                    ],
                }));
        }

        // 3.1 Create Outline Model Bind Group (Group 1 for outline pass)
        if bindings.outline_model_bind_group.is_none() {
            bindings.outline_model_bind_group =
                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("BindGroup Outline Object (Dynamic Instance Data)"),
                    layout: &library.layout_object,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: bindings.outline_instance_pool.buffer(),
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: bindings.bones_pool.buffer(),
                                offset: 0,
                                size: None,
                            }),
                        },
                    ],
                }));
        }

        // 4. Create Shadow Model Bind Group (Group 1 for shadow pass)
        if bindings.shadow_model_bind_group.is_none() {
            bindings.shadow_model_bind_group =
                Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("BindGroup Shadow Object (Dynamic Instance Data)"),
                    layout: &library.layout_object,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: bindings.shadow_instance_pool.buffer(),
                                offset: 0,
                                size: None,
                            }),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: bindings.bones_pool.buffer(),
                                offset: 0,
                                size: None,
                            }),
                        },
                    ],
                }));
        }
    }
}
