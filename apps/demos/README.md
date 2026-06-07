# Galfus TypeScript Demos

This package exposes four canonical demos:

- `001` (FrameGraph baseline parity)
- `002` (Optical Persistence style 3D motion/material showcase)
- `003` (Realm2D sprites/shapes/materials showcase)
- `004` (Realm2D lighting/shadows showcase with moving lights and occluders)

## Local build prerequisites

Build local core bindings before running demos:

```bash
bun run build:local
```

Main script: `scripts/build-local-bindings.ts`

Supported flags:

- `--mode debug|release`
- `--skip-bun`
- `--skip-napi`
- `--skip-wasm`

Expected outputs:

- `packages/transport-bun/dist/<platform>/...`
- `packages/transport-napi/dist/<platform>/...`
- `packages/transport-browser/dist/galfus_core*.wasm/js`

Operational prerequisites:

- Rust toolchain installed
- `wasm32-unknown-unknown` target
- `wasm-bindgen-cli` (auto-installed by script when missing)

Validation script for repo checks:

```bash
bun run check
```

## Run Demos (CLI Bun)

```bash
bun run --cwd apps/demos demo 001
bun run --cwd apps/demos demo 002
bun run --cwd apps/demos demo 003
bun run --cwd apps/demos demo 004
```

## Run Demos (Web/WASM)

```bash
bun run demo:web
```

Open with hash:

- `#demo-001`
- `#demo-002`
- `#demo-003`
- `#demo-004`
