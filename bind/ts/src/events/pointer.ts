import type { Vector2, ElementState, TouchPhase } from './common';

// MARK: Pointer Events

/** Mouse button types */
export enum MouseButton {
  Left = 0,
  Right = 1,
  Middle = 2,
  Back = 3,
  Forward = 4,
  // Other represented as number > 4
}

/** Pointer type for unified mouse/touch handling */
export enum PointerType {
  Mouse = 0,
  Touch = 1,
  Pen = 2,
}

/** Mouse scroll delta type */
export type ScrollDelta =
  | { type: 0; value: Vector2 } // Line
  | { type: 1; value: Vector2 }; // Pixel

/**
 * Event fired when the pointer (mouse, touch, or pen) moves.
 *
 * @example
 * ```typescript
 * const event: PointerMovedEvent = {
 *   event: 'on-move',
 *   data: {
 *     windowId: 1,
 *     pointerType: 'mouse',
 *     pointerId: 0,
 *     position: { x: 640, y: 360 }
 *   }
 * };
 * ```
 */
export interface PointerMovedEvent {
  event: 'on-move';
  data: {
    windowId: number;
    pointerType: PointerType;
    pointerId: number;
    position: Vector2;
  };
}

export interface PointerEnteredEvent {
  event: 'on-enter';
  data: {
    windowId: number;
    pointerType: PointerType;
    pointerId: number;
  };
}

export interface PointerLeftEvent {
  event: 'on-leave';
  data: {
    windowId: number;
    pointerType: PointerType;
    pointerId: number;
  };
}

/**
 * Event fired when a pointer button (mouse button) is pressed or released.
 *
 * @example
 * ```typescript
 * const event: PointerButtonEvent = {
 *   event: 'on-button',
 *   data: {
 *     windowId: 1,
 *     pointerType: 'mouse',
 *     pointerId: 0,
 *     button: 'left',
 *     state: 'pressed',
 *     position: { x: 100, y: 200 }
 *   }
 * };
 * ```
 */
export interface PointerButtonEvent {
  event: 'on-button';
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
  event: 'on-scroll';
  data: {
    windowId: number;
    delta: ScrollDelta;
    phase: TouchPhase;
  };
}

export interface PointerTouchEvent {
  event: 'on-touch';
  data: {
    windowId: number;
    pointerId: number;
    phase: TouchPhase;
    position: Vector2;
    pressure: number | null;
  };
}

export interface PointerPinchGestureEvent {
  event: 'on-pinch-gesture';
  data: {
    windowId: number;
    delta: number;
    phase: TouchPhase;
  };
}

export interface PointerPanGestureEvent {
  event: 'on-pan-gesture';
  data: {
    windowId: number;
    delta: Vector2;
    phase: TouchPhase;
  };
}

export interface PointerRotationGestureEvent {
  event: 'on-rotation-gesture';
  data: {
    windowId: number;
    delta: number;
    phase: TouchPhase;
  };
}

export interface PointerDoubleTapGestureEvent {
  event: 'on-double-tap-gesture';
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
