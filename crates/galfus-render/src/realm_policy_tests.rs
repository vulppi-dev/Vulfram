use super::{clear_alpha_for_realm_kind, graph_is_compatible_with_realm_kind};
use crate::{LogicalId, RenderGraphNode, RenderGraphPlan};
use galfus_realm_core::{RENDER_PASS_PREPARE, RENDER_PASS_SHADOW_3D, RealmKind};

#[test]
fn realm_policy_maps_clear_alpha_by_kind() {
    assert_eq!(clear_alpha_for_realm_kind(RealmKind::ThreeD), 1.0);
    assert_eq!(clear_alpha_for_realm_kind(RealmKind::TwoD), 0.0);
}

#[test]
fn realm_policy_validates_passes_by_kind() {
    let twod_plan = RenderGraphPlan {
        nodes: vec![RenderGraphNode {
            node_id: LogicalId::Str(RENDER_PASS_PREPARE.into()),
            pass_id: RENDER_PASS_PREPARE.into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            require: Vec::new(),
            priority: 0,
            enabled: true,
            params: Default::default(),
            shader: None,
        }],
        order: vec![0],
    };
    let threed_plan = RenderGraphPlan {
        nodes: vec![RenderGraphNode {
            node_id: LogicalId::Str(RENDER_PASS_SHADOW_3D.into()),
            pass_id: RENDER_PASS_SHADOW_3D.into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            require: Vec::new(),
            priority: 0,
            enabled: true,
            params: Default::default(),
            shader: None,
        }],
        order: vec![0],
    };

    assert!(graph_is_compatible_with_realm_kind(
        &twod_plan,
        RealmKind::TwoD
    ));
    assert!(!graph_is_compatible_with_realm_kind(
        &threed_plan,
        RealmKind::TwoD
    ));
    assert!(graph_is_compatible_with_realm_kind(
        &threed_plan,
        RealmKind::ThreeD
    ));
}
