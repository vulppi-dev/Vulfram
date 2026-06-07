use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MaterialShaderBasePreset {
    Standard,
    Pbr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MaterialShaderType {
    Model,
    Particle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MaterialShaderRealm {
    ThreeD,
    TwoD,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialShaderCompileSpec {
    pub base_preset: MaterialShaderBasePreset,
    #[serde(default = "default_material_shader_type")]
    pub shader_type: MaterialShaderType,
    pub shader_source: String,
    #[serde(default)]
    pub shader_params_schema: HashMap<String, String>,
    #[serde(default)]
    pub capabilities: MaterialShaderCapabilities,
}

fn default_material_shader_type() -> MaterialShaderType {
    MaterialShaderType::Model
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialShaderCapabilities {
    #[serde(default)]
    pub semantics: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CompiledMaterialShader {
    pub source: String,
    pub hash: u64,
}

const STANDARD_SOURCE: &str = r#"
fn shade_standard(
  base_color: vec3<f32>,
  normal: vec3<f32>,
  light_dir: vec3<f32>,
  view_dir: vec3<f32>,
  shadow_visibility: f32,
  roughness: f32,
  metallic: f32,
) -> vec3<f32> {
  let n = normalize(normal);
  let l = normalize(light_dir);
  let v = normalize(view_dir);
  let h = normalize(l + v);
  let ndotl = max(dot(n, l), 0.0);
  let spec_power = mix(64.0, 4.0, roughness);
  let specular = pow(max(dot(n, h), 0.0), spec_power) * (1.0 - metallic);
  return (base_color * ndotl + vec3<f32>(specular)) * shadow_visibility;
}

fn vertex(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.world_position = input.position;
  out.world_normal = input.normal;
  out.uv = input.uv;
  out.clip_position = vec4<f32>(0.0);
  return out;
}

fn fragment(input: FragmentInput) -> FragmentOutput {
  var out: FragmentOutput;
  let base_color = input_at(0u).rgb;
  let tint = select(vec3<f32>(1.0, 0.7, 0.6), base_color, has_material_input(0u));
  let normal = normalize(input.world_normal);
  let view_dir = normalize(camera.position.xyz - input.world_position);
  var lit = tint * 0.08;
  let max_lights = min(
    visible_counts[light_params.camera_index],
    light_params.max_lights_per_camera
  );
  var i = 0u;
  loop {
    if (i >= max_lights) { break; }
    let visible_index = light_params.camera_index * light_params.max_lights_per_camera + i;
    let light_index = visible_indices[visible_index];
    if (light_index < arrayLength(&lights)) {
      let light = lights[light_index];
      let light_dir = light_direction(light, input.world_position);
      let shadow_visibility = select(
        1.0,
        sample_shadow_for_light(light_index, input.world_position, normal),
        input.receive_shadow > 0.5
      );
      let direct = shade_standard(tint, normal, light_dir, view_dir, shadow_visibility, 0.05, 0.0);
      lit += direct * light_radiance(light);
    }
    i = i + 1u;
  }
  out.color = vec4<f32>(lit, 1.0);
  out.emissive = vec4<f32>(0.0);
  return out;
}
"#;

const PBR_SOURCE: &str = r#"
const PI: f32 = 3.14159265;

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
  return f0 + (vec3<f32>(1.0) - f0) * pow(max(1.0 - cos_theta, 0.0), 5.0);
}

fn distribution_ggx(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
  let a = roughness * roughness;
  let a2 = a * a;
  let ndoth = max(dot(n, h), 0.0);
  let ndoth2 = ndoth * ndoth;
  let num = a2;
  let denom = (ndoth2 * (a2 - 1.0) + 1.0);
  return num / max(PI * denom * denom, 0.0001);
}

fn geometry_schlick_ggx(ndotv: f32, roughness: f32) -> f32 {
  let r = roughness + 1.0;
  let k = (r * r) / 8.0;
  let denom = ndotv * (1.0 - k) + k;
  return ndotv / max(denom, 0.0001);
}

fn geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
  let ndotv = max(dot(n, v), 0.0);
  let ndotl = max(dot(n, l), 0.0);
  let ggx1 = geometry_schlick_ggx(ndotv, roughness);
  let ggx2 = geometry_schlick_ggx(ndotl, roughness);
  return ggx1 * ggx2;
}

fn shade_pbr(
  base_color: vec3<f32>,
  normal: vec3<f32>,
  light_dir: vec3<f32>,
  view_dir: vec3<f32>,
  shadow_visibility: f32,
  metallic: f32,
  roughness: f32,
) -> vec3<f32> {
  let n = normalize(normal);
  let l = normalize(light_dir);
  let v = normalize(view_dir);
  let h = normalize(v + l);
  let ndotl = max(dot(n, l), 0.0);
  let ndotv = max(dot(n, v), 0.0);
  let f0 = mix(vec3<f32>(0.04), base_color, metallic);
  let f = fresnel_schlick(max(dot(h, v), 0.0), f0);
  let d = distribution_ggx(n, h, roughness);
  let g = geometry_smith(n, v, l, roughness);
  let numerator = d * g * f;
  let denominator = max(4.0 * ndotv * ndotl, 0.0001);
  let specular = numerator / denominator;
  let kd = (vec3<f32>(1.0) - f) * (1.0 - metallic);
  let diffuse = kd * base_color / PI;
  return (diffuse + specular) * ndotl * shadow_visibility;
}

fn vertex(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.world_position = input.position;
  out.world_normal = input.normal;
  out.uv = input.uv;
  out.clip_position = vec4<f32>(0.0);
  return out;
}

fn fragment(input: FragmentInput) -> FragmentOutput {
  var out: FragmentOutput;
  let base_color = input_at(0u).rgb;
  let tint = select(vec3<f32>(0.6, 0.9, 0.65), base_color, has_material_input(0u));
  let metal_rough = input_at(1u).xy;
  let metallic = clamp(metal_rough.x, 0.0, 1.0);
  let roughness = clamp(metal_rough.y, 0.04, 1.0);
  let normal = normalize(input.world_normal);
  let view_dir = normalize(camera.position.xyz - input.world_position);
  var lit = tint * 0.03;
  let max_lights = min(
    visible_counts[light_params.camera_index],
    light_params.max_lights_per_camera
  );
  var i = 0u;
  loop {
    if (i >= max_lights) { break; }
    let visible_index = light_params.camera_index * light_params.max_lights_per_camera + i;
    let light_index = visible_indices[visible_index];
    if (light_index < arrayLength(&lights)) {
      let light = lights[light_index];
      let light_dir = light_direction(light, input.world_position);
      let shadow_visibility = select(
        1.0,
        sample_shadow_for_light(light_index, input.world_position, normal),
        input.receive_shadow > 0.5
      );
      let direct = shade_pbr(tint, normal, light_dir, view_dir, shadow_visibility, metallic, roughness);
      lit += direct * light_radiance(light);
    }
    i = i + 1u;
  }
  out.color = vec4<f32>(lit, 1.0);
  out.emissive = vec4<f32>(0.0);
  return out;
}
"#;

pub fn builtin_material_source(preset: MaterialShaderBasePreset) -> &'static str {
    match preset {
        MaterialShaderBasePreset::Standard => STANDARD_SOURCE,
        MaterialShaderBasePreset::Pbr => PBR_SOURCE,
    }
}

const STANDARD_2D_SOURCE: &str = r#"
fn vertex(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.clip_position = vec4<f32>(0.0);
  // Keep zero so the 2D composer injects the per-material tint from camera uniform.
  out.color = vec4<f32>(0.0);
  out.uv = input.uv;
  return out;
}

fn fragment(input: FragmentInput) -> FragmentOutput {
  var out: FragmentOutput;
  out.color = sample_material(input.uv) * input.color;
  return out;
}
"#;

pub fn builtin_material_source_2d() -> &'static str {
    STANDARD_2D_SOURCE
}

const FORBIDDEN_SHADER_TOKENS: [&str; 16] = [
    "@group",
    "@binding",
    "@vertex",
    "@fragment",
    "@compute",
    "@location",
    "@builtin",
    "var<uniform>",
    "var<storage>",
    "texture_2d",
    "texture_depth_2d",
    "texture_2d_array",
    "sampler",
    "sampler_comparison",
    "override ",
    "override\t",
];

fn validate_logical_shader_source(
    shader_type: MaterialShaderType,
    source: &str,
) -> Result<(), String> {
    for token in FORBIDDEN_SHADER_TOKENS {
        if source.contains(token) {
            return Err(format!(
                "Shader source contains forbidden token '{}'",
                token
            ));
        }
    }

    let has_vertex = source.contains("fn vertex(");
    let has_fragment = source.contains("fn fragment(");
    let has_compute = source.contains("fn compute(");

    match shader_type {
        MaterialShaderType::Model => {
            if !has_vertex || !has_fragment {
                return Err(
                    "Model shader must define both 'fn vertex(...)' and 'fn fragment(...)'"
                        .to_string(),
                );
            }
            if has_compute {
                return Err("Model shader cannot define 'fn compute(...)'".to_string());
            }
        }
        MaterialShaderType::Particle => {
            if !has_compute {
                return Err("Particle shader must define 'fn compute(...)'".to_string());
            }
            if has_vertex || has_fragment {
                return Err(
                    "Particle shader cannot define 'fn vertex(...)' or 'fn fragment(...)'"
                        .to_string(),
                );
            }
        }
    }

    Ok(())
}

fn model_composer_prelude() -> &'static str {
    r#"
struct Frame {
    time: f32,
    delta_time: f32,
    frame_index: u32,
    _padding: u32,
}

struct Camera {
    position: vec4<f32>,
    direction: vec4<f32>,
    up: vec4<f32>,
    near_far: vec2<f32>,
    kind_flags: vec2<u32>,
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    view_projection: mat4x4<f32>,
}

struct LightDrawParams {
    camera_index: u32,
    max_lights_per_camera: u32,
}

struct Light {
    position: vec4<f32>,
    direction: vec4<f32>,
    color: vec4<f32>,
    ground_color: vec4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    view_projection: mat4x4<f32>,
    intensity_range: vec2<f32>,
    spot_inner_outer: vec2<f32>,
    kind_flags: vec2<u32>,
    shadow_index: u32,
    _padding: u32,
}

struct ShadowPageEntry {
    scale_offset: vec4<f32>,
    layer_index: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
}

struct ShadowParams {
    virtual_grid_size: f32,
    pcf_range: i32,
    table_capacity: u32,
    point_vp_base: u32,
    bias_min: f32,
    bias_slope: f32,
    point_bias_min: f32,
    point_bias_slope: f32,
    normal_bias: f32,
    _padding0: f32,
    _padding1: f32,
}

struct Model {
    transform: mat4x4<f32>,
    translation: vec4<f32>,
    rotation: vec4<f32>,
    scale: vec4<f32>,
    flags: vec4<u32>,
    outline_color: vec4<f32>,
}

struct MaterialParams {
    input_indices: vec4<u32>,
    inputs_offset_count: vec2<u32>,
    surface_flags: vec2<u32>,
    texture_slots: array<vec4<u32>, 2>,
    sampler_indices: array<vec4<u32>, 2>,
    tex_sources: array<vec4<u32>, 2>,
    atlas_layers: array<vec4<u32>, 2>,
    atlas_scale_bias: array<vec4<f32>, 8>,
}

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<uniform> camera: Camera;
@group(0) @binding(2) var<uniform> light_params: LightDrawParams;
@group(0) @binding(3) var<storage, read> lights: array<Light>;
@group(0) @binding(4) var<storage, read> visible_indices: array<u32>;
@group(0) @binding(5) var<storage, read> visible_counts: array<u32>;
@group(0) @binding(6) var<uniform> shadow_params: ShadowParams;
@group(0) @binding(7) var shadow_atlas: texture_depth_2d_array;
@group(0) @binding(8) var<storage, read> shadow_page_table: array<ShadowPageEntry>;
@group(0) @binding(9) var<storage, read> point_light_vp: array<mat4x4<f32>>;
@group(0) @binding(10) var point_clamp_sampler: sampler;
@group(0) @binding(11) var linear_clamp_sampler: sampler;
@group(0) @binding(12) var point_repeat_sampler: sampler;
@group(0) @binding(13) var linear_repeat_sampler: sampler;
@group(0) @binding(14) var shadow_sampler: sampler_comparison;
@group(0) @binding(15) var forward_atlas: texture_2d_array<f32>;

@group(1) @binding(0) var<storage, read> models: array<Model>;
@group(1) @binding(1) var<uniform> material: MaterialParams;
@group(1) @binding(2) var<storage, read> material_inputs: array<vec4<f32>>;
@group(1) @binding(3) var material_tex0: texture_2d<f32>;
@group(1) @binding(4) var material_tex1: texture_2d<f32>;
@group(1) @binding(5) var material_tex2: texture_2d<f32>;
@group(1) @binding(6) var material_tex3: texture_2d<f32>;
@group(1) @binding(7) var material_tex4: texture_2d<f32>;
@group(1) @binding(8) var material_tex5: texture_2d<f32>;
@group(1) @binding(9) var material_tex6: texture_2d<f32>;
@group(1) @binding(10) var material_tex7: texture_2d<f32>;
@group(1) @binding(11) var<storage, read> bones: array<mat4x4<f32>>;

struct FrameSemanticMeta {
    resolution: vec2<f32>,
    inv_resolution: vec2<f32>,
    frame_index: u32,
    flags: u32,
}
@group(2) @binding(0) var frame_scene_color: texture_2d<f32>;
@group(2) @binding(1) var frame_scene_depth: texture_depth_2d;
@group(2) @binding(2) var frame_history0: texture_2d<f32>;
@group(2) @binding(3) var frame_history1: texture_2d<f32>;
@group(2) @binding(4) var frame_linear_sampler: sampler;
@group(2) @binding(5) var frame_point_sampler: sampler;
@group(2) @binding(6) var<uniform> frame_semantics: FrameSemanticMeta;

struct VertexInput {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    instance_index: u32,
}

struct VertexOutput {
    world_position: vec3<f32>,
    world_normal: vec3<f32>,
    uv: vec2<f32>,
    clip_position: vec4<f32>,
}

struct FragmentInput {
    world_position: vec3<f32>,
    world_normal: vec3<f32>,
    uv: vec2<f32>,
    receive_shadow: f32,
}

struct FragmentOutput {
    color: vec4<f32>,
    emissive: vec4<f32>,
}

struct VertexStageInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @builtin(instance_index) instance_index: u32,
};

