import type { ElementState, ModifiersState } from './common';

// MARK: Keyboard Events

/** Key location on keyboard */
export enum KeyLocation {
  Standard = 0,
  Left = 1,
  Right = 2,
  Numpad = 3,
}

/** Physical key code (scancode-like, layout independent) */
export enum KeyCode {
  // Writing System Keys
  Backquote = 0,
  Backslash = 1,
  BracketLeft = 2,
  BracketRight = 3,
  Comma = 4,
  Digit0 = 5,
  Digit1 = 6,
  Digit2 = 7,
  Digit3 = 8,
  Digit4 = 9,
  Digit5 = 10,
  Digit6 = 11,
  Digit7 = 12,
  Digit8 = 13,
  Digit9 = 14,
  Equal = 15,
  IntlBackslash = 16,
  IntlRo = 17,
  IntlYen = 18,
  KeyA = 19,
  KeyB = 20,
  KeyC = 21,
  KeyD = 22,
  KeyE = 23,
  KeyF = 24,
  KeyG = 25,
  KeyH = 26,
  KeyI = 27,
  KeyJ = 28,
  KeyK = 29,
  KeyL = 30,
  KeyM = 31,
  KeyN = 32,
  KeyO = 33,
  KeyP = 34,
  KeyQ = 35,
  KeyR = 36,
  KeyS = 37,
  KeyT = 38,
  KeyU = 39,
  KeyV = 40,
  KeyW = 41,
  KeyX = 42,
  KeyY = 43,
  KeyZ = 44,
  Minus = 45,
  Period = 46,
  Quote = 47,
  Semicolon = 48,
  Slash = 49,
  // Functional Keys
  AltLeft = 50,
  AltRight = 51,
  Backspace = 52,
  CapsLock = 53,
  ContextMenu = 54,
  ControlLeft = 55,
  ControlRight = 56,
  Enter = 57,
  SuperLeft = 58,
  SuperRight = 59,
  ShiftLeft = 60,
  ShiftRight = 61,
  Space = 62,
  Tab = 63,
  // Control Keys
  Delete = 64,
  End = 65,
  Help = 66,
  Home = 67,
  Insert = 68,
  PageDown = 69,
  PageUp = 70,
  // Arrow Keys
  ArrowDown = 71,
  ArrowLeft = 72,
  ArrowRight = 73,
  ArrowUp = 74,
  // Numpad Keys
  NumLock = 75,
  Numpad0 = 76,
  Numpad1 = 77,
  Numpad2 = 78,
  Numpad3 = 79,
  Numpad4 = 80,
  Numpad5 = 81,
  Numpad6 = 82,
  Numpad7 = 83,
  Numpad8 = 84,
  Numpad9 = 85,
  NumpadAdd = 86,
  NumpadBackspace = 87,
  NumpadClear = 88,
  NumpadClearEntry = 89,
  NumpadComma = 90,
  NumpadDecimal = 91,
  NumpadDivide = 92,
  NumpadEnter = 93,
  NumpadEqual = 94,
  NumpadHash = 95,
  NumpadMemoryAdd = 96,
  NumpadMemoryClear = 97,
  NumpadMemoryRecall = 98,
  NumpadMemoryStore = 99,
  NumpadMemorySubtract = 100,
  NumpadMultiply = 101,
  NumpadParenLeft = 102,
  NumpadParenRight = 103,
  NumpadStar = 104,
  NumpadSubtract = 105,
  // Function Keys
  Escape = 106,
  F1 = 107,
  F2 = 108,
  F3 = 109,
  F4 = 110,
  F5 = 111,
  F6 = 112,
  F7 = 113,
  F8 = 114,
  F9 = 115,
  F10 = 116,
  F11 = 117,
  F12 = 118,
  F13 = 119,
  F14 = 120,
  F15 = 121,
  F16 = 122,
  F17 = 123,
  F18 = 124,
  F19 = 125,
  F20 = 126,
  F21 = 127,
  F22 = 128,
  F23 = 129,
  F24 = 130,
  // Lock Keys
  ScrollLock = 131,
  // Media Keys
  AudioVolumeDown = 132,
  AudioVolumeMute = 133,
  AudioVolumeUp = 134,
  MediaPlayPause = 135,
  MediaStop = 136,
  MediaTrackNext = 137,
  MediaTrackPrevious = 138,
  // Browser Keys
  BrowserBack = 139,
  BrowserFavorites = 140,
  BrowserForward = 141,
  BrowserHome = 142,
  BrowserRefresh = 143,
  BrowserSearch = 144,
  BrowserStop = 145,
  // System Keys
  PrintScreen = 146,
  Pause = 147,
  // Unknown
  Unidentified = 148,
}

/**
 * Event fired when a keyboard key is pressed or released.
 *
 * @example
 * ```typescript
 * const event: KeyboardInputEvent = {
 *   event: 'on-input',
 *   data: {
 *     windowId: 1,
 *     keyCode: 'key-w',
 *     state: 'pressed',
 *     location: 'standard',
 *     repeat: false,
 *     text: 'w',
 *     modifiers: { shift: false, ctrl: false, alt: false, meta: false }
 *   }
 * };
 * ```
 */
export interface KeyboardInputEvent {
  event: 'on-input';
  data: {
    windowId: number;
    keyCode: KeyCode;
    state: ElementState;
    location: KeyLocation;
    repeat: boolean;
    text: string | null;
    modifiers: ModifiersState;
  };
}

export interface KeyboardModifiersChangedEvent {
  event: 'on-modifiers-change';
  data: {
    windowId: number;
    modifiers: ModifiersState;
  };
}

export interface KeyboardImeEnabledEvent {
  event: 'on-ime-enable';
  data: { windowId: number };
}

export interface KeyboardImePreeditEvent {
  event: 'on-ime-preedit';
  data: {
    windowId: number;
    text: string;
    cursorRange: [number, number] | null;
  };
}

export interface KeyboardImeCommitEvent {
  event: 'on-ime-commit';
  data: {
    windowId: number;
    text: string;
  };
}

export interface KeyboardImeDisabledEvent {
  event: 'on-ime-disable';
  data: { windowId: number };
}

export type KeyboardEvent =
  | KeyboardInputEvent
  | KeyboardModifiersChangedEvent
  | KeyboardImeEnabledEvent
  | KeyboardImePreeditEvent
  | KeyboardImeCommitEvent
  | KeyboardImeDisabledEvent;
