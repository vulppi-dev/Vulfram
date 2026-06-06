use super::RenderState;
use crate::core::realm::RealmId;

pub(super) fn sync_scene_from_realm_and_universal_resources(
    render_state: &mut RenderState,
    universal: &crate::core::realm::UniversalState,
    realm_id: RealmId,
) {
    let previous_cameras = std::mem::take(&mut render_state.scene.cameras);
    render_state.detached_cameras.extend(previous_cameras);
    let live_camera_ids: std::collections::HashSet<u32> = universal
        .scene
        .realm3d
        .entities
        .values()
        .flat_map(|entities| entities.cameras.keys().copied())
        .collect();
    galfus_realm_3d::retain_records_by_ids(&mut render_state.detached_cameras, &live_camera_ids);
    render_state.scene.cameras.clear();

    if let Some(entities) = universal.scene.realm3d.entities.get(&realm_id) {
        for (camera_id, node) in &entities.cameras {
            let mut record = render_state
                .detached_cameras
                .remove(camera_id)
                .unwrap_or_else(|| node.to_render_record());
            let previous_data = record.data;
            let projection_plan = galfus_realm_3d::plan_camera_projection_update(
                previous_data.kind_flags.to_array(),
                previous_data.near_far,
                record.ortho_scale,
                node.data.kind_flags.to_array(),
                node.data.near_far,
                node.ortho_scale,
                [record.last_projection_size.x, record.last_projection_size.y],
            );
            record.label = node.label.clone();
            record.data = node.data;
            if projection_plan.preserve_runtime_projection {
                // Keep runtime projection/aspect from the resolved target size.
                record.data.projection = previous_data.projection;
                record.data.view_projection = record.data.projection * record.data.view;
            } else if projection_plan.reset_projection_size {
                // Force projection rebuild on this frame with the current target size.
                record.last_projection_size = glam::UVec2::ZERO;
            }
            record.layer_mask = node.layer_mask;
            record.order = node.order;
            record.ortho_scale = node.ortho_scale;
            record.view_position = node.view_position.clone();
            record.mark_dirty();
            render_state.scene.cameras.insert(*camera_id, record);
        }
        galfus_realm_3d::rebuild_record_map(
            &mut render_state.scene.models,
            &entities.models,
            |record, node| {
                let current_meta = galfus_realm_3d::ModelRecordMeta {
                    transform: record.data.transform.to_cols_array(),
                    translation: record.data.translation,
                    rotation: record.data.rotation,
                    scale: record.data.scale,
                    flags: record.data.flags.to_array(),
                    outline_color: record.data.outline_color,
                    geometry_id: record.geometry_id,
                    material_id: record.material_id,
                    active: record.active,
                    layer_mask: record.layer_mask,
                    cast_shadow: record.cast_shadow,
                    receive_shadow: record.receive_shadow,
                    cast_outline: record.cast_outline,
                };
                let next_meta = galfus_realm_3d::ModelRecordMeta {
                    transform: node.data.transform.to_cols_array(),
                    translation: node.data.translation,
                    rotation: node.data.rotation,
                    scale: node.data.scale,
                    flags: node.data.flags.to_array(),
                    outline_color: node.data.outline_color,
                    geometry_id: node.geometry_id,
                    material_id: node.material_id,
                    active: node.active,
                    layer_mask: node.layer_mask,
                    cast_shadow: node.cast_shadow,
                    receive_shadow: node.receive_shadow,
                    cast_outline: node.cast_outline,
                };
                record.label = node.label.clone();
                record.data = node.data;
                record.geometry_id = node.geometry_id;
                record.material_id = node.material_id;
                record.active = node.active;
                record.layer_mask = node.layer_mask;
                record.cast_shadow = node.cast_shadow;
                record.receive_shadow = node.receive_shadow;
                record.cast_outline = node.cast_outline;
                let update_plan =
                    galfus_realm_3d::plan_model_record_update(&current_meta, &next_meta);
                if update_plan.mark_dirty {
                    record.mark_dirty();
                }
            },
        );
        galfus_realm_3d::rebuild_record_map(
            &mut render_state.scene.lights,
            &entities.lights,
            |record, node| {
                let current_meta = galfus_realm_3d::LightRecordMeta {
                    position: record.data.position,
                    direction: record.data.direction,
                    color: record.data.color,
                    ground_color: record.data.ground_color,
                    view: record.data.view.to_cols_array(),
                    projection: record.data.projection.to_cols_array(),
                    view_projection: record.data.view_projection.to_cols_array(),
                    intensity_range: record.data.intensity_range,
                    spot_inner_outer: record.data.spot_inner_outer,
                    kind_flags: record.data.kind_flags.to_array(),
                    active: record.active,
                    layer_mask: record.layer_mask,
                    shadow_layer_mask: record.shadow_layer_mask,
                    cast_shadow: record.cast_shadow,
                };
                let next_meta = galfus_realm_3d::LightRecordMeta {
                    position: node.data.position,
                    direction: node.data.direction,
                    color: node.data.color,
                    ground_color: node.data.ground_color,
                    view: node.data.view.to_cols_array(),
                    projection: node.data.projection.to_cols_array(),
                    view_projection: node.data.view_projection.to_cols_array(),
                    intensity_range: node.data.intensity_range,
                    spot_inner_outer: node.data.spot_inner_outer,
                    kind_flags: node.data.kind_flags.to_array(),
                    active: node.active,
                    layer_mask: node.layer_mask,
                    shadow_layer_mask: node.shadow_layer_mask,
                    cast_shadow: node.cast_shadow,
                };
                record.label = node.label.clone();
                record.data = node.data;
                record.active = node.active;
                record.layer_mask = node.layer_mask;
                record.shadow_layer_mask = node.shadow_layer_mask;
                record.cast_shadow = node.cast_shadow;
                let update_plan =
                    galfus_realm_3d::plan_light_record_update(&current_meta, &next_meta);
                if update_plan.mark_dirty {
                    record.mark_dirty();
                }
            },
        );
    } else {
        render_state.scene.models.clear();
        render_state.scene.lights.clear();
    }

    render_state.two_d_source.cameras.clear();
    render_state.two_d_source.lights.clear();
    render_state.two_d_source.sprites.clear();
    render_state.two_d_source.shapes.clear();
    render_state.two_d_source.shadow_config = universal
        .scene
        .realm2d_shadow_configs
        .get(&realm_id)
        .copied()
        .unwrap_or_default();
    if let Some(entities) = universal.scene.realm2d.entities.get(&realm_id) {
        render_state.two_d_source.cameras.extend(
            entities
                .cameras
                .iter()
                .map(|(id, record)| (*id, record.clone())),
        );
        render_state.two_d_source.lights.extend(
            entities
                .lights
                .iter()
                .map(|(id, record)| (*id, record.clone())),
        );
        render_state.two_d_source.sprites.extend(
            entities
                .sprites
                .iter()
                .map(|(id, record)| (*id, record.clone())),
        );
        render_state.two_d_source.shapes.extend(
            entities
                .shapes
                .iter()
                .map(|(id, record)| (*id, record.clone())),
        );
    }

    galfus_realm_3d::rebuild_record_map(
        &mut render_state.scene.materials,
        &universal.scene.realm3d.materials,
        |record, node| {
            let current_data_bytes = match record.preset {
                crate::core::resources::ShaderMaterialPreset::Standard => {
                    bytemuck::bytes_of(&record.data_standard).to_vec()
                }
                crate::core::resources::ShaderMaterialPreset::Pbr => {
                    bytemuck::bytes_of(&record.data_pbr).to_vec()
                }
            };
            let next_data_bytes = match node.preset {
                crate::core::resources::ShaderMaterialPreset::Standard => {
                    bytemuck::bytes_of(&node.data_standard).to_vec()
                }
                crate::core::resources::ShaderMaterialPreset::Pbr => {
                    bytemuck::bytes_of(&node.data_pbr).to_vec()
                }
            };
            let current_meta = galfus_realm_3d::MaterialRecordMeta {
                label: record.label.clone(),
                data_bytes: current_data_bytes,
                inputs_bytes: bytemuck::cast_slice(record.inputs.as_slice()).to_vec(),
                texture_ids: record.texture_ids.to_vec(),
                surface_type: record.surface_type as u32,
                topology: record.topology as u32,
                polygon_mode: record.polygon_mode as u32,
            };
            let next_meta = galfus_realm_3d::MaterialRecordMeta {
                label: node.label.clone(),
                data_bytes: next_data_bytes,
                inputs_bytes: bytemuck::cast_slice(node.inputs.as_slice()).to_vec(),
                texture_ids: node.texture_ids.to_vec(),
                surface_type: node.surface_type as u32,
                topology: node.topology as u32,
                polygon_mode: node.polygon_mode as u32,
            };
            record.label = node.label.clone();
            record.preset = node.preset;
            record.data_standard = node.data_standard;
            record.data_pbr = node.data_pbr;
            record.inputs = node.inputs.clone();
            record.texture_ids = node.texture_ids;
            record.surface_type = node.surface_type;
            record.topology = node.topology;
            record.polygon_mode = node.polygon_mode;
            record.render_side = node.render_side;
            let update_plan =
                galfus_realm_3d::plan_material_record_update(&current_meta, &next_meta);
            if update_plan.mark_dirty {
                record.mark_dirty();
            }
            if update_plan.reset_bind_group {
                record.bind_group = None;
            }
        },
    );
    let textures_hash = hash_texture_records(&universal.scene.render_resources.textures);
    if render_state.textures_sync_hash != textures_hash {
        sync_texture_records(
            &mut render_state.scene.textures,
            &universal.scene.render_resources.textures,
        );
        render_state.textures_sync_hash = textures_hash;
    }
    let atlas_hash =
        hash_forward_atlas_entries(&universal.scene.render_resources.forward_atlas_entries);
    if render_state.atlas_sync_hash != atlas_hash {
        sync_forward_atlas_entries(
            &mut render_state.scene.forward_atlas_entries,
            &universal.scene.render_resources.forward_atlas_entries,
        );
        render_state.atlas_sync_hash = atlas_hash;
    }
    let binds_hash =
        hash_target_texture_binds(&universal.scene.render_resources.target_texture_binds);
    if render_state.target_binds_sync_hash != binds_hash {
        sync_target_texture_binds(
            &mut render_state.target_texture_binds,
            &universal.scene.render_resources.target_texture_binds,
        );
        render_state.target_binds_sync_hash = binds_hash;
    }
}