struct VertexStageOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) receive_shadow: f32,
};

struct FragmentStageOutput {
    @location(0) color: vec4<f32>,
    @location(1) emissive: vec4<f32>,
};

const MATERIAL_INVALID_SLOT: u32 = 0xFFFFFFFFu;
const SAMPLER_POINT_CLAMP: u32 = 0u;
const SAMPLER_LINEAR_CLAMP: u32 = 1u;
const SAMPLER_POINT_REPEAT: u32 = 2u;
const SAMPLER_LINEAR_REPEAT: u32 = 3u;
const TEX_SOURCE_STANDALONE: u32 = 0u;
const TEX_SOURCE_ATLAS: u32 = 1u;
const TEX_SOURCE_INVALID: u32 = 2u;
fn sample_shadow_compare_atlas(uv: vec2<f32>, layer: u32, depth_ref: f32) -> f32 {
    return textureSampleCompare(shadow_atlas, shadow_sampler, uv, i32(layer), depth_ref);
}
fn resolve_point_light_face(dir: vec3<f32>) -> u32 {
    let ad = abs(dir);
    if (ad.x >= ad.y && ad.x >= ad.z) {
        return select(1u, 0u, dir.x >= 0.0);
    }
    if (ad.y >= ad.x && ad.y >= ad.z) {
        return select(3u, 2u, dir.y >= 0.0);
    }
    return select(5u, 4u, dir.z >= 0.0);
}

