use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::os::windows::thread;
use std::sync::Arc;
use std::thread::{ThreadId, current as current_thread};
use winit::{
    event_loop::EventLoop,
    window::{Window, WindowId},
};

#[derive(Debug)]
pub enum EngineAccessError {
    NotInitialized,
    WrongThread,
}

pub struct WindowState {
    pub id: WindowId,
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

pub struct EngineState {
    pub windows: HashMap<u32, WindowState>,

    pub event_loop: EventLoop<()>,
    pub wgpu: wgpu::Instance,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
}

impl EngineState {
    pub fn new() -> Self {
        let wgpu_descriptor = wgpu::InstanceDescriptor {
            backends: if cfg!(target_os = "ios") || cfg!(target_os = "macos") {
                wgpu::Backends::METAL | wgpu::Backends::VULKAN
            } else {
                wgpu::Backends::DX12 | wgpu::Backends::VULKAN
            },
            backend_options: wgpu::BackendOptions::default(),
            flags: wgpu::InstanceFlags::empty(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        };
        let wgpu_instance = wgpu::Instance::new(&wgpu_descriptor);

        Self {
            event_loop: EventLoop::new().unwrap(),
            windows: HashMap::new(),
            wgpu: wgpu_instance,
            device: None,
            queue: None,
        }
    }

    fn request_redraw(&self) {
        for window_state in self.windows.values() {
            window_state.window.request_redraw();
        }
    }

    // fn create_window(&mut self, args: &CmdWindowCreateArgs) {
    //     let attrs = WindowAttributes::default()
    //         .with_title(args.title.as_str())
    //         .with_resizable(args.resizable)
    //         .with_position(PhysicalPosition::new(args.pos[0], args.pos[1]))
    //         .with_inner_size(PhysicalSize::new(args.size[0], args.size[1]))
    //         .with_transparent(true);
    // }
}

// impl ApplicationHandler for EngineState {
//     fn resumed(&mut self, event_loop: &ActiveEventLoop) {
//         // let window_attributes =
//         //     WindowAttributes::default().with_title("Winit + WGPU — Loop externo");

//         // let window = event_loop
//         //     .create_window(window_attributes)
//         //     .expect("Falha ao criar janela");
//     }

//     fn window_event(
//         &mut self,
//         event_loop: &ActiveEventLoop,
//         window_id: WindowId,
//         event: WindowEvent,
//     ) {
//     }
// }

static ENGINE_INSTANCE: OnceCell<EngineState> = OnceCell::new();
static MAIN_THREAD_ID: OnceCell<ThreadId> = OnceCell::new();

/// Chamar apenas no main thread
pub fn engine_init(instance: EngineState) -> Result<(), EngineAccessError> {
    // guarda quem é o "main thread"
    MAIN_THREAD_ID
        .set(current_thread().id())
        .map_err(|_| EngineAccessError::NotInitialized)?; // já foi setado -> provavelmente double init

    ENGINE_INSTANCE
        .set(instance)
        .map_err(|_| EngineAccessError::NotInitialized)?;

    Ok(())
}

pub fn engine_dispose() {
    // opcional: se quiser limpar, use take()
    if let Some(engine) = ENGINE_INSTANCE.take() {
        drop(engine);
    }
}

/// Acesso imutável, só no main thread
pub fn with_engine<F, R>(f: F) -> Result<R, EngineAccessError>
where
    F: FnOnce(&EngineState) -> R,
{
    let main_id = MAIN_THREAD_ID
        .get()
        .ok_or(EngineAccessError::NotInitialized)?;
    let current_id = current_thread().id();

    if &current_id != main_id {
        return Err(EngineAccessError::WrongThread);
    }

    let engine = ENGINE_INSTANCE
        .get()
        .ok_or(EngineAccessError::NotInitialized)?;

    Ok(f(engine))
}

/// Acesso mutável, só no main thread
pub fn with_engine_mut<F, R>(f: F) -> Result<R, EngineAccessError>
where
    F: FnOnce(&mut EngineState) -> R,
{
    let main_id = MAIN_THREAD_ID
        .get()
        .ok_or(EngineAccessError::NotInitialized)?;
    let current_id = current_thread().id();

    if &current_id != main_id {
        return Err(EngineAccessError::WrongThread);
    }

    let engine = ENGINE_INSTANCE
        .get_mut()
        .ok_or(EngineAccessError::NotInitialized)?;

    Ok(f(engine))
}
