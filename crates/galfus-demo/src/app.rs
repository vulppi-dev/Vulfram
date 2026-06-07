use crate::demo::{DemoContext, create_window, run_demo, select_demo};
use galfus_core::core::GalfusResult;
use galfus_core::core::cmd::EngineCmd;
use galfus_core::core::window::CmdWindowCloseArgs;
use std::sync::Mutex;

static ENGINE_GUARD: Mutex<()> = Mutex::new(());

pub fn run() {
    let _lock = ENGINE_GUARD.lock().unwrap();

    assert_eq!(galfus_core::core::galfus_init(), GalfusResult::Success);

    let selection = select_demo();
    let window_id: u32 = 1;

    let title = selection.demo.title();
    let binding = create_window(window_id, &title);

    let close_sent = run_demo(
        selection.demo,
        DemoContext {
            window_id,
            realm_id: binding.realm_id,
        },
        selection.options,
    );

    if !close_sent {
        let close_cmd = EngineCmd::CmdWindowClose(CmdWindowCloseArgs { window_id });
        let _ = crate::demo::send_commands(vec![close_cmd]);
    }

    assert_eq!(galfus_core::core::galfus_dispose(), GalfusResult::Success);
}
