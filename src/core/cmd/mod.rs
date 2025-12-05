use serde::{Deserialize, Serialize};
use winit::event_loop::EventLoopProxy;

use crate::core::result::VulframResult;
use crate::core::singleton::EngineCustomEvents;
use crate::core::state::EngineState;

pub mod events;
pub mod win;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineCmd {
    CmdWindowCreate(win::CmdWindowCreateArgs),
    CmdWindowClose(win::CmdWindowCloseArgs),
    CmdWindowSetTitle(win::CmdWindowSetTitleArgs),
    CmdWindowSetPosition(win::CmdWindowSetPositionArgs),
    CmdWindowGetPosition(win::CmdWindowGetPositionArgs),
    CmdWindowSetSize(win::CmdWindowSetSizeArgs),
    CmdWindowGetSize(win::CmdWindowGetSizeArgs),
    CmdWindowGetOuterSize(win::CmdWindowGetOuterSizeArgs),
    CmdWindowGetSurfaceSize(win::CmdWindowGetSurfaceSizeArgs),
    CmdWindowSetState(win::CmdWindowSetStateArgs),
    CmdWindowGetState(win::CmdWindowGetStateArgs),
    CmdWindowSetIcon(win::CmdWindowSetIconArgs),
    CmdWindowSetDecorations(win::CmdWindowSetDecorationsArgs),
    CmdWindowHasDecorations(win::CmdWindowHasDecorationsArgs),
    CmdWindowSetResizable(win::CmdWindowSetResizableArgs),
    CmdWindowIsResizable(win::CmdWindowIsResizableArgs),
    CmdWindowRequestAttention(win::CmdWindowRequestAttentionArgs),
    CmdWindowFocus(win::CmdWindowFocusArgs),
    CmdWindowSetCursorVisible(win::CmdWindowSetCursorVisibleArgs),
    CmdWindowSetCursorGrab(win::CmdWindowSetCursorGrabArgs),
    CmdWindowSetCursorIcon(win::CmdWindowSetCursorIconArgs),
}

/// Spontaneous engine events (input, window changes, system events)
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum EngineEvent {
    Window(events::WindowEvent),
    Pointer(events::PointerEvent),
    Keyboard(events::KeyboardEvent),
    Gamepad(events::GamepadEvent),
    System(events::SystemEvent),
}

/// Command responses (answers to commands sent by user)
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "content", rename_all = "kebab-case")]
pub enum CommandResponse {
    WindowCreate(win::CmdResultWindowCreate),
    WindowClose(win::CmdResultWindowClose),
    WindowSetTitle(win::CmdResultWindowSetTitle),
    WindowSetPosition(win::CmdResultWindowSetPosition),
    WindowGetPosition(win::CmdResultWindowGetPosition),
    WindowSetSize(win::CmdResultWindowSetSize),
    WindowGetSize(win::CmdResultWindowGetSize),
    WindowGetOuterSize(win::CmdResultWindowGetOuterSize),
    WindowGetSurfaceSize(win::CmdResultWindowGetSurfaceSize),
    WindowSetState(win::CmdResultWindowSetState),
    WindowGetState(win::CmdResultWindowGetState),
    WindowSetIcon(win::CmdResultWindowSetIcon),
    WindowSetDecorations(win::CmdResultWindowSetDecorations),
    WindowHasDecorations(win::CmdResultWindowHasDecorations),
    WindowSetResizable(win::CmdResultWindowSetResizable),
    WindowIsResizable(win::CmdResultWindowIsResizable),
    WindowRequestAttention(win::CmdResultWindowRequestAttention),
    WindowFocus(win::CmdResultWindowFocus),
    WindowSetCursorVisible(win::CmdResultWindowSetCursorVisible),
    WindowSetCursorGrab(win::CmdResultWindowSetCursorGrab),
    WindowSetCursorIcon(win::CmdResultWindowSetCursorIcon),
}

#[derive(Debug, Deserialize, Clone)]
pub struct EngineCmdEnvelope {
    pub id: u64,
    #[serde(flatten)]
    pub cmd: EngineCmd,
}

