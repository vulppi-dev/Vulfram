use glam::{Vec2, Vec4};
use serde::{Deserialize, Serialize};

use crate::core::id_policy::validate_host_logical_id;
use crate::core::realm::{RealmId, RealmKind};
use crate::core::resources::common::{default_layer_mask, mark_realm_windows_dirty};
use crate::core::resources::{LightComponent, LightKind, LightRecord};
use crate::core::state::EngineState;
use crate::core::system::push_error_event;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdLightCreateArgs {
    pub realm_id: u32,
    pub light_id: u32,
    pub label: Option<String>,
    #[serde(default)]
    pub kind: Option<LightKind>,
    #[serde(default)]
    pub position: Option<Vec4>,
    #[serde(default)]
    pub direction: Option<Vec4>,
    #[serde(default)]
    pub color: Option<Vec4>,
    #[serde(default)]
    pub ground_color: Option<Vec4>,
    #[serde(default)]
    pub intensity: Option<f32>,
    #[serde(default)]
    pub range: Option<f32>,
    #[serde(default)]
    pub spot_inner_outer: Option<Vec2>,
    #[serde(default = "default_layer_mask")]
    pub layer_mask: u32,
    #[serde(default)]
    pub shadow_layer_mask: Option<u32>,
    #[serde(default = "crate::core::resources::common::default_true")]
    pub active: bool,
    #[serde(default = "crate::core::resources::common::default_true")]
    pub cast_shadow: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultLightCreate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdLightUpdateArgs {
    pub realm_id: u32,
    pub light_id: u32,
    pub label: Option<String>,
    pub kind: Option<LightKind>,
    pub position: Option<Vec4>,
    pub direction: Option<Vec4>,
    pub color: Option<Vec4>,
    pub ground_color: Option<Vec4>,
    pub intensity: Option<f32>,
    pub range: Option<f32>,
    pub spot_inner_outer: Option<Vec2>,
    pub layer_mask: Option<u32>,
    pub shadow_layer_mask: Option<u32>,
    pub active: Option<bool>,
    pub cast_shadow: Option<bool>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultLightUpdate {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdLight3dDisposeArgs {
    pub realm_id: u32,
    pub light_id: u32,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultLightDispose {
    pub success: bool,
    pub message: String,
}

fn resolve_realm_kind(engine: &EngineState, realm_id: RealmId) -> Result<RealmKind, String> {
    engine
        .universal_state
        .composition
        .realms
        .entries
        .get(&realm_id)
        .map(|realm| realm.value.kind)
        .ok_or_else(|| format!("Realm {} not found", realm_id.0))
}

pub fn engine_cmd_light_create(
    engine: &mut EngineState,
    args: &CmdLightCreateArgs,
) -> CmdResultLightCreate {
    if let Err(message) = validate_host_logical_id(args.realm_id, "realmId") {
        return CmdResultLightCreate {
            success: false,
            message,
        };
    }
    if let Err(message) = validate_host_logical_id(args.light_id, "lightId") {
        return CmdResultLightCreate {
            success: false,
            message,
        };
    }
    let realm_id = RealmId(args.realm_id);
    let realm_kind = match resolve_realm_kind(engine, realm_id) {
        Ok(kind) => kind,
        Err(message) => {
            return CmdResultLightCreate {
                success: false,
                message,
            };
        }
    };
    let light_exists = match realm_kind {
        RealmKind::ThreeD => engine
            .universal_state
            .scene
            .realm3d
            .entities
            .entry(realm_id)
            .or_default()
            .lights
            .contains_key(&args.light_id),
        RealmKind::TwoD => engine
            .universal_state
            .scene
            .realm2d
            .entities
            .entry(realm_id)
            .or_default()
            .lights
            .contains_key(&args.light_id),
    };
    if light_exists {
        let message = format!("Light with id {} already exists", args.light_id);
        push_error_event(
            engine,
            "light",
            message.clone(),
            None,
            Some("light3d-upsert".into()),
        );
        return CmdResultLightCreate {
            success: false,
            message,
        };
    }

    let kind = args.kind.unwrap_or(LightKind::Point);
    let position = args.position.unwrap_or(Vec4::new(0.0, 1.0, 0.0, 1.0));
    let direction = args.direction.unwrap_or(Vec4::new(0.0, -1.0, 0.0, 0.0));
    let color = args.color.unwrap_or(Vec4::new(1.0, 1.0, 1.0, 1.0));
    let ground_color = args.ground_color.unwrap_or(Vec4::new(0.0, 0.0, 0.0, 1.0));
    let intensity = args.intensity.unwrap_or(1.0);
    let range = args.range.unwrap_or(10.0);
    let spot_inner_outer = args.spot_inner_outer.unwrap_or(Vec2::new(0.5, 0.8));

    let component = LightComponent::new(
        position,
        direction,
        color,
        ground_color,
        intensity,
        range,
        spot_inner_outer,
        kind,
        args.cast_shadow,
    );
    let record = LightRecord::new(
        args.label.clone(),
        component,
        args.active,
        args.layer_mask,
        args.shadow_layer_mask.unwrap_or(args.layer_mask),
        args.cast_shadow,
    );
    match realm_kind {
        RealmKind::ThreeD => {
            engine
                .universal_state
                .scene
                .realm3d
                .entities
                .entry(realm_id)
                .or_default()
                .lights
                .insert(args.light_id, record);
        }
        RealmKind::TwoD => {
            engine
                .universal_state
                .scene
                .realm2d
                .entities
                .entry(realm_id)
                .or_default()
                .lights
                .insert(args.light_id, record);
        }
    }
    mark_realm_windows_dirty(engine, args.realm_id);
    galfus_log::galfus_log_debug!(
        engine,
        "realm.state",
        "light-created realm={} realm_kind={:?} light={} light_kind={:?} intensity={} range={} active={} layer_mask={} cast_shadow={}",
        args.realm_id,
        realm_kind,
        args.light_id,
        kind,
        intensity,
        range,
        args.active,
        args.layer_mask,
        args.cast_shadow
    );

    CmdResultLightCreate {
        success: true,
        message: "Light created successfully".into(),
    }
}

pub fn engine_cmd_light_update(
    engine: &mut EngineState,
    args: &CmdLightUpdateArgs,
) -> CmdResultLightUpdate {
    if let Err(message) = validate_host_logical_id(args.realm_id, "realmId") {
        return CmdResultLightUpdate {
            success: false,
            message,
        };
    }
    if let Err(message) = validate_host_logical_id(args.light_id, "lightId") {
        return CmdResultLightUpdate {
            success: false,
            message,
        };
    }
    let realm_id = RealmId(args.realm_id);
    let realm_kind = match resolve_realm_kind(engine, realm_id) {
        Ok(kind) => kind,
        Err(message) => {
            return CmdResultLightUpdate {
                success: false,
                message,
            };
        }
    };
    let record = match realm_kind {
        RealmKind::ThreeD => engine
            .universal_state
            .scene
            .realm3d
            .entities
            .get_mut(&realm_id)
            .and_then(|entities| entities.lights.get_mut(&args.light_id)),
        RealmKind::TwoD => engine
            .universal_state
            .scene
            .realm2d
            .entities
            .get_mut(&realm_id)
            .and_then(|entities| entities.lights.get_mut(&args.light_id)),
    };
    let Some(record) = record else {
        let message = format!("Light with id {} not found", args.light_id);
        push_error_event(
            engine,
            "light",
            message.clone(),
            None,
            Some("light3d-upsert".into()),
        );
        return CmdResultLightUpdate {
            success: false,
            message,
        };
    };

    if args.label.is_some() {
        record.label = args.label.clone();
    }
    if let Some(kind) = args.kind {
        record.data.kind_flags.x = kind.to_u32();
    }
    if let Some(cast_shadow) = args.cast_shadow {
        record.cast_shadow = cast_shadow;
        if cast_shadow {
            record.data.kind_flags.y |= LightComponent::FLAG_CAST_SHADOW;
        } else {
            record.data.kind_flags.y &= !LightComponent::FLAG_CAST_SHADOW;
        }
    }
    if let Some(position) = args.position {
        record.data.position = position;
    }
    if let Some(direction) = args.direction {
        record.data.direction = direction;
    }
    if let Some(color) = args.color {
        record.data.color = color;
    }
    if let Some(ground_color) = args.ground_color {
        record.data.ground_color = ground_color;
    }
    if let Some(intensity) = args.intensity {
        record.data.intensity_range.x = intensity;
    }
    if let Some(range) = args.range {
        record.data.intensity_range.y = range;
    }
    if let Some(spot_inner_outer) = args.spot_inner_outer {
        record.data.spot_inner_outer = spot_inner_outer;
    }
    if let Some(layer_mask) = args.layer_mask {
        record.layer_mask = layer_mask;
        if args.shadow_layer_mask.is_none() {
            record.shadow_layer_mask = layer_mask;
        }
    }
    if let Some(shadow_layer_mask) = args.shadow_layer_mask {
        record.shadow_layer_mask = shadow_layer_mask;
    }
    if let Some(active) = args.active {
        record.active = active;
    }

    record.data.update_matrices();
    record.mark_dirty();
    mark_realm_windows_dirty(engine, args.realm_id);

    CmdResultLightUpdate {
        success: true,
        message: "Light updated successfully".into(),
    }
}

pub fn engine_cmd_light_dispose(
    engine: &mut EngineState,
    args: &CmdLight3dDisposeArgs,
) -> CmdResultLightDispose {
    if let Err(message) = validate_host_logical_id(args.realm_id, "realmId") {
        return CmdResultLightDispose {
            success: false,
            message,
        };
    }
    if let Err(message) = validate_host_logical_id(args.light_id, "lightId") {
        return CmdResultLightDispose {
            success: false,
            message,
        };
    }
    let realm_id = RealmId(args.realm_id);
    let realm_kind = match resolve_realm_kind(engine, realm_id) {
        Ok(kind) => kind,
        Err(message) => {
            return CmdResultLightDispose {
                success: false,
                message,
            };
        }
    };
    let removed = match realm_kind {
        RealmKind::ThreeD => engine
            .universal_state
            .scene
            .realm3d
            .entities
            .get_mut(&realm_id)
            .and_then(|entities| entities.lights.remove(&args.light_id)),
        RealmKind::TwoD => engine
            .universal_state
            .scene
            .realm2d
            .entities
            .get_mut(&realm_id)
            .and_then(|entities| entities.lights.remove(&args.light_id)),
    };
    if removed.is_some() {
        if matches!(realm_kind, RealmKind::ThreeD) {
            for render_state in engine.render.states.values_mut() {
                if let Some(shadow) = render_state.shadow_3d.as_mut() {
                    shadow.mark_dirty();
                }
            }
        }
        mark_realm_windows_dirty(engine, args.realm_id);
        CmdResultLightDispose {
            success: true,
            message: "Light disposed successfully".into(),
        }
    } else {
        let message = format!("Light with id {} not found", args.light_id);
        push_error_event(
            engine,
            "light",
            message.clone(),
            None,
            Some("light3d-dispose".into()),
        );
        CmdResultLightDispose {
            success: false,
            message,
        }
    }
}
