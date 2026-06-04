use super::super::*;
use super::defer::{DeferredFailureKind, classify_failed_response, command_signature};
use crate::core::platform::EventLoopProxy;
use crate::core::platforms::PlatformProxy;
use crate::core::singleton::EngineCustomEvents;
use crate::core::state::EngineState;
use crate::core::window::{CmdResultWindowCreate, CmdWindowCreateArgs};
use galfus_log::LogLevel;
use galfus_runtime::defer_backoff_frames;

struct TestPlatformProxy;

impl PlatformProxy for TestPlatformProxy {
    fn event_loop_proxy(&self) -> &EventLoopProxy<EngineCustomEvents> {
        panic!("event_loop_proxy is not used in this test")
    }

    fn handle_window_create(
        &mut self,
        _state: &mut EngineState,
        _cmd_id: u64,
        _args: &CmdWindowCreateArgs,
    ) -> Result<(), CmdResultWindowCreate> {
        Err(CmdResultWindowCreate {
            success: false,
            message: "Window creation is not supported in this unit test proxy".into(),
            ..Default::default()
        })
    }

    fn process_gamepads(&mut self, _state: &mut EngineState) -> u64 {
        0
    }

    fn pump_events(&mut self, _state: &mut EngineState) -> u64 {
        0
    }

    fn render(&mut self, _state: &mut EngineState) -> u64 {
        0
    }
}

#[test]
fn defer_backoff_caps_at_64_frames() {
    assert_eq!(defer_backoff_frames(1), 1);
    assert_eq!(defer_backoff_frames(2), 2);
    assert_eq!(defer_backoff_frames(3), 4);
    assert_eq!(defer_backoff_frames(7), 64);
    assert_eq!(defer_backoff_frames(100), 64);
}

#[test]
fn command_signature_differs_for_different_payloads() {
    let cmd_a = EngineCmd::CmdWindowMeasurement(win::CmdWindowMeasurementArgs {
        window_id: 1,
        ..Default::default()
    });
    let cmd_b = EngineCmd::CmdWindowMeasurement(win::CmdWindowMeasurementArgs {
        window_id: 2,
        ..Default::default()
    });
    assert_ne!(command_signature(&cmd_a), command_signature(&cmd_b));
}

#[test]
fn classify_failed_response_marks_transient_dependency() {
    let engine = EngineState::new();
    let cmd = EngineCmd::CmdShadow3dConfigure(res::shadow::CmdShadow3dConfigureArgs {
        window_id: 10,
        config: res::shadow::Shadow3dConfig::default(),
    });
    let response = CommandResponse::Shadow3dConfigure(res::shadow::CmdResultShadow3dConfigure {
        success: false,
        message: "Window 10 not found or shadow manager not initialized".into(),
    });
    let classified = classify_failed_response(&engine, &cmd, &response);
    assert!(matches!(
        classified,
        Some((DeferredFailureKind::Transient, _))
    ));
}

#[test]
fn classify_failed_response_marks_permanent_invalid_payload() {
    let mut engine = EngineState::new();
    engine.buffers.insert_upload(
        77,
        crate::core::buffers::state::UploadBuffer {
            upload_type: crate::core::buffers::state::UploadType::Raw,
            data: vec![1, 2, 3],
        },
    );
    let cmd = EngineCmd::CmdTextureCreateFromBuffer(res::CmdTextureCreateFromBufferArgs {
        texture_id: 9,
        label: None,
        buffer_id: 77,
        srgb: Some(true),
        mode: res::TextureCreateMode::Standalone,
        atlas_options: None,
    });
    let response =
        CommandResponse::TextureCreateFromBuffer(res::CmdResultTextureCreateFromBuffer {
            success: false,
            message: "Invalid buffer type. Expected ImageData, got Raw".into(),
            pending: false,
        });
    let classified = classify_failed_response(&engine, &cmd, &response);
    assert!(matches!(
        classified,
        Some((DeferredFailureKind::Permanent, _))
    ));
}

#[test]
fn engine_process_batch_dispatches_window_measurement_and_returns_matching_response() {
    let mut engine = EngineState::new();
    let mut platform = TestPlatformProxy;
    let batch = vec![EngineCmdEnvelope {
        id: 42,
        cmd: EngineCmd::CmdWindowMeasurement(win::CmdWindowMeasurementArgs {
            window_id: 999,
            ..Default::default()
        }),
    }];

    let result = super::engine_process_batch(&mut engine, &mut platform, batch);
    assert!(matches!(result, crate::core::GalfusResult::Success));
    assert_eq!(engine.runtime.response_count(), 1);

    let response = engine
        .runtime
        .last_response_cloned()
        .expect("one response must be returned");
    assert_eq!(response.id, 42);

    match response.response {
        CommandResponse::WindowMeasurement(payload) => {
            assert!(payload.success);
            assert!(payload.message.contains("Window 999 not ready yet"));
        }
        other => panic!("unexpected response type: {other:?}"),
    }
}

#[test]
fn engine_process_batch_preserves_response_order_and_ids_for_multiple_commands() {
    let mut engine = EngineState::new();
    let mut platform = TestPlatformProxy;
    let batch = vec![
        EngineCmdEnvelope {
            id: 100,
            cmd: EngineCmd::CmdSystemBuildVersionGet(sys::CmdSystemBuildVersionGetArgs {}),
        },
        EngineCmdEnvelope {
            id: 101,
            cmd: EngineCmd::CmdWindowMeasurement(win::CmdWindowMeasurementArgs {
                window_id: 777,
                ..Default::default()
            }),
        },
    ];

    let result = super::engine_process_batch(&mut engine, &mut platform, batch);
    assert!(matches!(result, crate::core::GalfusResult::Success));

    let responses = engine.runtime.response_batch();
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0].id, 100);
    assert_eq!(responses[1].id, 101);

    match &responses[0].response {
        CommandResponse::SystemBuildVersionGet(payload) => {
            assert!(payload.success);
        }
        other => panic!("unexpected first response type: {other:?}"),
    }

    match &responses[1].response {
        CommandResponse::WindowMeasurement(payload) => {
            assert!(payload.success);
            assert!(payload.message.contains("Window 777 not ready yet"));
        }
        other => panic!("unexpected second response type: {other:?}"),
    }
}

#[test]
fn engine_process_batch_dispatches_system_log_level_set_and_get() {
    let mut engine = EngineState::new();
    let mut platform = TestPlatformProxy;
    let batch = vec![
        EngineCmdEnvelope {
            id: 200,
            cmd: EngineCmd::CmdSystemLogLevelSet(sys::CmdSystemLogLevelSetArgs {
                level: LogLevel::Error,
            }),
        },
        EngineCmdEnvelope {
            id: 201,
            cmd: EngineCmd::CmdSystemLogLevelGet(sys::CmdSystemLogLevelGetArgs {}),
        },
    ];

    let result = super::engine_process_batch(&mut engine, &mut platform, batch);
    assert!(matches!(result, crate::core::GalfusResult::Success));

    let responses = engine.runtime.response_batch();
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0].id, 200);
    assert_eq!(responses[1].id, 201);

    match &responses[0].response {
        CommandResponse::SystemLogLevelSet(payload) => {
            assert!(payload.success);
            assert_eq!(payload.current_level, LogLevel::Error);
        }
        other => panic!("unexpected first response type: {other:?}"),
    }

    match &responses[1].response {
        CommandResponse::SystemLogLevelGet(payload) => {
            assert!(payload.success);
            assert_eq!(payload.current_level, LogLevel::Error);
        }
        other => panic!("unexpected second response type: {other:?}"),
    }
}
