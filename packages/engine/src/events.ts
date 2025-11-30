// MARK: Common Types

/** Represents the state of an input element (pressed or released) */
export type ElementState = 'released' | 'pressed';

/** Represents the phase of a touch/gesture event */
export type TouchPhase = 'started' | 'moved' | 'ended' | 'cancelled';

/** Represents keyboard modifier keys state */
export interface ModifiersState {
  shift: boolean;
  ctrl: boolean;
  alt: boolean;
  meta: boolean;
}

/** 2D vector as [x, y] */
export type Vector2 = [number, number];

/** 2D integer vector as [x, y] */
export type IVector2 = [number, number];

// MARK: Window Events

export interface WindowCreatedEvent {
  event: 'created';
  data: { windowId: number };
}

export interface WindowResizedEvent {
  event: 'resized';
  data: { windowId: number; width: number; height: number };
}

export interface WindowMovedEvent {
  event: 'moved';
  data: { windowId: number; position: IVector2 };
}

export interface WindowCloseRequestedEvent {
  event: 'close-requested';
  data: { windowId: number };
}

export interface WindowDestroyedEvent {
  event: 'destroyed';
  data: { windowId: number };
}

export interface WindowFocusedEvent {
  event: 'focused';
  data: { windowId: number; focused: boolean };
}

export interface WindowScaleFactorChangedEvent {
  event: 'scale-factor-changed';
  data: {
    windowId: number;
    scaleFactor: number;
    newWidth: number;
    newHeight: number;
  };
}

export interface WindowOccludedEvent {
  event: 'occluded';
  data: { windowId: number; occluded: boolean };
}

export interface WindowRedrawRequestedEvent {
  event: 'redraw-requested';
  data: { windowId: number };
}

export interface WindowFileDroppedEvent {
  event: 'file-dropped';
  data: { windowId: number; path: string };
}

export interface WindowFileHoveredEvent {
  event: 'file-hovered';
  data: { windowId: number; path: string };
}

export interface WindowFileHoveredCancelledEvent {
  event: 'file-hovered-cancelled';
  data: { windowId: number };
}

export interface WindowThemeChangedEvent {
  event: 'theme-changed';
  data: { windowId: number; darkMode: boolean };
}

export type WindowEvent =
  | WindowCreatedEvent
  | WindowResizedEvent
  | WindowMovedEvent
  | WindowCloseRequestedEvent
  | WindowDestroyedEvent
  | WindowFocusedEvent
  | WindowScaleFactorChangedEvent
  | WindowOccludedEvent
  | WindowRedrawRequestedEvent
  | WindowFileDroppedEvent
  | WindowFileHoveredEvent
  | WindowFileHoveredCancelledEvent
  | WindowThemeChangedEvent;

// MARK: Pointer Events

/** Mouse button types */
export type MouseButton =
  | 'left'
  | 'right'
  | 'middle'
  | 'back'
  | 'forward'
  | { other: number };

/** Pointer type for unified mouse/touch handling */
export type PointerType = 'mouse' | 'touch' | 'pen';

/** Mouse scroll delta type */
export type ScrollDelta =
  | { type: 'line'; value: Vector2 }
  | { type: 'pixel'; value: Vector2 };

export interface PointerMovedEvent {
  event: 'moved';
  data: {
    windowId: number;
    pointerType: PointerType;
    pointerId: number;
    position: Vector2;
  };
}

export interface PointerEnteredEvent {
  event: 'entered';
  data: {
    windowId: number;
    pointerType: PointerType;
    pointerId: number;
  };
}

export interface PointerLeftEvent {
  event: 'left';
  data: {
    windowId: number;
    pointerType: PointerType;
    pointerId: number;
  };
}

export interface PointerButtonEvent {
  event: 'button';
  data: {
    windowId: number;
    pointerType: PointerType;
    pointerId: number;
    button: MouseButton;
    state: ElementState;
    position: Vector2;
  };
}

