use super::*;
use crate::core::cmd::EngineEvent;
use crate::core::render::graph::{
    LogicalId, RenderGraphDesc, RenderGraphLifetime, RenderGraphNode, RenderGraphResource,
    RenderGraphResourceKind,
};
use crate::core::state::EngineState;
use crate::core::system::SystemEvent;
use galfus_realm_core::{RENDER_PASS_FORWARD, RENDER_PASS_PREPARE, RENDER_PASS_SHADOW_3D};
use std::collections::HashMap;

fn valid_graph(graph_name: &str, resource_name: &str) -> RenderGraphDesc {
    RenderGraphDesc {
        graph_id: LogicalId::Str(graph_name.into()),
        nodes: vec![RenderGraphNode {
            node_id: LogicalId::Str(format!("{graph_name}-shadow")),
            pass_id: RENDER_PASS_SHADOW_3D.into(),
            inputs: Vec::new(),
            outputs: vec![LogicalId::Str(resource_name.into())],
            require: Vec::new(),
            priority: 0,
            enabled: true,
            params: HashMap::new(),
            shader: None,
        }],
        edges: Vec::new(),
        resources: vec![RenderGraphResource {
            res_id: LogicalId::Str(resource_name.into()),
            kind: RenderGraphResourceKind::Texture,
            lifetime: RenderGraphLifetime::Frame,
            alias_group: None,
        }],
        fallback: false,
    }
}

fn invalid_graph_missing_resource(graph_name: &str) -> RenderGraphDesc {
    RenderGraphDesc {
        graph_id: LogicalId::Str(graph_name.into()),
        nodes: vec![RenderGraphNode {
            node_id: LogicalId::Str(format!("{graph_name}-forward")),
            pass_id: RENDER_PASS_FORWARD.into(),
            inputs: vec![LogicalId::Str("missing".into())],
            outputs: Vec::new(),
            require: vec![LogicalId::Str("missing".into())],
            priority: 0,
            enabled: true,
            params: HashMap::new(),
            shader: None,
        }],
        edges: Vec::new(),
        resources: Vec::new(),
        fallback: false,
    }
}

fn realm2d_only_graph(graph_name: &str, resource_name: &str) -> RenderGraphDesc {
    RenderGraphDesc {
        graph_id: LogicalId::Str(graph_name.into()),
        nodes: vec![RenderGraphNode {
            node_id: LogicalId::Str(format!("{graph_name}-prepare")),
            pass_id: RENDER_PASS_PREPARE.into(),
            inputs: Vec::new(),
            outputs: vec![LogicalId::Str(resource_name.into())],
            require: Vec::new(),
            priority: 0,
            enabled: true,
            params: HashMap::new(),
            shader: None,
        }],
        edges: Vec::new(),
        resources: vec![RenderGraphResource {
            res_id: LogicalId::Str(resource_name.into()),
            kind: RenderGraphResourceKind::Attachment,
            lifetime: RenderGraphLifetime::Frame,
            alias_group: None,
        }],
        fallback: false,
    }
}

fn create_realm(engine: &mut EngineState, kind: RealmKindDto) -> u32 {
    let result = engine_cmd_realm_create(
        engine,
        &CmdRealmCreateArgs {
            kind,
            importance: None,
            cache_policy: None,
            flags: None,
        },
    );
    assert!(result.success, "realm create failed: {}", result.message);
    result.realm_id.expect("realm id should exist")
}

fn take_error_events(engine: &mut EngineState) -> Vec<(String, String, Option<String>)> {
    let mut errors = Vec::new();
    for event in engine.runtime.take_events() {
        let EngineEvent::System(SystemEvent::Error {
            scope,
            message,
            command_type,
            ..
        }) = event
        else {
            continue;
        };
        errors.push((scope, message, command_type));
    }
    errors
}

