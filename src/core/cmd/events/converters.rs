use super::common::TouchPhase;

/// Convert winit TouchPhase to our TouchPhase
pub fn convert_touch_phase(phase: winit::event::TouchPhase) -> TouchPhase {
    match phase {
        winit::event::TouchPhase::Started => TouchPhase::Started,
        winit::event::TouchPhase::Moved => TouchPhase::Moved,
        winit::event::TouchPhase::Ended => TouchPhase::Ended,
        winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

/// Convert winit MouseButton to u32
pub fn convert_mouse_button(button: winit::event::MouseButton) -> u32 {
    match button {
        winit::event::MouseButton::Left => 0,
        winit::event::MouseButton::Right => 1,
        winit::event::MouseButton::Middle => 2,
        winit::event::MouseButton::Back => 3,
        winit::event::MouseButton::Forward => 4,
        winit::event::MouseButton::Other(id) => id as u32,
    }
}

/// Convert winit KeyLocation to u32
pub fn convert_key_location(location: winit::keyboard::KeyLocation) -> u32 {
    match location {
        winit::keyboard::KeyLocation::Standard => 0,
        winit::keyboard::KeyLocation::Left => 1,
        winit::keyboard::KeyLocation::Right => 2,
        winit::keyboard::KeyLocation::Numpad => 3,
    }
}

/// Convert gilrs Button to u32
pub fn convert_gilrs_button(button: gilrs::Button) -> u32 {
    use gilrs::Button;

    match button {
        Button::South => 0,
        Button::East => 1,
        Button::West => 2,
        Button::North => 3,
        Button::LeftTrigger => 4,
        Button::RightTrigger => 5,
        Button::LeftTrigger2 => 6,
        Button::RightTrigger2 => 7,
        Button::Select => 8,
        Button::Start => 9,
        Button::Mode => 10,
        Button::LeftThumb => 11,
        Button::RightThumb => 12,
        Button::DPadUp => 13,
        Button::DPadDown => 14,
        Button::DPadLeft => 15,
        Button::DPadRight => 16,
        _ => 255, // Unknown button
    }
}

/// Convert gilrs Axis to u32
pub fn convert_gilrs_axis(axis: gilrs::Axis) -> u32 {
    use gilrs::Axis;

    match axis {
        Axis::LeftStickX => 0,
        Axis::LeftStickY => 1,
        Axis::RightStickX => 2,
        Axis::RightStickY => 3,
        Axis::LeftZ => 4,
        Axis::RightZ => 5,
        _ => 255, // Unknown axis
    }
}

/// Convert winit PhysicalKey to u32
pub fn convert_key_code(physical_key: &winit::keyboard::PhysicalKey) -> u32 {
    use winit::keyboard::KeyCode as WKeyCode;
    use winit::keyboard::PhysicalKey;

    match physical_key {
        PhysicalKey::Code(code) => match code {
            // Writing System Keys (0-50)
            WKeyCode::Backquote => 0,
            WKeyCode::Backslash => 1,
            WKeyCode::BracketLeft => 2,
            WKeyCode::BracketRight => 3,
            WKeyCode::Comma => 4,
            WKeyCode::Digit0 => 5,
            WKeyCode::Digit1 => 6,
            WKeyCode::Digit2 => 7,
            WKeyCode::Digit3 => 8,
            WKeyCode::Digit4 => 9,
            WKeyCode::Digit5 => 10,
            WKeyCode::Digit6 => 11,
            WKeyCode::Digit7 => 12,
            WKeyCode::Digit8 => 13,
            WKeyCode::Digit9 => 14,
            WKeyCode::Equal => 15,
            WKeyCode::IntlBackslash => 16,
            WKeyCode::IntlRo => 17,
            WKeyCode::IntlYen => 18,
            WKeyCode::KeyA => 19,
            WKeyCode::KeyB => 20,
            WKeyCode::KeyC => 21,
            WKeyCode::KeyD => 22,
            WKeyCode::KeyE => 23,
            WKeyCode::KeyF => 24,
            WKeyCode::KeyG => 25,
            WKeyCode::KeyH => 26,
            WKeyCode::KeyI => 27,
            WKeyCode::KeyJ => 28,
            WKeyCode::KeyK => 29,
            WKeyCode::KeyL => 30,
            WKeyCode::KeyM => 31,
            WKeyCode::KeyN => 32,
            WKeyCode::KeyO => 33,
            WKeyCode::KeyP => 34,
            WKeyCode::KeyQ => 35,
            WKeyCode::KeyR => 36,
            WKeyCode::KeyS => 37,
            WKeyCode::KeyT => 38,
            WKeyCode::KeyU => 39,
            WKeyCode::KeyV => 40,
            WKeyCode::KeyW => 41,
            WKeyCode::KeyX => 42,
            WKeyCode::KeyY => 43,
            WKeyCode::KeyZ => 44,
            WKeyCode::Minus => 45,
            WKeyCode::Period => 46,
            WKeyCode::Quote => 47,
            WKeyCode::Semicolon => 48,
            WKeyCode::Slash => 49,

            // Functional Keys (50-63)
            WKeyCode::AltLeft => 50,
            WKeyCode::AltRight => 51,
            WKeyCode::Backspace => 52,
            WKeyCode::CapsLock => 53,
            WKeyCode::ContextMenu => 54,
            WKeyCode::ControlLeft => 55,
            WKeyCode::ControlRight => 56,
            WKeyCode::Enter => 57,
            WKeyCode::SuperLeft => 58,
            WKeyCode::SuperRight => 59,
            WKeyCode::ShiftLeft => 60,
            WKeyCode::ShiftRight => 61,
            WKeyCode::Space => 62,
            WKeyCode::Tab => 63,

            // Control Keys (64-70)
            WKeyCode::Delete => 64,
            WKeyCode::End => 65,
            WKeyCode::Help => 66,
            WKeyCode::Home => 67,
            WKeyCode::Insert => 68,
            WKeyCode::PageDown => 69,
            WKeyCode::PageUp => 70,

            // Arrow Keys (71-74)
            WKeyCode::ArrowDown => 71,
            WKeyCode::ArrowLeft => 72,
            WKeyCode::ArrowRight => 73,
            WKeyCode::ArrowUp => 74,

            // Numpad Keys (75-104)
            WKeyCode::NumLock => 75,
            WKeyCode::Numpad0 => 76,
            WKeyCode::Numpad1 => 77,
            WKeyCode::Numpad2 => 78,
            WKeyCode::Numpad3 => 79,
            WKeyCode::Numpad4 => 80,
            WKeyCode::Numpad5 => 81,
            WKeyCode::Numpad6 => 82,
            WKeyCode::Numpad7 => 83,
            WKeyCode::Numpad8 => 84,
            WKeyCode::Numpad9 => 85,
            WKeyCode::NumpadAdd => 86,
            WKeyCode::NumpadBackspace => 87,
            WKeyCode::NumpadClear => 88,
            WKeyCode::NumpadClearEntry => 89,
            WKeyCode::NumpadComma => 90,
            WKeyCode::NumpadDecimal => 91,
            WKeyCode::NumpadDivide => 92,
            WKeyCode::NumpadEnter => 93,
            WKeyCode::NumpadEqual => 94,
            WKeyCode::NumpadHash => 95,
            WKeyCode::NumpadMemoryAdd => 96,
            WKeyCode::NumpadMemoryClear => 97,
            WKeyCode::NumpadMemoryRecall => 98,
            WKeyCode::NumpadMemoryStore => 99,
            WKeyCode::NumpadMemorySubtract => 100,
            WKeyCode::NumpadMultiply => 101,
            WKeyCode::NumpadParenLeft => 102,
            WKeyCode::NumpadParenRight => 103,
            WKeyCode::NumpadStar => 104,
            WKeyCode::NumpadSubtract => 105,

            // Function Keys (106-129)
            WKeyCode::Escape => 106,
            WKeyCode::F1 => 107,
            WKeyCode::F2 => 108,
            WKeyCode::F3 => 109,
            WKeyCode::F4 => 110,
            WKeyCode::F5 => 111,
            WKeyCode::F6 => 112,
            WKeyCode::F7 => 113,
            WKeyCode::F8 => 114,
            WKeyCode::F9 => 115,
            WKeyCode::F10 => 116,
            WKeyCode::F11 => 117,
            WKeyCode::F12 => 118,
            WKeyCode::F13 => 119,
            WKeyCode::F14 => 120,
            WKeyCode::F15 => 121,
            WKeyCode::F16 => 122,
            WKeyCode::F17 => 123,
            WKeyCode::F18 => 124,
            WKeyCode::F19 => 125,
            WKeyCode::F20 => 126,
            WKeyCode::F21 => 127,
            WKeyCode::F22 => 128,
            WKeyCode::F23 => 129,
            WKeyCode::F24 => 130,

            // Lock Keys (131)
            WKeyCode::ScrollLock => 131,

            // Media Keys (132-138)
            WKeyCode::AudioVolumeDown => 132,
            WKeyCode::AudioVolumeMute => 133,
            WKeyCode::AudioVolumeUp => 134,
            WKeyCode::MediaPlayPause => 135,
            WKeyCode::MediaStop => 136,
            WKeyCode::MediaTrackNext => 137,
            WKeyCode::MediaTrackPrevious => 138,

            // Browser Keys (139-145)
            WKeyCode::BrowserBack => 139,
            WKeyCode::BrowserFavorites => 140,
            WKeyCode::BrowserForward => 141,
            WKeyCode::BrowserHome => 142,
            WKeyCode::BrowserRefresh => 143,
            WKeyCode::BrowserSearch => 144,
            WKeyCode::BrowserStop => 145,

            // System Keys (146-147)
            WKeyCode::PrintScreen => 146,
            WKeyCode::Pause => 147,

            _ => 255, // Unidentified
        },
        PhysicalKey::Unidentified(_) => 255,
    }
}
