use crate::core::state::EngineState;
use winit::window::WindowId;

pub fn render_frames(_wid: WindowId, engine_state: &mut EngineState) {
    // Get device and queue
    let device = match &engine_state.device {
        Some(device) => device,
        None => return,
    };

    let queue = match &engine_state.queue {
        Some(queue) => queue,
        None => return,
    };

    // Render all windows
    for (_window_id, window_state) in engine_state.windows.iter() {
        // Get the surface texture
        let surface_texture = match window_state.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(e) => {
                log::error!("Failed to get surface texture: {:?}", e);
                continue;
            }
        };

        // Create a texture view
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create a command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // Create a render pass with purple clear color
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5, // Red component
                            g: 0.0, // Green component
                            b: 0.5, // Blue component
                            a: 1.0, // Alpha component
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        // Submit the commands
        queue.submit(std::iter::once(encoder.finish()));

        // Present the frame
        surface_texture.present();
    }
}
