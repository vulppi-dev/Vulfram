use std::collections::{HashMap, HashSet};

use crate::core::realm::{RealmId, RealmKind, SurfaceId, UniversalState};
use crate::core::resources::RenderTarget;
use crate::core::target::{
    dispose_realm_layers as dispose_target_realm_layers, dispose_surface_auto_links,
};

pub fn detach_realm_runtime(
    universal_state: &mut UniversalState,
    realm_id: RealmId,
    realm_kind: RealmKind,
) {
    let _ = realm_kind;
    universal_state.scene.realm3d.entities.remove(&realm_id);
    universal_state.scene.realm2d.entities.remove(&realm_id);
    universal_state
        .scene
        .realm2d_shadow_configs
        .remove(&realm_id);
    universal_state
        .targets
        .host_realm_index
        .retain(|_, indexed_realm_id| *indexed_realm_id != realm_id);
}

pub fn remove_connectors_for_realms(
    universal_state: &mut UniversalState,
    realm_ids: &HashSet<RealmId>,
    source_surfaces: &HashSet<SurfaceId>,
) {
    let mut removed_connectors = Vec::new();
    universal_state
        .composition
        .connectors
        .entries
        .retain(|connector_id, entry| {
            let remove = source_surfaces.contains(&entry.value.source_surface)
                || realm_ids.contains(&entry.value.target_realm);
            if remove {
                removed_connectors.push(*connector_id);
            }
            !remove
        });
    if removed_connectors.is_empty() {
        return;
    }

    let removed_set: HashSet<_> = removed_connectors.into_iter().collect();
    universal_state
        .interaction
        .input_routing
        .captures
        .retain(|_, capture| {
            !removed_set.contains(&crate::core::realm::ConnectorId(capture.connector_id))
        });
    universal_state
        .composition
        .surface_cache
        .last_good
        .retain(|connector_id, _| !removed_set.contains(connector_id));
    universal_state
        .composition
        .surface_cache
        .fallback
        .retain(|connector_id, _| !removed_set.contains(connector_id));
}

pub fn dispose_surface_links(
    universal_state: &mut UniversalState,
    realm_id: RealmId,
    surface_id: SurfaceId,
) {
    dispose_surface_auto_links(universal_state, realm_id, surface_id);

    universal_state
        .composition
        .surface_cache
        .last_good
        .retain(|_, source| *source != surface_id);
    universal_state
        .composition
        .surface_cache
        .fallback
        .retain(|_, source| *source != surface_id);
}

pub fn dispose_realm_layers(universal_state: &mut UniversalState, realm_id: RealmId) {
    dispose_target_realm_layers(universal_state, realm_id);
}

pub fn dispose_surfaces_for_window(
    universal_state: &mut UniversalState,
    surface_targets: &mut HashMap<SurfaceId, RenderTarget>,
    window_id: u32,
) {
    let surfaces_to_remove: Vec<_> = universal_state
        .composition
        .presents
        .entries
        .values()
        .filter(|present| present.value.window_id == window_id)
        .map(|present| present.value.surface)
        .collect();
    universal_state
        .composition
        .presents
        .remove_by_window(window_id);
    if surfaces_to_remove.is_empty() {
        return;
    }

    let surface_set: HashSet<_> = surfaces_to_remove.iter().copied().collect();
    let realms_to_remove: Vec<_> = universal_state
        .composition
        .realms
        .entries
        .iter()
        .filter_map(|(realm_id, entry)| {
            entry
                .value
                .output_surface
                .is_some_and(|surface| surface_set.contains(&surface))
                .then_some(*realm_id)
        })
        .collect();
    let realm_set: HashSet<_> = realms_to_remove.iter().copied().collect();

    for realm_id in realms_to_remove {
        if let Some(entry) = universal_state.composition.realms.remove(realm_id) {
            detach_realm_runtime(universal_state, realm_id, entry.value.kind);
        }
    }
    for surface_id in &surfaces_to_remove {
        universal_state.composition.surfaces.remove(*surface_id);
        surface_targets.remove(surface_id);
    }
    universal_state
        .targets
        .auto_links
        .retain(|_, link| !surface_set.contains(&link.surface_id));
    remove_connectors_for_realms(universal_state, &realm_set, &surface_set);
    universal_state
        .composition
        .surface_cache
        .last_good
        .retain(|_, source| !surface_set.contains(source));
    universal_state
        .composition
        .surface_cache
        .fallback
        .retain(|_, source| !surface_set.contains(source));
}

pub fn init_realm_runtime(
    universal_state: &mut UniversalState,
    realm_id: RealmId,
    kind: RealmKind,
) {
    if kind == RealmKind::TwoD {
        universal_state
            .scene
            .realm2d
            .entities
            .entry(realm_id)
            .or_default();
        universal_state
            .scene
            .realm2d_shadow_configs
            .entry(realm_id)
            .or_default();
    } else {
        universal_state
            .scene
            .realm3d
            .entities
            .entry(realm_id)
            .or_default();
    }
}