fn resolve_secondary_point_light_face(dir: vec3<f32>, primary_face: u32) -> u32 {
    let ad = abs(dir);
    if (primary_face == 0u || primary_face == 1u) {
        if (ad.y >= ad.z) { return select(3u, 2u, dir.y >= 0.0); }
        return select(5u, 4u, dir.z >= 0.0);
    }
    if (primary_face == 2u || primary_face == 3u) {
        if (ad.x >= ad.z) { return select(1u, 0u, dir.x >= 0.0); }
        return select(5u, 4u, dir.z >= 0.0);
    }
    if (ad.x >= ad.y) { return select(1u, 0u, dir.x >= 0.0); }
    return select(3u, 2u, dir.y >= 0.0);
}

fn sample_shadow_for_point_face(
    light: Light,
    world_position: vec3<f32>,
    world_normal: vec3<f32>,
    light_dir: vec3<f32>,
    face: u32
) -> f32 {
    let vp = point_light_vp[shadow_params.point_vp_base + light.shadow_index * 6u + face];
    let clip = vp * vec4<f32>(world_position, 1.0);
    if (abs(clip.w) < 1e-6) { return 1.0; }

    let ndc = clip.xyz / clip.w;
    if (ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0) { return 1.0; }

    let grid_size = max(u32(shadow_params.virtual_grid_size), 1u);
    let uv01 = vec2<f32>(ndc.x * 0.5 + 0.5, -ndc.y * 0.5 + 0.5);
    let page_x = min(u32(floor(uv01.x * f32(grid_size))), grid_size - 1u);
    let page_y = min(u32(floor(uv01.y * f32(grid_size))), grid_size - 1u);
    // Keep sampling away from page borders to reduce seams between virtual pages/faces.
    let page_uv = clamp(fract(uv01 * f32(grid_size)), vec2<f32>(2e-4), vec2<f32>(1.0 - 2e-4));

    let table_index = ((light.shadow_index * 6u + face) * grid_size * grid_size + page_y * grid_size + page_x) % shadow_params.table_capacity;
    let page = shadow_page_table[table_index];
    if (page.layer_index == 0xFFFFFFFFu) { return 1.0; }

    let atlas_uv = page.scale_offset.xy * page_uv + page.scale_offset.zw;
    let depth_ref = clamp(ndc.z, 0.0, 1.0);
    let n = normalize(world_normal);
    let slope = 1.0 - max(dot(n, light_dir), 0.0);
    let bias = (shadow_params.point_bias_min + shadow_params.point_bias_slope * slope) * 0.05;
    let compare_ref = clamp(depth_ref + bias, 0.0, 1.0);
    return sample_shadow_compare_atlas(atlas_uv, page.layer_index, compare_ref);
}
fn light_direction(light: Light, world_position: vec3<f32>) -> vec3<f32> {
    if (light.kind_flags.x == 0u) {
        return normalize(-light.direction.xyz);
    }
    return normalize(light.position.xyz - world_position);
}

fn light_radiance(light: Light) -> vec3<f32> {
    return light.color.rgb * max(light.intensity_range.x, 0.0);
}

fn sample_shadow_for_light(light_index: u32, world_position: vec3<f32>, world_normal: vec3<f32>) -> f32 {
    if (light_index >= arrayLength(&lights)) { return 1.0; }
    let light = lights[light_index];
    if (
        light.kind_flags.x != 1u ||
        (light.kind_flags.y & 1u) == 0u ||
        light.shadow_index == 0xFFFFFFFFu ||
        shadow_params.table_capacity == 0u
    ) { return 1.0; }

    let l = light_direction(light, world_position);
    let to_frag = world_position - light.position.xyz;
    let primary_face = resolve_point_light_face(to_frag);
    let primary_visibility =
        sample_shadow_for_point_face(light, world_position, world_normal, l, primary_face);

    // Near cube-face transition lines, sample a secondary face and keep the more occluding result.
    let ad = abs(to_frag);
    let major = max(ad.x, max(ad.y, ad.z));
    let minor = ad.x + ad.y + ad.z - major - min(ad.x, min(ad.y, ad.z));
    if (major > 0.0 && (major - minor) / major < 0.05) {
        let secondary_face = resolve_secondary_point_light_face(to_frag, primary_face);
        let secondary_visibility =
            sample_shadow_for_point_face(light, world_position, world_normal, l, secondary_face);
        return min(primary_visibility, secondary_visibility);
    }

    return primary_visibility;
}
fn sample_scene_color(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(frame_scene_color, frame_linear_sampler, uv);
}
fn sample_history0(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(frame_history0, frame_linear_sampler, uv);
}
fn sample_history1(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(frame_history1, frame_linear_sampler, uv);
}
fn load_scene_depth(pixel: vec2<u32>) -> f32 {
    let dim = textureDimensions(frame_scene_depth);
    let x = min(pixel.x, max(dim.x, 1u) - 1u);
    let y = min(pixel.y, max(dim.y, 1u) - 1u);
    return textureLoad(frame_scene_depth, vec2<i32>(i32(x), i32(y)), 0);
}
fn scene_resolution() -> vec2<f32> { return frame_semantics.resolution; }
fn scene_inv_resolution() -> vec2<f32> { return frame_semantics.inv_resolution; }
fn current_frame_index() -> u32 { return frame_semantics.frame_index; }
fn get_slot(slots: array<vec4<u32>, 2>, index: u32) -> u32 {
    let vec_index = index / 4u;
    let lane = index % 4u;
    let v = slots[vec_index];
    if (lane == 0u) { return v.x; }
    if (lane == 1u) { return v.y; }
    if (lane == 2u) { return v.z; }
    return v.w;
}