#[derive(Debug, Serialize, Clone)]
pub struct CommandResponseEnvelope {
    pub id: u64,
    #[serde(flatten)]
    pub response: CommandResponse,
}

pub type EngineBatchCmds = Vec<EngineCmdEnvelope>;

pub type EngineBatchEvents = Vec<EngineEvent>;

pub type EngineBatchResponses = Vec<CommandResponseEnvelope>;

pub fn engine_process_batch(
    engine: &mut EngineState,
    loop_proxy: &mut EventLoopProxy<EngineCustomEvents>,
    batch: EngineBatchCmds,
) -> VulframResult {
    for pack in batch {
        match pack.cmd {
            EngineCmd::CmdWindowCreate(args) => {
                let _ =
                    loop_proxy.send_event(EngineCustomEvents::CreateWindow(pack.id, args.clone()));
            }
            EngineCmd::CmdWindowClose(args) => {
                let result = win::engine_cmd_window_close(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowClose(result),
                });
            }
            EngineCmd::CmdWindowSetTitle(args) => {
                let result = win::engine_cmd_window_set_title(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetTitle(result),
                });
            }
            EngineCmd::CmdWindowSetPosition(args) => {
                let result = win::engine_cmd_window_set_position(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetPosition(result),
                });
            }
            EngineCmd::CmdWindowGetPosition(args) => {
                let result = win::engine_cmd_window_get_position(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetPosition(result),
                });
            }
            EngineCmd::CmdWindowSetSize(args) => {
                let result = win::engine_cmd_window_set_size(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetSize(result),
                });
            }
            EngineCmd::CmdWindowGetSize(args) => {
                let result = win::engine_cmd_window_get_size(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetSize(result),
                });
            }
            EngineCmd::CmdWindowGetOuterSize(args) => {
                let result = win::engine_cmd_window_get_outer_size(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetOuterSize(result),
                });
            }
            EngineCmd::CmdWindowGetSurfaceSize(args) => {
                let result = win::engine_cmd_window_get_surface_size(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetSurfaceSize(result),
                });
            }
            EngineCmd::CmdWindowSetState(args) => {
                let result = win::engine_cmd_window_set_state(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetState(result),
                });
            }
            EngineCmd::CmdWindowGetState(args) => {
                let result = win::engine_cmd_window_get_state(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowGetState(result),
                });
            }
            EngineCmd::CmdWindowSetIcon(args) => {
                let result = win::engine_cmd_window_set_icon(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetIcon(result),
                });
            }
            EngineCmd::CmdWindowSetDecorations(args) => {
                let result = win::engine_cmd_window_set_decorations(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetDecorations(result),
                });
            }
            EngineCmd::CmdWindowHasDecorations(args) => {
                let result = win::engine_cmd_window_has_decorations(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowHasDecorations(result),
                });
            }
            EngineCmd::CmdWindowSetResizable(args) => {
                let result = win::engine_cmd_window_set_resizable(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetResizable(result),
                });
            }
            EngineCmd::CmdWindowIsResizable(args) => {
                let result = win::engine_cmd_window_is_resizable(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowIsResizable(result),
                });
            }
            EngineCmd::CmdWindowRequestAttention(args) => {
                let result = win::engine_cmd_window_request_attention(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowRequestAttention(result),
                });
            }
            EngineCmd::CmdWindowFocus(args) => {
                let result = win::engine_cmd_window_focus(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowFocus(result),
                });
            }
            EngineCmd::CmdWindowSetCursorVisible(args) => {
                let result = win::engine_cmd_window_set_cursor_visible(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetCursorVisible(result),
                });
            }
            EngineCmd::CmdWindowSetCursorGrab(args) => {
                let result = win::engine_cmd_window_set_cursor_grab(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetCursorGrab(result),
                });
            }
            EngineCmd::CmdWindowSetCursorIcon(args) => {
                let result = win::engine_cmd_window_set_cursor_icon(engine, &args);
                engine.response_queue.push(CommandResponseEnvelope {
                    id: pack.id,
                    response: CommandResponse::WindowSetCursorIcon(result),
                });
            }
        }
    }

    VulframResult::Success
}
