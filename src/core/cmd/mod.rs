use serde::{Deserialize, Serialize};
use winit::event_loop::ActiveEventLoop;

use crate::core::{EngineResult, EngineState};

pub mod events;
pub mod win;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineCmd {
    CmdWindowCreate(win::CmdWindowCreateArgs),
}

/// Engine event types sent from native to JavaScript
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineEvent {
    Window(events::WindowEvent),
    Pointer(events::PointerEvent),
    Keyboard(events::KeyboardEvent),
    Gamepad(events::GamepadEvent),
    Joystick(events::JoystickEvent),
    System(events::SystemEvent),
    // MARK: Command answers
    WindowCreate(win::CmdResultWindowCreate),
}

#[derive(Debug, Deserialize, Clone)]
pub struct EngineCmdEnvelope {
    pub id: u64,
    #[serde(flatten)]
    pub cmd: EngineCmd,
}

#[derive(Debug, Serialize, Clone)]
pub struct EngineEventEnvelope {
    pub id: u64,
    #[serde(flatten)]
    pub event: EngineEvent,
}

pub type EngineBatchCmds = Vec<EngineCmdEnvelope>;

pub type EngineBatchEvents = Vec<EngineEventEnvelope>;

pub fn engine_process_batch(
    engine: &mut EngineState,
    event_loop: &ActiveEventLoop,
    batch: EngineBatchCmds,
) -> EngineResult {
    EngineResult::Success
}
