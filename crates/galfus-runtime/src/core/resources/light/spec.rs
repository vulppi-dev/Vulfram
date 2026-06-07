use bytemuck::{Pod, Zeroable};
use glam::{Mat4, UVec2, Vec2, Vec3, Vec4, Vec4Swizzles};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LightKind {
    Directional = 0,
    Point,
    Spot,
    Ambient,
    Hemisphere,
}

impl LightKind {
    pub fn to_u32(self) -> u32 {
        match self {
            LightKind::Directional => 0,
            LightKind::Point => 1,
            LightKind::Spot => 2,
            LightKind::Ambient => 3,
            LightKind::Hemisphere => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, Pod, Zeroable, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[repr(C)]
pub struct LightComponent {
    pub position: Vec4,
    pub direction: Vec4,
    pub color: Vec4,
    pub ground_color: Vec4,
    pub view: Mat4,
    pub projection: Mat4,
    pub view_projection: Mat4,
    pub intensity_range: Vec2,
    pub spot_inner_outer: Vec2,
    pub kind_flags: UVec2, // x: kind, y: flags (bit 0: cast_shadow)
    pub shadow_index: u32,
    pub _padding: u32,
}

impl LightComponent {
    pub const FLAG_CAST_SHADOW: u32 = 1 << 0;

    pub fn new(
        position: Vec4,
        direction: Vec4,
        color: Vec4,
        ground_color: Vec4,
        intensity: f32,
        range: f32,
        spot_inner_outer: Vec2,
        kind: LightKind,
        cast_shadow: bool,
    ) -> Self {
        let mut flags = 0u32;
        if cast_shadow {
            flags |= Self::FLAG_CAST_SHADOW;
        }

        let mut component = Self {
            position,
            direction,
            color,
            ground_color,
            view: Mat4::IDENTITY,
            projection: Mat4::IDENTITY,
            view_projection: Mat4::IDENTITY,
            intensity_range: Vec2::new(intensity, range),
            spot_inner_outer,
            kind_flags: UVec2::new(kind.to_u32(), flags),
            shadow_index: 0xFFFFFFFF,
            _padding: 0,
        };

        component.update_matrices();
        component
    }

    pub fn update_matrices(&mut self) {
        let kind = self.kind_flags.x;
        match kind {
            0 => {
                // Directional Light
                let dir = self.direction.xyz().normalize_or(Vec3::NEG_Y);
                let pos = self.position.xyz();

                let up = if dir.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };

                self.view = Mat4::look_to_rh(pos, dir, up);

                let size = 50.0;
                let near = -100.0;
                let far = 100.0;

                // Reverse Z: swap near/far
                self.projection = Mat4::orthographic_rh(-size, size, -size, size, far, near);
            }
            1 => {
                // Point Light
                self.view = Mat4::IDENTITY;
                self.projection = Mat4::IDENTITY;
            }
            2 => {
                // Spot Light
                let pos = self.position.xyz();
                let dir = self.direction.xyz().normalize_or(Vec3::NEG_Z);
                let up = if dir.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };

                self.view = Mat4::look_to_rh(pos, dir, up);

                let fov = self.spot_inner_outer.y * 2.0;
                let near = 0.1;
                let far = self.intensity_range.y;

                self.projection = Mat4::perspective_rh(fov, 1.0, far, near);
            }
            _ => {
                self.view = Mat4::IDENTITY;
                self.projection = Mat4::IDENTITY;
            }
        }

        self.view_projection = self.projection * self.view;
    }
}
#[derive(Debug, Clone)]
pub struct LightRecord {
    pub label: Option<String>,
    pub data: LightComponent,
    pub active: bool,
    pub layer_mask: u32,
    pub shadow_layer_mask: u32,
    pub cast_shadow: bool,
    pub is_dirty: bool,
}

impl LightRecord {
    pub fn new(
        label: Option<String>,
        data: LightComponent,
        active: bool,
        layer_mask: u32,
        shadow_layer_mask: u32,
        cast_shadow: bool,
    ) -> Self {
        Self {
            label,
            data,
            active,
            layer_mask,
            shadow_layer_mask,
            cast_shadow,
            is_dirty: true,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }
}
