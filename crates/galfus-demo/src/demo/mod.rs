mod io;
mod scenarios;
mod session;

use std::env;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoKind {
    FrameGraph001,
    FrameGraph002Persistence,
    Realm2D003,
    Realm2D004LightsShadows,
}

impl DemoKind {
    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "1" | "001" | "demo001" | "demo_001" | "framegraph" | "framegraph001" => {
                Some(Self::FrameGraph001)
            }
            "2" | "002" | "demo002" | "demo_002" | "framegraph002" | "persistence" => {
                Some(Self::FrameGraph002Persistence)
            }
            "3" | "003" | "demo003" | "demo_003" | "realm2d" | "2d" => Some(Self::Realm2D003),
            "4" | "004" | "demo004" | "demo_004" | "realm2d004" | "2dls" | "lights2d"
            | "shadows2d" => Some(Self::Realm2D004LightsShadows),
            _ => None,
        }
    }

    pub fn title(self) -> String {
        match self {
            Self::FrameGraph001 => "Galfus Demo 001 - FrameGraph".to_string(),
            Self::FrameGraph002Persistence => "Galfus Demo 002 - Optical Persistence".to_string(),
            Self::Realm2D003 => "Galfus Demo 003 - Realm2D".to_string(),
            Self::Realm2D004LightsShadows => {
                "Galfus Demo 004 - Realm2D Lights and Shadows".to_string()
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DemoContext {
    pub window_id: u32,
    pub realm_id: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DemoRunOptions {
    pub timeout: Option<Duration>,
}

#[derive(Debug, Clone, Copy)]
pub struct DemoSelection {
    pub demo: DemoKind,
    pub options: DemoRunOptions,
}

pub fn select_demo() -> DemoSelection {
    let mut demo_arg: Option<String> = None;
    let mut timeout_seconds: Option<u64> = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--timeout" {
            let value = args
                .next()
                .unwrap_or_else(|| panic!("--timeout requires a value in seconds"));
            timeout_seconds = Some(parse_timeout_seconds(&value));
            continue;
        }
        if let Some(value) = arg.strip_prefix("--timeout=") {
            timeout_seconds = Some(parse_timeout_seconds(value));
            continue;
        }
        if demo_arg.is_none() {
            demo_arg = Some(arg);
        }
    }

    if let Some(value) = demo_arg.as_ref()
        && let Some(demo) = DemoKind::from_str(value)
    {
        return DemoSelection {
            demo,
            options: DemoRunOptions {
                timeout: timeout_seconds.map(Duration::from_secs),
            },
        };
    }
    if let Ok(value) = env::var("GALFUS_DEMO")
        && let Some(demo) = DemoKind::from_str(&value)
    {
        return DemoSelection {
            demo,
            options: DemoRunOptions {
                timeout: timeout_seconds.map(Duration::from_secs),
            },
        };
    }

    DemoSelection {
        demo: DemoKind::FrameGraph001,
        options: DemoRunOptions {
            timeout: timeout_seconds.map(Duration::from_secs),
        },
    }
}

fn parse_timeout_seconds(value: &str) -> u64 {
    value
        .trim()
        .parse::<u64>()
        .ok()
        .filter(|seconds| *seconds > 0)
        .unwrap_or_else(|| panic!("invalid --timeout value '{}': expected integer > 0", value))
}

pub fn run_demo(demo: DemoKind, ctx: DemoContext, options: DemoRunOptions) -> bool {
    scenarios::run(demo, ctx, options)
}

pub use io::send_commands;
pub use session::create_window;

#[cfg(test)]
mod tests {
    use super::{DemoKind, parse_timeout_seconds};

    #[test]
    fn demo_004_aliases_select_realm2d_lights_shadows() {
        assert_eq!(
            DemoKind::from_str("4"),
            Some(DemoKind::Realm2D004LightsShadows)
        );
        assert_eq!(
            DemoKind::from_str("004"),
            Some(DemoKind::Realm2D004LightsShadows)
        );
        assert_eq!(
            DemoKind::from_str("demo004"),
            Some(DemoKind::Realm2D004LightsShadows)
        );
        assert_eq!(
            DemoKind::from_str("shadows2d"),
            Some(DemoKind::Realm2D004LightsShadows)
        );
    }

    #[test]
    fn timeout_parser_accepts_positive_integer() {
        assert_eq!(parse_timeout_seconds("5"), 5);
    }
}
