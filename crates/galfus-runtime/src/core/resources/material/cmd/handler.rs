use super::types::*;
use super::utils::pack_schema_material;
use crate::core::cmd::EngineEvent;
use crate::core::id_policy::validate_host_logical_id;
use crate::core::resources::{
    MATERIAL_DEFINITION_PBR_ID, MATERIAL_DEFINITION_PBR_SLUG, MATERIAL_DEFINITION_STANDARD_2D_ID,
    MATERIAL_DEFINITION_STANDARD_2D_SLUG, MATERIAL_DEFINITION_STANDARD_ID,
    MATERIAL_DEFINITION_STANDARD_SLUG, MATERIAL_FALLBACK_ID, MaterialDefinitionRecord,
    MaterialInstanceRecord, MaterialRealmKind, MaterialShaderType, ShaderMaterialPreset,
    ShaderMaterialRecord,
};
use crate::core::state::EngineState;
use crate::core::system::SystemEvent;
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

fn default_capabilities_for_realm(capabilities: Option<Vec<String>>) -> Vec<String> {
    if let Some(value) = capabilities {
        return value;
    }
    Vec::new()
}

fn to_render_preset(preset: ShaderMaterialPreset) -> galfus_render::MaterialShaderBasePreset {
    match preset {
        ShaderMaterialPreset::Standard => galfus_render::MaterialShaderBasePreset::Standard,
        ShaderMaterialPreset::Pbr => galfus_render::MaterialShaderBasePreset::Pbr,
    }
}

fn to_render_shader_type(shader_type: MaterialShaderType) -> galfus_render::MaterialShaderType {
    match shader_type {
        MaterialShaderType::Model => galfus_render::MaterialShaderType::Model,
        MaterialShaderType::Particle => galfus_render::MaterialShaderType::Particle,
    }
}

fn definition_is_broken(definition: &MaterialDefinitionRecord) -> bool {
    definition.compile_error.is_some() || definition.compiled_shader_source.is_none()
}

fn validate_preset_realm(
    preset: ShaderMaterialPreset,
    realm_kind: MaterialRealmKind,
) -> Result<(), String> {
    if preset == ShaderMaterialPreset::Pbr && realm_kind == MaterialRealmKind::TwoD {
        return Err("PBR preset is only supported for ThreeD realm".to_string());
    }
    Ok(())
}

fn compile_material_program(
    engine: &mut EngineState,
    realm_kind: MaterialRealmKind,
    preset: ShaderMaterialPreset,
    shader_type: MaterialShaderType,
    shader_source: String,
    shader_params_schema: HashMap<String, String>,
) -> Result<galfus_render::CompiledMaterialShader, String> {
    const MATERIAL_PROGRAM_CACHE_MAX_UNUSED_FRAMES: u64 = 600;
    const MATERIAL_PROGRAM_CACHE_MAX_ENTRIES_SOFT: usize = 512;
    const MATERIAL_PROGRAM_CACHE_MAX_ENTRIES_HARD: usize = 1024;

    let spec = galfus_render::MaterialShaderCompileSpec {
        base_preset: to_render_preset(preset),
        shader_type: to_render_shader_type(shader_type),
        shader_source,
        shader_params_schema,
        capabilities: Default::default(),
    };
    let realm = match realm_kind {
        MaterialRealmKind::ThreeD => galfus_render::MaterialShaderRealm::ThreeD,
        MaterialRealmKind::TwoD => galfus_render::MaterialShaderRealm::TwoD,
    };

    let cache_key = {
        let mut hasher = DefaultHasher::new();
        realm.hash(&mut hasher);
        to_render_preset(preset).hash(&mut hasher);
        to_render_shader_type(shader_type).hash(&mut hasher);
        spec.shader_source.hash(&mut hasher);
        let mut params: Vec<_> = spec.shader_params_schema.iter().collect();
        params.sort_by(|a, b| a.0.cmp(b.0));
        for (name, ty) in params {
            name.hash(&mut hasher);
            ty.hash(&mut hasher);
        }
        hasher.finish()
    };

    if let Some(compiled) = engine
        .universal_state
        .scene
        .material_program_cache
        .get(&cache_key)
        .cloned()
    {
        let frame_index = engine.runtime.frame_index();
        engine
            .universal_state
            .scene
            .material_program_cache_last_used_frame
            .insert(cache_key, frame_index);
        galfus_log::galfus_log_debug!(
            engine,
            "material.cache.hit",
            "key={} preset={:?} shader_type={:?}",
            cache_key,
            preset,
            shader_type
        );
        return Ok(compiled);
    }

    galfus_log::galfus_log_debug!(
        engine,
        "material.cache.miss",
        "key={} preset={:?} shader_type={:?}",
        cache_key,
        preset,
        shader_type
    );

    let compiled = galfus_render::compile_material_shader_spec_for_realm(&spec, realm)?;
    let frame_index = engine.runtime.frame_index();
    engine
        .universal_state
        .scene
        .material_program_cache
        .insert(cache_key, compiled.clone());
    engine
        .universal_state
        .scene
        .material_program_cache_last_used_frame
        .insert(cache_key, frame_index);
    let active_hashes: HashSet<u64> = engine
        .universal_state
        .scene
        .material_definitions
        .values()
        .filter_map(|record| {
            if record.compiled_shader_hash != 0 {
                Some(record.compiled_shader_hash)
            } else {
                None
            }
        })
        .collect();
    let should_prune = engine.universal_state.scene.material_program_cache.len()
        > MATERIAL_PROGRAM_CACHE_MAX_ENTRIES_SOFT;
    if should_prune {
        let before_prune = engine.universal_state.scene.material_program_cache.len();
        engine
            .universal_state
            .scene
            .material_program_cache
            .retain(|key, value| {
                let last_used = engine
                    .universal_state
                    .scene
                    .material_program_cache_last_used_frame
                    .get(key)
                    .copied()
                    .unwrap_or(frame_index);
                let within_window = frame_index.saturating_sub(last_used)
                    <= MATERIAL_PROGRAM_CACHE_MAX_UNUSED_FRAMES;
                within_window || active_hashes.contains(&value.hash)
            });
        let evicted =
            before_prune.saturating_sub(engine.universal_state.scene.material_program_cache.len());
        engine.profiling.render.material_program_cache_evictions = engine
            .profiling
            .render
            .material_program_cache_evictions
            .saturating_add(evicted as u32);
    }
    if engine.universal_state.scene.material_program_cache.len()
        > MATERIAL_PROGRAM_CACHE_MAX_ENTRIES_HARD
    {
        let mut stale_keys: Vec<(u64, u64)> = engine
            .universal_state
            .scene
            .material_program_cache
            .iter()
            .map(|(key, _)| {
                let last_used = engine
                    .universal_state
                    .scene
                    .material_program_cache_last_used_frame
                    .get(key)
                    .copied()
                    .unwrap_or(0);
                (*key, last_used)
            })
            .collect();
        stale_keys.sort_by_key(|(_, last_used)| *last_used);
        let overflow = engine
            .universal_state
            .scene
            .material_program_cache
            .len()
            .saturating_sub(MATERIAL_PROGRAM_CACHE_MAX_ENTRIES_HARD);
        for (key, _) in stale_keys.into_iter().take(overflow) {
            engine
                .universal_state
                .scene
                .material_program_cache
                .remove(&key);
            engine
                .universal_state
                .scene
                .material_program_cache_last_used_frame
                .remove(&key);
        }
        engine.profiling.render.material_program_cache_evictions = engine
            .profiling
            .render
            .material_program_cache_evictions
            .saturating_add(overflow as u32);
    }
    engine
        .universal_state
        .scene
        .material_program_cache_last_used_frame
        .retain(|key, _| {
            engine
                .universal_state
                .scene
                .material_program_cache
                .contains_key(key)
        });
    Ok(compiled)
}

