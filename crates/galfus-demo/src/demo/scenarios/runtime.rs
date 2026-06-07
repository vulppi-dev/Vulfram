use galfus_core::core;
use galfus_core::core::GalfusResult;
use galfus_core::core::cmd::{EngineCmd, EngineEvent};
use galfus_core::core::window::{CmdWindowCloseArgs, WindowEvent};
use std::collections::HashSet;
use std::time::{Duration, Instant};

use crate::demo::io::{receive_events, receive_responses, send_commands};

pub fn run_with_window_loop<FFrame, FEvents>(
    window_id: u32,
    frame_ms: u32,
    timeout: Option<Duration>,
    mut on_frame: FFrame,
    mut on_events: FEvents,
) -> bool
where
    FFrame: FnMut(f32),
    FEvents: FnMut(Vec<EngineEvent>),
{
    let start = Instant::now();
    let mut total_ms: u64 = 0;
    let mut close_sent = false;
    let mut active_windows = HashSet::new();
    active_windows.insert(window_id);

    loop {
        if let Some(limit) = timeout
            && start.elapsed() >= limit
        {
            break;
        }

        let time_seconds = total_ms as f32 / 1000.0;
        on_frame(time_seconds);
        assert_eq!(
            core::galfus_tick(total_ms as i64, frame_ms),
            GalfusResult::Success
        );
        total_ms = total_ms.saturating_add(frame_ms as u64);

        let _responses = receive_responses();
        let events = receive_events();
        let mut should_exit = false;
        for event in &events {
            match event {
                EngineEvent::Window(WindowEvent::OnCreate {
                    window_id: event_window_id,
                }) => {
                    active_windows.insert(*event_window_id);
                }
                EngineEvent::Window(WindowEvent::OnCloseRequest {
                    window_id: event_window_id,
                }) if *event_window_id == window_id => {
                    if !close_sent {
                        let close_cmd = EngineCmd::CmdWindowClose(CmdWindowCloseArgs { window_id });
                        let _ = send_commands(vec![close_cmd]);
                        close_sent = true;
                        // Native close command cleans up the window state immediately and may not
                        // produce an OnDestroy event in the same frame.
                        active_windows.remove(&window_id);
                        should_exit = active_windows.is_empty();
                    }
                }
                EngineEvent::Window(WindowEvent::OnDestroy {
                    window_id: event_window_id,
                }) if *event_window_id == window_id => {
                    should_exit = true;
                }
                EngineEvent::Window(WindowEvent::OnDestroy {
                    window_id: event_window_id,
                }) => {
                    active_windows.remove(event_window_id);
                }
                _ => {}
            }
        }
        on_events(events);
        if should_exit || active_windows.is_empty() {
            return close_sent;
        }

        std::thread::sleep(Duration::from_millis(frame_ms as u64));
    }

    close_sent
}
