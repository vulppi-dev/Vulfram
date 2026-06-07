# Realm2D Parity Roadmap (Incremental)

This roadmap closes the architecture parity gap between `realm2d` and `realm3d` using one commit per phase.

## Execution Rules

- One phase per commit.
- Each phase must end with objective validation.
- No phase is considered done without explicit `done criteria` satisfaction.

## Phase 0: Baseline and Contracts

Status: `completed` (commit `28bd02a`)

Goal:
- Freeze parity scope and acceptance contracts before feature work.

Scope:
- Create a parity matrix (`realm2d` vs `realm3d`) for: lighting, shadows, forward material layout, custom pass lifecycle, tests, docs.
- Define mandatory host/core contracts:
  - `realm_kind` exclusivity must be validated for all relevant resources.
  - Host logical IDs must reject core-reserved range.
  - Material/pass definition and instance lifecycle rules must be explicit.
- Classify all differences as `intentional` or `gap`.

Done criteria:
- Binary checklist exists and is reviewable.
- Intended differences and missing pieces are explicit and non-ambiguous.

Commit message:
- `docs(realm2d): define parity baseline and acceptance contracts`

Validation:
- Docs consistency review against existing runtime architecture docs.

## Phase 1: Realm2D Forward Material Layout Completion

Status: `completed`

Goal:
- Align 2D forward material data layout semantics with 3D mindset (`global + material + frame semantics`).

Scope:
- Expand 2D bind/layout contract where needed.
- Preserve existing fallback behavior.
- Enforce lifecycle validation on definition/instance operations.

Done criteria:
- 2D material flow follows complete contract shape.
- No regression in current 2D demos.
- Contract validation tests pass.

Commit message:
- `feat(realm2d): align forward material layout with realm3d semantics`

Validation:
- Targeted runtime tests + demo smoke.

## Phase 2: Realm2D Custom Pass Definition and Instance Lifecycle

Status: `completed`

Goal:
- Formalize `pass definition + pass instance` lifecycle for `realm2d` with material-like contracts.

Scope:
- Create/update/dispose flow for pass definitions and instances.
- Validate `realm_kind` compatibility and contract integrity.
- Add deterministic error reporting for invalid lifecycle operations.

Done criteria:
- End-to-end custom pass lifecycle works for `realm2d`.
- `realm_kind` exclusivity checks are enforced and tested.

Commit message:
- `feat(realm2d): formalize custom pass definition and instance lifecycle`

Validation:
- Targeted lifecycle tests + contract negative tests.

## Phase 3: Native Realm2D Lighting

Status: `completed`

Goal:
- Introduce a native 2D lighting pipeline with architecture equivalent to 3D where applicable.

Scope:
- 2D light model and runtime resources.
- Light culling/selection strategy.
- Light buffer integration into forward shading.

Done criteria:
- Multi-light 2D scenes render correctly.
- No-light fallback path remains stable.
- Baseline performance is measured and accepted.

Commit message:
- `feat(realm2d): add native 2d lighting pipeline with culling and light buffer`

Validation:
- Stress demo scene + targeted performance check.

## Phase 4: Native Realm2D Shadows

Status: `completed`

Goal:
- Add native 2D shadow pass with `cast/receive` semantics at pipeline level.

Scope:
- Implement shadow pass integration with 2D lighting.
- Add `cast_shadow` and `receive_shadow` controls.
- Keep predictable fallback behavior.

Done criteria:
- Shadows respond correctly to light/object movement.
- `cast` and `receive` controls are independently validated.
- No critical rendering regressions.

Commit message:
- `feat(realm2d): implement native 2d shadows with cast/receive pipeline flags`

Validation:
- Visual regression checks + targeted behavior tests.

## Phase 5: Dedicated Realm2D Parity Test Suite

Status: `completed`

Goal:
- Add explicit parity-focused tests for 2D.

Scope:
- Material definition + instance lifecycle tests.
- Fallback behavior tests.
- `realm_kind` exclusivity tests.
- Basic visual regression checks for demos.

Done criteria:
- Suite is stable and actionable in CI.
- Critical parity contracts have direct test coverage.

Commit message:
- `test(realm2d): add parity suite for lifecycle validation fallback and visuals`

Validation:
- CI pass + deterministic local rerun.

## Phase 6: Final Host Contract and Docs Consolidation

Status: `completed`

Goal:
- Publish final 2D flow contracts mirroring 3D doc quality.

Scope:
- Clarify preset vs custom behavior.
- Document lifecycle, limits, guarantees, and expected errors.
- Provide host-side integration examples.

Done criteria:
- Docs cover all delivered behavior from phases 1-5.
- Contract rules are implementation-aligned and validated.

Commit message:
- `docs(realm2d): finalize host contract and lifecycle documentation`

Validation:
- Cross-check with tests and demo behavior.

## Phase 7: Demo 004 (Lighting and Shadows Validation)

Status: `completed`

Goal:
- Create `demo 004` in `apps/demos` as a practical validation target for native 2D lights/shadows.

Scope:
- Add `004.demo.ts` focused on 2D lights, occluders, cast/receive combinations, and fallback toggles.
- Register demo id and hash routing.
- Ensure the demo can be used as a visual regression baseline for parity checks.

Done criteria:
- Demo 004 runs on Bun CLI and Web/WASM path.
- Scene clearly exercises all required light/shadow paths.
- Output is usable for regression snapshots.

Commit message:
- `feat(demos): add demo 004 for realm2d lighting and shadows validation`

Validation:
- `bun run --cwd apps/demos demo 004`
- `bun run demo:web` with `#demo-004`

## Binary Checklist for Parity Closure

- [x] Forward 2D uses complete material contract semantics.
- [x] Custom passes for 2D support formal definition + instance lifecycle.
- [x] Native 2D lighting exists (culling + light buffer + shading integration).
- [x] Native 2D shadows exist (`cast/receive` pipeline-level behavior).
- [x] Dedicated 2D parity test suite exists and is stable.
- [x] Final host contract docs for 2D are complete and implementation-aligned.
- [x] Demo 004 exists and validates 2D light/shadow behavior end-to-end.
