#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureRecordMeta {
    pub id: u32,
    pub label: Option<String>,
    pub width: u32,
    pub height: u32,
    pub depth_or_array_layers: u32,
    pub format: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForwardAtlasEntryMeta {
    pub id: u32,
    pub label: Option<String>,
    pub layer: u32,
    pub uv_scale_bias: glam::Vec4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetTextureBindingMeta {
    pub texture_id: u32,
    pub target_id: galfus_realm_core::TargetId,
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncPlan {
    pub stale_ids: Vec<u32>,
    pub replace_ids: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraProjectionPlan {
    pub preserve_runtime_projection: bool,
    pub reset_projection_size: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityRecordUpdatePlan {
    pub mark_dirty: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelRecordMeta {
    pub transform: [f32; 16],
    pub translation: glam::Vec4,
    pub rotation: glam::Vec4,
    pub scale: glam::Vec4,
    pub flags: [u32; 4],
    pub outline_color: glam::Vec4,
    pub geometry_id: u32,
    pub material_id: Option<u32>,
    pub active: bool,
    pub layer_mask: u32,
    pub cast_shadow: bool,
    pub receive_shadow: bool,
    pub cast_outline: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LightRecordMeta {
    pub position: glam::Vec4,
    pub direction: glam::Vec4,
    pub color: glam::Vec4,
    pub ground_color: glam::Vec4,
    pub view: [f32; 16],
    pub projection: [f32; 16],
    pub view_projection: [f32; 16],
    pub intensity_range: glam::Vec2,
    pub spot_inner_outer: glam::Vec2,
    pub kind_flags: [u32; 2],
    pub active: bool,
    pub layer_mask: u32,
    pub shadow_layer_mask: u32,
    pub cast_shadow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialRecordMeta {
    pub label: Option<String>,
    pub data_bytes: Vec<u8>,
    pub inputs_bytes: Vec<u8>,
    pub texture_ids: Vec<u32>,
    pub surface_type: u32,
    pub topology: u32,
    pub polygon_mode: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialRecordUpdatePlan {
    pub mark_dirty: bool,
    pub reset_bind_group: bool,
}
