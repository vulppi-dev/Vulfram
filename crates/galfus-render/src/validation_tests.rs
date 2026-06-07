use std::collections::HashMap;

use crate::{
    LogicalId, RenderGraphDesc, RenderGraphEdge, RenderGraphEdgeReason, RenderGraphLifetime,
    RenderGraphNode, RenderGraphResource, RenderGraphResourceKind, RenderGraphShaderSpec,
    RenderGraphShaderType, RenderGraphState, validate_graph,
};
use galfus_realm_core::{
    RENDER_PASS_BLOOM, RENDER_PASS_FORWARD, RENDER_PASS_LIGHT_CULL, RENDER_PASS_POST,
    RENDER_PASS_SHADOW_3D, RENDER_PASS_SKYBOX,
};

fn id(name: &str) -> LogicalId {
    LogicalId::Str(name.into())
}

fn resource(name: &str) -> RenderGraphResource {
    RenderGraphResource {
        res_id: id(name),
        kind: RenderGraphResourceKind::Texture,
        lifetime: RenderGraphLifetime::Frame,
        alias_group: None,
    }
}

fn node(
    pass_id: &str,
    node_id: &str,
    inputs: Vec<LogicalId>,
    outputs: Vec<LogicalId>,
) -> RenderGraphNode {
    RenderGraphNode {
        node_id: id(node_id),
        pass_id: pass_id.into(),
        inputs,
        outputs,
        require: Vec::new(),
        priority: 0,
        enabled: true,
        params: HashMap::new(),
        shader: None,
    }
}

#[test]
fn fallback_graph_remains_semantically_valid() {
    let fallback = crate::fallback_graph();
    let result = validate_graph(&fallback);
    assert!(result.is_ok());
    assert!(RenderGraphState::from_desc(fallback).is_ok());
}

#[test]
fn allows_input_without_any_producer_when_not_required() {
    let desc = RenderGraphDesc {
        graph_id: id("g"),
        nodes: vec![node(RENDER_PASS_FORWARD, "n0", vec![id("missing")], vec![])],
        edges: vec![],
        resources: vec![resource("missing")],
        fallback: false,
    };
    assert!(validate_graph(&desc).is_ok());
}

#[test]
fn rejects_required_resource_without_any_producer() {
    let desc = RenderGraphDesc {
        graph_id: id("g"),
        nodes: vec![RenderGraphNode {
            require: vec![id("r")],
            ..node(RENDER_PASS_FORWARD, "consume", vec![id("r")], vec![])
        }],
        edges: vec![],
        resources: vec![resource("r")],
        fallback: false,
    };
    let err = validate_graph(&desc).expect_err("graph must fail");
    assert!(err.contains("Required resource"));
}

#[test]
fn rejects_multiple_writers_without_read_before_overwrite() {
    let desc = RenderGraphDesc {
        graph_id: id("g"),
        nodes: vec![
            node(RENDER_PASS_SHADOW_3D, "w0", vec![], vec![id("r")]),
            node(RENDER_PASS_POST, "w1", vec![], vec![id("r")]),
        ],
        edges: vec![RenderGraphEdge {
            from_node_id: id("w0"),
            to_node_id: id("w1"),
            reason: None,
        }],
        resources: vec![resource("r")],
        fallback: false,
    };
    let err = validate_graph(&desc).expect_err("graph must fail");
    assert!(err.contains("must read it before overwrite"));
}

#[test]
fn rejects_edge_reason_without_matching_dependency() {
    let desc = RenderGraphDesc {
        graph_id: id("g"),
        nodes: vec![
            node(RENDER_PASS_LIGHT_CULL, "a", vec![], vec![]),
            node(RENDER_PASS_SKYBOX, "b", vec![], vec![]),
        ],
        edges: vec![RenderGraphEdge {
            from_node_id: id("a"),
            to_node_id: id("b"),
            reason: Some(RenderGraphEdgeReason::ReadAfterWrite),
        }],
        resources: vec![],
        fallback: false,
    };
    let err = validate_graph(&desc).expect_err("graph must fail");
    assert!(err.contains("no matching resource dependency"));
}

#[test]
fn rejects_dangerous_same_priority_overwrite_when_both_read_write_same_output() {
    let desc = RenderGraphDesc {
        graph_id: id("g"),
        nodes: vec![
            RenderGraphNode {
                inputs: vec![id("color")],
                outputs: vec![id("color")],
                priority: 10,
                ..node(RENDER_PASS_POST, "a", vec![id("color")], vec![id("color")])
            },
            RenderGraphNode {
                inputs: vec![id("color")],
                outputs: vec![id("color")],
                priority: 10,
                ..node(RENDER_PASS_BLOOM, "b", vec![id("color")], vec![id("color")])
            },
            node(RENDER_PASS_FORWARD, "seed", vec![], vec![id("color")]),
        ],
        edges: vec![],
        resources: vec![resource("color")],
        fallback: false,
    };
    let err = validate_graph(&desc).expect_err("graph must fail");
    assert!(err.contains("dangerous same-priority overwrite"));
}

#[test]
fn rejects_node_shader_with_forbidden_tokens() {
    let mut custom = node(
        RENDER_PASS_POST,
        "post_with_shader",
        vec![id("color")],
        vec![id("color")],
    );
    custom.shader = Some(RenderGraphShaderSpec {
        shader_type: RenderGraphShaderType::Screen,
        source: "@group(0) @binding(0) var<uniform> x: vec4<f32>; fn fragment(input: FragmentInput) -> FragmentOutput { var out: FragmentOutput; out.color = sample_color(input.uv); return out; }".into(),
        params: HashMap::new(),
        capabilities: Default::default(),
    });
    let desc = RenderGraphDesc {
        graph_id: id("g"),
        nodes: vec![
            node(RENDER_PASS_FORWARD, "seed", vec![], vec![id("color")]),
            custom,
        ],
        edges: vec![],
        resources: vec![resource("color")],
        fallback: false,
    };
    let err = validate_graph(&desc).expect_err("shader with forbidden tokens should fail");
    assert!(err.contains("Invalid shader in node"));
}
