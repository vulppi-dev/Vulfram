use once_cell::sync::OnceCell;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread::{self, ThreadId};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent as WinitWindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::platform::pump_events::EventLoopExtPumpEvents;
use winit::window::{Window, WindowId};

pub mod cmd;
pub mod units;

use cmd::EngineEvent;
use cmd::events::{
    ElementState, KeyCode, KeyLocation, KeyboardEvent, ModifiersState, MouseButton, PointerEvent,
    PointerType, ScrollDelta, SystemEvent, TouchPhase, WindowEvent,
};

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
    WinitError = 1000,
    // Reserved error codes for WGPU 2000-2999
    WgpuInstanceError = 2000,
    // Reserved error codes for Command Processing 3000-3999
    CmdInvalidCborError = 3000,
}

pub struct WindowState {
    pub id: WindowId,
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

pub struct EngineState {
    pub windows: HashMap<u32, WindowState>,
    pub window_id_map: HashMap<WindowId, u32>,
    pub buffers: HashMap<u64, Vec<u8>>,
    pub event_queue: cmd::EngineBatchEvents,
    pub event_loop: Option<EventLoop<()>>,

    pub wgpu: wgpu::Instance,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,

    pub time: u64,
    pub delta_time: u32,

    modifiers_state: ModifiersState,
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
        let event_loop = EventLoop::new().unwrap();

        Self {
            windows: HashMap::new(),
            window_id_map: HashMap::new(),
            buffers: HashMap::new(),
            event_queue: Vec::new(),
            event_loop: Some(event_loop),

            wgpu: wgpu_instance,
            device: None,
            queue: None,
            time: 0,
            delta_time: 0,

            modifiers_state: ModifiersState::default(),
        }
    }

    fn request_redraw(&self) {
        for window_state in self.windows.values() {
            window_state.window.request_redraw();
        }
    }
}