fn hash_texture_records(
    textures: &std::collections::HashMap<u32, crate::core::resources::TextureRecord>,
) -> u64 {
    galfus_realm_3d::hash_map_by_meta(
        textures,
        |id, record| {
            let size = record._texture.size();
            galfus_realm_3d::TextureRecordMeta {
                id,
                label: record.label.clone(),
                width: size.width,
                height: size.height,
                depth_or_array_layers: size.depth_or_array_layers,
                format: format!("{:?}", record._texture.format()),
            }
        },
        galfus_realm_3d::hash_texture_records,
    )
}

fn hash_forward_atlas_entries(
    entries: &std::collections::HashMap<u32, crate::core::resources::ForwardAtlasEntry>,
) -> u64 {
    galfus_realm_3d::hash_map_by_meta(
        entries,
        |id, entry| galfus_realm_3d::ForwardAtlasEntryMeta {
            id,
            label: entry.label.clone(),
            layer: entry.layer,
            uv_scale_bias: entry.uv_scale_bias,
        },
        galfus_realm_3d::hash_forward_atlas_entries,
    )
}

fn hash_target_texture_binds(
    binds: &std::collections::HashMap<u32, crate::core::resources::TargetTextureBinding>,
) -> u64 {
    galfus_realm_3d::hash_map_by_meta(
        binds,
        |texture_id, bind| galfus_realm_3d::TargetTextureBindingMeta {
            texture_id,
            target_id: bind.target_id,
            label: bind.label.clone(),
        },
        galfus_realm_3d::hash_target_texture_binds,
    )
}

