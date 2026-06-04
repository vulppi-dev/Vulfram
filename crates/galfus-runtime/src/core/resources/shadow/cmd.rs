use super::Shadow3dConfig;
use crate::core::state::EngineState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CmdShadow3dConfigureArgs {
    pub window_id: u32,
    pub config: Shadow3dConfig,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(default, rename_all = "camelCase")]
pub struct CmdResultShadow3dConfigure {
    pub success: bool,
    pub message: String,
}

pub fn engine_cmd_shadow3d_configure(
    engine: &mut EngineState,
    args: &CmdShadow3dConfigureArgs,
) -> CmdResultShadow3dConfigure {
    let device = match engine.device.as_ref() {
        Some(d) => d,
        None => {
            return CmdResultShadow3dConfigure {
                success: false,
                message: "GPU Device not initialized".into(),
            };
        }
    };

    if let Some(render_state) = engine.render.get_mut(&args.window_id) {
        let Some(shadow) = render_state.shadow_3d.as_mut() else {
            return CmdResultShadow3dConfigure {
                success: false,
                message: format!(
                    "Window {} not found or shadow manager not initialized",
                    args.window_id
                ),
            };
        };
        shadow.configure(device, args.config);
        if let Some(bindings) = render_state.bindings.as_mut() {
            bindings.shared_group = None;
            bindings.shadow_model_bind_group = None;
        }
        if let Some(window_state) = engine.window.states.get_mut(&args.window_id) {
            window_state.is_dirty = true;
        }
        CmdResultShadow3dConfigure {
            success: true,
            message: "Shadow configuration updated successfully".into(),
        }
    } else {
        CmdResultShadow3dConfigure {
            success: false,
            message: format!(
                "Window {} not found or shadow manager not initialized",
                args.window_id
            ),
        }
    }
}