fn builtin_shader_source(preset: ShaderMaterialPreset) -> &'static str {
    match preset {
        ShaderMaterialPreset::Standard => galfus_render::builtin_material_source(
            galfus_render::MaterialShaderBasePreset::Standard,
        ),
        ShaderMaterialPreset::Pbr => {
            galfus_render::builtin_material_source(galfus_render::MaterialShaderBasePreset::Pbr)
        }
    }
}

fn ensure_fallback_material_instance(engine: &mut EngineState) {
    let Some(standard_definition) = engine
        .universal_state
        .scene
        .material_definitions
        .get(&MATERIAL_DEFINITION_STANDARD_ID)
        .cloned()
    else {
        return;
    };
    if definition_is_broken(&standard_definition) {
        return;
    }
    let resources = &mut engine.universal_state.scene.realm3d;
    let mut fallback = resources
        .materials
        .remove(&MATERIAL_FALLBACK_ID)
        .unwrap_or_else(|| ShaderMaterialRecord::new_standard(Some("Fallback Material".into())));
    sync_material_from_definition(&mut fallback, &standard_definition);
    fallback.bind_group = None;
    resources.materials.insert(MATERIAL_FALLBACK_ID, fallback);

    engine.universal_state.scene.material_instances.insert(
        MATERIAL_FALLBACK_ID,
        MaterialInstanceRecord {
            material_id: MATERIAL_FALLBACK_ID,
            definition_id: MATERIAL_DEFINITION_STANDARD_ID,
            label: Some("fallback-standard".to_string()),
        },
    );
}

fn broken_definition_result(
    message: String,
    fallback_preserved: bool,
) -> CmdResultMaterialDefinition {
    let suffix = if fallback_preserved {
        " (fallback instance preserved)"
    } else {
        ""
    };
    CmdResultMaterialDefinition {
        success: false,
        message: format!("{message}{suffix}"),
    }
}

