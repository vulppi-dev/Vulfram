# Galfus Documentation Index

## Start Here

- high-level overview: [OVERVIEW.md](OVERVIEW.md)
- public ABI contract: [ABI.md](ABI.md)
- current runtime architecture: [ARCH.md](ARCH.md)
- internal API snapshot: [API.md](API.md)
- realm architecture: [REALM-ARCH.md](REALM-ARCH.md)
- realm2d parity roadmap: [REALM2D-PARITY-ROADMAP.md](REALM2D-PARITY-ROADMAP.md)
- realm2d host contract: [REALM2D-HOST-CONTRACT.md](REALM2D-HOST-CONTRACT.md)
- render graph model: [RENDER-GRAPH.md](RENDER-GRAPH.md)
- custom material shaders: [SHADERS-MATERIALS.md](SHADERS-MATERIALS.md)
- custom pass shaders: [SHADERS-PASSES.md](SHADERS-PASSES.md)
- shader helpers glossary: [SHADERS-GLOSSARY.md](SHADERS-GLOSSARY.md)
- terminology: [GLOSSARY.md](GLOSSARY.md)
- command reference: [cmds/](cmds/)

## For Binding Authors and Integrators

Read in this order:

1. [OVERVIEW.md](OVERVIEW.md)
2. [ABI.md](ABI.md)
3. [REALM-ARCH.md](REALM-ARCH.md)
4. [RENDER-GRAPH.md](RENDER-GRAPH.md)
5. [cmds/](cmds/)

Current baseline rules:

- host composition uses `Realm`, `Target`, `TargetLayer`
- targets are only `window` and `texture`
- pointer flow is global (no target-routed relay)
- materials are unified as `ShaderMaterial` with composer-generated WGSL

## For Core Contributors

Read in this order:

1. [ARCH.md](ARCH.md)
2. [API.md](API.md)
3. [REALM-ARCH.md](REALM-ARCH.md)
4. [RENDER-GRAPH.md](RENDER-GRAPH.md)
5. [GLOSSARY.md](GLOSSARY.md)

## Release / Validation Supporting Docs

- [CI-RELEASE-PUBLISH.md](CI-RELEASE-PUBLISH.md)
- [OIDC-PUBLISH-SETUP.md](OIDC-PUBLISH-SETUP.md)