export interface PointerScrollEvent {
  event: 'scroll';
  data: {
    windowId: number;
    delta: ScrollDelta;
    phase: TouchPhase;
  };
}

export interface PointerTouchEvent {
  event: 'touch';
  data: {
    windowId: number;
    pointerId: number;
    phase: TouchPhase;
    position: Vector2;
    pressure: number | null;
  };
}

export interface PointerPinchGestureEvent {
  event: 'pinch-gesture';
  data: {
    windowId: number;
    delta: number;
    phase: TouchPhase;
  };
}

export interface PointerPanGestureEvent {
  event: 'pan-gesture';
  data: {
    windowId: number;
    delta: Vector2;
    phase: TouchPhase;
  };
}

export interface PointerRotationGestureEvent {
  event: 'rotation-gesture';
  data: {
    windowId: number;
    delta: number;
    phase: TouchPhase;
  };
}

export interface PointerDoubleTapGestureEvent {
  event: 'double-tap-gesture';
  data: { windowId: number };
}

export type PointerEvent =
  | PointerMovedEvent
  | PointerEnteredEvent
  | PointerLeftEvent
  | PointerButtonEvent
  | PointerScrollEvent
  | PointerTouchEvent
  | PointerPinchGestureEvent
  | PointerPanGestureEvent
  | PointerRotationGestureEvent
  | PointerDoubleTapGestureEvent;

// MARK: Keyboard Events

/** Key location on keyboard */
export type KeyLocation = 'standard' | 'left' | 'right' | 'numpad';

/** Physical key code (scancode-like, layout independent) */
export type KeyCode =
  // Writing System Keys
  | 'backquote'
  | 'backslash'
  | 'bracket-left'
  | 'bracket-right'
  | 'comma'
  | 'digit0'
  | 'digit1'
  | 'digit2'
  | 'digit3'
  | 'digit4'
  | 'digit5'
  | 'digit6'
  | 'digit7'
  | 'digit8'
  | 'digit9'
  | 'equal'
  | 'intl-backslash'
  | 'intl-ro'
  | 'intl-yen'
  | 'key-a'
  | 'key-b'
  | 'key-c'
  | 'key-d'
  | 'key-e'
  | 'key-f'
  | 'key-g'
  | 'key-h'
  | 'key-i'
  | 'key-j'
  | 'key-k'
  | 'key-l'
  | 'key-m'
  | 'key-n'
  | 'key-o'
  | 'key-p'
  | 'key-q'
  | 'key-r'
  | 'key-s'
  | 'key-t'
  | 'key-u'
  | 'key-v'
  | 'key-w'
  | 'key-x'
  | 'key-y'
  | 'key-z'
  | 'minus'
  | 'period'
  | 'quote'
  | 'semicolon'
  | 'slash'
  // Functional Keys
  | 'alt-left'
  | 'alt-right'
  | 'backspace'
  | 'caps-lock'
  | 'context-menu'
  | 'control-left'
  | 'control-right'
  | 'enter'
  | 'super-left'
  | 'super-right'
  | 'shift-left'
  | 'shift-right'
  | 'space'
  | 'tab'
  // Control Keys
  | 'delete'
  | 'end'
  | 'help'
  | 'home'
  | 'insert'
  | 'page-down'
  | 'page-up'
  // Arrow Keys
  | 'arrow-down'
  | 'arrow-left'
  | 'arrow-right'
  | 'arrow-up'
  // Numpad Keys
  | 'num-lock'
  | 'numpad0'
  | 'numpad1'
  | 'numpad2'
  | 'numpad3'
  | 'numpad4'
  | 'numpad5'
  | 'numpad6'
  | 'numpad7'
  | 'numpad8'
  | 'numpad9'
  | 'numpad-add'
  | 'numpad-backspace'
  | 'numpad-clear'
  | 'numpad-clear-entry'
  | 'numpad-comma'
  | 'numpad-decimal'
  | 'numpad-divide'
  | 'numpad-enter'
  | 'numpad-equal'
  | 'numpad-hash'
  | 'numpad-memory-add'
  | 'numpad-memory-clear'
  | 'numpad-memory-recall'
  | 'numpad-memory-store'
  | 'numpad-memory-subtract'
  | 'numpad-multiply'
  | 'numpad-paren-left'
  | 'numpad-paren-right'
  | 'numpad-star'
  | 'numpad-subtract'
  // Function Keys
  | 'escape'
  | 'f1'
  | 'f2'
  | 'f3'
  | 'f4'
  | 'f5'
  | 'f6'
  | 'f7'
  | 'f8'
  | 'f9'
  | 'f10'
  | 'f11'
  | 'f12'
  | 'f13'
  | 'f14'
  | 'f15'
  | 'f16'
  | 'f17'
  | 'f18'
  | 'f19'
  | 'f20'
  | 'f21'
  | 'f22'
  | 'f23'
  | 'f24'
  // Lock Keys
  | 'scroll-lock'
  // Media Keys
  | 'audio-volume-down'
  | 'audio-volume-mute'
  | 'audio-volume-up'
  | 'media-play-pause'
  | 'media-stop'
  | 'media-track-next'
  | 'media-track-previous'
  // Browser Keys
  | 'browser-back'
  | 'browser-favorites'
  | 'browser-forward'
  | 'browser-home'
  | 'browser-refresh'
  | 'browser-search'
  | 'browser-stop'
  // System Keys
  | 'print-screen'
  | 'pause'
  // Unknown
  | 'unidentified';