fn has_material_input(index: u32) -> bool {
    return index < material.inputs_offset_count.y;
}

fn input_at(index: u32) -> vec4<f32> {
    if (!has_material_input(index)) {
        return vec4<f32>(0.0);
    }
    return material_inputs[material.inputs_offset_count.x + index];
}

fn sample_texture_slot(tex_slot: u32, sampler_index: u32, uv: vec2<f32>) -> vec4<f32> {
    if (tex_slot == 0u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex0, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex0, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex0, point_repeat_sampler, uv); }
        return textureSample(material_tex0, linear_repeat_sampler, uv);
    }
    if (tex_slot == 1u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex1, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex1, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex1, point_repeat_sampler, uv); }
        return textureSample(material_tex1, linear_repeat_sampler, uv);
    }
    if (tex_slot == 2u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex2, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex2, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex2, point_repeat_sampler, uv); }
        return textureSample(material_tex2, linear_repeat_sampler, uv);
    }
    if (tex_slot == 3u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex3, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex3, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex3, point_repeat_sampler, uv); }
        return textureSample(material_tex3, linear_repeat_sampler, uv);
    }
    if (tex_slot == 4u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex4, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex4, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex4, point_repeat_sampler, uv); }
        return textureSample(material_tex4, linear_repeat_sampler, uv);
    }
    if (tex_slot == 5u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex5, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex5, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex5, point_repeat_sampler, uv); }
        return textureSample(material_tex5, linear_repeat_sampler, uv);
    }
    if (tex_slot == 6u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex6, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex6, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex6, point_repeat_sampler, uv); }
        return textureSample(material_tex6, linear_repeat_sampler, uv);
    }
    if (tex_slot == 7u) {
        if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(material_tex7, point_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(material_tex7, linear_clamp_sampler, uv); }
        if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(material_tex7, point_repeat_sampler, uv); }
        return textureSample(material_tex7, linear_repeat_sampler, uv);
    }
    return vec4<f32>(1.0);
}

fn sample_atlas(sampler_index: u32, uv: vec2<f32>, layer: u32) -> vec4<f32> {
    let layer_i = i32(layer);
    if (sampler_index == SAMPLER_POINT_CLAMP) { return textureSample(forward_atlas, point_clamp_sampler, uv, layer_i); }
    if (sampler_index == SAMPLER_LINEAR_CLAMP) { return textureSample(forward_atlas, linear_clamp_sampler, uv, layer_i); }
    if (sampler_index == SAMPLER_POINT_REPEAT) { return textureSample(forward_atlas, point_repeat_sampler, uv, layer_i); }
    return textureSample(forward_atlas, linear_repeat_sampler, uv, layer_i);
}

fn sample_material(tex_slot: u32, sampler_index: u32, uv: vec2<f32>) -> vec4<f32> {
    if (tex_slot == MATERIAL_INVALID_SLOT) {
        return vec4<f32>(1.0);
    }
    let source = get_slot(material.tex_sources, tex_slot);
    let scale_bias = material.atlas_scale_bias[tex_slot];
    let uv_transformed = uv * scale_bias.xy + scale_bias.zw;
    if (source == TEX_SOURCE_ATLAS) {
        let layer = get_slot(material.atlas_layers, tex_slot);
        return sample_atlas(sampler_index, uv_transformed, layer);
    }
    if (source == TEX_SOURCE_INVALID) {
        return vec4<f32>(1.0);
    }
    return sample_texture_slot(tex_slot, sampler_index, uv_transformed);
}

"#
}

fn model_composer_postlude() -> &'static str {
    r#"
@vertex
fn vs_main(input: VertexStageInput) -> VertexStageOutput {
    let model = models[input.instance_index];
    let logical_input = VertexInput(
        input.position,
        input.normal,
        input.uv,
        input.instance_index,
    );
    var logical_output = vertex(logical_input);
    if all(logical_output.clip_position == vec4<f32>(0.0)) {
        let world = model.transform * vec4<f32>(input.position, 1.0);
        logical_output.world_position = world.xyz;
        logical_output.world_normal = normalize((model.transform * vec4<f32>(input.normal, 0.0)).xyz);
        logical_output.uv = input.uv;
        logical_output.clip_position = camera.view_projection * world;
    }

    var out: VertexStageOutput;
    out.clip_position = logical_output.clip_position;
    out.world_position = logical_output.world_position;
    out.world_normal = logical_output.world_normal;
    out.uv = logical_output.uv;
    out.receive_shadow = select(0.0, 1.0, (model.flags.x & 1u) != 0u);
    return out;
}

@fragment
fn fs_main(input: VertexStageOutput) -> FragmentStageOutput {
    let logical_input = FragmentInput(
        input.world_position,
        normalize(input.world_normal),
        input.uv,
        input.receive_shadow,
    );
    let logical_output = fragment(logical_input);

    var out: FragmentStageOutput;
    out.color = logical_output.color;
    out.emissive = logical_output.emissive;
    return out;
}
"#
}

fn two_d_composer_prelude() -> &'static str {
    r#"
