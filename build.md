# Build Instructions

## FFI Build (para Bun/Node.js com dlopen)

```powershell
# Build DLL com C ABI
cargo build --release --features ffi

# A DLL estará em:
# target/release/vulfram_core.dll (Windows)
# target/release/libvulfram_core.so (Linux)
# target/release/libvulfram_core.dylib (macOS)
```

Copie a DLL para `binding/bun/assets/x64/win32/` (ou equivalente).

## N-API Build (para Node.js nativo)

```powershell
# Instalar dependências
npm install

# Build com N-API
cargo build --release --features napi --no-default-features

# Ou use o napi-rs CLI (recomendado)
npm run build:napi

# O módulo nativo estará em:
# target/release/vulfram_core.node
```

## Usando ambos

### Com FFI (Bun - já configurado):

```typescript
import { dlopen } from 'bun:ffi';
// Seu código atual em binding/bun/src/index.ts
```

### Com N-API (Node.js nativo):

```typescript
const vulframCore = require('./vulfram_core.node');

const result = vulframCore.engine_init();
```

## Diferenças

- **FFI**: Simples, funciona com Bun e Node.js, usa ponteiros
- **N-API**: Nativo, melhor performance, apenas Node.js, usa Buffers JavaScript
