use gilrs::{Event as GilrsEvent, EventType as GilrsEventType, Gilrs};
use once_cell::sync::OnceCell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::{self, ThreadId};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent as WinitWindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::{Window, WindowId};

pub mod cmd;
pub mod units;

use cmd::EngineEvent;
use cmd::events::{
    ElementState, KeyboardEvent, ModifiersState, PointerEvent, PointerType, ScrollDelta,
    SystemEvent, WindowEvent,
};

use crate::core::cmd::EngineEventEnvelope;

#[derive(Debug)]
#[repr(u32)]
pub enum EngineResult {
    Success = 0,
    UnknownError = 1,
    NotInitialized,
    AlreadyInitialized,
    WrongThread,
    BufferOverflow,
    // Reserved error codes for Winit 1000-1999
    WinitEventLoopNotInitializedError = 1000,
    WinitCreateWindowError,
    // Reserved error codes for WGPU 2000-2999
    WgpuInstanceError = 2000,
    // Reserved error codes for Command Processing 3000-3999
    CmdInvalidCborError = 3000,
}

pub struct WindowState {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

pub struct EngineState {
    // Públicos - acessados em cmd/win.rs
    pub windows: HashMap<u32, WindowState>,
    pub window_id_map: HashMap<WindowId, u32>,
    pub window_id_counter: u32,
    pub wgpu: wgpu::Instance,
    pub caps: Option<wgpu::SurfaceCapabilities>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,

    // Públicos - acessados em funções públicas do módulo
    pub buffers: HashMap<u64, Vec<u8>>,
    pub event_queue: cmd::EngineBatchEvents,

    // Privados - apenas uso interno
    time: u64,
    delta_time: u32,
    modifiers_state: ModifiersState,
    gilrs: Option<Gilrs>,
}

struct EngineSingleton {
    pub state: EngineState,
    pub event_loop: Option<EventLoop<EngineCustomEvents>>,
    pub proxy: Option<EventLoopProxy<EngineCustomEvents>>,
}

enum EngineCustomEvents {
    ProcessCommands(cmd::EngineBatchCmds),
}

impl EngineState {
    pub fn new() -> Self {
        let wgpu_descriptor = wgpu::InstanceDescriptor {
            backends: if cfg!(target_os = "ios") || cfg!(target_os = "macos") {
                wgpu::Backends::METAL | wgpu::Backends::VULKAN
            } else {
                wgpu::Backends::DX12 | wgpu::Backends::VULKAN
            },
            backend_options: wgpu::BackendOptions::default(),
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        };
        let wgpu_instance = wgpu::Instance::new(&wgpu_descriptor);

        // Initialize gilrs for gamepad support
        let gilrs = match Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(e) => {
                log::warn!("Failed to initialize gamepad support: {:?}", e);
                None
            }
        };

        Self {
            windows: HashMap::new(),
            window_id_map: HashMap::new(),
            buffers: HashMap::new(),
            event_queue: Vec::new(),

            window_id_counter: 0,

            wgpu: wgpu_instance,
            caps: None,
            device: None,
            queue: None,
            time: 0,
            delta_time: 0,

            modifiers_state: ModifiersState::default(),
            gilrs,
        }
    }

