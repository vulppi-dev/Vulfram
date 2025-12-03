use std::collections::HashMap;
use std::sync::Arc;
use winit::window::{Window, WindowId};

use crate::core::buffers::Buffer;
use crate::core::cmd::EngineBatchEvents;
use crate::core::cmd::events::ModifiersState;
use crate::core::units::{IVector2, Size, Vector2};

/// Represents a window with its associated WGPU resources
pub struct WindowState {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,

    // Window state tracking
    pub inner_position: IVector2,
    pub outer_position: IVector2,
    pub inner_size: Size,
    pub outer_size: Size,
}

/// Main engine state holding all runtime data
pub struct EngineState {
    // Window management
    pub windows: HashMap<u32, WindowState>,
    pub window_id_map: HashMap<WindowId, u32>,
    pub window_id_counter: u32,

    // WGPU resources
    pub wgpu: wgpu::Instance,
    pub caps: Option<wgpu::SurfaceCapabilities>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,

    // Buffer management
    pub buffers: HashMap<u64, Buffer>,

    // Event system
    pub event_queue: EngineBatchEvents,

    // Time tracking
    pub(crate) time: u64,
    pub(crate) delta_time: u32,

    // Input state
    pub(crate) modifiers_state: ModifiersState,
    pub(crate) cursor_positions: HashMap<u32, Vector2>,
    pub(crate) gilrs: Option<gilrs::Gilrs>,
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

        // Initialize gilrs for gamepad support
        let gilrs = match gilrs::Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(e) => {
                log::warn!("Failed to initialize gamepad support: {:?}", e);
                None
            }
        };

        Self {
            windows: HashMap::new(),
            window_id_map: HashMap::new(),
            buffers: HashMap::new(),
            event_queue: Vec::new(),

            window_id_counter: 0,

            wgpu: wgpu_instance,
            caps: None,
            device: None,
            queue: None,
            time: 0,
            delta_time: 0,

            modifiers_state: ModifiersState::default(),
            cursor_positions: HashMap::new(),
            gilrs,
        }
    }

    pub fn request_redraw(&self) {
        for window_state in self.windows.values() {
            window_state.window.request_redraw();
        }
    }
}
