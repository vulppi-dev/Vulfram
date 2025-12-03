use std::sync::Arc;

use pollster::FutureExt;
use serde::{Deserialize, Serialize};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize, Position},
    event_loop::ActiveEventLoop,
    window::Window,
};

use crate::core::state::{EngineState, WindowState};
use crate::core::units::{IVector2, Size};

use super::{EngineWindowState, window_size_default};

// MARK: - Create Window

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCreateArgs {
    pub title: String,
    #[serde(default = "window_size_default")]
    pub size: Size,
    pub position: IVector2,
    pub borderless: bool,
    pub resizable: bool,
    pub initial_state: EngineWindowState,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowCreate {
    success: bool,
    message: String,
    content: u32,
}

pub fn engine_cmd_window_create(
    engine: &mut EngineState,
    event_loop: &ActiveEventLoop,
    args: &CmdWindowCreateArgs,
) -> CmdResultWindowCreate {
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
            return CmdResultWindowCreate {
                success: false,
                message: format!("Winit create window error: {}", e),
                content: 0,
            };
        }
    };

    let win_id = engine.window_id_counter;
    engine.window_id_counter += 1;
    engine.window_id_map.insert(window.id(), win_id);

    let surface = match engine.wgpu.create_surface(window.clone()) {
        Ok(surface) => surface,
        Err(e) => {
            return CmdResultWindowCreate {
                success: false,
                message: format!("WGPU create surface error: {}", e),
                content: 0,
            };
        }
    };

    // Get or create adapter and device
    let (adapter, is_new_device) = if engine.device.is_none() {
        // First window - create new adapter and device
        let adapter =
            match pollster::block_on(engine.wgpu.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })) {
                Ok(adapter) => adapter,
                Err(_) => {
                    return CmdResultWindowCreate {
                        success: false,
                        message: "WGPU adapter request error".to_string(),
                        content: 0,
                    };
                }
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
                return CmdResultWindowCreate {
                    success: false,
                    message: format!("WGPU device request error: {}", e),
                    content: 0,
                };
            }
        };

        engine.caps = Some(surface.get_capabilities(&adapter));
        engine.device = Some(device);
        engine.queue = Some(queue);
        (adapter, true)
    } else {
        // Subsequent windows - validate surface compatibility with existing adapter
        let adapter = match pollster::block_on(engine.wgpu.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        )) {
            Ok(adapter) => adapter,
            Err(_) => {
                return CmdResultWindowCreate {
                        success: false,
                        message: "Surface is not compatible with existing WGPU adapter. Cannot create window.".to_string(),
                        content: 0,
                    };
            }
        };
        (adapter, false)
    };

    // Get surface capabilities
    let caps = if is_new_device {
        engine.caps.as_ref().unwrap()
    } else {
        // For subsequent windows, get fresh capabilities and store them
        let new_caps = surface.get_capabilities(&adapter);
        engine.caps = Some(new_caps);
        engine.caps.as_ref().unwrap()
    };

    let format = caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(caps.formats[0]);

    // Select alpha mode with transparency support (PreMultiplied or PostMultiplied)
    // Fallback to Opaque if transparency is not supported
    let alpha_mode = caps
        .alpha_modes
        .iter()
        .copied()
        .find(|mode| {
            matches!(
                mode,
                wgpu::CompositeAlphaMode::PreMultiplied | wgpu::CompositeAlphaMode::PostMultiplied
            )
        })
        .unwrap_or(wgpu::CompositeAlphaMode::Opaque);

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
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    // Configure the surface with the device
    surface.configure(engine.device.as_ref().unwrap(), &config);

    // Get initial window positions and sizes
    let inner_position = window.inner_position().unwrap_or_default();
    let outer_position = window.outer_position().unwrap_or_default();
    let inner_size = window.inner_size();
    let outer_size = window.outer_size();

    engine.windows.insert(
        win_id,
        WindowState {
            window,
            surface,
            config,
            inner_position: [inner_position.x, inner_position.y],
            outer_position: [outer_position.x, outer_position.y],
            inner_size: [inner_size.width, inner_size.height],
            outer_size: [outer_size.width, outer_size.height],
        },
    );

    CmdResultWindowCreate {
        success: true,
        message: "Window created successfully".to_string(),
        content: win_id,
    }
}

// MARK: - Close Window

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdWindowCloseArgs {
    pub window_id: u32,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultWindowClose {
    success: bool,
    message: String,
}

pub fn engine_cmd_window_close(
    engine: &mut EngineState,
    args: &CmdWindowCloseArgs,
) -> CmdResultWindowClose {
    // Check if window exists
    if !engine.windows.contains_key(&args.window_id) {
        return CmdResultWindowClose {
            success: false,
            message: format!("Window with id {} not found", args.window_id),
        };
    }

    // Remove window from state
    if let Some(window_state) = engine.windows.remove(&args.window_id) {
        // Remove from window_id_map
        engine.window_id_map.remove(&window_state.window.id());

        // Window and surface will be dropped automatically
        CmdResultWindowClose {
            success: true,
            message: "Window closed successfully".to_string(),
        }
    } else {
        CmdResultWindowClose {
            success: false,
            message: "Failed to close window".to_string(),
        }
    }
}
