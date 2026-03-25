import {
  Align,
  AppState,
  AudioState,
  AutoCountStyle,
  BottomPanelTab,
  BracketingApproach,
  BracketStyle,
  FontFamily,
  Justify,
  LayoutType,
  NoteDuration,
  PluginInventoryState,
  Tab,
  Tool,
  TransportState,
} from "../types";
import { createContext, useContext } from "react";

export const DEFAULT_TRANSPORT_TEMPO = 120;

export const initialAppState: AppState = {
  current_file_path: null,
  resolved_playback_route_by_instrument_id: {},
  route_target_options_by_slot_id: {},
  ui: {
    tab: Tab.Setup,
    layout_id: null,
    flow_id: null,
    renderer: {
      zoom: 1,
      pxPerMm: 96 / 25.4,
      pan_x: 0,
      pan_y: 0,
      bounds: { x: 0, y: 0, width: 0, height: 0 },
      debug: false,
    },
    sequencer: {
      x: 0,
      y: 1008,
      tool: Tool.Select,
      snap: NoteDuration.Eighth,
      instrument: null,
      track: null,
      selection: null,
    },
    screens: {
      setup: { selection: null, expanded: [] },
      write: {
        bottom_panel_minimized: false,
        bottom_panel_tab: BottomPanelTab.Sequencer,
      },
      sequence: {
        bottom_panel_minimized: false,
        bottom_panel_tab: BottomPanelTab.Sequencer,
        show_active_only: true,
      },
    },
  },
  preferences: {
    audition: true,
  },
  score: {
    playback: {
      rack: { order: [], by_id: {} },
    },
    layouts: { order: [], by_id: {} },
    flows: { order: [], by_id: {} },
    players: { order: [], by_id: {} },
    instruments: { by_id: {} },
    tracks: { by_id: {} },
    config: {
      auto_count_styles: {
        solo: AutoCountStyle.Arabic,
        section: AutoCountStyle.Roman,
      },
    },
    engraves: {
      by_layout_type: {
        [LayoutType.FullScore]: {
          space: 1.5,

          page_padding: { top: 35, right: 20, bottom: 35, left: 20 },
          frame_padding: { top: 25, right: 0, bottom: 25, left: 0 },
          instrument_spacing: 8,
          stave_spacing: 6,
          system_start_padding: 0.75,

          instrument_name_font: {
            size: 1.75,
            family: FontFamily.LibreBaskerville,
            justify: Justify.End,
            align: Align.Middle,
          },
          instrument_name_spacing: 2,

          systemic_barline_single_instrument_system: true,
          bracketing_approach: BracketingApproach.Orchestral,
          bracket_style: BracketStyle.Wing,
          bracket_single_staves: false,

          base_note_space: 3.0,
          minimum_note_space: 0.6,
          minimum_tie_space: 1.6,
          note_space_ratio: 1.41,

          maximum_beam_slant: 0.5,
        },
        [LayoutType.Part]: {
          space: 2.0,

          page_padding: { top: 35, right: 20, bottom: 35, left: 20 },
          frame_padding: { top: 25, right: 0, bottom: 25, left: 0 },
          instrument_spacing: 8,
          stave_spacing: 6,
          system_start_padding: 0.75,

          instrument_name_font: {
            size: 1.75,
            family: FontFamily.LibreBaskerville,
            justify: Justify.End,
            align: Align.Middle,
          },
          instrument_name_spacing: 2,

          systemic_barline_single_instrument_system: false,
          bracketing_approach: BracketingApproach.Orchestral,
          bracket_style: BracketStyle.Wing,
          bracket_single_staves: false,

          base_note_space: 3.0,
          minimum_note_space: 0.6,
          minimum_tie_space: 1.6,
          note_space_ratio: 1.41,

          maximum_beam_slant: 0.5,
        },
      },
    },
    meta: {
      app_version: "0.15.0",
      title: "",
      subtitle: "",
      dedication: "",
      composer: "",
      arranger: "",
      lyricist: "",
      artist: "",
      copyist: "",
      publisher: "",
      editor: "",
      copyright: "",
      work_number: "",
      composition_year: "",
      composer_dates: "",
      other_information: "",
      created: Date.now(),
    },
  },
};

export const AppContext = createContext<AppState | null>(null);

export const initialAudioState: AudioState = {
  devices: [],
  active_device_id: "",
  current_sample_rate: 44100,
};

export const AudioContext = createContext<AudioState | null>(null);

export const initialPluginInventoryState: PluginInventoryState = {
  entries: [],
  errors: [],
  last_discovered_at_ms: null,
};

export const PluginInventoryContext =
  createContext<PluginInventoryState | null>(null);

export const initialTransportState: TransportState = {
  playing: false,
  current_tick: 0,
  tempo: DEFAULT_TRANSPORT_TEMPO,
};

export const TransportContext = createContext<TransportState | null>(null);

export const useAppState = () => {
  const context = useContext(AppContext);

  if (!context) {
    throw new Error("useAppState must be used within an AppStateProvider");
  }

  return context;
};

export const useAudioState = () => {
  const context = useContext(AudioContext);

  if (!context) {
    throw new Error("useAudioState must be used within an AudioStateProvider");
  }

  return context;
};

export const usePluginInventoryState = () => {
  const context = useContext(PluginInventoryContext);

  if (!context) {
    throw new Error(
      "usePluginInventoryState must be used within an AppStateProvider",
    );
  }

  return context;
};

export const useTransportState = () => {
  const context = useContext(TransportContext);

  if (!context) {
    throw new Error(
      "useTransportState must be used within an AppStateProvider",
    );
  }

  return context;
};