fn sync_texture_records(
    current: &mut std::collections::HashMap<u32, crate::core::resources::TextureRecord>,
    next: &std::collections::HashMap<u32, crate::core::resources::TextureRecord>,
) {
    galfus_realm_3d::sync_map_by_meta(
        current,
        next,
        |id, record| {
            let size = record._texture.size();
            galfus_realm_3d::TextureRecordMeta {
                id,
                label: record.label.clone(),
                width: size.width,
                height: size.height,
                depth_or_array_layers: size.depth_or_array_layers,
                format: format!("{:?}", record._texture.format()),
            }
        },
        galfus_realm_3d::plan_texture_record_sync,
    );
}

fn sync_forward_atlas_entries(
    current: &mut std::collections::HashMap<u32, crate::core::resources::ForwardAtlasEntry>,
    next: &std::collections::HashMap<u32, crate::core::resources::ForwardAtlasEntry>,
) {
    galfus_realm_3d::sync_map_by_meta(
        current,
        next,
        |id, entry| galfus_realm_3d::ForwardAtlasEntryMeta {
            id,
            label: entry.label.clone(),
            layer: entry.layer,
            uv_scale_bias: entry.uv_scale_bias,
        },
        galfus_realm_3d::plan_forward_atlas_sync,
    );
}

fn sync_target_texture_binds(
    current: &mut std::collections::HashMap<u32, crate::core::resources::TargetTextureBinding>,
    next: &std::collections::HashMap<u32, crate::core::resources::TargetTextureBinding>,
) {
    galfus_realm_3d::sync_map_by_meta(
        current,
        next,
        |texture_id, bind| galfus_realm_3d::TargetTextureBindingMeta {
            texture_id,
            target_id: bind.target_id,
            label: bind.label.clone(),
        },
        galfus_realm_3d::plan_target_texture_bind_sync,
    );
}

pub(super) fn sync_window_geometry_registry(
    render_state: &mut RenderState,
    geometries: &std::collections::HashMap<u32, crate::core::render::UniversalGeometryRecord>,
) {
    let Some(vertex) = render_state.vertex.as_mut() else {
        return;
    };
    let current_ids: Vec<u32> = vertex.records().keys().copied().collect();
    let next_ids: Vec<u32> = geometries.keys().copied().collect();
    let plan = galfus_realm_3d::plan_geometry_registry_sync(&current_ids, &next_ids);

    for geometry_id in plan.replace_ids {
        let Some(record) = geometries.get(&geometry_id) else {
            continue;
        };
        let _ = vertex.create_geometry(geometry_id, record.label.clone(), record.entries.clone());
    }

    for geometry_id in plan.stale_ids {
        let _ = vertex.destroy_geometry(geometry_id);
    }
}
