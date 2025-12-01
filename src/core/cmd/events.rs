use serde::{Deserialize, Serialize};

use crate::core::units::{IVector2, Vector2};

// MARK: Common Types

/// Represents the state of an input element (pressed or released)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ElementState {
    Released = 0,
    Pressed = 1,
}

/// Represents the phase of a touch/gesture event
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TouchPhase {
    Started = 0,
    Moved = 1,
    Ended = 2,
    Cancelled = 3,
}

/// Represents keyboard modifier keys state
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModifiersState {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

// MARK: Window Events

/// Window-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum WindowEvent {
    /// Window was created successfully
    OnCreate { window_id: u32 },

    /// Window was resized
    OnResize {
        window_id: u32,
        width: u32,
        height: u32,
    },

    /// Window was moved
    OnMove { window_id: u32, position: IVector2 },

    /// Window close was requested by user
    OnCloseRequest { window_id: u32 },

    /// Window was destroyed
    OnDestroy { window_id: u32 },

    /// Window gained or lost focus
    OnFocus { window_id: u32, focused: bool },

    /// Window scale factor changed
    OnScaleFactorChange {
        window_id: u32,
        scale_factor: f64,
        new_width: u32,
        new_height: u32,
    },

    /// Window was occluded (completely hidden from view)
    OnOcclude { window_id: u32, occluded: bool },

    /// Window redraw was requested
    OnRedrawRequest { window_id: u32 },

    /// File was dropped into window
    OnFileDrop { window_id: u32, path: String },

    /// File is being hovered over window
    OnFileHover { window_id: u32, path: String },

    /// Hovered file left the window
    OnFileHoverCancel { window_id: u32 },

    /// System theme changed
    OnThemeChange { window_id: u32, dark_mode: bool },
}

// MARK: Pointer Events

/// Mouse button types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MouseButton {
    Left = 0,
    Right = 1,
    Middle = 2,
    Back = 3,
    Forward = 4,
    Other(u8),
}

/// Pointer type for unified mouse/touch handling
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PointerType {
    Mouse = 0,
    Touch = 1,
    Pen = 2,
}

/// Mouse scroll delta type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "kebab-case")]
pub enum ScrollDelta {
    /// Line-based scrolling (traditional mouse wheel)
    Line(Vector2),
    /// Pixel-based scrolling (touchpad)
    Pixel(Vector2),
}

/// Pointer (Mouse/Touch) events - unified for both input types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum PointerEvent {
    /// Pointer moved
    OnMove {
        window_id: u32,
        pointer_type: PointerType,
        pointer_id: u64,
        position: Vector2,
    },

    /// Pointer entered window area
    OnEnter {
        window_id: u32,
        pointer_type: PointerType,
        pointer_id: u64,
    },

    /// Pointer left window area
    OnLeave {
        window_id: u32,
        pointer_type: PointerType,
        pointer_id: u64,
    },

    /// Pointer button pressed/released (mouse) or touch started/ended
    OnButton {
        window_id: u32,
        pointer_type: PointerType,
        pointer_id: u64,
        button: MouseButton,
        state: ElementState,
        position: Vector2,
    },

    /// Mouse wheel/touchpad scroll
    OnScroll {
        window_id: u32,
        delta: ScrollDelta,
        phase: TouchPhase,
    },

    /// Touch event with pressure and additional info
    OnTouch {
        window_id: u32,
        pointer_id: u64,
        phase: TouchPhase,
        position: Vector2,
        pressure: Option<f32>,
    },

    /// Pinch gesture (zoom)
    OnPinchGesture {
        window_id: u32,
        delta: f64,
        phase: TouchPhase,
    },

    /// Pan gesture
    OnPanGesture {
        window_id: u32,
        delta: Vector2,
        phase: TouchPhase,
    },

    /// Rotation gesture
    OnRotationGesture {
        window_id: u32,
        delta: f32,
        phase: TouchPhase,
    },

    /// Double tap gesture
    OnDoubleTapGesture { window_id: u32 },
}

// MARK: Keyboard Events

/// Key location on keyboard
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyLocation {
    Standard = 0,
    Left = 1,
    Right = 2,
    Numpad = 3,
}

