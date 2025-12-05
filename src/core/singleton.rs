use once_cell::sync::OnceCell;
use std::cell::RefCell;
use std::thread::{self, ThreadId};
use winit::event_loop::{EventLoop, EventLoopProxy};

use super::cmd::win::CmdWindowCreateArgs;
use super::result::VulframResult;
use super::state::EngineState;

/// Custom events sent through the event loop
pub enum EngineCustomEvents {
    CreateWindow(u64, CmdWindowCreateArgs),
}

/// Singleton container for engine state and event loop
pub struct EngineSingleton {
    pub state: EngineState,
    pub event_loop: Option<EventLoop<EngineCustomEvents>>,
    pub proxy: Option<EventLoopProxy<EngineCustomEvents>>,
}

thread_local! {
    pub(crate) static ENGINE_INSTANCE: RefCell<Option<EngineSingleton>> = RefCell::new(None);
}

pub(crate) static MAIN_THREAD_ID: OnceCell<ThreadId> = OnceCell::new();

/// Validates that the current thread is the main engine thread
pub fn validate_thread() -> Result<(), VulframResult> {
    let current_id = thread::current().id();
    let main_id = MAIN_THREAD_ID.get().ok_or(VulframResult::NotInitialized)?;

    if &current_id != main_id {
        return Err(VulframResult::WrongThread);
    }

    Ok(())
}

/// Execute a closure with mutable access to the engine state
pub fn with_engine<F, R>(f: F) -> Result<R, VulframResult>
where
    F: FnOnce(&mut EngineState) -> R,
{
    validate_thread()?;

    ENGINE_INSTANCE.with(|cell| {
        let mut opt = cell.borrow_mut();
        let engine_state = opt.as_mut().ok_or(VulframResult::NotInitialized)?;
        Ok(f(&mut engine_state.state))
    })
}

/// Execute a closure with mutable access to the entire engine singleton
pub fn with_engine_singleton<F, R>(f: F) -> Result<R, VulframResult>
where
    F: FnOnce(&mut EngineSingleton) -> R,
{
    validate_thread()?;

    ENGINE_INSTANCE.with(|cell| {
        let mut opt = cell.borrow_mut();
        let engine_state = opt.as_mut().ok_or(VulframResult::NotInitialized)?;
        Ok(f(engine_state))
    })
}
