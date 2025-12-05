import type { ElementState } from './common';

// MARK: Gamepad Events

/** Gamepad button types following standard gamepad mapping */
export enum GamepadButton {
  // Face buttons
  South = 0, // A / Cross
  East = 1, // B / Circle
  West = 2, // X / Square
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
  // Other represented as number > 16
}

/** Gamepad axis types */
export enum GamepadAxis {
  LeftStickX = 0,
  LeftStickY = 1,
  RightStickX = 2,
  RightStickY = 3,
  LeftTrigger = 4,
  RightTrigger = 5,
  // Other represented as number > 5
}

/**
 * Event fired when a gamepad is connected to the system.
 *
 * @example
 * ```typescript
 * const event: GamepadConnectedEvent = {
 *   event: 'on-connect',
 *   data: {
 *     gamepadId: 0,
 *     name: 'Xbox Controller'
 *   }
 * };
 * ```
 */
export interface GamepadConnectedEvent {
  event: 'on-connect';
  data: {
    gamepadId: number;
    name: string;
  };
}

export interface GamepadDisconnectedEvent {
  event: 'on-disconnect';
  data: { gamepadId: number };
}

/**
 * Event fired when a gamepad button is pressed or released.
 *
 * @example
 * ```typescript
 * const event: GamepadButtonEvent = {
 *   event: 'on-button',
 *   data: {
 *     gamepadId: 0,
 *     button: 'south',
 *     state: 'pressed',
 *     value: 1.0
 *   }
 * };
 * ```
 */
export interface GamepadButtonEvent {
  event: 'on-button';
  data: {
    gamepadId: number;
    button: GamepadButton;
    state: ElementState;
    value: number; // 0.0-1.0 for analog triggers
  };
}

export interface GamepadAxisEvent {
  event: 'on-axis';
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
