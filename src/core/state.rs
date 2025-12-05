use glam::{IVec2, UVec2, Vec2};
use std::collections::HashMap;
use std::sync::Arc;
use winit::window::{Window, WindowId};

use crate::core::buffers::Buffer;
use crate::core::cache::{GamepadCacheManager, InputCacheManager, WindowCacheManager};
use crate::core::cmd::events::ModifiersState;
use crate::core::cmd::{EngineBatchEvents, EngineBatchResponses};

/// Represents a window with its associated WGPU resources
pub struct WindowState {
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,

    // Window state tracking
    pub inner_position: IVec2,
    pub outer_position: IVec2,
    pub inner_size: UVec2,
    pub outer_size: UVec2,
    pub(crate) is_dirty: bool,
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
    pub response_queue: EngineBatchResponses,
    pub(crate) serialized_events_buffer: Vec<u8>,
    pub(crate) serialized_responses_buffer: Vec<u8>,

    // Time tracking
    pub(crate) time: u64,
    pub(crate) delta_time: u32,

    // Input state
    pub(crate) modifiers_state: ModifiersState,
    pub(crate) cursor_positions: HashMap<u32, Vec2>,
    pub(crate) gilrs: Option<gilrs::Gilrs>,

    // Event caching for optimization
    pub(crate) window_cache: WindowCacheManager,
    pub(crate) input_cache: InputCacheManager,
    pub(crate) gamepad_cache: GamepadCacheManager,

    // Profiling data
    pub(crate) profiling: TickProfiling,
}

/// Detailed profiling data for tick operations
#[derive(Debug, Clone, Default)]
pub struct TickProfiling {
    pub gamepad_processing_ns: u64,
    pub event_loop_pump_ns: u64,
    pub request_redraw_ns: u64,
    pub serialization_ns: u64,
    pub total_events_dispatched: usize,
    pub total_events_cached: usize,
    pub(crate) custom_events_ns: u64,
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
            response_queue: Vec::new(),
            serialized_events_buffer: Vec::new(),
            serialized_responses_buffer: Vec::new(),

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

            window_cache: WindowCacheManager::new(),
            input_cache: InputCacheManager::new(),
            gamepad_cache: GamepadCacheManager::new(),

            profiling: TickProfiling::default(),
        }
    }

    /// Mark a specific window as dirty to trigger redraw on next tick
    #[allow(dead_code)]
    pub fn mark_window_dirty(&mut self, window_id: u32) {
        if let Some(window_state) = self.windows.get_mut(&window_id) {
            window_state.is_dirty = true;
        }
    }

    /// Mark all windows as dirty to trigger redraw on next tick
    #[allow(dead_code)]
    pub fn mark_all_windows_dirty(&mut self) {
        for window_state in self.windows.values_mut() {
            window_state.is_dirty = true;
        }
    }
}
