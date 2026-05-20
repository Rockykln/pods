use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::model::{
    ConvAwareness, DeviceState, Event, MicSelection, Mode, PairedDevice, PressAction, Profile,
    SpatialAudio,
};

pub fn socket_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(dir).join("podctl.sock");
    }
    let uid = unsafe { libc_getuid() };
    PathBuf::from(format!("/tmp/podctl-{uid}.sock"))
}

unsafe extern "C" {
    #[link_name = "getuid"]
    fn libc_getuid() -> u32;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    Status,
    Battery,
    Ping,

    SetMode {
        mode: Mode,
    },
    SetConv {
        conv: ConvAwareness,
    },
    SetSpatial {
        mode: SpatialAudio,
    },

    SetEarDetection {
        on: bool,
    },
    SetMic {
        mic: MicSelection,
    },
    SetLoudReduction {
        on: bool,
    },
    SetPressAction {
        side: PressSide,
        action: PressAction,
    },
    SetToneOnPress {
        on: bool,
    },
    Rename {
        name: String,
    },

    Connect,
    Disconnect,
    Pair,
    Unpair,
    List,
    SetAutoConnect {
        on: bool,
    },

    SetVolume {
        percent: u8,
    },
    SetMuted {
        muted: bool,
    },
    SetProfile {
        profile: Profile,
    },
    SetCodec {
        codec: String,
    },
    MakeDefaultSink,
    SetLatencyOffset {
        ms: i32,
    },

    SetOneBudAnc {
        on: bool,
    },
    SetChimeVolume {
        level: u8,
    },
    SetAutoAncLevel {
        level: u8,
    },

    DebugEmitCaseLid {
        open: bool,
    },
    ShowPopup,

    Watch,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PressSide {
    Left,
    Right,
}

// `State` is the dominant variant for view-style requests; boxing it
// would buy us bytes-on-stack at the cost of an allocation on every
// happy-path response. Not worth it for a single-shot IPC over Unix.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum Response {
    State(DeviceState),
    List(Vec<PairedDevice>),
    Event(Event),
    Pong,
    Done,
    Pending(String),
    Err(String),
}

// Legacy alias so existing call sites keep compiling.
pub type OkPayload = Response;

impl Response {
    pub fn ok_state(s: DeviceState) -> Self {
        Response::State(s)
    }
    pub fn ok_list(v: Vec<PairedDevice>) -> Self {
        Response::List(v)
    }
    pub fn ok_done() -> Self {
        Response::Done
    }
    pub fn ok_pending(reason: impl Into<String>) -> Self {
        Response::Pending(reason.into())
    }
    pub fn ok_pong() -> Self {
        Response::Pong
    }
    pub fn ok_event(e: Event) -> Self {
        Response::Event(e)
    }
    pub fn err(msg: impl Into<String>) -> Self {
        Response::Err(msg.into())
    }
    pub fn is_err(&self) -> bool {
        matches!(self, Response::Err(_))
    }
}