impl ApplicationHandler for EngineState {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        self.event_queue
            .push(EngineEvent::System(SystemEvent::Resumed));
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.event_queue
            .push(EngineEvent::System(SystemEvent::Suspended));
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.event_queue
            .push(EngineEvent::System(SystemEvent::Exiting));
    }

    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
        self.event_queue
            .push(EngineEvent::System(SystemEvent::MemoryWarning));
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
                self.event_queue
                    .push(EngineEvent::Window(WindowEvent::Resized {
                        window_id,
                        width: size.width,
                        height: size.height,
                    }));
            }

            WinitWindowEvent::Moved(position) => {
                self.event_queue
                    .push(EngineEvent::Window(WindowEvent::Moved {
                        window_id,
                        position: [position.x, position.y],
                    }));
            }

            WinitWindowEvent::CloseRequested => {
                self.event_queue
                    .push(EngineEvent::Window(WindowEvent::CloseRequested {
                        window_id,
                    }));
            }

            WinitWindowEvent::Destroyed => {
                self.event_queue
                    .push(EngineEvent::Window(WindowEvent::Destroyed { window_id }));
            }

            WinitWindowEvent::DroppedFile(path) => {
                self.event_queue
                    .push(EngineEvent::Window(WindowEvent::FileDropped {
                        window_id,
                        path: path.to_string_lossy().to_string(),
                    }));
            }

            WinitWindowEvent::HoveredFile(path) => {
                self.event_queue
                    .push(EngineEvent::Window(WindowEvent::FileHovered {
                        window_id,
                        path: path.to_string_lossy().to_string(),
                    }));
            }

            WinitWindowEvent::HoveredFileCancelled => {
                self.event_queue
                    .push(EngineEvent::Window(WindowEvent::FileHoveredCancelled {
                        window_id,
                    }));
            }

            WinitWindowEvent::Focused(focused) => {
                self.event_queue
                    .push(EngineEvent::Window(WindowEvent::Focused {
                        window_id,
                        focused,
                    }));
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

                self.event_queue
                    .push(EngineEvent::Keyboard(KeyboardEvent::Input {
                        window_id,
                        key_code,
                        state,
                        location,
                        repeat: event.repeat,
                        text: event.text.map(|s| s.to_string()),
                        modifiers: self.modifiers_state,
                    }));
            }

            WinitWindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers_state = ModifiersState {
                    shift: modifiers.state().shift_key(),
                    ctrl: modifiers.state().control_key(),
                    alt: modifiers.state().alt_key(),
                    meta: modifiers.state().super_key(),
                };

                self.event_queue
                    .push(EngineEvent::Keyboard(KeyboardEvent::ModifiersChanged {
                        window_id,
                        modifiers: self.modifiers_state,
                    }));
            }

            WinitWindowEvent::Ime(ime) => {
                let ime_event = match ime {
                    winit::event::Ime::Enabled => KeyboardEvent::ImeEnabled { window_id },
                    winit::event::Ime::Preedit(text, cursor) => KeyboardEvent::ImePreedit {
                        window_id,
                        text,
                        cursor_range: cursor,
                    },
                    winit::event::Ime::Commit(text) => KeyboardEvent::ImeCommit { window_id, text },
                    winit::event::Ime::Disabled => KeyboardEvent::ImeDisabled { window_id },
                };
                self.event_queue.push(EngineEvent::Keyboard(ime_event));
            }

            WinitWindowEvent::CursorMoved { position, .. } => {
                self.event_queue
                    .push(EngineEvent::Pointer(PointerEvent::Moved {
                        window_id,
                        pointer_type: PointerType::Mouse,
                        pointer_id: 0,
                        position: [position.x as f32, position.y as f32],
                    }));
            }

            WinitWindowEvent::CursorEntered { .. } => {
                self.event_queue
                    .push(EngineEvent::Pointer(PointerEvent::Entered {
                        window_id,
                        pointer_type: PointerType::Mouse,
                        pointer_id: 0,
                    }));
            }

            WinitWindowEvent::CursorLeft { .. } => {
                self.event_queue
                    .push(EngineEvent::Pointer(PointerEvent::Left {
                        window_id,
                        pointer_type: PointerType::Mouse,
                        pointer_id: 0,
                    }));
            }

            WinitWindowEvent::MouseWheel { delta, phase, .. } => {
                let scroll_delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => ScrollDelta::Line([x, y]),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        ScrollDelta::Pixel([pos.x as f32, pos.y as f32])
                    }
                };
                let touch_phase = convert_touch_phase(phase);

                self.event_queue
                    .push(EngineEvent::Pointer(PointerEvent::Scroll {
                        window_id,
                        delta: scroll_delta,
                        phase: touch_phase,
                    }));
            }

            WinitWindowEvent::MouseInput { state, button, .. } => {
                let btn = convert_mouse_button(button);
                let elem_state = if state.is_pressed() {
                    ElementState::Pressed
                } else {
                    ElementState::Released
                };

                self.event_queue
                    .push(EngineEvent::Pointer(PointerEvent::Button {
                        window_id,
                        pointer_type: PointerType::Mouse,
                        pointer_id: 0,
                        button: btn,
                        state: elem_state,
                        position: [0.0, 0.0], // Position is sent separately via CursorMoved
                    }));
            }

            WinitWindowEvent::PinchGesture { delta, phase, .. } => {
                self.event_queue
                    .push(EngineEvent::Pointer(PointerEvent::PinchGesture {
                        window_id,
                        delta,
                        phase: convert_touch_phase(phase),
                    }));
            }

            WinitWindowEvent::PanGesture { delta, phase, .. } => {
                self.event_queue
                    .push(EngineEvent::Pointer(PointerEvent::PanGesture {
                        window_id,
                        delta: [delta.x, delta.y],
                        phase: convert_touch_phase(phase),
                    }));
            }

            WinitWindowEvent::RotationGesture { delta, phase, .. } => {
                self.event_queue
                    .push(EngineEvent::Pointer(PointerEvent::RotationGesture {
                        window_id,
                        delta,
                        phase: convert_touch_phase(phase),
                    }));
            }

            WinitWindowEvent::DoubleTapGesture { .. } => {
                self.event_queue
                    .push(EngineEvent::Pointer(PointerEvent::DoubleTapGesture {
                        window_id,
                    }));
            }

            WinitWindowEvent::Touch(touch) => {
                let phase = convert_touch_phase(touch.phase);
                let pressure = touch.force.map(|f| f.normalized() as f32);

                self.event_queue
                    .push(EngineEvent::Pointer(PointerEvent::Touch {
                        window_id,
                        pointer_id: touch.id,
                        phase,
                        position: [touch.location.x as f32, touch.location.y as f32],
                        pressure,
                    }));
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

                self.event_queue
                    .push(EngineEvent::Window(WindowEvent::ScaleFactorChanged {
                        window_id,
                        scale_factor,
                        new_width,
                        new_height,
                    }));
            }

            WinitWindowEvent::ThemeChanged(theme) => {
                let dark_mode = matches!(theme, winit::window::Theme::Dark);
                self.event_queue
                    .push(EngineEvent::Window(WindowEvent::ThemeChanged {
                        window_id,
                        dark_mode,
                    }));
            }

            WinitWindowEvent::Occluded(occluded) => {
                self.event_queue
                    .push(EngineEvent::Window(WindowEvent::Occluded {
                        window_id,
                        occluded,
                    }));
            }

            WinitWindowEvent::RedrawRequested => {
                self.event_queue
                    .push(EngineEvent::Window(WindowEvent::RedrawRequested {
                        window_id,
                    }));
            }

            // Events we don't need to handle
            WinitWindowEvent::ActivationTokenDone { .. } => {}
            WinitWindowEvent::AxisMotion { .. } => {}
            WinitWindowEvent::TouchpadPressure { .. } => {}
        }
    }
}

