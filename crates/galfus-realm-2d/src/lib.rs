mod state;

use galfus_realm_core::{
    RENDER_PASS_BATCH, RENDER_PASS_COMPOSE, RENDER_PASS_CUSTOM_POST_FORWARD,
    RENDER_PASS_CUSTOM_PRE_FORWARD, RENDER_PASS_FORWARD, RENDER_PASS_PREPARE,
    RENDER_PASS_SHADOW_2D,
};

pub const SUPPORTED_RENDER_PASSES: &[&str] = &[
    RENDER_PASS_PREPARE,
    RENDER_PASS_SHADOW_2D,
    RENDER_PASS_BATCH,
    RENDER_PASS_FORWARD,
    RENDER_PASS_CUSTOM_PRE_FORWARD,
    RENDER_PASS_CUSTOM_POST_FORWARD,
    RENDER_PASS_COMPOSE,
];

pub fn supports_render_pass(pass_id: &str) -> bool {
    SUPPORTED_RENDER_PASSES.contains(&pass_id)
}

pub fn graph_is_compatible<'a>(pass_ids: impl IntoIterator<Item = &'a str>) -> bool {
    pass_ids.into_iter().all(supports_render_pass)
}

pub use state::{Realm2dState, RealmEntities};

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