    fn request_redraw(&self) {
        for window_state in self.windows.values() {
            window_state.window.request_redraw();
        }
    }
}

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

                let key_code = cmd::events::convert_key_code(&event.physical_key);
                let location = cmd::events::convert_key_location(event.location);
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
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnMove {
                        window_id,
                        pointer_type: PointerType::Mouse,
                        pointer_id: 0,
                        position: [position.x as f32, position.y as f32],
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
                let touch_phase = cmd::events::convert_touch_phase(phase);

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
                let btn = cmd::events::convert_mouse_button(button);
                let elem_state = if state.is_pressed() {
                    ElementState::Pressed
                } else {
                    ElementState::Released
                };

                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnButton {
                        window_id,
                        pointer_type: PointerType::Mouse,
                        pointer_id: 0,
                        button: btn,
                        state: elem_state,
                        position: [0.0, 0.0], // Position is sent separately via CursorMoved
                    }),
                });
            }

            WinitWindowEvent::PinchGesture { delta, phase, .. } => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnPinchGesture {
                        window_id,
                        delta,
                        phase: cmd::events::convert_touch_phase(phase),
                    }),
                });
            }

            WinitWindowEvent::PanGesture { delta, phase, .. } => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnPanGesture {
                        window_id,
                        delta: [delta.x, delta.y],
                        phase: cmd::events::convert_touch_phase(phase),
                    }),
                });
            }

            WinitWindowEvent::RotationGesture { delta, phase, .. } => {
                self.event_queue.push(EngineEventEnvelope {
                    id: 0,
                    event: EngineEvent::Pointer(PointerEvent::OnRotationGesture {
                        window_id,
                        delta,
                        phase: cmd::events::convert_touch_phase(phase),
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
                let phase = cmd::events::convert_touch_phase(touch.phase);
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

// MARK: - Engine Management

thread_local! {
    static ENGINE_INSTANCE: RefCell<Option<EngineSingleton>> = RefCell::new(None);
}
static MAIN_THREAD_ID: OnceCell<ThreadId> = OnceCell::new();

pub fn engine_init() -> EngineResult {
    let _ = env_logger::try_init();
    let current_id = thread::current().id();

    if let Err(_) = MAIN_THREAD_ID.set(current_id) {
        if MAIN_THREAD_ID.get().unwrap() != &current_id {
            return EngineResult::WrongThread;
        }
    }

    ENGINE_INSTANCE.with(|cell| {
        let mut opt = cell.borrow_mut();
        if opt.is_some() {
            return EngineResult::AlreadyInitialized;
        } else {
            let event_loop = EventLoop::<EngineCustomEvents>::with_user_event()
                .build()
                .unwrap();
            let proxy = event_loop.create_proxy();

            *opt = Some(EngineSingleton {
                state: EngineState::new(),
                event_loop: Some(event_loop),
                proxy: Some(proxy),
            });
            return EngineResult::Success;
        }
    })
}

pub fn engine_dispose() -> EngineResult {
    let current_id = thread::current().id();

    if let Some(main_id) = MAIN_THREAD_ID.get() {
        if &current_id != main_id {
            return EngineResult::WrongThread;
        }
    } else {
        return EngineResult::NotInitialized;
    }

    ENGINE_INSTANCE.with(|cell| {
        let mut opt = cell.borrow_mut();
        *opt = None;
    });

    EngineResult::Success
}

pub fn with_engine<F, R>(f: F) -> Result<R, EngineResult>
where
    F: FnOnce(&mut EngineState) -> R,
{
    let current_id = thread::current().id();
    let main_id = MAIN_THREAD_ID.get().ok_or(EngineResult::NotInitialized)?;

    if &current_id != main_id {
        return Err(EngineResult::WrongThread);
    }

    ENGINE_INSTANCE.with(|cell| {
        let mut opt = cell.borrow_mut();
        let engine_state = opt.as_mut().ok_or(EngineResult::NotInitialized)?;
        Ok(f(&mut engine_state.state))
    })
}

fn with_engine_singleton<F, R>(f: F) -> Result<R, EngineResult>
where
    F: FnOnce(&mut EngineSingleton) -> R,
{
    let current_id = thread::current().id();
    let main_id = MAIN_THREAD_ID.get().ok_or(EngineResult::NotInitialized)?;

    if &current_id != main_id {
        return Err(EngineResult::WrongThread);
    }

    ENGINE_INSTANCE.with(|cell| {
        let mut opt = cell.borrow_mut();
        let engine_state = opt.as_mut().ok_or(EngineResult::NotInitialized)?;
        Ok(f(engine_state))
    })
}

pub fn engine_send_queue(ptr: *const u8, length: usize) -> EngineResult {
    let data = unsafe { std::slice::from_raw_parts(ptr, length).to_vec() };

    let batch = match serde_cbor::from_slice::<cmd::EngineBatchCmds>(&data) {
        Err(_e) => {
            return EngineResult::CmdInvalidCborError;
        }
        Ok(batch) => batch,
    };

    match with_engine_singleton(|engine| {
        if let Some(proxy) = &engine.proxy {
            let _ = proxy.send_event(EngineCustomEvents::ProcessCommands(batch));
        }
    }) {
        Err(e) => return e,
        Ok(_) => EngineResult::Success,
    }
}

pub fn engine_receive_queue(out_ptr: *mut u8, out_length: *mut usize) -> EngineResult {
    match with_engine(|engine| {
        let serialized = match serde_cbor::to_vec(&engine.event_queue) {
            Ok(data) => data,
            Err(_) => return EngineResult::UnknownError,
        };

        let required_length = serialized.len();

        unsafe {
            if out_ptr.is_null() {
                *out_length = required_length;
                return EngineResult::Success;
            }

            let available_length = *out_length;

            if required_length <= available_length {
                std::ptr::copy_nonoverlapping(serialized.as_ptr(), out_ptr, required_length);
                *out_length = required_length;
                engine.event_queue.clear();
                return EngineResult::Success;
            } else {
                *out_length = required_length;
                return EngineResult::BufferOverflow;
            }
        }
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}

pub fn engine_upload_buffer(bfr_id: u64, bfr_ptr: *const u8, bfr_length: usize) -> EngineResult {
    let data = unsafe { std::slice::from_raw_parts(bfr_ptr, bfr_length).to_vec() };

    match with_engine(|engine| {
        engine.buffers.insert(bfr_id, data);
    }) {
        Err(e) => e,
        Ok(_) => EngineResult::Success,
    }
}

pub fn engine_download_buffer(
    bfr_id: u64,
    bfr_ptr: *mut u8,
    bfr_length: *mut usize,
) -> EngineResult {
    match with_engine(|engine| {
        let buffer = match engine.buffers.get(&bfr_id) {
            Some(buf) => buf,
            None => return EngineResult::UnknownError,
        };

        let required_length = buffer.len();

        unsafe {
            if bfr_ptr.is_null() {
                *bfr_length = required_length;
                return EngineResult::Success;
            }

            let available_length = *bfr_length;

            if required_length <= available_length {
                std::ptr::copy_nonoverlapping(buffer.as_ptr(), bfr_ptr, required_length);
                *bfr_length = required_length;
                return EngineResult::Success;
            } else {
                *bfr_length = required_length;
                return EngineResult::BufferOverflow;
            }
        }
    }) {
        Err(e) => e,
        Ok(result) => result,
    }
}

pub fn engine_clear_buffer(bfr_id: u64) -> EngineResult {
    match with_engine(|engine| {
        engine.buffers.remove(&bfr_id);
    }) {
        Err(e) => return e,
        Ok(_) => EngineResult::Success,
    }
}

pub fn engine_tick(time: u64, delta_time: u32) -> EngineResult {
    match with_engine_singleton(|engine| {
        engine.state.time = time;
        engine.state.delta_time = delta_time;

        // Process gamepad/joystick events
        let mut gilrs_events = Vec::new();
        if let Some(gilrs) = &mut engine.state.gilrs {
            while let Some(event) = gilrs.next_event() {
                gilrs_events.push(event);
            }
        }

        for event in gilrs_events {
            process_gilrs_event(&mut engine.state, event);
        }

        if let Some(mut event_loop) = engine.event_loop.take() {
            event_loop.set_control_flow(ControlFlow::Poll);
            event_loop.pump_app_events(None, &mut engine.state);
            engine.event_loop = Some(event_loop);
        }

        engine.state.request_redraw();
    }) {
        Err(e) => e,
        Ok(_) => EngineResult::Success,
    }
}

fn process_gilrs_event(engine_state: &mut EngineState, event: GilrsEvent) {
    let gamepad_id: u32 = usize::from(event.id) as u32;

    match event.event {
        GilrsEventType::Connected => {
            let name = if let Some(gilrs) = &engine_state.gilrs {
                gilrs.gamepad(event.id).name().to_string()
            } else {
                "Unknown".to_string()
            };

            engine_state.event_queue.push(EngineEventEnvelope {
                id: 0,
                event: EngineEvent::Gamepad(cmd::events::GamepadEvent::OnConnect {
                    gamepad_id,
                    name,
                }),
            });
        }
        GilrsEventType::Disconnected => {
            engine_state.event_queue.push(EngineEventEnvelope {
                id: 0,
                event: EngineEvent::Gamepad(cmd::events::GamepadEvent::OnDisconnect { gamepad_id }),
            });
        }
        GilrsEventType::ButtonPressed(button, _code) => {
            let button_mapped = cmd::events::convert_gilrs_button(button);
            engine_state.event_queue.push(EngineEventEnvelope {
                id: 0,
                event: EngineEvent::Gamepad(cmd::events::GamepadEvent::OnButton {
                    gamepad_id,
                    button: button_mapped,
                    state: cmd::events::ElementState::Pressed,
                    value: 1.0,
                }),
            });
        }
        GilrsEventType::ButtonReleased(button, _code) => {
            let button_mapped = cmd::events::convert_gilrs_button(button);
            engine_state.event_queue.push(EngineEventEnvelope {
                id: 0,
                event: EngineEvent::Gamepad(cmd::events::GamepadEvent::OnButton {
                    gamepad_id,
                    button: button_mapped,
                    state: cmd::events::ElementState::Released,
                    value: 0.0,
                }),
            });
        }
        GilrsEventType::ButtonChanged(button, value, _code) => {
            let button_mapped = cmd::events::convert_gilrs_button(button);
            let state = if value > 0.5 {
                cmd::events::ElementState::Pressed
            } else {
                cmd::events::ElementState::Released
            };
            engine_state.event_queue.push(EngineEventEnvelope {
                id: 0,
                event: EngineEvent::Gamepad(cmd::events::GamepadEvent::OnButton {
                    gamepad_id,
                    button: button_mapped,
                    state,
                    value,
                }),
            });
        }
        GilrsEventType::AxisChanged(axis, value, _code) => {
            let axis_mapped = cmd::events::convert_gilrs_axis(axis);
            engine_state.event_queue.push(EngineEventEnvelope {
                id: 0,
                event: EngineEvent::Gamepad(cmd::events::GamepadEvent::OnAxis {
                    gamepad_id,
                    axis: axis_mapped,
                    value,
                }),
            });
        }
        _ => {}
    }
}