// MARK: - Conversion Functions

fn convert_touch_phase(phase: winit::event::TouchPhase) -> TouchPhase {
    match phase {
        winit::event::TouchPhase::Started => TouchPhase::Started,
        winit::event::TouchPhase::Moved => TouchPhase::Moved,
        winit::event::TouchPhase::Ended => TouchPhase::Ended,
        winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

fn convert_mouse_button(button: winit::event::MouseButton) -> MouseButton {
    match button {
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        winit::event::MouseButton::Back => MouseButton::Back,
        winit::event::MouseButton::Forward => MouseButton::Forward,
        winit::event::MouseButton::Other(id) => MouseButton::Other(id as u8),
    }
}

fn convert_key_location(location: winit::keyboard::KeyLocation) -> KeyLocation {
    match location {
        winit::keyboard::KeyLocation::Standard => KeyLocation::Standard,
        winit::keyboard::KeyLocation::Left => KeyLocation::Left,
        winit::keyboard::KeyLocation::Right => KeyLocation::Right,
        winit::keyboard::KeyLocation::Numpad => KeyLocation::Numpad,
    }
}

fn convert_key_code(physical_key: &winit::keyboard::PhysicalKey) -> KeyCode {
    use winit::keyboard::KeyCode as WKeyCode;
    use winit::keyboard::PhysicalKey;

    match physical_key {
        PhysicalKey::Code(code) => match code {
            // Writing System Keys
            WKeyCode::Backquote => KeyCode::Backquote,
            WKeyCode::Backslash => KeyCode::Backslash,
            WKeyCode::BracketLeft => KeyCode::BracketLeft,
            WKeyCode::BracketRight => KeyCode::BracketRight,
            WKeyCode::Comma => KeyCode::Comma,
            WKeyCode::Digit0 => KeyCode::Digit0,
            WKeyCode::Digit1 => KeyCode::Digit1,
            WKeyCode::Digit2 => KeyCode::Digit2,
            WKeyCode::Digit3 => KeyCode::Digit3,
            WKeyCode::Digit4 => KeyCode::Digit4,
            WKeyCode::Digit5 => KeyCode::Digit5,
            WKeyCode::Digit6 => KeyCode::Digit6,
            WKeyCode::Digit7 => KeyCode::Digit7,
            WKeyCode::Digit8 => KeyCode::Digit8,
            WKeyCode::Digit9 => KeyCode::Digit9,
            WKeyCode::Equal => KeyCode::Equal,
            WKeyCode::IntlBackslash => KeyCode::IntlBackslash,
            WKeyCode::IntlRo => KeyCode::IntlRo,
            WKeyCode::IntlYen => KeyCode::IntlYen,
            WKeyCode::KeyA => KeyCode::KeyA,
            WKeyCode::KeyB => KeyCode::KeyB,
            WKeyCode::KeyC => KeyCode::KeyC,
            WKeyCode::KeyD => KeyCode::KeyD,
            WKeyCode::KeyE => KeyCode::KeyE,
            WKeyCode::KeyF => KeyCode::KeyF,
            WKeyCode::KeyG => KeyCode::KeyG,
            WKeyCode::KeyH => KeyCode::KeyH,
            WKeyCode::KeyI => KeyCode::KeyI,
            WKeyCode::KeyJ => KeyCode::KeyJ,
            WKeyCode::KeyK => KeyCode::KeyK,
            WKeyCode::KeyL => KeyCode::KeyL,
            WKeyCode::KeyM => KeyCode::KeyM,
            WKeyCode::KeyN => KeyCode::KeyN,
            WKeyCode::KeyO => KeyCode::KeyO,
            WKeyCode::KeyP => KeyCode::KeyP,
            WKeyCode::KeyQ => KeyCode::KeyQ,
            WKeyCode::KeyR => KeyCode::KeyR,
            WKeyCode::KeyS => KeyCode::KeyS,
            WKeyCode::KeyT => KeyCode::KeyT,
            WKeyCode::KeyU => KeyCode::KeyU,
            WKeyCode::KeyV => KeyCode::KeyV,
            WKeyCode::KeyW => KeyCode::KeyW,
            WKeyCode::KeyX => KeyCode::KeyX,
            WKeyCode::KeyY => KeyCode::KeyY,
            WKeyCode::KeyZ => KeyCode::KeyZ,
            WKeyCode::Minus => KeyCode::Minus,
            WKeyCode::Period => KeyCode::Period,
            WKeyCode::Quote => KeyCode::Quote,
            WKeyCode::Semicolon => KeyCode::Semicolon,
            WKeyCode::Slash => KeyCode::Slash,

            // Functional Keys
            WKeyCode::AltLeft => KeyCode::AltLeft,
            WKeyCode::AltRight => KeyCode::AltRight,
            WKeyCode::Backspace => KeyCode::Backspace,
            WKeyCode::CapsLock => KeyCode::CapsLock,
            WKeyCode::ContextMenu => KeyCode::ContextMenu,
            WKeyCode::ControlLeft => KeyCode::ControlLeft,
            WKeyCode::ControlRight => KeyCode::ControlRight,
            WKeyCode::Enter => KeyCode::Enter,
            WKeyCode::SuperLeft => KeyCode::SuperLeft,
            WKeyCode::SuperRight => KeyCode::SuperRight,
            WKeyCode::ShiftLeft => KeyCode::ShiftLeft,
            WKeyCode::ShiftRight => KeyCode::ShiftRight,
            WKeyCode::Space => KeyCode::Space,
            WKeyCode::Tab => KeyCode::Tab,

            // Control Keys
            WKeyCode::Delete => KeyCode::Delete,
            WKeyCode::End => KeyCode::End,
            WKeyCode::Help => KeyCode::Help,
            WKeyCode::Home => KeyCode::Home,
            WKeyCode::Insert => KeyCode::Insert,
            WKeyCode::PageDown => KeyCode::PageDown,
            WKeyCode::PageUp => KeyCode::PageUp,

            // Arrow Keys
            WKeyCode::ArrowDown => KeyCode::ArrowDown,
            WKeyCode::ArrowLeft => KeyCode::ArrowLeft,
            WKeyCode::ArrowRight => KeyCode::ArrowRight,
            WKeyCode::ArrowUp => KeyCode::ArrowUp,

            // Numpad Keys
            WKeyCode::NumLock => KeyCode::NumLock,
            WKeyCode::Numpad0 => KeyCode::Numpad0,
            WKeyCode::Numpad1 => KeyCode::Numpad1,
            WKeyCode::Numpad2 => KeyCode::Numpad2,
            WKeyCode::Numpad3 => KeyCode::Numpad3,
            WKeyCode::Numpad4 => KeyCode::Numpad4,
            WKeyCode::Numpad5 => KeyCode::Numpad5,
            WKeyCode::Numpad6 => KeyCode::Numpad6,
            WKeyCode::Numpad7 => KeyCode::Numpad7,
            WKeyCode::Numpad8 => KeyCode::Numpad8,
            WKeyCode::Numpad9 => KeyCode::Numpad9,
            WKeyCode::NumpadAdd => KeyCode::NumpadAdd,
            WKeyCode::NumpadBackspace => KeyCode::NumpadBackspace,
            WKeyCode::NumpadClear => KeyCode::NumpadClear,
            WKeyCode::NumpadClearEntry => KeyCode::NumpadClearEntry,
            WKeyCode::NumpadComma => KeyCode::NumpadComma,
            WKeyCode::NumpadDecimal => KeyCode::NumpadDecimal,
            WKeyCode::NumpadDivide => KeyCode::NumpadDivide,
            WKeyCode::NumpadEnter => KeyCode::NumpadEnter,
            WKeyCode::NumpadEqual => KeyCode::NumpadEqual,
            WKeyCode::NumpadHash => KeyCode::NumpadHash,
            WKeyCode::NumpadMemoryAdd => KeyCode::NumpadMemoryAdd,
            WKeyCode::NumpadMemoryClear => KeyCode::NumpadMemoryClear,
            WKeyCode::NumpadMemoryRecall => KeyCode::NumpadMemoryRecall,
            WKeyCode::NumpadMemoryStore => KeyCode::NumpadMemoryStore,
            WKeyCode::NumpadMemorySubtract => KeyCode::NumpadMemorySubtract,
            WKeyCode::NumpadMultiply => KeyCode::NumpadMultiply,
            WKeyCode::NumpadParenLeft => KeyCode::NumpadParenLeft,
            WKeyCode::NumpadParenRight => KeyCode::NumpadParenRight,
            WKeyCode::NumpadStar => KeyCode::NumpadStar,
            WKeyCode::NumpadSubtract => KeyCode::NumpadSubtract,

            // Function Keys
            WKeyCode::Escape => KeyCode::Escape,
            WKeyCode::F1 => KeyCode::F1,
            WKeyCode::F2 => KeyCode::F2,
            WKeyCode::F3 => KeyCode::F3,
            WKeyCode::F4 => KeyCode::F4,
            WKeyCode::F5 => KeyCode::F5,
            WKeyCode::F6 => KeyCode::F6,
            WKeyCode::F7 => KeyCode::F7,
            WKeyCode::F8 => KeyCode::F8,
            WKeyCode::F9 => KeyCode::F9,
            WKeyCode::F10 => KeyCode::F10,
            WKeyCode::F11 => KeyCode::F11,
            WKeyCode::F12 => KeyCode::F12,
            WKeyCode::F13 => KeyCode::F13,
            WKeyCode::F14 => KeyCode::F14,
            WKeyCode::F15 => KeyCode::F15,
            WKeyCode::F16 => KeyCode::F16,
            WKeyCode::F17 => KeyCode::F17,
            WKeyCode::F18 => KeyCode::F18,
            WKeyCode::F19 => KeyCode::F19,
            WKeyCode::F20 => KeyCode::F20,
            WKeyCode::F21 => KeyCode::F21,
            WKeyCode::F22 => KeyCode::F22,
            WKeyCode::F23 => KeyCode::F23,
            WKeyCode::F24 => KeyCode::F24,

            // Lock Keys
            WKeyCode::ScrollLock => KeyCode::ScrollLock,

            // Media Keys
            WKeyCode::AudioVolumeDown => KeyCode::AudioVolumeDown,
            WKeyCode::AudioVolumeMute => KeyCode::AudioVolumeMute,
            WKeyCode::AudioVolumeUp => KeyCode::AudioVolumeUp,
            WKeyCode::MediaPlayPause => KeyCode::MediaPlayPause,
            WKeyCode::MediaStop => KeyCode::MediaStop,
            WKeyCode::MediaTrackNext => KeyCode::MediaTrackNext,
            WKeyCode::MediaTrackPrevious => KeyCode::MediaTrackPrevious,

            // Browser Keys
            WKeyCode::BrowserBack => KeyCode::BrowserBack,
            WKeyCode::BrowserFavorites => KeyCode::BrowserFavorites,
            WKeyCode::BrowserForward => KeyCode::BrowserForward,
            WKeyCode::BrowserHome => KeyCode::BrowserHome,
            WKeyCode::BrowserRefresh => KeyCode::BrowserRefresh,
            WKeyCode::BrowserSearch => KeyCode::BrowserSearch,
            WKeyCode::BrowserStop => KeyCode::BrowserStop,

            // System Keys
            WKeyCode::PrintScreen => KeyCode::PrintScreen,
            WKeyCode::Pause => KeyCode::Pause,

            _ => KeyCode::Unidentified,
        },
        PhysicalKey::Unidentified(_) => KeyCode::Unidentified,
    }
}

// MARK: - Engine Management

thread_local! {
    static ENGINE_INSTANCE: RefCell<Option<EngineState>> = RefCell::new(None);
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
            *opt = Some(EngineState::new());
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

    match with_engine(|engine_state| cmd::engine_process_batch(engine_state, batch)) {
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
    match with_engine(|engine_state| {
        engine_state.time = time;
        engine_state.delta_time = delta_time;

        if let Some(mut event_loop) = engine_state.event_loop.take() {
            event_loop.set_control_flow(ControlFlow::Poll);
            event_loop.pump_app_events(None, engine_state);
            engine_state.event_loop = Some(event_loop);
        }

        engine_state.request_redraw();
    }) {
        Err(e) => e,
        Ok(_) => EngineResult::Success,
    }
}