#[test]
fn render_graph_upsert_and_list_includes_desc_hash_and_passes() {
    let mut engine = EngineState::new();
    let graph = valid_graph("custom_a", "shadow_atlas_custom_a");
    let expected_hash = crate::core::render::graph::render_graph_desc_hash(&graph);

    let upsert = engine_cmd_render_graph_upsert(
        &mut engine,
        &CmdRenderGraphUpsertArgs {
            render_graph_id: 100,
            graph,
        },
    );
    assert!(upsert.success, "upsert failed: {}", upsert.message);

    let listed = engine_cmd_render_graph_list(&mut engine, &CmdRenderGraphListArgs::default());
    assert!(listed.success, "list failed: {}", listed.message);
    let entry = listed
        .render_graphs
        .iter()
        .find(|entry| entry.render_graph_id == 100)
        .expect("custom graph must be listed");
    assert_eq!(entry.desc_hash, expected_hash);
    assert_eq!(entry.graph_kind, "3d");
    assert_eq!(entry.pass_count, 1);
    assert_eq!(entry.pass_ids, vec![RENDER_PASS_SHADOW_3D.to_string()]);
    assert_eq!(entry.bound_realm_ids, Vec::<u32>::new());
    assert_eq!(
        engine
            .universal_state
            .render_catalog
            .render_graph_compile_cache_misses,
        1
    );
    assert_eq!(
        engine
            .universal_state
            .render_catalog
            .render_graph_compile_cache_hits,
        0
    );

    let upsert_again = engine_cmd_render_graph_upsert(
        &mut engine,
        &CmdRenderGraphUpsertArgs {
            render_graph_id: 110,
            graph: valid_graph("custom_a", "shadow_atlas_custom_a"),
        },
    );
    assert!(
        upsert_again.success,
        "upsert again failed: {}",
        upsert_again.message
    );
    assert_eq!(
        engine
            .universal_state
            .render_catalog
            .render_graph_compile_cache_hits,
        1
    );
}

#[test]
fn realm_bind_and_rebind_updates_realm_graph_id() {
    let mut engine = EngineState::new();
    let realm_id = create_realm(&mut engine, RealmKindDto::ThreeD);

    assert!(
        engine_cmd_render_graph_upsert(
            &mut engine,
            &CmdRenderGraphUpsertArgs {
                render_graph_id: 101,
                graph: valid_graph("custom_b", "shadow_atlas_custom_b"),
            },
        )
        .success
    );
    assert!(
        engine_cmd_render_graph_upsert(
            &mut engine,
            &CmdRenderGraphUpsertArgs {
                render_graph_id: 102,
                graph: valid_graph("custom_c", "shadow_atlas_custom_c"),
            },
        )
        .success
    );

    let first_bind = engine_cmd_realm_render_graph_bind(
        &mut engine,
        &CmdRealmRenderGraphBindArgs {
            realm_id,
            render_graph_id: 101,
        },
    );
    assert!(
        first_bind.success,
        "first bind failed: {}",
        first_bind.message
    );

    let second_bind = engine_cmd_realm_render_graph_bind(
        &mut engine,
        &CmdRealmRenderGraphBindArgs {
            realm_id,
            render_graph_id: 102,
        },
    );
    assert!(
        second_bind.success,
        "second bind failed: {}",
        second_bind.message
    );

    let realm_entry = engine
        .universal_state
        .composition
        .realms
        .entries
        .get(&crate::core::realm::RealmId(realm_id))
        .expect("realm entry should exist");
    assert_eq!(realm_entry.value.render_graph_id, Some(102));
}

#[test]
fn render_graph_dispose_respects_realm_bindings() {
    let mut engine = EngineState::new();
    let realm_id = create_realm(&mut engine, RealmKindDto::ThreeD);
    assert!(
        engine_cmd_render_graph_upsert(
            &mut engine,
            &CmdRenderGraphUpsertArgs {
                render_graph_id: 103,
                graph: valid_graph("custom_d", "shadow_atlas_custom_d"),
            },
        )
        .success
    );

    assert!(
        engine_cmd_realm_render_graph_bind(
            &mut engine,
            &CmdRealmRenderGraphBindArgs {
                realm_id,
                render_graph_id: 103,
            },
        )
        .success
    );

    let dispose_bound = engine_cmd_render_graph_dispose(
        &mut engine,
        &CmdRenderGraphDisposeArgs {
            render_graph_id: 103,
        },
    );
    assert!(!dispose_bound.success);
    let errors = take_error_events(&mut engine);
    assert!(errors.iter().any(|(scope, _, command_type)| {
        scope == "render-graph" && command_type.as_deref() == Some("render-graph-dispose")
    }));

    assert!(
        engine_cmd_realm_render_graph_bind(
            &mut engine,
            &CmdRealmRenderGraphBindArgs {
                realm_id,
                render_graph_id: crate::core::render::graph::DEFAULT_3D_RENDER_GRAPH_ID,
            },
        )
        .success
    );
    let dispose_unbound = engine_cmd_render_graph_dispose(
        &mut engine,
        &CmdRenderGraphDisposeArgs {
            render_graph_id: 103,
        },
    );
    assert!(
        dispose_unbound.success,
        "dispose should succeed after unbind: {}",
        dispose_unbound.message
    );
}

