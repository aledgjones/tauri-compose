use crate::utils::UniqueId;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScorePlayback {
    pub rack: PlaybackRack,
}

impl ScorePlayback {
    pub fn new() -> Self {
        Self {
            rack: PlaybackRack::new(),
        }
    }
}

impl Default for ScorePlayback {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackRack {
    pub order: Vec<UniqueId>,
    pub by_id: FxHashMap<UniqueId, PlaybackRackSlot>,
}

impl PlaybackRack {
    pub fn new() -> Self {
        Self {
            order: Vec::new(),
            by_id: FxHashMap::default(),
        }
    }
}

impl Default for PlaybackRack {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackRackSlot {
    pub id: UniqueId,
    pub target: Option<PlaybackSlotTarget>,
}

impl PlaybackRackSlot {
    pub fn new(id: UniqueId) -> Self {
        Self { id, target: None }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackSlotTarget {
    pub plugin_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_state: Option<PlaybackSlotPluginState>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackSlotPluginState {
    pub vst3_component_state_base64: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PlaybackRouteTargetMode {
    Plugin = 0,
    PluginPart = 1,
    MidiChannel = 2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackRouteTarget {
    pub mode: PlaybackRouteTargetMode,
    pub index: Option<u8>,
}

impl PlaybackRouteTarget {
    pub fn plugin() -> Self {
        Self {
            mode: PlaybackRouteTargetMode::Plugin,
            index: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum PlaybackRouteAddressMode {
    Root = 0,
    Part = 1,
    MidiChannel = 2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackRouteAddress {
    pub mode: PlaybackRouteAddressMode,
    pub index: Option<u8>,
}

impl PlaybackRouteAddress {
    pub fn root() -> Self {
        Self {
            mode: PlaybackRouteAddressMode::Root,
            index: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstrumentPlaybackRoute {
    pub slot_id: UniqueId,
    pub target: PlaybackRouteTarget,
    pub address: PlaybackRouteAddress,
}

impl InstrumentPlaybackRoute {
    #[cfg(test)]
    pub fn direct(slot_id: UniqueId) -> Self {
        Self {
            slot_id,
            target: PlaybackRouteTarget::plugin(),
            address: PlaybackRouteAddress::root(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaybackRouteChoice {
    pub key: String,
    pub label: String,
    pub target: PlaybackRouteTarget,
    pub address: PlaybackRouteAddress,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum ResolvedPlaybackRouteStatus {
    Unrouted = 0,
    Ready = 1,
    NeedsAttention = 2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportedPlaybackMixControls {
    pub volume: bool,
    pub pan: bool,
    pub mute: bool,
    pub solo: bool,
}

impl SupportedPlaybackMixControls {
    pub fn direct_plugin_root() -> Self {
        Self {
            volume: true,
            pan: true,
            mute: true,
            solo: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedPlaybackRoute {
    pub status: ResolvedPlaybackRouteStatus,
    pub slot_id: Option<UniqueId>,
    pub slot_target: Option<PlaybackSlotTarget>,
    pub effective_plugin_id: Option<String>,
    pub target: Option<PlaybackRouteTarget>,
    pub address: Option<PlaybackRouteAddress>,
    pub supported_mix_controls: SupportedPlaybackMixControls,
}
