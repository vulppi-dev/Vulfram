use std::sync::Arc;

use pollster::FutureExt;
use serde::{Deserialize, Serialize};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize, Position},
    event_loop::ActiveEventLoop,
    platform::windows::WindowExtWindows,
    window::{Fullscreen, Window, WindowAttributes},
};

use super::super::units::{IVector2, Size};
use crate::core::{EngineResult, EngineState, WindowState};

#[repr(u32)]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum EngineWindowState {
    Minimized = 0,
    Maximized,
    Windowed,
    Fullscreen,
    WindowedFullscreen,
}

impl Default for EngineWindowState {
    fn default() -> Self {
        EngineWindowState::Windowed
    }
}

fn window_size_default() -> Size {
    [800, 600]
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCreateArgs {
    pub title: String,
    #[serde(default = "window_size_default")]
    pub size: Size,
    pub position: IVector2,
    pub borderless: bool,
    pub resizable: bool,
    pub always_on_top: bool,
    pub initial_state: EngineWindowState,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowCreate {
    id: u32,
    success: bool,
}

pub fn engine_cmd_window_create(
    engine: &mut EngineState,
    event_loop: &ActiveEventLoop,
    args: &CmdWindowCreateArgs,
) -> EngineResult {
    let win_attrs = Window::default_attributes()
        .with_title(args.title.as_str())
        .with_decorations(!args.borderless)
        .with_resizable(args.resizable)
        .with_inner_size(PhysicalSize::new(args.size[0], args.size[1]))
        .with_position(Position::Physical(PhysicalPosition::new(
            args.position[0],
            args.position[1],
        )))
        .with_transparent(true);

    let window = match event_loop.create_window(win_attrs) {
        Ok(window) => Arc::new(window),
        Err(e) => {
            println!("Failed to create window: {}", e);
            return EngineResult::WinitCreateWindowError;
        }
    };

    let win_id = engine.window_id_counter;
    engine.window_id_counter += 1;
    engine.window_id_map.insert(window.id(), win_id);

    let surface = match engine.wgpu.create_surface(window.clone()) {
        Ok(surface) => surface,
        Err(e) => {
            println!("Failed to create surface: {}", e);
            return EngineResult::WinitCreateWindowError;
        }
    };

    if engine.device.is_none() {
        let adapter =
            match pollster::block_on(engine.wgpu.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })) {
                Ok(adapter) => adapter,
                Err(_) => return EngineResult::WgpuInstanceError,
            };

        let (device, queue) = match adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                ..Default::default()
            })
            .block_on()
        {
            Ok((device, queue)) => (device, queue),
            Err(e) => {
                println!("Failed to create device: {}", e);
                return EngineResult::WgpuInstanceError;
            }
        };

        engine.caps = Some(surface.get_capabilities(&adapter));
        engine.device = Some(device);
        engine.queue = Some(queue);
    }

    let caps = engine.caps.as_ref().unwrap();
    let format = caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        width: args.size[0],
        height: args.size[1],
        present_mode: if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::Fifo
        },
        format,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    engine.windows.insert(
        win_id,
        WindowState {
            window,
            surface,
            config,
        },
    );

    EngineResult::Success
}
