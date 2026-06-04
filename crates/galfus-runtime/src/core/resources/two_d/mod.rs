use glam::Mat4;
use serde::{Deserialize, Serialize};

use crate::core::id_policy::validate_host_logical_id;
use crate::core::realm::{RealmId, RealmKind};
use crate::core::resources::common::mark_realm_windows_dirty;
use crate::core::state::EngineState;
use crate::core::system::push_error_event;

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Realm2dShadowQualityPreset {
    Performance,
    Balanced,
    Quality,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Realm2dShadowConfig {
    pub softness: f32,
    pub shadow_contact_offset: f32,
    pub shadow_back_gradient_strength: f32,
    pub shadow_debug_light_index: i32,
    pub shadow_debug_mode: i32,
    pub ambient: f32,
    pub light_radius: f32,
    pub quality_preset: Realm2dShadowQualityPreset,
    pub max_shadow_updates_per_frame: u32,
    pub angular_resolution: u32,
    pub map_resolution: u32,
}

impl Default for Realm2dShadowConfig {
    fn default() -> Self {
        Self {
            softness: 0.08,
            shadow_contact_offset: 0.0,
            shadow_back_gradient_strength: 0.0,
            shadow_debug_light_index: -1,
            shadow_debug_mode: 0,
            ambient: 0.06,
            light_radius: 0.25,
            quality_preset: Realm2dShadowQualityPreset::Balanced,
            max_shadow_updates_per_frame: 16,
            angular_resolution: 1024,
            map_resolution: 512,
        }
    }
}

impl Realm2dShadowConfig {
    pub fn sanitized(self) -> Self {
        Self {
            softness: self.softness.max(0.0),
            shadow_contact_offset: self.shadow_contact_offset.max(0.0),
            shadow_back_gradient_strength: self.shadow_back_gradient_strength.max(0.0),
            shadow_debug_light_index: self.shadow_debug_light_index.max(-1),
            shadow_debug_mode: self.shadow_debug_mode.clamp(0, 1),
            ambient: self.ambient.max(0.0),
            light_radius: self.light_radius.max(0.01),
            quality_preset: self.quality_preset,
            max_shadow_updates_per_frame: self.max_shadow_updates_per_frame.clamp(1, 64),
            angular_resolution: self.angular_resolution.clamp(128, 4096),
            map_resolution: self.map_resolution.clamp(64, 2048),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Camera2dRecord {
    pub label: Option<String>,
    pub transform: Mat4,
    pub near_far: glam::Vec2,
    pub ortho_scale: f32,
    pub layer_mask: u32,
    pub order: i32,
}

#[derive(Debug, Clone)]
pub struct Sprite2dRecord {
    pub label: Option<String>,
    pub transform: Mat4,
    pub geometry_id: u32,
    pub material_id: Option<u32>,
    pub layer: i32,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
    pub occluder_only: bool,
    pub shadow_height: f32,
    pub shadow_layer_mask: u32,
}

#[derive(Debug, Clone)]
pub struct Shape2dRecord {
    pub label: Option<String>,
    pub transform: Mat4,
    pub geometry_id: u32,
    pub material_id: Option<u32>,
    pub layer: i32,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
    pub occluder_only: bool,
    pub shadow_height: f32,
    pub shadow_layer_mask: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdCamera2dCreateArgs {
    pub realm_id: u32,
    pub camera_id: u32,
    pub label: Option<String>,
    pub transform: Mat4,
    pub near_far: glam::Vec2,
    #[serde(default = "default_ortho_scale")]
    pub ortho_scale: f32,
    #[serde(default = "default_layer_mask")]
    pub layer_mask: u32,
    #[serde(default)]
    pub order: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdCamera2dUpdateArgs {
    pub realm_id: u32,
    pub camera_id: u32,
    pub label: Option<String>,
    pub transform: Option<Mat4>,
    pub near_far: Option<glam::Vec2>,
    pub ortho_scale: Option<f32>,
    pub layer_mask: Option<u32>,
    pub order: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdCamera2dUpsertArgs {
    Create(CmdCamera2dCreateArgs),
    Update(CmdCamera2dUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdCamera2dDisposeArgs {
    pub realm_id: u32,
    pub camera_id: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdSprite2dCreateArgs {
    pub realm_id: u32,
    pub sprite_id: u32,
    pub label: Option<String>,
    pub transform: Mat4,
    pub geometry_id: u32,
    #[serde(default)]
    pub material_id: Option<u32>,
    #[serde(default)]
    pub layer: i32,
    #[serde(default = "crate::core::resources::common::default_true")]
    pub cast_shadow: bool,
    #[serde(default = "crate::core::resources::common::default_true")]
    pub receive_shadow: bool,
    #[serde(default)]
    pub occluder_only: bool,
    #[serde(default = "default_shadow_height")]
    pub shadow_height: f32,
    #[serde(default = "default_shadow_layer_mask")]
    pub shadow_layer_mask: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdSprite2dUpdateArgs {
    pub realm_id: u32,
    pub sprite_id: u32,
    pub label: Option<String>,
    pub transform: Option<Mat4>,
    pub geometry_id: Option<u32>,
    #[serde(default)]
    pub material_id: Option<u32>,
    pub layer: Option<i32>,
    pub cast_shadow: Option<bool>,
    pub receive_shadow: Option<bool>,
    pub occluder_only: Option<bool>,
    pub shadow_height: Option<f32>,
    pub shadow_layer_mask: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdSprite2dUpsertArgs {
    Create(CmdSprite2dCreateArgs),
    Update(CmdSprite2dUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdSprite2dDisposeArgs {
    pub realm_id: u32,
    pub sprite_id: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdShape2dCreateArgs {
    pub realm_id: u32,
    pub shape_id: u32,
    pub label: Option<String>,
    pub transform: Mat4,
    pub geometry_id: u32,
    #[serde(default)]
    pub material_id: Option<u32>,
    #[serde(default)]
    pub layer: i32,
    #[serde(default = "crate::core::resources::common::default_true")]
    pub cast_shadow: bool,
    #[serde(default = "crate::core::resources::common::default_true")]
    pub receive_shadow: bool,
    #[serde(default)]
    pub occluder_only: bool,
    #[serde(default = "default_shadow_height")]
    pub shadow_height: f32,
    #[serde(default = "default_shadow_layer_mask")]
    pub shadow_layer_mask: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdShape2dUpdateArgs {
    pub realm_id: u32,
    pub shape_id: u32,
    pub label: Option<String>,
    pub transform: Option<Mat4>,
    pub geometry_id: Option<u32>,
    #[serde(default)]
    pub material_id: Option<u32>,
    pub layer: Option<i32>,
    pub cast_shadow: Option<bool>,
    pub receive_shadow: Option<bool>,
    pub occluder_only: Option<bool>,
    pub shadow_height: Option<f32>,
    pub shadow_layer_mask: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum CmdShape2dUpsertArgs {
    Create(CmdShape2dCreateArgs),
    Update(CmdShape2dUpdateArgs),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdShape2dDisposeArgs {
    pub realm_id: u32,
    pub shape_id: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdRealm2dShadowConfigUpdateArgs {
    pub realm_id: u32,
    pub softness: Option<f32>,
    pub shadow_contact_offset: Option<f32>,
    pub shadow_back_gradient_strength: Option<f32>,
    pub shadow_debug_light_index: Option<i32>,
    pub shadow_debug_mode: Option<i32>,
    pub ambient: Option<f32>,
    pub light_radius: Option<f32>,
    pub quality_preset: Option<Realm2dShadowQualityPreset>,
    pub max_shadow_updates_per_frame: Option<u32>,
    pub angular_resolution: Option<u32>,
    pub map_resolution: Option<u32>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTwoDUpsert {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultTwoDDispose {
    pub success: bool,
    pub message: String,
}

fn default_ortho_scale() -> f32 {
    1.0
}

fn default_layer_mask() -> u32 {
    1
}

fn default_shadow_height() -> f32 {
    1.0
}

fn default_shadow_layer_mask() -> u32 {
    u32::MAX
}

fn default_realm2d_shadow_config() -> Realm2dShadowConfig {
    Realm2dShadowConfig::default()
}

fn ensure_realm_is_2d(
    engine: &mut EngineState,
    realm_id: RealmId,
    command: &str,
) -> Result<(), String> {
    let Some(realm) = engine
        .universal_state
        .composition
        .realms
        .entries
        .get(&realm_id)
    else {
        return Err(format!("Realm {} not found", realm_id.0));
    };
    if realm.value.kind != RealmKind::TwoD {
        return Err(format!("Realm {} is not a two-d realm", realm_id.0));
    }
    let _ = command;
    Ok(())
}

fn upsert_error(
    engine: &mut EngineState,
    scope: &str,
    command: &str,
    message: String,
) -> CmdResultTwoDUpsert {
    push_error_event(
        engine,
        scope,
        message.clone(),
        None,
        Some(command.to_string()),
    );
    CmdResultTwoDUpsert {
        success: false,
        message,
    }
}

fn dispose_error(
    engine: &mut EngineState,
    scope: &str,
    command: &str,
    message: String,
) -> CmdResultTwoDDispose {
    push_error_event(
        engine,
        scope,
        message.clone(),
        None,
        Some(command.to_string()),
    );
    CmdResultTwoDDispose {
        success: false,
        message,
    }
}

fn validate_realm_entity_ids(
    realm_id: u32,
    entity_id: u32,
    realm_field: &str,
    entity_field: &str,
) -> Result<(), String> {
    validate_host_logical_id(realm_id, realm_field)?;
    validate_host_logical_id(entity_id, entity_field)?;
    Ok(())
}

pub fn engine_cmd_camera2d_upsert(
    engine: &mut EngineState,
    args: CmdCamera2dUpsertArgs,
) -> CmdResultTwoDUpsert {
    match args {
        CmdCamera2dUpsertArgs::Create(create) => {
            if let Err(message) =
                validate_realm_entity_ids(create.realm_id, create.camera_id, "realmId", "cameraId")
            {
                return upsert_error(engine, "camera2d", "camera2d-upsert", message);
            }
            let realm_id = RealmId(create.realm_id);
            if let Err(message) = ensure_realm_is_2d(engine, realm_id, "camera2d-upsert") {
                return upsert_error(engine, "camera2d", "camera2d-upsert", message);
            }
            let entities = engine
                .universal_state
                .scene
                .realm2d
                .entities
                .entry(realm_id)
                .or_default();
            if entities.cameras.contains_key(&create.camera_id) {
                return upsert_error(
                    engine,
                    "camera2d",
                    "camera2d-upsert",
                    format!("Camera2D with id {} already exists", create.camera_id),
                );
            }
            entities.cameras.insert(
                create.camera_id,
                Camera2dRecord {
                    label: create.label,
                    transform: create.transform,
                    near_far: create.near_far,
                    ortho_scale: create.ortho_scale,
                    layer_mask: create.layer_mask,
                    order: create.order,
                },
            );
            mark_realm_windows_dirty(engine, create.realm_id);
            CmdResultTwoDUpsert {
                success: true,
                message: "Camera2D upserted successfully".to_string(),
            }
        }
        CmdCamera2dUpsertArgs::Update(update) => {
            if let Err(message) =
                validate_realm_entity_ids(update.realm_id, update.camera_id, "realmId", "cameraId")
            {
                return upsert_error(engine, "camera2d", "camera2d-upsert", message);
            }
            let realm_id = RealmId(update.realm_id);
            if let Err(message) = ensure_realm_is_2d(engine, realm_id, "camera2d-upsert") {
                return upsert_error(engine, "camera2d", "camera2d-upsert", message);
            }
            let Some(entities) = engine
                .universal_state
                .scene
                .realm2d
                .entities
                .get_mut(&realm_id)
            else {
                return upsert_error(
                    engine,
                    "camera2d",
                    "camera2d-upsert",
                    format!("Realm {} not found", update.realm_id),
                );
            };
            let Some(record) = entities.cameras.get_mut(&update.camera_id) else {
                return upsert_error(
                    engine,
                    "camera2d",
                    "camera2d-upsert",
                    format!("Camera2D with id {} not found", update.camera_id),
                );
            };
            if update.label.is_some() {
                record.label = update.label;
            }
            if let Some(transform) = update.transform {
                record.transform = transform;
            }
            if let Some(near_far) = update.near_far {
                record.near_far = near_far;
            }
            if let Some(ortho_scale) = update.ortho_scale {
                record.ortho_scale = ortho_scale;
            }
            if let Some(layer_mask) = update.layer_mask {
                record.layer_mask = layer_mask;
            }
            if let Some(order) = update.order {
                record.order = order;
            }
            mark_realm_windows_dirty(engine, update.realm_id);
            CmdResultTwoDUpsert {
                success: true,
                message: "Camera2D upserted successfully".to_string(),
            }
        }
    }
}

pub fn engine_cmd_camera2d_dispose(
    engine: &mut EngineState,
    args: &CmdCamera2dDisposeArgs,
) -> CmdResultTwoDDispose {
    if let Err(message) =
        validate_realm_entity_ids(args.realm_id, args.camera_id, "realmId", "cameraId")
    {
        return dispose_error(engine, "camera2d", "camera2d-dispose", message);
    }
    let realm_id = RealmId(args.realm_id);
    if let Err(message) = ensure_realm_is_2d(engine, realm_id, "camera2d-dispose") {
        return dispose_error(engine, "camera2d", "camera2d-dispose", message);
    }
    let Some(entities) = engine
        .universal_state
        .scene
        .realm2d
        .entities
        .get_mut(&realm_id)
    else {
        return dispose_error(
            engine,
            "camera2d",
            "camera2d-dispose",
            format!("Realm {} not found", args.realm_id),
        );
    };
    if entities.cameras.remove(&args.camera_id).is_none() {
        return dispose_error(
            engine,
            "camera2d",
            "camera2d-dispose",
            format!("Camera2D with id {} not found", args.camera_id),
        );
    }
    mark_realm_windows_dirty(engine, args.realm_id);
    CmdResultTwoDDispose {
        success: true,
        message: "Camera2D disposed successfully".to_string(),
    }
}

pub fn engine_cmd_sprite2d_upsert(
    engine: &mut EngineState,
    args: CmdSprite2dUpsertArgs,
) -> CmdResultTwoDUpsert {
    match args {
        CmdSprite2dUpsertArgs::Create(create) => {
            if let Err(message) =
                validate_realm_entity_ids(create.realm_id, create.sprite_id, "realmId", "spriteId")
            {
                return upsert_error(engine, "sprite2d", "sprite2d-upsert", message);
            }
            if let Err(message) = validate_host_logical_id(create.geometry_id, "geometryId") {
                return upsert_error(engine, "sprite2d", "sprite2d-upsert", message);
            }
            if let Some(material_id) = create.material_id
                && let Err(message) = validate_host_logical_id(material_id, "materialId")
            {
                return upsert_error(engine, "sprite2d", "sprite2d-upsert", message);
            }
            let realm_id = RealmId(create.realm_id);
            if let Err(message) = ensure_realm_is_2d(engine, realm_id, "sprite2d-upsert") {
                return upsert_error(engine, "sprite2d", "sprite2d-upsert", message);
            }
            let entities = engine
                .universal_state
                .scene
                .realm2d
                .entities
                .entry(realm_id)
                .or_default();
            if entities.sprites.contains_key(&create.sprite_id) {
                return upsert_error(
                    engine,
                    "sprite2d",
                    "sprite2d-upsert",
                    format!("Sprite2D with id {} already exists", create.sprite_id),
                );
            }
            entities.sprites.insert(
                create.sprite_id,
                Sprite2dRecord {
                    label: create.label,
                    transform: create.transform,
                    geometry_id: create.geometry_id,
                    material_id: create.material_id,
                    layer: create.layer,
                    cast_shadow: create.cast_shadow,
                    receive_shadow: create.receive_shadow,
                    occluder_only: create.occluder_only,
                    shadow_height: create.shadow_height.max(0.0),
                    shadow_layer_mask: create.shadow_layer_mask,
                },
            );
            mark_realm_windows_dirty(engine, create.realm_id);
            CmdResultTwoDUpsert {
                success: true,
                message: "Sprite2D upserted successfully".to_string(),
            }
        }
        CmdSprite2dUpsertArgs::Update(update) => {
            if let Err(message) =
                validate_realm_entity_ids(update.realm_id, update.sprite_id, "realmId", "spriteId")
            {
                return upsert_error(engine, "sprite2d", "sprite2d-upsert", message);
            }
            if let Some(geometry_id) = update.geometry_id
                && let Err(message) = validate_host_logical_id(geometry_id, "geometryId")
            {
                return upsert_error(engine, "sprite2d", "sprite2d-upsert", message);
            }
            if let Some(material_id) = update.material_id
                && let Err(message) = validate_host_logical_id(material_id, "materialId")
            {
                return upsert_error(engine, "sprite2d", "sprite2d-upsert", message);
            }
            let realm_id = RealmId(update.realm_id);
            if let Err(message) = ensure_realm_is_2d(engine, realm_id, "sprite2d-upsert") {
                return upsert_error(engine, "sprite2d", "sprite2d-upsert", message);
            }
            let Some(entities) = engine
                .universal_state
                .scene
                .realm2d
                .entities
                .get_mut(&realm_id)
            else {
                return upsert_error(
                    engine,
                    "sprite2d",
                    "sprite2d-upsert",
                    format!("Realm {} not found", update.realm_id),
                );
            };
            let Some(record) = entities.sprites.get_mut(&update.sprite_id) else {
                return upsert_error(
                    engine,
                    "sprite2d",
                    "sprite2d-upsert",
                    format!("Sprite2D with id {} not found", update.sprite_id),
                );
            };
            if update.label.is_some() {
                record.label = update.label;
            }
            if let Some(transform) = update.transform {
                record.transform = transform;
            }
            if let Some(geometry_id) = update.geometry_id {
                record.geometry_id = geometry_id;
            }
            if update.material_id.is_some() {
                record.material_id = update.material_id;
            }
            if let Some(layer) = update.layer {
                record.layer = layer;
            }
            if let Some(cast_shadow) = update.cast_shadow {
                record.cast_shadow = cast_shadow;
            }
            if let Some(receive_shadow) = update.receive_shadow {
                record.receive_shadow = receive_shadow;
            }
            if let Some(occluder_only) = update.occluder_only {
                record.occluder_only = occluder_only;
            }
            if let Some(shadow_height) = update.shadow_height {
                record.shadow_height = shadow_height.max(0.0);
            }
            if let Some(shadow_layer_mask) = update.shadow_layer_mask {
                record.shadow_layer_mask = shadow_layer_mask;
            }
            mark_realm_windows_dirty(engine, update.realm_id);
            CmdResultTwoDUpsert {
                success: true,
                message: "Sprite2D upserted successfully".to_string(),
            }
        }
    }
}

pub fn engine_cmd_sprite2d_dispose(
    engine: &mut EngineState,
    args: &CmdSprite2dDisposeArgs,
) -> CmdResultTwoDDispose {
    if let Err(message) =
        validate_realm_entity_ids(args.realm_id, args.sprite_id, "realmId", "spriteId")
    {
        return dispose_error(engine, "sprite2d", "sprite2d-dispose", message);
    }
    let realm_id = RealmId(args.realm_id);
    if let Err(message) = ensure_realm_is_2d(engine, realm_id, "sprite2d-dispose") {
        return dispose_error(engine, "sprite2d", "sprite2d-dispose", message);
    }
    let Some(entities) = engine
        .universal_state
        .scene
        .realm2d
        .entities
        .get_mut(&realm_id)
    else {
        return dispose_error(
            engine,
            "sprite2d",
            "sprite2d-dispose",
            format!("Realm {} not found", args.realm_id),
        );
    };
    if entities.sprites.remove(&args.sprite_id).is_none() {
        return dispose_error(
            engine,
            "sprite2d",
            "sprite2d-dispose",
            format!("Sprite2D with id {} not found", args.sprite_id),
        );
    }
    mark_realm_windows_dirty(engine, args.realm_id);
    CmdResultTwoDDispose {
        success: true,
        message: "Sprite2D disposed successfully".to_string(),
    }
}

pub fn engine_cmd_shape2d_upsert(
    engine: &mut EngineState,
    args: CmdShape2dUpsertArgs,
) -> CmdResultTwoDUpsert {
    match args {
        CmdShape2dUpsertArgs::Create(create) => {
            if let Err(message) =
                validate_realm_entity_ids(create.realm_id, create.shape_id, "realmId", "shapeId")
            {
                return upsert_error(engine, "shape2d", "shape2d-upsert", message);
            }
            if let Err(message) = validate_host_logical_id(create.geometry_id, "geometryId") {
                return upsert_error(engine, "shape2d", "shape2d-upsert", message);
            }
            if let Some(material_id) = create.material_id
                && let Err(message) = validate_host_logical_id(material_id, "materialId")
            {
                return upsert_error(engine, "shape2d", "shape2d-upsert", message);
            }
            let realm_id = RealmId(create.realm_id);
            if let Err(message) = ensure_realm_is_2d(engine, realm_id, "shape2d-upsert") {
                return upsert_error(engine, "shape2d", "shape2d-upsert", message);
            }
            let entities = engine
                .universal_state
                .scene
                .realm2d
                .entities
                .entry(realm_id)
                .or_default();
            if entities.shapes.contains_key(&create.shape_id) {
                return upsert_error(
                    engine,
                    "shape2d",
                    "shape2d-upsert",
                    format!("Shape2D with id {} already exists", create.shape_id),
                );
            }
            entities.shapes.insert(
                create.shape_id,
                Shape2dRecord {
                    label: create.label,
                    transform: create.transform,
                    geometry_id: create.geometry_id,
                    material_id: create.material_id,
                    layer: create.layer,
                    cast_shadow: create.cast_shadow,
                    receive_shadow: create.receive_shadow,
                    occluder_only: create.occluder_only,
                    shadow_height: create.shadow_height.max(0.0),
                    shadow_layer_mask: create.shadow_layer_mask,
                },
            );
            mark_realm_windows_dirty(engine, create.realm_id);
            CmdResultTwoDUpsert {
                success: true,
                message: "Shape2D upserted successfully".to_string(),
            }
        }
        CmdShape2dUpsertArgs::Update(update) => {
            if let Err(message) =
                validate_realm_entity_ids(update.realm_id, update.shape_id, "realmId", "shapeId")
            {
                return upsert_error(engine, "shape2d", "shape2d-upsert", message);
            }
            if let Some(geometry_id) = update.geometry_id
                && let Err(message) = validate_host_logical_id(geometry_id, "geometryId")
            {
                return upsert_error(engine, "shape2d", "shape2d-upsert", message);
            }
            if let Some(material_id) = update.material_id
                && let Err(message) = validate_host_logical_id(material_id, "materialId")
            {
                return upsert_error(engine, "shape2d", "shape2d-upsert", message);
            }
            let realm_id = RealmId(update.realm_id);
            if let Err(message) = ensure_realm_is_2d(engine, realm_id, "shape2d-upsert") {
                return upsert_error(engine, "shape2d", "shape2d-upsert", message);
            }
            let Some(entities) = engine
                .universal_state
                .scene
                .realm2d
                .entities
                .get_mut(&realm_id)
            else {
                return upsert_error(
                    engine,
                    "shape2d",
                    "shape2d-upsert",
                    format!("Realm {} not found", update.realm_id),
                );
            };
            let Some(record) = entities.shapes.get_mut(&update.shape_id) else {
                return upsert_error(
                    engine,
                    "shape2d",
                    "shape2d-upsert",
                    format!("Shape2D with id {} not found", update.shape_id),
                );
            };
            if update.label.is_some() {
                record.label = update.label;
            }
            if let Some(transform) = update.transform {
                record.transform = transform;
            }
            if let Some(geometry_id) = update.geometry_id {
                record.geometry_id = geometry_id;
            }
            if update.material_id.is_some() {
                record.material_id = update.material_id;
            }
            if let Some(layer) = update.layer {
                record.layer = layer;
            }
            if let Some(cast_shadow) = update.cast_shadow {
                record.cast_shadow = cast_shadow;
            }
            if let Some(receive_shadow) = update.receive_shadow {
                record.receive_shadow = receive_shadow;
            }
            if let Some(occluder_only) = update.occluder_only {
                record.occluder_only = occluder_only;
            }
            if let Some(shadow_height) = update.shadow_height {
                record.shadow_height = shadow_height.max(0.0);
            }
            if let Some(shadow_layer_mask) = update.shadow_layer_mask {
                record.shadow_layer_mask = shadow_layer_mask;
            }
            mark_realm_windows_dirty(engine, update.realm_id);
            CmdResultTwoDUpsert {
                success: true,
                message: "Shape2D upserted successfully".to_string(),
            }
        }
    }
}

pub fn engine_cmd_shape2d_dispose(
    engine: &mut EngineState,
    args: &CmdShape2dDisposeArgs,
) -> CmdResultTwoDDispose {
    if let Err(message) =
        validate_realm_entity_ids(args.realm_id, args.shape_id, "realmId", "shapeId")
    {
        return dispose_error(engine, "shape2d", "shape2d-dispose", message);
    }
    let realm_id = RealmId(args.realm_id);
    if let Err(message) = ensure_realm_is_2d(engine, realm_id, "shape2d-dispose") {
        return dispose_error(engine, "shape2d", "shape2d-dispose", message);
    }
    let Some(entities) = engine
        .universal_state
        .scene
        .realm2d
        .entities
        .get_mut(&realm_id)
    else {
        return dispose_error(
            engine,
            "shape2d",
            "shape2d-dispose",
            format!("Realm {} not found", args.realm_id),
        );
    };
    if entities.shapes.remove(&args.shape_id).is_none() {
        return dispose_error(
            engine,
            "shape2d",
            "shape2d-dispose",
            format!("Shape2D with id {} not found", args.shape_id),
        );
    }
    mark_realm_windows_dirty(engine, args.realm_id);
    CmdResultTwoDDispose {
        success: true,
        message: "Shape2D disposed successfully".to_string(),
    }
}

pub fn engine_cmd_realm2d_shadow_config_update(
    engine: &mut EngineState,
    args: &CmdRealm2dShadowConfigUpdateArgs,
) -> CmdResultTwoDUpsert {
    if let Err(message) = validate_host_logical_id(args.realm_id, "realmId") {
        return upsert_error(
            engine,
            "realm2d-shadow-config",
            "realm2d-shadow-config-update",
            message,
        );
    }
    let realm_id = RealmId(args.realm_id);
    if let Err(message) = ensure_realm_is_2d(engine, realm_id, "realm2d-shadow-config-update") {
        return upsert_error(
            engine,
            "realm2d-shadow-config",
            "realm2d-shadow-config-update",
            message,
        );
    }

    let current = engine
        .universal_state
        .scene
        .realm2d_shadow_configs
        .get(&realm_id)
        .copied()
        .unwrap_or_else(default_realm2d_shadow_config);
    let next = Realm2dShadowConfig {
        softness: args.softness.unwrap_or(current.softness),
        shadow_contact_offset: args
            .shadow_contact_offset
            .unwrap_or(current.shadow_contact_offset),
        shadow_back_gradient_strength: args
            .shadow_back_gradient_strength
            .unwrap_or(current.shadow_back_gradient_strength),
        shadow_debug_light_index: args
            .shadow_debug_light_index
            .unwrap_or(current.shadow_debug_light_index),
        shadow_debug_mode: args.shadow_debug_mode.unwrap_or(current.shadow_debug_mode),
        ambient: args.ambient.unwrap_or(current.ambient),
        light_radius: args.light_radius.unwrap_or(current.light_radius),
        quality_preset: args.quality_preset.unwrap_or(current.quality_preset),
        max_shadow_updates_per_frame: args
            .max_shadow_updates_per_frame
            .unwrap_or(current.max_shadow_updates_per_frame),
        angular_resolution: args
            .angular_resolution
            .or(args.map_resolution)
            .unwrap_or(current.angular_resolution),
        map_resolution: args.map_resolution.unwrap_or(current.map_resolution),
    }
    .sanitized();
    engine
        .universal_state
        .scene
        .realm2d_shadow_configs
        .insert(realm_id, next);
    mark_realm_windows_dirty(engine, args.realm_id);
    CmdResultTwoDUpsert {
        success: true,
        message: "Realm2D shadow config updated successfully".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::realm::cmd::{CmdRealmCreateArgs, RealmKindDto, engine_cmd_realm_create};
    use crate::core::test_support::test_engine;

    #[test]
    fn camera2d_upsert_and_dispose_work_on_twod_realm() {
        let mut engine = test_engine();
        let realm_id = engine_cmd_realm_create(
            &mut engine,
            &CmdRealmCreateArgs {
                kind: RealmKindDto::TwoD,
                importance: None,
                cache_policy: None,
                flags: None,
            },
        )
        .realm_id
        .expect("realm id should exist");

        let create = engine_cmd_camera2d_upsert(
            &mut engine,
            CmdCamera2dUpsertArgs::Create(CmdCamera2dCreateArgs {
                realm_id,
                camera_id: 10,
                label: Some("main".to_string()),
                transform: Mat4::IDENTITY,
                near_far: glam::Vec2::new(0.1, 10.0),
                ortho_scale: 1.0,
                layer_mask: 1,
                order: 0,
            }),
        );
        assert!(create.success);

        let update = engine_cmd_camera2d_upsert(
            &mut engine,
            CmdCamera2dUpsertArgs::Update(CmdCamera2dUpdateArgs {
                realm_id,
                camera_id: 10,
                label: None,
                transform: None,
                near_far: None,
                ortho_scale: Some(2.0),
                layer_mask: None,
                order: None,
            }),
        );
        assert!(update.success);

        let dispose = engine_cmd_camera2d_dispose(
            &mut engine,
            &CmdCamera2dDisposeArgs {
                realm_id,
                camera_id: 10,
            },
        );
        assert!(dispose.success);
    }

    #[test]
    fn sprite2d_create_rejects_threed_realm() {
        let mut engine = test_engine();
        let realm_id = engine_cmd_realm_create(
            &mut engine,
            &CmdRealmCreateArgs {
                kind: RealmKindDto::ThreeD,
                importance: None,
                cache_policy: None,
                flags: None,
            },
        )
        .realm_id
        .expect("realm id should exist");

        let result = engine_cmd_sprite2d_upsert(
            &mut engine,
            CmdSprite2dUpsertArgs::Create(CmdSprite2dCreateArgs {
                realm_id,
                sprite_id: 22,
                label: None,
                transform: Mat4::IDENTITY,
                geometry_id: 100,
                material_id: Some(200),
                layer: 0,
                cast_shadow: true,
                receive_shadow: true,
                occluder_only: false,
                shadow_height: 1.0,
                shadow_layer_mask: u32::MAX,
            }),
        );
        assert!(!result.success);
    }
}