#[test]
fn invalid_upsert_and_unknown_bind_emit_error_events() {
    let mut engine = EngineState::new();
    let realm_id = create_realm(&mut engine, RealmKindDto::ThreeD);

    let invalid_upsert = engine_cmd_render_graph_upsert(
        &mut engine,
        &CmdRenderGraphUpsertArgs {
            render_graph_id: 104,
            graph: invalid_graph_missing_resource("invalid_a"),
        },
    );
    assert!(!invalid_upsert.success);

    let unknown_bind = engine_cmd_realm_render_graph_bind(
        &mut engine,
        &CmdRealmRenderGraphBindArgs {
            realm_id,
            render_graph_id: 999_999,
        },
    );
    assert!(!unknown_bind.success);

    let errors = take_error_events(&mut engine);
    assert!(errors.iter().any(|(scope, _, command_type)| {
        scope == "render-graph" && command_type.as_deref() == Some("render-graph-upsert")
    }));
    assert!(errors.iter().any(|(scope, _, command_type)| {
        scope == "render-graph" && command_type.as_deref() == Some("realm-render-graph-bind")
    }));
}

#[test]
fn twod_realm_rejects_graph_with_non_twod_passes() {
    let mut engine = EngineState::new();
    let twod_realm_id = create_realm(&mut engine, RealmKindDto::TwoD);

    assert!(
        engine_cmd_render_graph_upsert(
            &mut engine,
            &CmdRenderGraphUpsertArgs {
                render_graph_id: 105,
                graph: valid_graph("custom_3d_only", "shadow_atlas_custom"),
            },
        )
        .success
    );

    let incompatible_bind = engine_cmd_realm_render_graph_bind(
        &mut engine,
        &CmdRealmRenderGraphBindArgs {
            realm_id: twod_realm_id,
            render_graph_id: 105,
        },
    );
    assert!(!incompatible_bind.success);

    assert!(
        engine_cmd_render_graph_upsert(
            &mut engine,
            &CmdRenderGraphUpsertArgs {
                render_graph_id: 106,
                graph: realm2d_only_graph("custom_2d_only", "swapchain_custom"),
            },
        )
        .success
    );

    let compatible_bind = engine_cmd_realm_render_graph_bind(
        &mut engine,
        &CmdRealmRenderGraphBindArgs {
            realm_id: twod_realm_id,
            render_graph_id: 106,
        },
    );
    assert!(
        compatible_bind.success,
        "TwoD realm should accept 2D-only graph: {}",
        compatible_bind.message
    );
}

#[test]
fn upsert_rejects_incompatible_update_for_bound_realms() {
    let mut engine = EngineState::new();
    let twod_realm_id = create_realm(&mut engine, RealmKindDto::TwoD);

    assert!(
        engine_cmd_render_graph_upsert(
            &mut engine,
            &CmdRenderGraphUpsertArgs {
                render_graph_id: 107,
                graph: realm2d_only_graph("custom_2d_only_for_update", "swapchain_custom_update"),
            },
        )
        .success
    );
    assert!(
        engine_cmd_realm_render_graph_bind(
            &mut engine,
            &CmdRealmRenderGraphBindArgs {
                realm_id: twod_realm_id,
                render_graph_id: 107,
            },
        )
        .success
    );

    let incompatible_update = engine_cmd_render_graph_upsert(
        &mut engine,
        &CmdRenderGraphUpsertArgs {
            render_graph_id: 107,
            graph: valid_graph("custom_3d_replacement", "shadow_atlas_replacement"),
        },
    );
    assert!(!incompatible_update.success);
    let rejected_hash = crate::core::render::graph::render_graph_desc_hash(&valid_graph(
        "custom_3d_replacement",
        "shadow_atlas_replacement",
    ));
    assert!(
        !engine
            .universal_state
            .render_catalog
            .render_graph_plan_cache_3d
            .contains_key(&rejected_hash),
        "incompatible upsert must not populate plan cache"
    );

    let errors = take_error_events(&mut engine);
    assert!(errors.iter().any(|(scope, _, command_type)| {
        scope == "render-graph" && command_type.as_deref() == Some("render-graph-upsert")
    }));
}

#[test]
fn dispose_prunes_orphaned_render_graph_plan_cache() {
    let mut engine = EngineState::new();
    let graph = valid_graph("custom_cache_prune", "shadow_atlas_cache_prune");
    let desc_hash = crate::core::render::graph::render_graph_desc_hash(&graph);

    assert!(
        engine_cmd_render_graph_upsert(
            &mut engine,
            &CmdRenderGraphUpsertArgs {
                render_graph_id: 109,
                graph,
            },
        )
        .success
    );
    assert!(
        engine
            .universal_state
            .render_catalog
            .render_graph_plan_cache_3d
            .contains_key(&desc_hash)
    );

    assert!(
        engine_cmd_render_graph_dispose(
            &mut engine,
            &CmdRenderGraphDisposeArgs {
                render_graph_id: 109,
            },
        )
        .success
    );
    assert!(
        !engine
            .universal_state
            .render_catalog
            .render_graph_plan_cache_3d
            .contains_key(&desc_hash),
        "cache entry should be dropped when no graph references desc hash"
    );
}

