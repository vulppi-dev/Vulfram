use gilrs::{Event as GilrsEvent, EventType as GilrsEventType};
use std::time::Instant;
use winit::platform::pump_events::EventLoopExtPumpEvents;

use super::cmd::EngineEvent;
use super::cmd::events::{ElementState, GamepadEvent};
use super::result::VulframResult;
use super::singleton::with_engine_singleton;
use super::state::EngineState;

/// Main engine tick - processes events and updates state
pub fn engine_tick(time: u64, delta_time: u32) -> VulframResult {
    match with_engine_singleton(|engine| {
        engine.state.time = time;
        engine.state.delta_time = delta_time;

        // Reset profiling counters
        engine.state.profiling.gamepad_processing_ns = 0;
        engine.state.profiling.event_loop_pump_ns = 0;
        engine.state.profiling.total_events_cached = 0;
        engine.state.profiling.custom_events_ns = 0;

        let events_before = engine.state.event_queue.len();

        // MARK: Gamepad Processing
        let gamepad_start = Instant::now();
        let mut gilrs_events = Vec::new();
        if let Some(gilrs) = &mut engine.state.gilrs {
            while let Some(event) = gilrs.next_event() {
                gilrs_events.push(event);
            }
        }

        for event in gilrs_events {
            process_gilrs_event(&mut engine.state, event);
        }
        engine.state.profiling.gamepad_processing_ns = gamepad_start.elapsed().as_nanos() as u64;

        // MARK: Event Loop Pump
        if let Some(event_loop) = &mut engine.event_loop {
            // Only set control flow if it's not already set to Poll
            // This avoids unnecessary overhead
            let pump_start = Instant::now();

            // pump_app_events with timeout=None processes all pending events
            // without blocking or yielding to the OS
            event_loop.pump_app_events(None, &mut engine.state);

            let total_pump_time = pump_start.elapsed().as_nanos() as u64;
            // Subtract custom events time (window creation) from event loop pump time
            engine.state.profiling.event_loop_pump_ns =
                total_pump_time.saturating_sub(engine.state.profiling.custom_events_ns);
        }

        let events_after = engine.state.event_queue.len();
        engine.state.profiling.total_events_dispatched = events_after - events_before;

        // MARK: Request Redraw
        // Always request redraw to keep event loop active
        // Render will only happen for dirty windows (see RedrawRequested handler)
        let start = std::time::Instant::now();

        // Always request redraw to keep event loop active
        // The render function will check is_dirty before actually rendering
        for window_state in engine.state.windows.values_mut() {
            window_state.window.request_redraw();
        }

        engine.state.profiling.request_redraw_ns = start.elapsed().as_nanos() as u64;
    }) {
        Err(e) => e,
        Ok(_) => VulframResult::Success,
    }
}

/// Process a gilrs gamepad event and convert it to engine events
fn process_gilrs_event(engine_state: &mut EngineState, event: GilrsEvent) {
    let gamepad_id: u32 = usize::from(event.id) as u32;

    match event.event {
        GilrsEventType::Connected => {
            let name = if let Some(gilrs) = &engine_state.gilrs {
                gilrs.gamepad(event.id).name().to_string()
            } else {
                "Unknown".to_string()
            };

            // Add to cache
            engine_state
                .gamepad_cache
                .add_gamepad(gamepad_id, name.clone());

            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnConnect {
                    gamepad_id,
                    name,
                }));
        }
        GilrsEventType::Disconnected => {
            // Remove from cache
            engine_state.gamepad_cache.remove_gamepad(gamepad_id);

            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnDisconnect {
                    gamepad_id,
                }));
        }
        GilrsEventType::ButtonPressed(button, _code) => {
            let button_mapped = super::cmd::events::convert_gilrs_button(button);
            let value = 1.0;
            let state = ElementState::Pressed;

            // Check if button state actually changed
            if let Some(cache) = engine_state.gamepad_cache.get_mut(gamepad_id) {
                if !cache.button_changed(button_mapped, state, value) {
                    return;
                }
                cache.update_button(button_mapped, state, value);
            }

            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnButton {
                    gamepad_id,
                    button: button_mapped,
                    state,
                    value,
                }));
        }
        GilrsEventType::ButtonReleased(button, _code) => {
            let button_mapped = super::cmd::events::convert_gilrs_button(button);
            let value = 0.0;
            let state = ElementState::Released;

            // Check if button state actually changed
            if let Some(cache) = engine_state.gamepad_cache.get_mut(gamepad_id) {
                if !cache.button_changed(button_mapped, state, value) {
                    return;
                }
                cache.update_button(button_mapped, state, value);
            }

            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnButton {
                    gamepad_id,
                    button: button_mapped,
                    state,
                    value,
                }));
        }
        GilrsEventType::ButtonChanged(button, value, _code) => {
            let button_mapped = super::cmd::events::convert_gilrs_button(button);
            let state = if value > 0.5 {
                ElementState::Pressed
            } else {
                ElementState::Released
            };

            // Check if button state or value actually changed significantly
            if let Some(cache) = engine_state.gamepad_cache.get_mut(gamepad_id) {
                if !cache.button_changed(button_mapped, state, value) {
                    return;
                }
                cache.update_button(button_mapped, state, value);
            }

            engine_state
                .event_queue
                .push(EngineEvent::Gamepad(GamepadEvent::OnButton {
                    gamepad_id,
                    button: button_mapped,
                    state,
                    value,
                }));
        }
        GilrsEventType::AxisChanged(axis, value, _code) => {
            let axis_mapped = super::cmd::events::convert_gilrs_axis(axis);

            // Check if axis value actually changed significantly (with dead zone)
            if let Some(cache) = engine_state.gamepad_cache.get_mut(gamepad_id) {
                if !cache.axis_changed(axis_mapped, value) {
                    return;
                }

                // Get the adjusted value with dead zone applied
                let adjusted_value = cache.get_axis_value(axis_mapped);
                cache.update_axis(axis_mapped, value);

                engine_state
                    .event_queue
                    .push(EngineEvent::Gamepad(GamepadEvent::OnAxis {
                        gamepad_id,
                        axis: axis_mapped,
                        value: adjusted_value,
                    }));
            }
        }
        _ => {}
    }
}