export interface KeyboardInputEvent {
  event: 'input';
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
  event: 'modifiers-changed';
  data: {
    windowId: number;
    modifiers: ModifiersState;
  };
}

export interface KeyboardImeEnabledEvent {
  event: 'ime-enabled';
  data: { windowId: number };
}

export interface KeyboardImePreeditEvent {
  event: 'ime-preedit';
  data: {
    windowId: number;
    text: string;
    cursorRange: [number, number] | null;
  };
}

export interface KeyboardImeCommitEvent {
  event: 'ime-commit';
  data: {
    windowId: number;
    text: string;
  };
}

export interface KeyboardImeDisabledEvent {
  event: 'ime-disabled';
  data: { windowId: number };
}

export type KeyboardEvent =
  | KeyboardInputEvent
  | KeyboardModifiersChangedEvent
  | KeyboardImeEnabledEvent
  | KeyboardImePreeditEvent
  | KeyboardImeCommitEvent
  | KeyboardImeDisabledEvent;

// MARK: Gamepad Events

/** Gamepad button types following standard gamepad mapping */
export type GamepadButton =
  // Face buttons
  | 'south' // A / Cross
  | 'east' // B / Circle
  | 'west' // X / Square
  | 'north' // Y / Triangle
  // Shoulder buttons
  | 'left-bumper'
  | 'right-bumper'
  | 'left-trigger'
  | 'right-trigger'
  // Center buttons
  | 'select'
  | 'start'
  | 'mode' // Guide / Home
  // Stick buttons
  | 'left-stick'
  | 'right-stick'
  // D-pad
  | 'dpad-up'
  | 'dpad-down'
  | 'dpad-left'
  | 'dpad-right'
  // Other
  | { other: number };

/** Gamepad axis types */
export type GamepadAxis =
  | 'left-stick-x'
  | 'left-stick-y'
  | 'right-stick-x'
  | 'right-stick-y'
  | 'left-trigger'
  | 'right-trigger'
  | { other: number };

export interface GamepadConnectedEvent {
  event: 'connected';
  data: {
    gamepadId: number;
    name: string;
  };
}

export interface GamepadDisconnectedEvent {
  event: 'disconnected';
  data: { gamepadId: number };
}