#[test]
fn render_graph_list_reports_bound_realms() {
    let mut engine = EngineState::new();
    let realm_a = create_realm(&mut engine, RealmKindDto::ThreeD);
    let realm_b = create_realm(&mut engine, RealmKindDto::ThreeD);

    assert!(
        engine_cmd_render_graph_upsert(
            &mut engine,
            &CmdRenderGraphUpsertArgs {
                render_graph_id: 108,
                graph: valid_graph("custom_list_bind", "shadow_atlas_list_bind"),
            },
        )
        .success
    );
    assert!(
        engine_cmd_realm_render_graph_bind(
            &mut engine,
            &CmdRealmRenderGraphBindArgs {
                realm_id: realm_a,
                render_graph_id: 108,
            },
        )
        .success
    );
    assert!(
        engine_cmd_realm_render_graph_bind(
            &mut engine,
            &CmdRealmRenderGraphBindArgs {
                realm_id: realm_b,
                render_graph_id: 108,
            },
        )
        .success
    );

    let listed = engine_cmd_render_graph_list(&mut engine, &CmdRenderGraphListArgs::default());
    assert!(listed.success, "list failed: {}", listed.message);
    let entry = listed
        .render_graphs
        .iter()
        .find(|entry| entry.render_graph_id == 108)
        .expect("custom graph must be listed");
    assert_eq!(entry.bound_realm_ids, vec![realm_a, realm_b]);
}

#[test]
fn twod_definition_lifecycle_and_instance_contracts_are_enforced() {
    let mut engine = EngineState::new();
    let twod_realm_id = create_realm(&mut engine, RealmKindDto::TwoD);
    let threed_realm_id = create_realm(&mut engine, RealmKindDto::ThreeD);

    let upsert = engine_cmd_render_graph_upsert(
        &mut engine,
        &CmdRenderGraphUpsertArgs {
            render_graph_id: 120,
            graph: realm2d_only_graph("custom_2d_lifecycle", "swapchain_2d_lifecycle"),
        },
    );
    assert!(
        upsert.success,
        "2D definition upsert failed: {}",
        upsert.message
    );

    let listed = engine_cmd_render_graph_list(&mut engine, &CmdRenderGraphListArgs::default());
    let entry = listed
        .render_graphs
        .iter()
        .find(|item| item.render_graph_id == 120)
        .expect("2D graph entry must exist");
    assert_eq!(entry.graph_kind, "2d");
    assert_eq!(entry.bound_realm_ids, Vec::<u32>::new());

    let twod_bind = engine_cmd_realm_render_graph_bind(
        &mut engine,
        &CmdRealmRenderGraphBindArgs {
            realm_id: twod_realm_id,
            render_graph_id: 120,
        },
    );
    assert!(
        twod_bind.success,
        "TwoD realm should bind 2D graph: {}",
        twod_bind.message
    );

    let threed_bind = engine_cmd_realm_render_graph_bind(
        &mut engine,
        &CmdRealmRenderGraphBindArgs {
            realm_id: threed_realm_id,
            render_graph_id: 120,
        },
    );
    assert!(!threed_bind.success);
    assert!(
        threed_bind.message.contains("not found in 3d registry"),
        "unexpected message: {}",
        threed_bind.message
    );

    let dispose_bound = engine_cmd_render_graph_dispose(
        &mut engine,
        &CmdRenderGraphDisposeArgs {
            render_graph_id: 120,
        },
    );
    assert!(!dispose_bound.success);
    assert!(
        dispose_bound.message.contains("bound to realms"),
        "unexpected dispose error: {}",
        dispose_bound.message
    );

    let unbind_to_default = engine_cmd_realm_render_graph_bind(
        &mut engine,
        &CmdRealmRenderGraphBindArgs {
            realm_id: twod_realm_id,
            render_graph_id: crate::core::render::graph::DEFAULT_2D_RENDER_GRAPH_ID,
        },
    );
    assert!(
        unbind_to_default.success,
        "unbind to default 2D graph failed: {}",
        unbind_to_default.message
    );

    let dispose_unbound = engine_cmd_render_graph_dispose(
        &mut engine,
        &CmdRenderGraphDisposeArgs {
            render_graph_id: 120,
        },
    );
    assert!(
        dispose_unbound.success,
        "dispose after unbind should succeed: {}",
        dispose_unbound.message
    );
}