fn bootstrap_builtin_material_definitions(engine: &mut EngineState) {
    if engine
        .universal_state
        .scene
        .material_definitions
        .contains_key(&MATERIAL_DEFINITION_STANDARD_ID)
        && engine
            .universal_state
            .scene
            .material_definitions
            .contains_key(&MATERIAL_DEFINITION_PBR_ID)
        && engine
            .universal_state
            .scene
            .material_definitions
            .contains_key(&MATERIAL_DEFINITION_STANDARD_2D_ID)
    {
        return;
    }

    let candidates = [
        (
            MATERIAL_DEFINITION_STANDARD_ID,
            MATERIAL_DEFINITION_STANDARD_SLUG,
            ShaderMaterialPreset::Standard,
            MaterialRealmKind::ThreeD,
            Some("builtin-standard".to_string()),
        ),
        (
            MATERIAL_DEFINITION_PBR_ID,
            MATERIAL_DEFINITION_PBR_SLUG,
            ShaderMaterialPreset::Pbr,
            MaterialRealmKind::ThreeD,
            Some("builtin-pbr".to_string()),
        ),
        (
            MATERIAL_DEFINITION_STANDARD_2D_ID,
            MATERIAL_DEFINITION_STANDARD_2D_SLUG,
            ShaderMaterialPreset::Standard,
            MaterialRealmKind::TwoD,
            Some("builtin-standard-2d".to_string()),
        ),
    ];

    for (definition_id, slug, preset, realm_kind, label) in candidates {
        if engine
            .universal_state
            .scene
            .material_definitions
            .contains_key(&definition_id)
        {
            continue;
        }
        let shader_source = if realm_kind == MaterialRealmKind::TwoD {
            galfus_render::builtin_material_source_2d().to_string()
        } else {
            builtin_shader_source(preset).to_string()
        };
        let shader_capabilities = default_capabilities_for_realm(None);
        let compile_result = compile_material_program(
            engine,
            realm_kind,
            preset,
            MaterialShaderType::Model,
            shader_source.clone(),
            HashMap::new(),
        )
        .map(|compiled| (compiled.source, compiled.hash));
        let (compiled_source, compiled_hash, compile_error) = match compile_result {
            Ok((source, hash)) => (Some(source), hash, None),
            Err(error) => (None, 0, Some(error)),
        };
        engine.universal_state.scene.material_definitions.insert(
            definition_id,
            MaterialDefinitionRecord {
                definition_id,
                slug: slug.to_string(),
                label,
                realm_kind,
                base_preset: preset,
                shader_type: MaterialShaderType::Model,
                shader_source: Some(shader_source),
                shader_params_schema: HashMap::new(),
                shader_capabilities,
                compiled_shader_hash: compiled_hash,
                compiled_shader_source: compiled_source,
                compile_error,
            },
        );
    }
    ensure_fallback_material_instance(engine);
}

pub fn ensure_material_bootstrap_defaults(engine: &mut EngineState) {
    bootstrap_builtin_material_definitions(engine);
}

fn definition_by_slug(engine: &mut EngineState, slug: &str) -> Option<MaterialDefinitionRecord> {
    bootstrap_builtin_material_definitions(engine);
    engine
        .universal_state
        .scene
        .material_definitions
        .values()
        .find(|definition| definition.slug == slug)
        .cloned()
}

fn definition_by_id(
    engine: &mut EngineState,
    definition_id: u32,
) -> Option<MaterialDefinitionRecord> {
    bootstrap_builtin_material_definitions(engine);
    engine
        .universal_state
        .scene
        .material_definitions
        .get(&definition_id)
        .cloned()
}

fn sync_material_from_definition(
    record: &mut ShaderMaterialRecord,
    definition: &MaterialDefinitionRecord,
) {
    record.realm_kind = definition.realm_kind;
    record.base_preset = definition.base_preset;
    record.preset = definition.base_preset;
    record.shader_type = definition.shader_type;
    record.shader_source = definition.shader_source.clone();
    record.shader_params_schema = definition.shader_params_schema.clone();
    record.shader_capabilities = definition.shader_capabilities.clone();
    record.compiled_shader_hash = definition.compiled_shader_hash;
    record.compiled_shader_source = definition.compiled_shader_source.clone();
    record.compile_error = definition.compile_error.clone();
}

fn fallback_definition_for_realm(
    engine: &mut EngineState,
    realm_kind: MaterialRealmKind,
) -> Option<MaterialDefinitionRecord> {
    let fallback_id = match realm_kind {
        MaterialRealmKind::TwoD => MATERIAL_DEFINITION_STANDARD_2D_ID,
        MaterialRealmKind::ThreeD => MATERIAL_DEFINITION_STANDARD_ID,
    };
    definition_by_id(engine, fallback_id)
}