struct CameraUniform {
    view_projection: mat4x4<f32>,
    model_matrix: mat4x4<f32>,
    tint: vec4<f32>,
    model_position: vec4<f32>,
    light_offset_count: vec4<u32>,
    shadow_params: vec4<f32>,
    shadow_controls: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;
@group(0) @binding(1) var point_clamp_sampler: sampler;
@group(0) @binding(2) var linear_clamp_sampler: sampler;
@group(0) @binding(3) var point_repeat_sampler: sampler;
@group(0) @binding(4) var linear_repeat_sampler: sampler;
struct Light2D {
    position: vec4<f32>,
    color: vec4<f32>,
    intensity_range: vec2<f32>,
    light_radius: f32,
    _padding0: f32,
    kind_flags: vec2<u32>,
    shadow_layer_mask: u32,
    shadow_index: u32,
}
@group(0) @binding(5) var<storage, read> lights_2d: array<Light2D>;
struct ShadowSample2D {
    blocker_distance: f32,
    blocker_left: f32,
    blocker_right: f32,
    flags: f32,
    penumbra_left: f32,
    penumbra_right: f32,
    support_left_distance: f32,
    support_right_distance: f32,
    occluder_v0: vec2<f32>,
    occluder_v1: vec2<f32>,
    occluder_v2: vec2<f32>,
    occluder_v3: vec2<f32>,
}
@group(0) @binding(6) var<storage, read> shadow_samples_2d: array<ShadowSample2D>;

struct MaterialParams {
    input_indices: vec4<u32>,
    inputs_offset_count: vec2<u32>,
    surface_flags: vec2<u32>,
    texture_slots: array<vec4<u32>, 2>,
    sampler_indices: array<vec4<u32>, 2>,
    tex_sources: array<vec4<u32>, 2>,
    atlas_layers: array<vec4<u32>, 2>,
    atlas_scale_bias: array<vec4<f32>, 8>,
}

@group(1) @binding(1) var<uniform> material: MaterialParams;
@group(1) @binding(2) var<storage, read> material_inputs: array<vec4<f32>>;
@group(1) @binding(3) var material_tex0: texture_2d<f32>;
@group(1) @binding(4) var material_tex1: texture_2d<f32>;
@group(1) @binding(5) var material_tex2: texture_2d<f32>;
@group(1) @binding(6) var material_tex3: texture_2d<f32>;
@group(1) @binding(7) var material_tex4: texture_2d<f32>;
@group(1) @binding(8) var material_tex5: texture_2d<f32>;
@group(1) @binding(9) var material_tex6: texture_2d<f32>;
@group(1) @binding(10) var material_tex7: texture_2d<f32>;

struct FrameSemanticMeta {
    resolution: vec2<f32>,
    inv_resolution: vec2<f32>,
    frame_index: u32,
    flags: u32,
}
@group(2) @binding(0) var frame_scene_color: texture_2d<f32>;
@group(2) @binding(1) var frame_scene_depth: texture_depth_2d;
@group(2) @binding(2) var frame_history0: texture_2d<f32>;
@group(2) @binding(3) var frame_history1: texture_2d<f32>;
@group(2) @binding(4) var frame_linear_sampler: sampler;
@group(2) @binding(5) var frame_point_sampler: sampler;
@group(2) @binding(6) var<uniform> frame_semantics: FrameSemanticMeta;

struct VertexInput {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
    instance_index: u32,
}

struct VertexOutput {
    clip_position: vec4<f32>,
    color: vec4<f32>,
    uv: vec2<f32>,
    world_pos: vec3<f32>,
}

struct FragmentInput {
    color: vec4<f32>,
    uv: vec2<f32>,
    world_pos: vec3<f32>,
}

struct FragmentOutput {
    color: vec4<f32>,
}

fn sample_material(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(material_tex0, linear_clamp_sampler, uv);
}

fn scene_resolution() -> vec2<f32> {
    return frame_semantics.resolution;
}

fn scene_inv_resolution() -> vec2<f32> {
    return frame_semantics.inv_resolution;
}

fn current_frame_index() -> u32 {
    return frame_semantics.frame_index;
}

fn sample_scene_color(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(frame_scene_color, frame_linear_sampler, uv);
}

fn sample_history0(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(frame_history0, frame_linear_sampler, uv);
}

fn sample_history1(uv: vec2<f32>) -> vec4<f32> {
    return textureSample(frame_history1, frame_linear_sampler, uv);
}

fn load_scene_depth(pixel: vec2<i32>) -> f32 {
    return textureLoad(frame_scene_depth, pixel, 0);
}

fn unwrap_angle_near(angle: f32, reference: f32) -> f32 {
    var unwrapped = angle;
    let pi = 3.14159265359;
    let tau = 6.28318530718;
    loop {
        if (unwrapped - reference > pi) {
            unwrapped = unwrapped - tau;
            continue;
        }
        if (unwrapped - reference < -pi) {
            unwrapped = unwrapped + tau;
            continue;
        }
        break;
    }
    return unwrapped;
}

fn sample_shadow_data_2d(
    layer: i32,
    shadow_res: f32,
    shadow_res_i: i32,
    angular_x: f32,
    candidate_slot: u32,
) -> ShadowSample2D {
    let wrapped = angular_x - floor(angular_x / shadow_res) * shadow_res;
    let x0 = i32(floor(wrapped)) % shadow_res_i;
    let samples_per_direction = 8u;
    let direction_index = u32(max(layer, 0)) * u32(shadow_res_i) + u32(max(x0, 0));
    let index = direction_index * samples_per_direction + min(candidate_slot, samples_per_direction - 1u);
    return shadow_samples_2d[index];
}

fn cross2_2d(a: vec2<f32>, b: vec2<f32>) -> f32 {
    return a.x * b.y - a.y * b.x;
}

fn ray_segment_hit_distance_2d(
    ray_origin: vec2<f32>,
    ray_dir: vec2<f32>,
    segment_a: vec2<f32>,
    segment_b: vec2<f32>,
) -> f32 {
    let v1 = ray_origin - segment_a;
    let v2 = segment_b - segment_a;
    let den = cross2_2d(ray_dir, v2);
    if (abs(den) <= 1e-6) {
        return -1.0;
    }
    let t = cross2_2d(v2, v1) / den;
    let q = cross2_2d(ray_dir, v1) / den;
    if (t >= 0.0 && q >= 0.0 && q <= 1.0) {
        return t;
    }
    return -1.0;
}

fn ray_hits_occluder_before_light_2d(
    ray_origin: vec2<f32>,
    ray_dir: vec2<f32>,
    max_distance: f32,
    sample: ShadowSample2D,
) -> bool {
    let hit0 = ray_segment_hit_distance_2d(ray_origin, ray_dir, sample.occluder_v0, sample.occluder_v1);
    let hit1 = ray_segment_hit_distance_2d(ray_origin, ray_dir, sample.occluder_v1, sample.occluder_v2);
    let hit2 = ray_segment_hit_distance_2d(ray_origin, ray_dir, sample.occluder_v2, sample.occluder_v3);
    let hit3 = ray_segment_hit_distance_2d(ray_origin, ray_dir, sample.occluder_v3, sample.occluder_v0);
    let min_hit = min(min(select(1e20, hit0, hit0 >= 0.0), select(1e20, hit1, hit1 >= 0.0)), min(select(1e20, hit2, hit2 >= 0.0), select(1e20, hit3, hit3 >= 0.0)));
    return min_hit < max(max_distance, 0.0);
}

fn occluder_covers_light_disk_from_receiver(
    sample: ShadowSample2D,
    world_xy: vec2<f32>,
    light_xy: vec2<f32>,
    light_radius: f32,
) -> bool {
    let to_light = light_xy - world_xy;
    let light_distance = length(to_light);
    if (light_distance <= max(light_radius, 0.0001)) {
        return false;
    }

    let center_dir = to_light / light_distance;
    let light_radius_clamped = max(light_radius, 0.0);
    let half_angle = asin(clamp(light_radius_clamped / max(light_distance, 0.0001), 0.0, 0.9999));
    let tangent_distance = sqrt(max(light_distance * light_distance - light_radius_clamped * light_radius_clamped, 0.0));
    let c = cos(half_angle);
    let s = sin(half_angle);
    let left_dir = vec2<f32>(
        center_dir.x * c - center_dir.y * s,
        center_dir.x * s + center_dir.y * c,
    );
    let right_dir = vec2<f32>(
        center_dir.x * c + center_dir.y * s,
        -center_dir.x * s + center_dir.y * c,
    );

    return ray_hits_occluder_before_light_2d(world_xy, center_dir, light_distance, sample)
        && ray_hits_occluder_before_light_2d(world_xy, left_dir, tangent_distance, sample)
        && ray_hits_occluder_before_light_2d(world_xy, right_dir, tangent_distance, sample);
}

fn occluder_intersects_light_disk_from_receiver(
    sample: ShadowSample2D,
    world_xy: vec2<f32>,
    light_xy: vec2<f32>,
    light_radius: f32,
) -> bool {
    let to_light = light_xy - world_xy;
    let light_distance = length(to_light);
    if (light_distance <= max(light_radius, 0.0001)) {
        return false;
    }

    let center_dir = to_light / light_distance;
    let light_radius_clamped = max(light_radius, 0.0);
    let half_angle = asin(clamp(light_radius_clamped / max(light_distance, 0.0001), 0.0, 0.9999));
    let tangent_distance = sqrt(max(light_distance * light_distance - light_radius_clamped * light_radius_clamped, 0.0));
    let c = cos(half_angle);
    let s = sin(half_angle);
    let left_dir = vec2<f32>(
        center_dir.x * c - center_dir.y * s,
        center_dir.x * s + center_dir.y * c,
    );
    let right_dir = vec2<f32>(
        center_dir.x * c + center_dir.y * s,
        -center_dir.x * s + center_dir.y * c,
    );

    return ray_hits_occluder_before_light_2d(world_xy, center_dir, light_distance, sample)
        || ray_hits_occluder_before_light_2d(world_xy, left_dir, tangent_distance, sample)
        || ray_hits_occluder_before_light_2d(world_xy, right_dir, tangent_distance, sample);
}

fn clipped_edge_light_overlap(
    world_xy: vec2<f32>,
    edge_a: vec2<f32>,
    edge_b: vec2<f32>,
    light_angle: f32,
    light_left: f32,
    light_right: f32,
) -> vec2<f32> {
    let raw_a = atan2((edge_a - world_xy).y, (edge_a - world_xy).x);
    let raw_b = atan2((edge_b - world_xy).y, (edge_b - world_xy).x);
    var a = unwrap_angle_near(raw_a, light_angle);
    var b = unwrap_angle_near(raw_b, a);
    if (b < a) {
        let tmp = a;
        a = b;
        b = tmp;
    }
    if (b - a > 3.14159265359) {
        let tmp = a;
        a = b;
        b = tmp + 6.28318530718;
    }
    a = unwrap_angle_near(a, light_angle);
    b = unwrap_angle_near(b, a);
    if (b < a) {
        b = b + 6.28318530718;
    }

    let overlap_left = max(light_left, a);
    let overlap_right = min(light_right, b);
    if (overlap_right <= overlap_left) {
        return vec2<f32>(0.0, 0.0);
    }
    return vec2<f32>(overlap_left, overlap_right);
}

fn merge_interval_length_4(
    i0: vec2<f32>,
    i1: vec2<f32>,
    i2: vec2<f32>,
    i3: vec2<f32>,
) -> f32 {
    var starts = array<f32, 4>(i0.x, i1.x, i2.x, i3.x);
    var ends = array<f32, 4>(i0.y, i1.y, i2.y, i3.y);
    for (var i = 0u; i < 4u; i = i + 1u) {
        for (var j = i + 1u; j < 4u; j = j + 1u) {
            if (starts[j] < starts[i]) {
                let start_tmp = starts[i];
                let end_tmp = ends[i];
                starts[i] = starts[j];
                ends[i] = ends[j];
                starts[j] = start_tmp;
                ends[j] = end_tmp;
            }
        }
    }

    var total = 0.0;
    var current_start = 0.0;
    var current_end = 0.0;
    var has_interval = false;
    for (var i = 0u; i < 4u; i = i + 1u) {
        if (ends[i] <= starts[i]) {
            continue;
        }
        if (!has_interval) {
            current_start = starts[i];
            current_end = ends[i];
            has_interval = true;
            continue;
        }
        if (starts[i] <= current_end) {
            current_end = max(current_end, ends[i]);
        } else {
            total = total + max(current_end - current_start, 0.0);
            current_start = starts[i];
            current_end = ends[i];
        }
    }
    if (has_interval) {
        total = total + max(current_end - current_start, 0.0);
    }
    return total;
}

fn occluder_light_disk_visibility_from_receiver(
    sample: ShadowSample2D,
    world_xy: vec2<f32>,
    light_xy: vec2<f32>,
    light_radius: f32,
) -> f32 {
    let to_light = light_xy - world_xy;
    let light_distance = length(to_light);
    let light_radius_clamped = max(light_radius, 0.0);
    if (light_distance <= max(light_radius_clamped, 0.0001)) {
        return 1.0;
    }

    let light_angle = atan2(to_light.y, to_light.x);
    let light_half_angle = asin(clamp(light_radius_clamped / max(light_distance, 0.0001), 0.0, 0.9999));
    let light_left = light_angle - light_half_angle;
    let light_right = light_angle + light_half_angle;

    let edge0 = clipped_edge_light_overlap(world_xy, sample.occluder_v0, sample.occluder_v1, light_angle, light_left, light_right);
    let edge1 = clipped_edge_light_overlap(world_xy, sample.occluder_v1, sample.occluder_v2, light_angle, light_left, light_right);
    let edge2 = clipped_edge_light_overlap(world_xy, sample.occluder_v2, sample.occluder_v3, light_angle, light_left, light_right);
    let edge3 = clipped_edge_light_overlap(world_xy, sample.occluder_v3, sample.occluder_v0, light_angle, light_left, light_right);
    let blocked_angle = merge_interval_length_4(edge0, edge1, edge2, edge3);
    let coverage = clamp(blocked_angle / max(light_right - light_left, 1e-5), 0.0, 1.0);
    return 1.0 - coverage;
}

fn shadow_visibility_from_sample(
    sample: ShadowSample2D,
    angle: f32,
    dist_ratio: f32,
    contact_offset: f32,
    world_xy: vec2<f32>,
    light_xy: vec2<f32>,
    light_radius: f32,
) -> f32 {
    if (sample.flags < 0.5) {
        return select(0.0, 1.0, dist_ratio <= sample.blocker_distance + contact_offset);
    }

    if (occluder_covers_light_disk_from_receiver(sample, world_xy, light_xy, light_radius)) {
        return 0.0;
    }
    if (!occluder_intersects_light_disk_from_receiver(sample, world_xy, light_xy, light_radius)) {
        return 1.0;
    }

    return occluder_light_disk_visibility_from_receiver(sample, world_xy, light_xy, light_radius);
}

fn apply_2d_lighting(base_color: vec4<f32>, world_pos: vec3<f32>) -> vec4<f32> {
    var lit = vec3<f32>(camera.shadow_params.z, camera.shadow_params.z, camera.shadow_params.z);
    var max_shadow_occlusion = 0.0;
    let receives_shadow = camera.shadow_params.y > 0.5;
    let debug_mode = i32(camera.shadow_controls.w);
    let light_count = min(camera.light_offset_count.y, 64u);
    for (var i: u32 = 0u; i < light_count; i = i + 1u) {
        let debug_light_index = i32(camera.shadow_controls.z);
        if (debug_light_index >= 0 && i32(i) != debug_light_index) {
            continue;
        }
        let l = lights_2d[camera.light_offset_count.x + i];
        if ((l.shadow_layer_mask & camera.light_offset_count.z) == 0u) {
            continue;
        }
        let from_light = world_pos.xy - l.position.xy;
        let dist = length(from_light);
        let range = max(l.intensity_range.y, 0.0001);
        let dist_ratio = clamp(dist / range, 0.0, 1.0);
        let angle = atan2(from_light.y, from_light.x);
        let tau = 6.28318530718;
        let angular_u = fract((angle / tau) + 1.0);
        let shadow_res = max(camera.shadow_params.w, 1.0);
        let layer = i32(l.shadow_index);
        let shadow_res_i = i32(shadow_res);
        let angular_x = angular_u * shadow_res;
        let contact_offset = max(camera.shadow_controls.x, 0.0);
        var shadow_visibility = 1.0;
        var full_umbra = false;
        for (var direction_offset = -1i; direction_offset <= 1i; direction_offset = direction_offset + 1i) {
            let sample_x = angular_x + f32(direction_offset);
            for (var candidate_slot = 0u; candidate_slot < 8u; candidate_slot = candidate_slot + 1u) {
                let shadow_sample = sample_shadow_data_2d(
                    layer,
                    shadow_res,
                    shadow_res_i,
                    sample_x,
                    candidate_slot
                );
                if (shadow_sample.flags >= 0.5 && occluder_covers_light_disk_from_receiver(
                    shadow_sample,
                    world_pos.xy,
                    l.position.xy,
                    l.light_radius
                )) {
                    full_umbra = true;
                }
            }
        }
        if (full_umbra) {
            shadow_visibility = 0.0;
        } else {
            for (var candidate_slot = 0u; candidate_slot < 8u; candidate_slot = candidate_slot + 1u) {
                let shadow_sample = sample_shadow_data_2d(
                    layer,
                    shadow_res,
                    shadow_res_i,
                    angular_x,
                    candidate_slot
                );
                shadow_visibility = shadow_visibility * shadow_visibility_from_sample(
                        shadow_sample,
                        angle,
                        dist_ratio,
                        contact_offset,
                        world_pos.xy,
                        l.position.xy,
                        l.light_radius
                    );
            }
        }
        let visibility = select(1.0, shadow_visibility, receives_shadow);
        max_shadow_occlusion = max(max_shadow_occlusion, 1.0 - visibility);
        let t = clamp(1.0 - (dist / range), 0.0, 1.0);
        let attenuation = t * t * (3.0 - 2.0 * t);
        lit += l.color.rgb * attenuation * l.intensity_range.x * visibility;
    }
    if (receives_shadow) {
        // Preserve readable shadow silhouettes even with strong opposing lights.
        lit *= (1.0 - max_shadow_occlusion * 0.35);
    }
    if (debug_mode == 1) {
        return vec4<f32>(vec3<f32>(max_shadow_occlusion), 1.0);
    }
    return vec4<f32>(base_color.rgb * lit * camera.shadow_params.x, base_color.a);
}

struct VertexStageInput {
    @location(0) position: vec3<f32>,
    @location(4) uv: vec2<f32>,
    @builtin(instance_index) instance_index: u32,
}

struct VertexStageOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) world_pos: vec3<f32>,
}
"#
}

