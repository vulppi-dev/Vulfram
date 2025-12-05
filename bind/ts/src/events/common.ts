// MARK: Common Types

/** Represents the state of an input element (pressed or released) */
export enum ElementState {
  Released = 0,
  Pressed = 1,
}

/** Represents the phase of a touch/gesture event */
export enum TouchPhase {
  Started = 0,
  Moved = 1,
  Ended = 2,
  Cancelled = 3,
}

/** Represents keyboard modifier keys state */
export interface ModifiersState {
  shift: boolean;
  ctrl: boolean;
  alt: boolean;
  meta: boolean;
}

// MARK: Vector Types

/** 2D vector as [x, y] */
export type Vector2 = [number, number];

/** 2D integer vector as [x, y] */
export type IVector2 = [number, number];

/** 3D vector as [x, y, z] */
export type Vector3 = [number, number, number];

/** 3D integer vector as [x, y, z] */
export type IVector3 = [number, number, number];

/** 4D vector as [x, y, z, w] */
export type Vector4 = [number, number, number, number];

/** 4D integer vector as [x, y, z, w] */
export type IVector4 = [number, number, number, number];

/** Size as [width, height] */
export type Size = [number, number];

/** Color as [r, g, b, a] (0.0-1.0) */
export type Color = [number, number, number, number];

/** Rectangle as [x, y, width, height] */
export type Rect = [number, number, number, number];

/** Cube as [x, y, z, width, height, depth] */
export type Cube = [number, number, number, number, number, number];

/** 3x3 Matrix (9 elements, row-major) */
export type Matrix3x3 = [
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
];

/** 4x4 Matrix (16 elements, row-major) */
export type Matrix4x4 = [
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
];

/** Quaternion as [x, y, z, w] */
export type Quaternion = [number, number, number, number];
