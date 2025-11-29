use serde::{Deserialize, Serialize};

use crate::core::{EngineResult, EngineState};

pub mod args;
pub mod events;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineCmd {
    CmdWindowCreate(args::CmdWindowCreateArgs),
}

/// Engine event types sent from native to JavaScript
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineEvent {
    // Window events
    Window(events::WindowEvent),

    // Pointer (Mouse/Touch) events
    Pointer(events::PointerEvent),

    // Keyboard events
    Keyboard(events::KeyboardEvent),

    // Gamepad events
    Gamepad(events::GamepadEvent),

    // Joystick events
    Joystick(events::JoystickEvent),

    // System events
    System(events::SystemEvent),
}

#[derive(Debug, Deserialize, Clone)]
pub struct EngineCmdEnvelope {
    pub id: u64,
    #[serde(flatten)]
    pub cmd: EngineCmd,
}

pub type EngineBatchCmds = Vec<EngineCmdEnvelope>;

pub type EngineBatchEvents = Vec<EngineEvent>;

pub fn engine_process_batch(engine: &mut EngineState, batch: EngineBatchCmds) -> EngineResult {
    EngineResult::Success
}