fn two_d_composer_postlude() -> &'static str {
    r#"
@vertex
fn vs_main(input: VertexStageInput) -> VertexStageOutput {
    let logical_input = VertexInput(
        input.position,
        vec3<f32>(0.0, 0.0, 1.0),
        input.uv,
        input.instance_index,
    );
    var logical_output = vertex(logical_input);
    if all(logical_output.clip_position == vec4<f32>(0.0)) {
        logical_output.clip_position = camera.view_projection * (camera.model_matrix * vec4<f32>(input.position, 1.0));
    }
    if all(logical_output.color == vec4<f32>(0.0)) {
        logical_output.color = camera.tint;
    }
    if all(logical_output.uv == vec2<f32>(0.0)) {
        logical_output.uv = input.uv;
    }
    if all(logical_output.world_pos == vec3<f32>(0.0)) {
        logical_output.world_pos = (camera.model_matrix * vec4<f32>(input.position, 1.0)).xyz;
    }

    var out: VertexStageOutput;
    out.clip_position = logical_output.clip_position;
    out.color = logical_output.color;
    out.uv = logical_output.uv;
    out.world_pos = logical_output.world_pos;
    return out;
}

@fragment
fn fs_main(input: VertexStageOutput) -> @location(0) vec4<f32> {
    let logical_input = FragmentInput(input.color, input.uv, input.world_pos);
    let logical_output = fragment(logical_input);
    return apply_2d_lighting(logical_output.color, input.world_pos);
}
"#
}