pub fn engine_cmd_material_definition_create(
    engine: &mut EngineState,
    args: &CmdMaterialDefinitionCreateArgs,
) -> CmdResultMaterialDefinition {
    if let Err(message) = validate_host_logical_id(args.definition_id, "definitionId") {
        return CmdResultMaterialDefinition {
            success: false,
            message,
        };
    }
    bootstrap_builtin_material_definitions(engine);

    if engine
        .universal_state
        .scene
        .material_definitions
        .contains_key(&args.definition_id)
    {
        return CmdResultMaterialDefinition {
            success: false,
            message: format!(
                "Material definition with id {} already exists",
                args.definition_id
            ),
        };
    }

    if definition_by_slug(engine, &args.slug).is_some() {
        return CmdResultMaterialDefinition {
            success: false,
            message: format!("Material definition slug '{}' already exists", args.slug),
        };
    }

    let create_mode = match (&args.preset, &args.shader_type, &args.shader_source) {
        (Some(preset), None, None) => Ok((
            *preset,
            MaterialShaderType::Model,
            if args.realm_kind == MaterialRealmKind::TwoD {
                galfus_render::builtin_material_source_2d().to_string()
            } else {
                builtin_shader_source(*preset).to_string()
            },
        )),
        (None, Some(shader_type), Some(shader_source)) => {
            if shader_source.trim().is_empty() {
                Err("shaderSource must not be empty when creating with shaderType".to_string())
            } else {
                Ok((
                    ShaderMaterialPreset::Standard,
                    *shader_type,
                    shader_source.clone(),
                ))
            }
        }
        _ => Err(
            "Material definition create requires either preset, or shaderType + shaderSource"
                .to_string(),
        ),
    };
    let (preset, shader_type, shader_source) = match create_mode {
        Ok(values) => values,
        Err(message) => {
            return CmdResultMaterialDefinition {
                success: false,
                message,
            };
        }
    };
    if let Err(message) = validate_preset_realm(preset, args.realm_kind) {
        return CmdResultMaterialDefinition {
            success: false,
            message,
        };
    }
    galfus_log::galfus_log_debug!(
        engine,
        "material.definition.compile.start",
        "definition={} slug={} preset={:?} shader_type={:?}",
        args.definition_id,
        args.slug,
        preset,
        shader_type
    );

    let shader_capabilities = default_capabilities_for_realm(
        args.capabilities
            .as_ref()
            .map(|value| value.semantics.clone()),
    );
    let compile_result = compile_material_program(
        engine,
        args.realm_kind,
        preset,
        shader_type,
        shader_source.clone(),
        args.shader_params_schema.clone().unwrap_or_default(),
    )
    .map(|compiled| (compiled.source, compiled.hash));

    let (compiled_source, compiled_hash, compile_error, compile_failed_msg) = match compile_result {
        Ok((source, hash)) => {
            galfus_log::galfus_log_debug!(
                engine,
                "material.definition.compile.ok",
                "definition={} slug={} hash={}",
                args.definition_id,
                args.slug,
                hash
            );
            (Some(source), hash, None, None)
        }
        Err(error) => {
            galfus_log::galfus_log_error!(
                engine,
                "material.definition.compile.fail",
                "definition={} slug={} error={}",
                args.definition_id,
                args.slug,
                error
            );
            let msg = format!(
                "Material definition '{}' compile failed: {}",
                args.slug, error
            );
            (None, 0, Some(error), Some(msg))
        }
    };

    engine.universal_state.scene.material_definitions.insert(
        args.definition_id,
        MaterialDefinitionRecord {
            definition_id: args.definition_id,
            slug: args.slug.clone(),
            label: args.label.clone(),
            realm_kind: args.realm_kind,
            base_preset: preset,
            shader_type,
            shader_source: Some(shader_source),
            shader_params_schema: args.shader_params_schema.clone().unwrap_or_default(),
            shader_capabilities,
            compiled_shader_hash: compiled_hash,
            compiled_shader_source: compiled_source,
            compile_error,
        },
    );
    ensure_fallback_material_instance(engine);

    if let Some(message) = compile_failed_msg {
        return broken_definition_result(message, true);
    }

    CmdResultMaterialDefinition {
        success: true,
        message: "Material definition created successfully".to_string(),
    }
}

pub fn engine_cmd_material_definition_update(
    engine: &mut EngineState,
    args: &CmdMaterialDefinitionUpdateArgs,
) -> CmdResultMaterialDefinition {
    if args.definition_id == MATERIAL_DEFINITION_STANDARD_ID
        || args.definition_id == MATERIAL_DEFINITION_PBR_ID
        || args.definition_id == MATERIAL_DEFINITION_STANDARD_2D_ID
    {
        return CmdResultMaterialDefinition {
            success: false,
            message: "Builtin material definitions are immutable".to_string(),
        };
    }
    if let Err(message) = validate_host_logical_id(args.definition_id, "definitionId") {
        return CmdResultMaterialDefinition {
            success: false,
            message,
        };
    }
    bootstrap_builtin_material_definitions(engine);
    let Some(current) = definition_by_id(engine, args.definition_id) else {
        return CmdResultMaterialDefinition {
            success: false,
            message: format!(
                "Material definition with id {} not found",
                args.definition_id
            ),
        };
    };

    let slug = args.slug.clone().unwrap_or_else(|| current.slug.clone());
    if slug != current.slug {
        if definition_by_slug(engine, &slug).is_some() {
            return CmdResultMaterialDefinition {
                success: false,
                message: format!("Material definition slug '{}' already exists", slug),
            };
        }
    }

    let compile_requested = args.preset.is_some()
        || args.shader_type.is_some()
        || args.shader_source.is_some()
        || args.shader_params_schema.is_some();
    let realm_kind = args.realm_kind.unwrap_or(current.realm_kind);

    let shader_capabilities = default_capabilities_for_realm(
        args.capabilities
            .as_ref()
            .map(|value| value.semantics.clone())
            .or_else(|| Some(current.shader_capabilities.clone())),
    );

    if !compile_requested {
        if let Some(record) = engine
            .universal_state
            .scene
            .material_definitions
            .get_mut(&args.definition_id)
        {
            if let Some(label) = &args.label {
                record.label = Some(label.clone());
            }
            record.slug = slug;
            record.realm_kind = realm_kind;
            record.shader_capabilities = shader_capabilities;
        }
        ensure_fallback_material_instance(engine);
        return CmdResultMaterialDefinition {
            success: true,
            message: "Material definition updated successfully".to_string(),
        };
    }

    let update_mode = match (&args.preset, &args.shader_type, &args.shader_source) {
        (Some(preset), None, None) => Ok((
            *preset,
            MaterialShaderType::Model,
            if realm_kind == MaterialRealmKind::TwoD {
                galfus_render::builtin_material_source_2d().to_string()
            } else {
                builtin_shader_source(*preset).to_string()
            },
        )),
        (None, Some(shader_type), Some(shader_source)) => {
            if shader_source.trim().is_empty() {
                Err("shaderSource must not be empty when updating with shaderType".to_string())
            } else {
                Ok((current.base_preset, *shader_type, shader_source.clone()))
            }
        }
        (None, None, Some(shader_source)) => {
            if shader_source.trim().is_empty() {
                Err("shaderSource must not be empty when updating shader".to_string())
            } else {
                Ok((
                    current.base_preset,
                    current.shader_type,
                    shader_source.clone(),
                ))
            }
        }
        (None, Some(shader_type), None) => Ok((
            current.base_preset,
            *shader_type,
            current.shader_source.clone().unwrap_or_default(),
        )),
        _ => Err(
            "Material definition update requires either preset, or shaderType + shaderSource"
                .to_string(),
        ),
    };
    let (preset, shader_type, shader_source) = match update_mode {
        Ok(values) => values,
        Err(message) => {
            return CmdResultMaterialDefinition {
                success: false,
                message,
            };
        }
    };
    let shader_params_schema = args
        .shader_params_schema
        .clone()
        .unwrap_or_else(|| current.shader_params_schema.clone());
    if let Err(message) = validate_preset_realm(preset, realm_kind) {
        return CmdResultMaterialDefinition {
            success: false,
            message,
        };
    }

    galfus_log::galfus_log_debug!(
        engine,
        "material.definition.compile.start",
        "definition={} slug={} preset={:?} shader_type={:?}",
        args.definition_id,
        slug,
        preset,
        shader_type
    );

    let compile_result = compile_material_program(
        engine,
        realm_kind,
        preset,
        shader_type,
        shader_source.clone(),
        shader_params_schema.clone(),
    )
    .map(|compiled| (compiled.source, compiled.hash));

    let (compiled_source, compiled_hash, compile_error, compile_failed_msg) = match compile_result {
        Ok((source, hash)) => {
            galfus_log::galfus_log_debug!(
                engine,
                "material.definition.compile.ok",
                "definition={} slug={} hash={}",
                args.definition_id,
                slug,
                hash
            );
            (Some(source), hash, None, None)
        }
        Err(error) => {
            galfus_log::galfus_log_error!(
                engine,
                "material.definition.compile.fail",
                "definition={} slug={} error={}",
                args.definition_id,
                slug,
                error
            );
            let msg = format!("Material definition '{}' compile failed: {}", slug, error);
            (None, 0, Some(error), Some(msg))
        }
    };

    if let Some(record) = engine
        .universal_state
        .scene
        .material_definitions
        .get_mut(&args.definition_id)
    {
        if let Some(label) = &args.label {
            record.label = Some(label.clone());
        }
        record.slug = slug;
        record.realm_kind = realm_kind;
        record.base_preset = preset;
        record.shader_type = shader_type;
        record.shader_source = Some(shader_source);
        record.shader_params_schema = shader_params_schema;
        record.shader_capabilities = shader_capabilities;
        record.compiled_shader_hash = compiled_hash;
        record.compiled_shader_source = compiled_source;
        record.compile_error = compile_error;
    }
    ensure_fallback_material_instance(engine);

    if let Some(message) = compile_failed_msg {
        return broken_definition_result(message, true);
    }

    CmdResultMaterialDefinition {
        success: true,
        message: "Material definition updated successfully".to_string(),
    }
}