export interface GamepadButtonEvent {
  event: 'button';
  data: {
    gamepadId: number;
    button: GamepadButton;
    state: ElementState;
    value: number; // 0.0-1.0 for analog triggers
  };
}

export interface GamepadAxisEvent {
  event: 'axis';
  data: {
    gamepadId: number;
    axis: GamepadAxis;
    value: number; // -1.0 to 1.0 for sticks, 0.0 to 1.0 for triggers
  };
}

export type GamepadEvent =
  | GamepadConnectedEvent
  | GamepadDisconnectedEvent
  | GamepadButtonEvent
  | GamepadAxisEvent;

// MARK: Joystick Events

/** Joystick hat position */
export type JoystickHatPosition =
  | 'centered'
  | 'up'
  | 'right-up'
  | 'right'
  | 'right-down'
  | 'down'
  | 'left-down'
  | 'left'
  | 'left-up';

export interface JoystickConnectedEvent {
  event: 'connected';
  data: {
    joystickId: number;
    name: string;
    axesCount: number;
    buttonsCount: number;
    hatsCount: number;
  };
}

export interface JoystickDisconnectedEvent {
  event: 'disconnected';
  data: { joystickId: number };
}

export interface JoystickButtonEvent {
  event: 'button';
  data: {
    joystickId: number;
    buttonIndex: number;
    state: ElementState;
  };
}

export interface JoystickAxisEvent {
  event: 'axis';
  data: {
    joystickId: number;
    axisIndex: number;
    value: number; // -1.0 to 1.0
  };
}

export interface JoystickHatEvent {
  event: 'hat';
  data: {
    joystickId: number;
    hatIndex: number;
    position: JoystickHatPosition;
  };
}

export type JoystickEvent =
  | JoystickConnectedEvent
  | JoystickDisconnectedEvent
  | JoystickButtonEvent
  | JoystickAxisEvent
  | JoystickHatEvent;

// MARK: System Events

export interface SystemResumedEvent {
  event: 'resumed';
}

export interface SystemSuspendedEvent {
  event: 'suspended';
}

export interface SystemMemoryWarningEvent {
  event: 'memory-warning';
}

export interface SystemExitingEvent {
  event: 'exiting';
}

export type SystemEvent =
  | SystemResumedEvent
  | SystemSuspendedEvent
  | SystemMemoryWarningEvent
  | SystemExitingEvent;

// MARK: Engine Event (Union of all events)

export type EngineEvent =
  | { type: 'window'; content: WindowEvent }
  | { type: 'pointer'; content: PointerEvent }
  | { type: 'keyboard'; content: KeyboardEvent }
  | { type: 'gamepad'; content: GamepadEvent }
  | { type: 'joystick'; content: JoystickEvent }
  | { type: 'system'; content: SystemEvent };

/** Batch of engine events received from native */
export type EngineBatchEvents = EngineEvent[];

// MARK: Type Guards

export function isWindowEvent(
  event: EngineEvent,
): event is { type: 'window'; content: WindowEvent } {
  return event.type === 'window';
}

export function isPointerEvent(
  event: EngineEvent,
): event is { type: 'pointer'; content: PointerEvent } {
  return event.type === 'pointer';
}

export function isKeyboardEvent(
  event: EngineEvent,
): event is { type: 'keyboard'; content: KeyboardEvent } {
  return event.type === 'keyboard';
}

export function isGamepadEvent(
  event: EngineEvent,
): event is { type: 'gamepad'; content: GamepadEvent } {
  return event.type === 'gamepad';
}

export function isJoystickEvent(
  event: EngineEvent,
): event is { type: 'joystick'; content: JoystickEvent } {
  return event.type === 'joystick';
}

export function isSystemEvent(
  event: EngineEvent,
): event is { type: 'system'; content: SystemEvent } {
  return event.type === 'system';
}
