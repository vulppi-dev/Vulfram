mod bindings;
mod lights;
mod materials;

use super::RenderState;
use crate::core::resources::FrameComponent;

impl RenderState {
    pub fn prepare_render(
        &mut self,
        device: &wgpu::Device,
        frame_spec: FrameComponent,
        with_shadows: bool,
    ) {
        // 0. Cache sorted cameras for this tick
        self.camera_order.clear();
        self.camera_order.extend(self.scene.cameras.keys().copied());
        self.camera_order.sort_by_key(|id| {
            self.scene
                .cameras
                .get(id)
                .map(|record| (record.order, *id))
                .unwrap_or((0, *id))
        });
        self.camera_uniform_slots.clear();
        self.camera_uniform_slots.extend(
            self.camera_order
                .iter()
                .copied()
                .enumerate()
                .map(|(slot, camera_id)| (camera_id, slot as u32)),
        );

        // 1. Pre-prepare lights
        self.prepare_lights(device);

        let bindings = match self.bindings.as_mut() {
            Some(b) => b,
            None => return,
        };

        if self.library.is_none() {
            return;
        }

        // 2. Upload global data
        bindings.frame_pool.write(0, &frame_spec);

        let mut any_pool_resized = false;
        let mut check_pool = |name: &'static str, current_version: u64| {
            let entry = bindings
                .pool_versions
                .entry(name)
                .or_insert(current_version);
            if *entry != current_version {
                *entry = current_version;
                any_pool_resized = true;
            }
        };

        check_pool("frame", bindings.frame_pool.version());
        check_pool("camera", bindings.camera_pool.version());
        check_pool("shadow_camera", bindings.shadow_camera_pool.version());
        check_pool("model", bindings.model_pool.version());
        check_pool("instance", bindings.instance_pool.version());
        check_pool("outline_instance", bindings.outline_instance_pool.version());
        check_pool("shadow_instance", bindings.shadow_instance_pool.version());
        check_pool("mat_3d", bindings.material_3d_pool.version());
        check_pool("mat_3d_in", bindings.material_3d_inputs.version());
        check_pool("bones", bindings.bones_pool.version());

        if let Some(light_system) = self.light_system.as_ref() {
            check_pool("light_cull", light_system.lights.version());
            check_pool("light_indices", light_system.visible_indices.version());
            check_pool("light_counts", light_system.visible_counts.version());
            check_pool("light_params", light_system.light_params.version());
        }

        if let Some(shadow_manager) = self.shadow_3d.as_ref() {
            check_pool("shadow_params", shadow_manager.params_pool.version());
            check_pool("shadow_page_table", shadow_manager.page_table.version());
            check_pool("shadow_point_vp", shadow_manager.point_light_vp.version());
        }

        if with_shadows != bindings.last_with_shadows {
            bindings.last_with_shadows = with_shadows;
            bindings.shared_group = None;
            bindings.shadow_shared_group = None;
        }

        if any_pool_resized {
            bindings.shared_group = None;
            bindings.shadow_shared_group = None;
            bindings.model_bind_group = None;
            bindings.outline_model_bind_group = None;
            bindings.shadow_model_bind_group = None;
            // Clear material bind groups
            for mat in self.scene.materials.values_mut() {
                mat.bind_group = None;
            }
        }

        // 3. Upload camera data
        for (id, record) in &mut self.scene.cameras {
            if record.is_dirty {
                let Some(slot) = self.camera_uniform_slots.get(id).copied() else {
                    continue;
                };
                bindings.camera_pool.write(slot, &record.data);
                record.clear_dirty();
            }
        }

        // 4. Upload model data
        for (id, record) in &mut self.scene.models {
            if record.is_dirty {
                bindings.model_pool.write(*id, &record.data);
                record.clear_dirty();
                if let Some(shadow) = self.shadow_3d.as_mut() {
                    shadow.mark_dirty();
                }
            }
        }

        // 5. Update and upload materials
        self.prepare_materials(device);

        // 6. Build/Update bind groups
        self.update_bind_groups(device, with_shadows);
    }
}
