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
    Created { window_id: u32 },

    /// Window was resized
    Resized {
        window_id: u32,
        width: u32,
        height: u32,
    },

    /// Window was moved
    Moved { window_id: u32, position: IVector2 },

    /// Window close was requested by user
    CloseRequested { window_id: u32 },

    /// Window was destroyed
    Destroyed { window_id: u32 },

    /// Window gained or lost focus
    Focused { window_id: u32, focused: bool },

    /// Window scale factor changed
    ScaleFactorChanged {
        window_id: u32,
        scale_factor: f64,
        new_width: u32,
        new_height: u32,
    },

    /// Window was occluded (completely hidden from view)
    Occluded { window_id: u32, occluded: bool },

    /// Window redraw was requested
    RedrawRequested { window_id: u32 },

    /// File was dropped into window
    FileDropped { window_id: u32, path: String },

    /// File is being hovered over window
    FileHovered { window_id: u32, path: String },

    /// Hovered file left the window
    FileHoveredCancelled { window_id: u32 },

    /// System theme changed
    ThemeChanged { window_id: u32, dark_mode: bool },
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
    Moved {
        window_id: u32,
        pointer_type: PointerType,
        pointer_id: u64,
        position: Vector2,
    },

    /// Pointer entered window area
    Entered {
        window_id: u32,
        pointer_type: PointerType,
        pointer_id: u64,
    },

    /// Pointer left window area
    Left {
        window_id: u32,
        pointer_type: PointerType,
        pointer_id: u64,
    },

    /// Pointer button pressed/released (mouse) or touch started/ended
    Button {
        window_id: u32,
        pointer_type: PointerType,
        pointer_id: u64,
        button: MouseButton,
        state: ElementState,
        position: Vector2,
    },

    /// Mouse wheel/touchpad scroll
    Scroll {
        window_id: u32,
        delta: ScrollDelta,
        phase: TouchPhase,
    },

    /// Touch event with pressure and additional info
    Touch {
        window_id: u32,
        pointer_id: u64,
        phase: TouchPhase,
        position: Vector2,
        pressure: Option<f32>,
    },

    /// Pinch gesture (zoom)
    PinchGesture {
        window_id: u32,
        delta: f64,
        phase: TouchPhase,
    },

    /// Pan gesture
    PanGesture {
        window_id: u32,
        delta: Vector2,
        phase: TouchPhase,
    },

    /// Rotation gesture
    RotationGesture {
        window_id: u32,
        delta: f32,
        phase: TouchPhase,
    },

    /// Double tap gesture
    DoubleTapGesture { window_id: u32 },
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
    Input {
        window_id: u32,
        key_code: KeyCode,
        state: ElementState,
        location: KeyLocation,
        repeat: bool,
        text: Option<String>,
        modifiers: ModifiersState,
    },

    /// Modifiers changed
    ModifiersChanged {
        window_id: u32,
        modifiers: ModifiersState,
    },

    /// IME composition started
    ImeEnabled { window_id: u32 },

    /// IME composition in progress
    ImePreedit {
        window_id: u32,
        text: String,
        cursor_range: Option<(usize, usize)>,
    },

    /// IME composition committed
    ImeCommit { window_id: u32, text: String },

    /// IME disabled
    ImeDisabled { window_id: u32 },
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
    Connected { gamepad_id: u32, name: String },

    /// Gamepad was disconnected
    Disconnected { gamepad_id: u32 },

    /// Gamepad button pressed/released
    Button {
        gamepad_id: u32,
        button: GamepadButton,
        state: ElementState,
        value: f32, // 0.0-1.0 for analog triggers
    },

    /// Gamepad axis moved
    Axis {
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
    Connected {
        joystick_id: u32,
        name: String,
        axes_count: u32,
        buttons_count: u32,
        hats_count: u32,
    },

    /// Joystick was disconnected
    Disconnected { joystick_id: u32 },

    /// Raw joystick button pressed/released
    Button {
        joystick_id: u32,
        button_index: u32,
        state: ElementState,
    },

    /// Raw joystick axis moved
    Axis {
        joystick_id: u32,
        axis_index: u32,
        value: f32, // -1.0 to 1.0
    },

    /// Raw joystick hat/POV moved
    Hat {
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
    Resumed,

    /// Application was suspended
    Suspended,

    /// Low memory warning
    MemoryWarning,

    /// Application is about to exit
    Exiting,
}