pub fn engine_cmd_material_definition_dispose(
    engine: &mut EngineState,
    args: &CmdMaterialDefinitionDisposeArgs,
) -> CmdResultMaterialDefinition {
    if args.definition_id == MATERIAL_DEFINITION_STANDARD_ID
        || args.definition_id == MATERIAL_DEFINITION_PBR_ID
        || args.definition_id == MATERIAL_DEFINITION_STANDARD_2D_ID
    {
        return CmdResultMaterialDefinition {
            success: false,
            message: "Builtin material definitions cannot be disposed".to_string(),
        };
    }
    if let Err(message) = validate_host_logical_id(args.definition_id, "definitionId") {
        return CmdResultMaterialDefinition {
            success: false,
            message,
        };
    }

    if engine
        .universal_state
        .scene
        .material_definitions
        .remove(&args.definition_id)
        .is_some()
    {
        let impacted_materials: Vec<(u32, MaterialRealmKind)> = engine
            .universal_state
            .scene
            .material_instances
            .iter()
            .filter_map(|(material_id, instance)| {
                if instance.definition_id == args.definition_id {
                    let realm_kind = engine
                        .universal_state
                        .scene
                        .realm3d
                        .materials
                        .get(material_id)
                        .map(|record| record.realm_kind)?;
                    Some((*material_id, realm_kind))
                } else {
                    None
                }
            })
            .collect();

        for (material_id, realm_kind) in impacted_materials {
            let Some(fallback_definition) = fallback_definition_for_realm(engine, realm_kind)
            else {
                continue;
            };
            let Some(record) = engine
                .universal_state
                .scene
                .realm3d
                .materials
                .get_mut(&material_id)
            else {
                continue;
            };
            sync_material_from_definition(record, &fallback_definition);
            record.mark_structural_dirty();
            if let Some(instance) = engine
                .universal_state
                .scene
                .material_instances
                .get_mut(&material_id)
            {
                instance.definition_id = fallback_definition.definition_id;
            }
            engine.runtime.push_event(EngineEvent::System(
                SystemEvent::MaterialInstanceFallbackApplied {
                    material_id,
                    previous_definition_id: args.definition_id,
                    fallback_definition_id: fallback_definition.definition_id,
                    reason: "definition-disposed".to_string(),
                },
            ));
        }

        let active_hashes: HashSet<u64> = engine
            .universal_state
            .scene
            .material_definitions
            .values()
            .filter_map(|record| {
                if record.compiled_shader_hash != 0 {
                    Some(record.compiled_shader_hash)
                } else {
                    None
                }
            })
            .collect();
        let before_prune = engine.universal_state.scene.material_program_cache.len();
        engine
            .universal_state
            .scene
            .material_program_cache
            .retain(|_, value| active_hashes.contains(&value.hash));
        let evicted =
            before_prune.saturating_sub(engine.universal_state.scene.material_program_cache.len());
        engine.profiling.render.material_program_cache_evictions = engine
            .profiling
            .render
            .material_program_cache_evictions
            .saturating_add(evicted as u32);
        engine
            .universal_state
            .scene
            .material_program_cache_last_used_frame
            .retain(|key, _| {
                engine
                    .universal_state
                    .scene
                    .material_program_cache
                    .contains_key(key)
            });
        CmdResultMaterialDefinition {
            success: true,
            message: "Material definition disposed successfully".to_string(),
        }
    } else {
        CmdResultMaterialDefinition {
            success: false,
            message: format!(
                "Material definition with id {} not found",
                args.definition_id
            ),
        }
    }
}