fn compose_material_wgsl(
    realm: MaterialShaderRealm,
    shader_type: MaterialShaderType,
    snippet: &str,
) -> Result<String, String> {
    match shader_type {
        MaterialShaderType::Model => {
            validate_logical_shader_source(shader_type, snippet)?;
            let (prelude, postlude) = match realm {
                MaterialShaderRealm::ThreeD => {
                    (model_composer_prelude(), model_composer_postlude())
                }
                MaterialShaderRealm::TwoD => (two_d_composer_prelude(), two_d_composer_postlude()),
            };
            Ok(format!(
                "// generated_common_prelude\n{}\n// source\n{}\n// generated_postlude\n{}",
                prelude,
                snippet.trim(),
                postlude
            ))
        }
        MaterialShaderType::Particle => {
            validate_logical_shader_source(shader_type, snippet)?;
            Err("Particle material shader generation is not implemented yet".to_string())
        }
    }
}

pub fn compile_material_shader_spec(
    spec: &MaterialShaderCompileSpec,
) -> Result<CompiledMaterialShader, String> {
    compile_material_shader_spec_for_realm(spec, MaterialShaderRealm::ThreeD)
}

pub fn compile_material_shader_spec_for_realm(
    spec: &MaterialShaderCompileSpec,
    realm: MaterialShaderRealm,
) -> Result<CompiledMaterialShader, String> {
    if spec.shader_source.trim().is_empty() {
        return Err("shader_source is required and cannot be empty".to_string());
    }
    let source = compose_material_wgsl(realm, spec.shader_type, &spec.shader_source)?;

    if let Err(err) = naga::front::wgsl::parse_str(&source) {
        return Err(format!(
            "Material WGSL is invalid: {}",
            err.emit_to_string(&source)
        ));
    }

    let mut hasher = DefaultHasher::new();
    spec.base_preset.hash(&mut hasher);
    realm.hash(&mut hasher);
    spec.shader_type.hash(&mut hasher);
    source.hash(&mut hasher);
    let mut params: Vec<_> = spec.shader_params_schema.iter().collect();
    params.sort_by(|a, b| a.0.cmp(b.0));
    for (name, ty) in params {
        name.hash(&mut hasher);
        ty.hash(&mut hasher);
    }
    let hash = hasher.finish();

    Ok(CompiledMaterialShader { source, hash })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_standard_preset() {
        let spec = MaterialShaderCompileSpec {
            base_preset: MaterialShaderBasePreset::Standard,
            shader_type: MaterialShaderType::Model,
            shader_source: builtin_material_source(MaterialShaderBasePreset::Standard).to_string(),
            shader_params_schema: HashMap::new(),
            capabilities: Default::default(),
        };
        let compiled = compile_material_shader_spec(&spec).expect("standard should compile");
        assert!(!compiled.source.is_empty());
        assert_ne!(compiled.hash, 0);
        assert!(compiled.source.contains("fn shade_standard("));
        assert!(!compiled.source.contains("fn shade_pbr("));
        assert!(compiled.source.contains("sample_shadow_for_light("));
        assert!(
            compiled
                .source
                .contains("visible_counts[light_params.camera_index]")
        );
        assert!(!compiled.source.contains("sample_shadow_primary_light("));
        assert!(!compiled.source.contains("get_primary_light_direction("));
    }

    #[test]
    fn compiles_pbr_preset() {
        let spec = MaterialShaderCompileSpec {
            base_preset: MaterialShaderBasePreset::Pbr,
            shader_type: MaterialShaderType::Model,
            shader_source: builtin_material_source(MaterialShaderBasePreset::Pbr).to_string(),
            shader_params_schema: HashMap::new(),
            capabilities: Default::default(),
        };
        let compiled = compile_material_shader_spec(&spec).expect("pbr should compile");
        assert!(!compiled.source.is_empty());
        assert_ne!(compiled.hash, 0);
        assert!(compiled.source.contains("fn shade_pbr("));
        assert!(!compiled.source.contains("fn shade_standard("));
        assert!(compiled.source.contains("input.receive_shadow > 0.5"));
        assert!(compiled.source.contains("visible_indices[visible_index]"));
    }

    #[test]
    fn composes_model_logical_snippet() {
        let spec = MaterialShaderCompileSpec {
            base_preset: MaterialShaderBasePreset::Standard,
            shader_type: MaterialShaderType::Model,
            shader_source: r#"
fn vertex(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.world_position = input.position;
  out.world_normal = input.normal;
  out.uv = input.uv;
  out.clip_position = vec4<f32>(0.0);
  return out;
}
fn fragment(input: FragmentInput) -> FragmentOutput {
  var out: FragmentOutput;
  out.color = vec4<f32>(input.uv, 1.0, 1.0);
  out.emissive = vec4<f32>(0.0);
  return out;
}
"#
            .to_string(),
            shader_params_schema: HashMap::new(),
            capabilities: Default::default(),
        };
        let compiled = compile_material_shader_spec(&spec).expect("custom should compile");
        assert_ne!(compiled.hash, 0);
        assert!(compiled.source.contains("@vertex"));
    }

    #[test]
    fn rejects_invalid_model_contract() {
        let spec = MaterialShaderCompileSpec {
            base_preset: MaterialShaderBasePreset::Standard,
            shader_type: MaterialShaderType::Model,
            shader_source: "fn fragment() -> i32 { return 0; }".to_string(),
            shader_params_schema: HashMap::new(),
            capabilities: Default::default(),
        };
        let err = compile_material_shader_spec(&spec).expect_err("invalid model contract");
        assert!(err.contains("Model shader must define both"));
    }

    #[test]
    fn hash_is_stable() {
        let spec = MaterialShaderCompileSpec {
            base_preset: MaterialShaderBasePreset::Standard,
            shader_type: MaterialShaderType::Model,
            shader_source: r#"
fn vertex(input: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.world_position = input.position;
  out.world_normal = input.normal;
  out.uv = input.uv;
  out.clip_position = vec4<f32>(0.0);
  return out;
}
fn fragment(input: FragmentInput) -> FragmentOutput {
  var out: FragmentOutput;
  out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
  out.emissive = vec4<f32>(0.0);
  return out;
}
"#
            .to_string(),
            shader_params_schema: HashMap::from([(String::from("a"), String::from("f32"))]),
            capabilities: Default::default(),
        };
        let a = compile_material_shader_spec(&spec).expect("first compile");
        let b = compile_material_shader_spec(&spec).expect("second compile");
        assert_eq!(a.hash, b.hash);
    }

    #[test]
    fn compiles_standard_2d_realm() {
        let spec = MaterialShaderCompileSpec {
            base_preset: MaterialShaderBasePreset::Standard,
            shader_type: MaterialShaderType::Model,
            shader_source: builtin_material_source_2d().to_string(),
            shader_params_schema: HashMap::new(),
            capabilities: Default::default(),
        };
        let compiled = compile_material_shader_spec_for_realm(&spec, MaterialShaderRealm::TwoD)
            .expect("2d standard should compile");
        assert!(compiled.source.contains("struct CameraUniform"));
        assert!(compiled.source.contains("fn sample_material("));
        assert!(compiled.source.contains("@vertex"));
        assert!(compiled.source.contains("@fragment"));
        assert!(
            compiled
                .source
                .contains("let angular_x = angular_u * shadow_res;")
        );
        assert!(compiled.source.contains("fn sample_shadow_data_2d("));
    }
}
