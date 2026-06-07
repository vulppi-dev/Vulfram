mod runtime;
mod scenario_001_frame_graph;
mod scenario_002_optical_persistence;
mod scenario_003_realm2d;
mod scenario_004_realm2d_lights_shadows;

use crate::demo::{DemoContext, DemoKind, DemoRunOptions};
pub(crate) use runtime::run_with_window_loop;

pub fn run(demo: DemoKind, ctx: DemoContext, options: DemoRunOptions) -> bool {
    match demo {
        DemoKind::FrameGraph001 => scenario_001_frame_graph::run(ctx, options),
        DemoKind::FrameGraph002Persistence => scenario_002_optical_persistence::run(ctx, options),
        DemoKind::Realm2D003 => scenario_003_realm2d::run(ctx, options),
        DemoKind::Realm2D004LightsShadows => scenario_004_realm2d_lights_shadows::run(ctx, options),
    }
}