pub fn engine_cmd_material_instance_create(
    engine: &mut EngineState,
    args: &CmdMaterialInstanceCreateArgs,
) -> CmdResultMaterialInstance {
    if let Err(message) = validate_host_logical_id(args.material_id, "materialId") {
        return CmdResultMaterialInstance {
            success: false,
            message,
        };
    }
    let Some(definition) = definition_by_slug(engine, &args.slug) else {
        return CmdResultMaterialInstance {
            success: false,
            message: format!("Material definition slug '{}' not found", args.slug),
        };
    };
    if definition_is_broken(&definition) {
        return CmdResultMaterialInstance {
            success: false,
            message: format!(
                "Material definition slug '{}' is broken and cannot be instanced",
                args.slug
            ),
        };
    }

    let create_args = CmdMaterialCreateArgs {
        material_id: args.material_id,
        label: args.label.clone(),
        slug: args.slug.clone(),
        kind: MaterialKind::Shader,
        realm_kind: definition.realm_kind,
        options: args.options.clone(),
    };

    let result = engine_cmd_material_create(engine, &create_args);
    if !result.success {
        return CmdResultMaterialInstance {
            success: false,
            message: result.message,
        };
    }

    engine.universal_state.scene.material_instances.insert(
        args.material_id,
        MaterialInstanceRecord {
            material_id: args.material_id,
            definition_id: definition.definition_id,
            label: args.label.clone(),
        },
    );

    CmdResultMaterialInstance {
        success: true,
        message: "Material instance created successfully".to_string(),
    }
}

pub fn engine_cmd_material_instance_update(
    engine: &mut EngineState,
    args: &CmdMaterialInstanceUpdateArgs,
) -> CmdResultMaterialInstance {
    if let Err(message) = validate_host_logical_id(args.material_id, "materialId") {
        return CmdResultMaterialInstance {
            success: false,
            message,
        };
    }
    if let Some(slug) = args.slug.as_ref() {
        let Some(definition) = definition_by_slug(engine, slug) else {
            return CmdResultMaterialInstance {
                success: false,
                message: format!("Material definition slug '{}' not found", slug),
            };
        };
        if definition_is_broken(&definition) {
            return CmdResultMaterialInstance {
                success: false,
                message: format!(
                    "Material definition slug '{}' is broken and cannot be instanced",
                    slug
                ),
            };
        }
    }
    let update_args = CmdMaterialUpdateArgs {
        material_id: args.material_id,
        label: args.label.clone(),
        slug: args.slug.clone(),
        kind: Some(MaterialKind::Shader),
        realm_kind: None,
        options: args.options.clone(),
    };
    let result = engine_cmd_material_update(engine, &update_args);
    CmdResultMaterialInstance {
        success: result.success,
        message: result.message,
    }
}

pub fn engine_cmd_material_instance_dispose(
    engine: &mut EngineState,
    args: &CmdMaterialInstanceDisposeArgs,
) -> CmdResultMaterialInstance {
    if let Err(message) = validate_host_logical_id(args.material_id, "materialId") {
        return CmdResultMaterialInstance {
            success: false,
            message,
        };
    }
    if engine
        .universal_state
        .scene
        .material_instances
        .remove(&args.material_id)
        .is_none()
    {
        return CmdResultMaterialInstance {
            success: false,
            message: format!("Material instance with id {} not found", args.material_id),
        };
    }

    let dispose_result = engine_cmd_material_dispose(
        engine,
        &CmdMaterialDisposeArgs {
            material_id: args.material_id,
        },
    );

    CmdResultMaterialInstance {
        success: dispose_result.success,
        message: dispose_result.message,
    }
}

