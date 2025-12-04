use winit::application::ApplicationHandler;
use winit::event::WindowEvent as WinitWindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::core::render::render_frames;

use super::cmd::events::*;
use super::cmd::{self, EngineEvent, EngineEventEnvelope};
use super::singleton::EngineCustomEvents;
use super::state::EngineState;

impl ApplicationHandler<EngineCustomEvents> for EngineState {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        self.event_queue.push(EngineEventEnvelope {
            id: 0,
            event: EngineEvent::System(SystemEvent::OnResume),
        });
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.event_queue.push(EngineEventEnvelope {
            id: 0,
            event: EngineEvent::System(SystemEvent::OnSuspend),
        });
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.event_queue.push(EngineEventEnvelope {
            id: 0,
            event: EngineEvent::System(SystemEvent::OnExit),
        });
    }

    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
        self.event_queue.push(EngineEventEnvelope {
            id: 0,
            event: EngineEvent::System(SystemEvent::OnMemoryWarning),
        });
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        winit_window_id: WindowId,
        event: WinitWindowEvent,
    ) {
        let window_id = match self.window_id_map.get(&winit_window_id) {
            Some(id) => *id,
            None => return,
        };

        match event {
            WinitWindowEvent::Resized(size) => {
                // Update surface configuration with new size
                if let Some(window_state) = self.windows.get_mut(&window_id) {
                    if size.width > 0 && size.height > 0 {
                        window_state.config.width = size.width;
                        window_state.config.height = size.height;
                        window_state
                            .surface
                            .configure(self.device.as_ref().unwrap(), &window_state.config);

                        // Update size state
                        window_state.inner_size = [size.width, size.height];
                        let outer_size = window_state.window.outer_size();
                        window_state.outer_size = [outer_size.width, outer_size.height];

                        // Mark window as dirty to trigger redraw
                        window_state.is_dirty = true;
                    }
                }

                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Window(WindowEvent::OnResize {
                        window_id,
                        width: size.width,
                        height: size.height,
                    }),
                });
            }

            WinitWindowEvent::Moved(position) => {
                // Update window state
                if let Some(window_state) = self.windows.get_mut(&window_id) {
                    if let Ok(inner_pos) = window_state.window.inner_position() {
                        window_state.inner_position = [inner_pos.x, inner_pos.y];
                    }
                    if let Ok(outer_pos) = window_state.window.outer_position() {
                        window_state.outer_position = [outer_pos.x, outer_pos.y];
                    }
                }

                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Window(WindowEvent::OnMove {
                        window_id,
                        position: [position.x, position.y],
                    }),
                });
            }

            WinitWindowEvent::CloseRequested => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Window(WindowEvent::OnCloseRequest { window_id }),
                });
            }

            WinitWindowEvent::Destroyed => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Window(WindowEvent::OnDestroy { window_id }),
                });
            }

            WinitWindowEvent::DroppedFile(path) => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Window(WindowEvent::OnFileDrop {
                        window_id,
                        path: path.to_string_lossy().to_string(),
                    }),
                });
            }

            WinitWindowEvent::HoveredFile(path) => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Window(WindowEvent::OnFileHover {
                        window_id,
                        path: path.to_string_lossy().to_string(),
                    }),
                });
            }

            WinitWindowEvent::HoveredFileCancelled => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Window(WindowEvent::OnFileHoverCancel { window_id }),
                });
            }

            WinitWindowEvent::Focused(focused) => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Window(WindowEvent::OnFocus { window_id, focused }),
                });
            }

            WinitWindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                if is_synthetic {
                    return;
                }

                let key_code = convert_key_code(&event.physical_key);
                let location = convert_key_location(event.location);
                let state = if event.state.is_pressed() {
                    ElementState::Pressed
                } else {
                    ElementState::Released
                };

                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Keyboard(KeyboardEvent::OnInput {
                        window_id,
                        key_code,
                        state,
                        location,
                        repeat: event.repeat,
                        text: event.text.map(|s| s.to_string()),
                        modifiers: self.modifiers_state,
                    }),
                });
            }

            WinitWindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers_state = ModifiersState {
                    shift: modifiers.state().shift_key(),
                    ctrl: modifiers.state().control_key(),
                    alt: modifiers.state().alt_key(),
                    meta: modifiers.state().super_key(),
                };

                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Keyboard(KeyboardEvent::OnModifiersChange {
                        window_id,
                        modifiers: self.modifiers_state,
                    }),
                });
            }

            WinitWindowEvent::Ime(ime) => {
                let ime_event = match ime {
                    winit::event::Ime::Enabled => KeyboardEvent::OnImeEnable { window_id },
                    winit::event::Ime::Preedit(text, cursor) => KeyboardEvent::OnImePreedit {
                        window_id,
                        text,
                        cursor_range: cursor,
                    },
                    winit::event::Ime::Commit(text) => {
                        KeyboardEvent::OnImeCommit { window_id, text }
                    }
                    winit::event::Ime::Disabled => KeyboardEvent::OnImeDisable { window_id },
                };
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Keyboard(ime_event),
                });
            }

            WinitWindowEvent::CursorMoved { position, .. } => {
                // Save cursor position for this window
                let cursor_pos = [position.x as f32, position.y as f32];
                self.cursor_positions.insert(window_id, cursor_pos);

                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnMove {
                        window_id,
                        pointer_type: PointerType::Mouse,
                        pointer_id: 0,
                        position: cursor_pos,
                    }),
                });
            }

            WinitWindowEvent::CursorEntered { .. } => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnEnter {
                        window_id,
                        pointer_type: PointerType::Mouse,
                        pointer_id: 0,
                    }),
                });
            }

            WinitWindowEvent::CursorLeft { .. } => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnLeave {
                        window_id,
                        pointer_type: PointerType::Mouse,
                        pointer_id: 0,
                    }),
                });
            }

            WinitWindowEvent::MouseWheel { delta, phase, .. } => {
                let scroll_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => ScrollDelta::Line([x, y]),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        ScrollDelta::Pixel([pos.x as f32, pos.y as f32])
                    }
                };
                let touch_phase = convert_touch_phase(phase);

                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnScroll {
                        window_id,
                        delta: scroll_delta,
                        phase: touch_phase,
                    }),
                });
            }

            WinitWindowEvent::MouseInput { state, button, .. } => {
                let btn = convert_mouse_button(button);
                let elem_state = if state.is_pressed() {
                    ElementState::Pressed
                } else {
                    ElementState::Released
                };

                // Get the last known cursor position for this window
                let position = self
                    .cursor_positions
                    .get(&window_id)
                    .copied()
                    .unwrap_or([0.0, 0.0]);

                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnButton {
                        window_id,
                        pointer_type: PointerType::Mouse,
                        pointer_id: 0,
                        button: btn,
                        state: elem_state,
                        position,
                    }),
                });
            }

            WinitWindowEvent::PinchGesture { delta, phase, .. } => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnPinchGesture {
                        window_id,
                        delta,
                        phase: convert_touch_phase(phase),
                    }),
                });
            }

            WinitWindowEvent::PanGesture { delta, phase, .. } => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnPanGesture {
                        window_id,
                        delta: [delta.x, delta.y],
                        phase: convert_touch_phase(phase),
                    }),
                });
            }

            WinitWindowEvent::RotationGesture { delta, phase, .. } => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnRotationGesture {
                        window_id,
                        delta,
                        phase: convert_touch_phase(phase),
                    }),
                });
            }

            WinitWindowEvent::DoubleTapGesture { .. } => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnDoubleTapGesture { window_id }),
                });
            }

            WinitWindowEvent::Touch(touch) => {
                let phase = convert_touch_phase(touch.phase);
                let pressure = touch.force.map(|f| f.normalized() as f32);

                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnTouch {
                        window_id,
                        pointer_id: touch.id,
                        phase,
                        position: [touch.location.x as f32, touch.location.y as f32],
                        pressure,
                    }),
                });
            }

            WinitWindowEvent::ScaleFactorChanged {
                scale_factor,
                inner_size_writer: _,
            } => {
                // Get the current window inner size for the event
                let (new_width, new_height) = self
                    .windows
                    .get(&window_id)
                    .map(|ws| {
                        let size = ws.window.inner_size();
                        (size.width, size.height)
                    })
                    .unwrap_or((0, 0));

                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Window(WindowEvent::OnScaleFactorChange {
                        window_id,
                        scale_factor,
                        new_width,
                        new_height,
                    }),
                });
            }

            WinitWindowEvent::ThemeChanged(theme) => {
                let dark_mode = matches!(theme, winit::window::Theme::Dark);
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Window(WindowEvent::OnThemeChange {
                        window_id,
                        dark_mode,
                    }),
                });
            }

            WinitWindowEvent::Occluded(occluded) => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Window(WindowEvent::OnOcclude {
                        window_id,
                        occluded,
                    }),
                });
            }

            WinitWindowEvent::RedrawRequested => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Window(WindowEvent::OnRedrawRequest { window_id }),
                });

                render_frames(winit_window_id, self);
            }

            // Events we don't need to handle
            WinitWindowEvent::ActivationTokenDone { .. } => {}
            WinitWindowEvent::AxisMotion { .. } => {}
            WinitWindowEvent::TouchpadPressure { .. } => {}
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: EngineCustomEvents) {
        match event {
            EngineCustomEvents::ProcessCommands(batch) => {
                let _ = cmd::engine_process_batch(self, event_loop, batch);
            }
        }
    }
}