/// Physical key code (scancode-like, layout independent)
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyCode {
    // Writing System Keys
    Backquote,
    Backslash,
    BracketLeft,
    BracketRight,
    Comma,
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Equal,
    IntlBackslash,
    IntlRo,
    IntlYen,
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
    Minus,
    Period,
    Quote,
    Semicolon,
    Slash,

    // Functional Keys
    AltLeft,
    AltRight,
    Backspace,
    CapsLock,
    ContextMenu,
    ControlLeft,
    ControlRight,
    Enter,
    SuperLeft,
    SuperRight,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,

    // Control Keys
    Delete,
    End,
    Help,
    Home,
    Insert,
    PageDown,
    PageUp,

    // Arrow Keys
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,

    // Numpad Keys
    NumLock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadBackspace,
    NumpadClear,
    NumpadClearEntry,
    NumpadComma,
    NumpadDecimal,
    NumpadDivide,
    NumpadEnter,
    NumpadEqual,
    NumpadHash,
    NumpadMemoryAdd,
    NumpadMemoryClear,
    NumpadMemoryRecall,
    NumpadMemoryStore,
    NumpadMemorySubtract,
    NumpadMultiply,
    NumpadParenLeft,
    NumpadParenRight,
    NumpadStar,
    NumpadSubtract,

    // Function Keys
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    // Lock Keys
    ScrollLock,

    // Media Keys
    AudioVolumeDown,
    AudioVolumeMute,
    AudioVolumeUp,
    MediaPlayPause,
    MediaStop,
    MediaTrackNext,
    MediaTrackPrevious,

    // Browser Keys
    BrowserBack,
    BrowserFavorites,
    BrowserForward,
    BrowserHome,
    BrowserRefresh,
    BrowserSearch,
    BrowserStop,

    // System Keys
    PrintScreen,
    Pause,

    // Unknown/Unidentified key
    Unidentified,
}

/// Keyboard input event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum KeyboardEvent {
    /// Key was pressed or released
    OnInput {
        window_id: u32,
        key_code: KeyCode,
        state: ElementState,
        location: KeyLocation,
        repeat: bool,
        text: Option<String>,
        modifiers: ModifiersState,
    },

    /// Modifiers changed
    OnModifiersChange {
        window_id: u32,
        modifiers: ModifiersState,
    },

    /// IME composition started
    OnImeEnable { window_id: u32 },

    /// IME composition in progress
    OnImePreedit {
        window_id: u32,
        text: String,
        cursor_range: Option<(usize, usize)>,
    },

    /// IME composition committed
    OnImeCommit { window_id: u32, text: String },

    /// IME disabled
    OnImeDisable { window_id: u32 },
}

// MARK: Gamepad Events

/// Gamepad button types following standard gamepad mapping
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GamepadButton {
    // Face buttons
    South = 0, // A / Cross
    East = 1,  // B / Circle
    West = 2,  // X / Square
    North = 3, // Y / Triangle

    // Shoulder buttons
    LeftBumper = 4,
    RightBumper = 5,
    LeftTrigger = 6,
    RightTrigger = 7,

    // Center buttons
    Select = 8,
    Start = 9,
    Mode = 10, // Guide / Home

    // Stick buttons
    LeftStick = 11,
    RightStick = 12,

    // D-pad
    DpadUp = 13,
    DpadDown = 14,
    DpadLeft = 15,
    DpadRight = 16,

    // Other
    Other(u8),
}

/// Gamepad axis types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GamepadAxis {
    LeftStickX = 0,
    LeftStickY = 1,
    RightStickX = 2,
    RightStickY = 3,
    LeftTrigger = 4,
    RightTrigger = 5,
    Other(u8),
}

/// Gamepad events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum GamepadEvent {
    /// Gamepad was connected
    OnConnect { gamepad_id: u32, name: String },

    /// Gamepad was disconnected
    OnDisconnect { gamepad_id: u32 },

    /// Gamepad button pressed/released
    OnButton {
        gamepad_id: u32,
        button: GamepadButton,
        state: ElementState,
        value: f32, // 0.0-1.0 for analog triggers
    },

    /// Gamepad axis moved
    OnAxis {
        gamepad_id: u32,
        axis: GamepadAxis,
        value: f32, // -1.0 to 1.0 for sticks, 0.0 to 1.0 for triggers
    },
}

// MARK: Joystick Events