pub fn engine_cmd_material_create(
    engine: &mut EngineState,
    args: &CmdMaterialCreateArgs,
) -> CmdResultMaterialCreate {
    if let Err(message) = validate_host_logical_id(args.material_id, "materialId") {
        return CmdResultMaterialCreate {
            success: false,
            message,
        };
    }
    bootstrap_builtin_material_definitions(engine);

    let Some(definition) = definition_by_slug(engine, &args.slug) else {
        return CmdResultMaterialCreate {
            success: false,
            message: format!("Material definition slug '{}' not found", args.slug),
        };
    };
    if definition_is_broken(&definition) {
        return CmdResultMaterialCreate {
            success: false,
            message: format!(
                "Material definition slug '{}' is broken and cannot be used",
                args.slug
            ),
        };
    }
    if args.realm_kind != definition.realm_kind {
        return CmdResultMaterialCreate {
            success: false,
            message: format!(
                "Material realm kind mismatch: definition '{}' is {:?} but request is {:?}",
                args.slug, definition.realm_kind, args.realm_kind
            ),
        };
    }

    {
        let resources = &mut engine.universal_state.scene.realm3d;
        if resources.materials.contains_key(&args.material_id) {
            return CmdResultMaterialCreate {
                success: false,
                message: format!("Material with id {} already exists", args.material_id),
            };
        }
        if args.kind != MaterialKind::Shader {
            return CmdResultMaterialCreate {
                success: false,
                message: "Unsupported material kind".into(),
            };
        }
    }

    let mut record = match definition.base_preset {
        ShaderMaterialPreset::Standard => {
            if args.realm_kind == MaterialRealmKind::TwoD {
                ShaderMaterialRecord::new_standard_2d(args.label.clone())
            } else {
                ShaderMaterialRecord::new_standard(args.label.clone())
            }
        }
        ShaderMaterialPreset::Pbr => ShaderMaterialRecord::new_pbr(args.label.clone()),
    };
    record.realm_kind = definition.realm_kind;
    sync_material_from_definition(&mut record, &definition);

    match (&definition.base_preset, &args.options) {
        (_, Some(MaterialOptions::Schema(schema_params))) => {
            pack_schema_material(args.material_id, schema_params, &mut record);
        }
        _ => {}
    }

    record.bind_group = None;
    {
        let resources = &mut engine.universal_state.scene.realm3d;
        resources.materials.insert(args.material_id, record);
    }

    engine.universal_state.scene.material_instances.insert(
        args.material_id,
        MaterialInstanceRecord {
            material_id: args.material_id,
            definition_id: definition.definition_id,
            label: args.label.clone(),
        },
    );

    for window_state in engine.window.states.values_mut() {
        window_state.is_dirty = true;
    }

    CmdResultMaterialCreate {
        success: true,
        message: "Material created successfully".into(),
    }
}

pub fn engine_cmd_material_update(
    engine: &mut EngineState,
    args: &CmdMaterialUpdateArgs,
) -> CmdResultMaterialUpdate {
    if let Err(message) = validate_host_logical_id(args.material_id, "materialId") {
        return CmdResultMaterialUpdate {
            success: false,
            message,
        };
    }
    let mut target_slug: Option<String> = args.slug.clone();

    let instance_definition_id = engine
        .universal_state
        .scene
        .material_instances
        .get(&args.material_id)
        .map(|instance| instance.definition_id);

    if target_slug.is_none() {
        if let Some(definition_id) = instance_definition_id {
            target_slug = engine
                .universal_state
                .scene
                .material_definitions
                .get(&definition_id)
                .map(|d| d.slug.clone());
        }
    }

    let Some(slug) = target_slug else {
        return CmdResultMaterialUpdate {
            success: false,
            message: format!(
                "Material {} has no instance slug; provide slug in update",
                args.material_id
            ),
        };
    };

    let Some(definition) = definition_by_slug(engine, &slug) else {
        return CmdResultMaterialUpdate {
            success: false,
            message: format!("Material definition slug '{}' not found", slug),
        };
    };
    if definition_is_broken(&definition) {
        return CmdResultMaterialUpdate {
            success: false,
            message: format!(
                "Material definition slug '{}' is broken and cannot be used",
                slug
            ),
        };
    }

    let Some(record) = engine
        .universal_state
        .scene
        .realm3d
        .materials
        .get_mut(&args.material_id)
    else {
        return CmdResultMaterialUpdate {
            success: false,
            message: format!("Material with id {} not found", args.material_id),
        };
    };

    sync_material_from_definition(record, &definition);
    if let Some(label) = &args.label {
        record.label = Some(label.clone());
    }
    if let Some(realm_kind) = args.realm_kind
        && realm_kind != definition.realm_kind
    {
        return CmdResultMaterialUpdate {
            success: false,
            message: format!(
                "Material realm kind mismatch: definition '{}' is {:?} but request is {:?}",
                slug, definition.realm_kind, realm_kind
            ),
        };
    }
    record.realm_kind = definition.realm_kind;

    if let Some(opts) = &args.options {
        match opts {
            MaterialOptions::Schema(schema_params) => {
                pack_schema_material(args.material_id, schema_params, record)
            }
        }
    }
    record.mark_structural_dirty();

    engine.universal_state.scene.material_instances.insert(
        args.material_id,
        MaterialInstanceRecord {
            material_id: args.material_id,
            definition_id: definition.definition_id,
            label: args.label.clone(),
        },
    );

    for window_state in engine.window.states.values_mut() {
        window_state.is_dirty = true;
    }

    CmdResultMaterialUpdate {
        success: true,
        message: "Material updated successfully".into(),
    }
}

