# Realm2D Host Contract

This document defines the host-facing contract for `realm2d` in the current runtime.

## 1. Scope

`realm2d` supports:

- cameras (`camera2d`)
- draw items (`sprite2d`, `shape2d`)
- shader materials through material definition + material instance lifecycle
- native 2D forward rendering with:
  - complete material bind/layout semantics
  - native 2D lighting (camera-visible selection + light buffer + per-light shading)
  - native 2D shadow integration through per-light shadow mask array sampling (`cast_shadow`/`receive_shadow`)

## 2. Preset vs Custom

Builtin material definitions:

- `standard` (`realm_kind=ThreeD`)
- `pbr` (`realm_kind=ThreeD`)
- `standard-2d` (`realm_kind=TwoD`)

Rules:

- `pbr` cannot be used for `realm_kind=TwoD`.
- `standard-2d` is the builtin preset baseline for 2D.
- custom materials in 2D still use the same lifecycle as 3D:
  1. material definition upsert
  2. material instance upsert
  3. resource assignment to draw items

## 3. Lifecycle Contracts

### 3.1 Material Definition

- Definition IDs are host-controlled logical IDs and must pass host ID policy validation.
- Builtin definitions are immutable and cannot be disposed.
- Definition updates must keep contract validity:
  - valid create/update mode
  - realm compatibility
  - compileable logical shader source

### 3.2 Material Instance

- Instance creation references definition by `slug`.
- Instance realm is derived from the definition realm.
- Broken definitions cannot be instanced.

### 3.3 Fallback Behavior

When a custom definition is disposed:

- dependent instances are preserved
- instances are rebound to realm fallback definition
- for `realm2d`, fallback is `standard-2d`
- fallback application emits `MaterialInstanceFallbackApplied`

## 4. Realm Exclusivity Guarantees

The core enforces realm compatibility:

- `realm_kind` mismatch between requested material and definition is rejected.
- definitions incompatible with requested realm are rejected.
- pass-definition/pass-instance binding for `realm2d` rejects incompatible graph registry/pass sets when the graph contract requires realm-specific resources.

## 5. 2D Lighting and Shadows Contract

### 5.1 Lighting

`realm2d` lighting path includes:

- visible-light selection per camera
- light data upload into a dedicated 2D light buffer
- per-light shading contribution in 2D material shading

### 5.2 Shadows

`sprite2d` and `shape2d` support:

- `cast_shadow`
- `receive_shadow`

Runtime behavior:

- non-casters do not occlude
- non-receivers are not darkened by shadow factor
- shadowing is applied in the 2D forward shading path using per-light mask layers generated in the 2D shadow pass

Realm-level shadow tuning command (`cmd-realm2d-shadow-config-update`) supports:

- `lightRadius`
- `shadowContactOffset`
- `ambient`
- `angularResolution`
- `mapResolution` (clamped to `64..=2048`)

Current limits:

- shadow mask generation runs in the 2D shadow GPU pass
- runtime quality/performance depends primarily on angular resolution and visible occluder complexity
- cone opening is derived from `lightRadius`

## 6. IDs and Reserved Range

Host logical IDs must never use core-reserved IDs.

For `u32`, reserved range is:

- `4294901761..=4294967295`

Commands that violate this range are rejected by core validation.

## 7. Expected Failure Cases

Common explicit failures:

- `Material definition slug '<slug>' not found`
- `Material realm kind mismatch`
- `PBR preset is only supported for ThreeD realm`
- builtin material definition mutation/dispose rejection
- realm-pass instance binding rejection for incompatible `realm_kind`

## 8. Operational Guarantees

- Definition and instance operations are deterministic and validated.
- Fallback rebinding preserves instance continuity.
- `realm2d` and `realm3d` keep parity on material and lifecycle contracts, with realm-specific compatibility enforcement.