/// Joystick hat position
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JoystickHatPosition {
    Centered = 0,
    Up = 1,
    RightUp = 2,
    Right = 3,
    RightDown = 4,
    Down = 5,
    LeftDown = 6,
    Left = 7,
    LeftUp = 8,
}

/// Raw joystick events (for non-standard controllers)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum JoystickEvent {
    /// Joystick was connected
    OnConnect {
        joystick_id: u32,
        name: String,
        axes_count: u32,
        buttons_count: u32,
        hats_count: u32,
    },

    /// Joystick was disconnected
    OnDisconnect { joystick_id: u32 },

    /// Raw joystick button pressed/released
    OnButton {
        joystick_id: u32,
        button_index: u32,
        state: ElementState,
    },

    /// Raw joystick axis moved
    OnAxis {
        joystick_id: u32,
        axis_index: u32,
        value: f32, // -1.0 to 1.0
    },

    /// Raw joystick hat/POV moved
    OnHat {
        joystick_id: u32,
        hat_index: u32,
        position: JoystickHatPosition,
    },
}

// MARK: System Events

/// System-level events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "kebab-case")]
pub enum SystemEvent {
    /// Application was resumed (from suspended state)
    OnResume,

    /// Application was suspended
    OnSuspend,

    /// Low memory warning
    OnMemoryWarning,

    /// Application is about to exit
    OnExit,
}

// MARK: Conversion Functions

pub fn convert_touch_phase(phase: winit::event::TouchPhase) -> TouchPhase {
    match phase {
        winit::event::TouchPhase::Started => TouchPhase::Started,
        winit::event::TouchPhase::Moved => TouchPhase::Moved,
        winit::event::TouchPhase::Ended => TouchPhase::Ended,
        winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

pub fn convert_mouse_button(button: winit::event::MouseButton) -> MouseButton {
    match button {
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        winit::event::MouseButton::Back => MouseButton::Back,
        winit::event::MouseButton::Forward => MouseButton::Forward,
        winit::event::MouseButton::Other(id) => MouseButton::Other(id as u8),
    }
}

pub fn convert_key_location(location: winit::keyboard::KeyLocation) -> KeyLocation {
    match location {
        winit::keyboard::KeyLocation::Standard => KeyLocation::Standard,
        winit::keyboard::KeyLocation::Left => KeyLocation::Left,
        winit::keyboard::KeyLocation::Right => KeyLocation::Right,
        winit::keyboard::KeyLocation::Numpad => KeyLocation::Numpad,
    }
}

pub fn convert_gilrs_button(button: gilrs::Button) -> GamepadButton {
    use gilrs::Button;

    match button {
        Button::South => GamepadButton::South,
        Button::East => GamepadButton::East,
        Button::West => GamepadButton::West,
        Button::North => GamepadButton::North,
        Button::LeftTrigger => GamepadButton::LeftBumper,
        Button::RightTrigger => GamepadButton::RightBumper,
        Button::LeftTrigger2 => GamepadButton::LeftTrigger,
        Button::RightTrigger2 => GamepadButton::RightTrigger,
        Button::Select => GamepadButton::Select,
        Button::Start => GamepadButton::Start,
        Button::Mode => GamepadButton::Mode,
        Button::LeftThumb => GamepadButton::LeftStick,
        Button::RightThumb => GamepadButton::RightStick,
        Button::DPadUp => GamepadButton::DpadUp,
        Button::DPadDown => GamepadButton::DpadDown,
        Button::DPadLeft => GamepadButton::DpadLeft,
        Button::DPadRight => GamepadButton::DpadRight,
        _ => GamepadButton::Other(255),
    }
}

pub fn convert_gilrs_axis(axis: gilrs::Axis) -> GamepadAxis {
    use gilrs::Axis;

    match axis {
        Axis::LeftStickX => GamepadAxis::LeftStickX,
        Axis::LeftStickY => GamepadAxis::LeftStickY,
        Axis::RightStickX => GamepadAxis::RightStickX,
        Axis::RightStickY => GamepadAxis::RightStickY,
        Axis::LeftZ => GamepadAxis::LeftTrigger,
        Axis::RightZ => GamepadAxis::RightTrigger,
        _ => GamepadAxis::Other(255),
    }
}

pub fn convert_key_code(physical_key: &winit::keyboard::PhysicalKey) -> KeyCode {
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