pub fn engine_cmd_material_dispose(
    engine: &mut EngineState,
    args: &CmdMaterialDisposeArgs,
) -> CmdResultMaterialDispose {
    if let Err(message) = validate_host_logical_id(args.material_id, "materialId") {
        return CmdResultMaterialDispose {
            success: false,
            message,
        };
    }
    if args.material_id == MATERIAL_FALLBACK_ID {
        return CmdResultMaterialDispose {
            success: false,
            message: "Fallback material cannot be disposed".into(),
        };
    }

    engine
        .universal_state
        .scene
        .material_instances
        .remove(&args.material_id);

    if engine
        .universal_state
        .scene
        .realm3d
        .materials
        .remove(&args.material_id)
        .is_some()
    {
        for window_state in engine.window.states.values_mut() {
            window_state.is_dirty = true;
        }
        CmdResultMaterialDispose {
            success: true,
            message: "Material disposed successfully".into(),
        }
    } else {
        CmdResultMaterialDispose {
            success: false,
            message: format!("Material with id {} not found", args.material_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::resources::{
        MATERIAL_DEFINITION_STANDARD_2D_ID, MATERIAL_DEFINITION_STANDARD_2D_SLUG, MaterialKind,
        MaterialRealmKind, ShaderMaterialPreset,
    };
    use crate::core::test_support::test_engine;

    #[test]
    fn material_definition_and_instance_lifecycle_for_twod() {
        let mut engine = test_engine();
        let create_definition = engine_cmd_material_definition_create(
            &mut engine,
            &CmdMaterialDefinitionCreateArgs {
                definition_id: 200_100,
                slug: "unit-2d-def-a".to_string(),
                label: Some("Unit 2D Definition".to_string()),
                realm_kind: MaterialRealmKind::TwoD,
                preset: Some(ShaderMaterialPreset::Standard),
                shader_type: None,
                shader_source: None,
                shader_params_schema: None,
                capabilities: None,
            },
        );
        assert!(create_definition.success, "{}", create_definition.message);

        let update_definition = engine_cmd_material_definition_update(
            &mut engine,
            &CmdMaterialDefinitionUpdateArgs {
                definition_id: 200_100,
                slug: Some("unit-2d-def-b".to_string()),
                label: Some("Unit 2D Definition Updated".to_string()),
                realm_kind: Some(MaterialRealmKind::TwoD),
                preset: None,
                shader_type: None,
                shader_source: None,
                shader_params_schema: None,
                capabilities: None,
            },
        );
        assert!(update_definition.success, "{}", update_definition.message);

        let create_instance = engine_cmd_material_instance_create(
            &mut engine,
            &CmdMaterialInstanceCreateArgs {
                material_id: 200_101,
                slug: "unit-2d-def-b".to_string(),
                label: Some("Unit 2D Instance".to_string()),
                options: None,
            },
        );
        assert!(create_instance.success, "{}", create_instance.message);

        let update_instance = engine_cmd_material_instance_update(
            &mut engine,
            &CmdMaterialInstanceUpdateArgs {
                material_id: 200_101,
                slug: Some(MATERIAL_DEFINITION_STANDARD_2D_SLUG.to_string()),
                label: Some("Unit 2D Instance Updated".to_string()),
                options: None,
            },
        );
        assert!(update_instance.success, "{}", update_instance.message);

        let dispose_instance = engine_cmd_material_instance_dispose(
            &mut engine,
            &CmdMaterialInstanceDisposeArgs {
                material_id: 200_101,
            },
        );
        assert!(dispose_instance.success, "{}", dispose_instance.message);

        let dispose_definition = engine_cmd_material_definition_dispose(
            &mut engine,
            &CmdMaterialDefinitionDisposeArgs {
                definition_id: 200_100,
            },
        );
        assert!(dispose_definition.success, "{}", dispose_definition.message);
    }

    #[test]
    fn disposing_twod_definition_falls_back_to_standard_2d_definition() {
        let mut engine = test_engine();
        let create_definition = engine_cmd_material_definition_create(
            &mut engine,
            &CmdMaterialDefinitionCreateArgs {
                definition_id: 200_200,
                slug: "unit-2d-fallback".to_string(),
                label: None,
                realm_kind: MaterialRealmKind::TwoD,
                preset: Some(ShaderMaterialPreset::Standard),
                shader_type: None,
                shader_source: None,
                shader_params_schema: None,
                capabilities: None,
            },
        );
        assert!(create_definition.success, "{}", create_definition.message);

        let create_material = engine_cmd_material_create(
            &mut engine,
            &CmdMaterialCreateArgs {
                material_id: 200_201,
                label: Some("2D Material".to_string()),
                slug: "unit-2d-fallback".to_string(),
                kind: MaterialKind::Shader,
                realm_kind: MaterialRealmKind::TwoD,
                options: None,
            },
        );
        assert!(create_material.success, "{}", create_material.message);

        let dispose_definition = engine_cmd_material_definition_dispose(
            &mut engine,
            &CmdMaterialDefinitionDisposeArgs {
                definition_id: 200_200,
            },
        );
        assert!(dispose_definition.success, "{}", dispose_definition.message);

        let definition_id = engine
            .universal_state
            .scene
            .material_instances
            .get(&200_201)
            .map(|instance| instance.definition_id);
        assert_eq!(definition_id, Some(MATERIAL_DEFINITION_STANDARD_2D_ID));
    }

    #[test]
    fn twod_definition_rejects_wrong_realm_on_material_create() {
        let mut engine = test_engine();
        let create_result = engine_cmd_material_create(
            &mut engine,
            &CmdMaterialCreateArgs {
                material_id: 200_300,
                label: Some("invalid-realm".to_string()),
                slug: MATERIAL_DEFINITION_STANDARD_2D_SLUG.to_string(),
                kind: MaterialKind::Shader,
                realm_kind: MaterialRealmKind::ThreeD,
                options: None,
            },
        );
        assert!(!create_result.success);
        assert!(
            create_result
                .message
                .contains("Material realm kind mismatch"),
            "{}",
            create_result.message
        );
    }
}
