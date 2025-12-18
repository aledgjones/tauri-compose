import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  Align,
  AppState,
  AutoCountStyle,
  BracketingApproach,
  BracketStyle,
  FontFamily,
  Justify,
  LayoutType,
  NoteDuration,
  Tab,
  Tool,
} from "./types";
import { actions } from "./actions";
import { useSampler } from "./use-sampler";
import {
  createContext,
  FC,
  PropsWithChildren,
  useContext,
  useEffect,
  useState,
} from "react";

export const initialAppState: AppState = {
  ui: {
    tab: Tab.Setup,
    layout_id: 0,
    setup: { selection: null, expanded: [] },
    sequencer: {
      x: 0,
      y: 0,
      tool: Tool.Select,
      snap: NoteDuration.Eighth,
      flow: null,
      instrument: null,
      track: null,
    },
  },
  score: {
    layouts: { order: [], by_id: {} },
    flows: { order: [], by_id: {} },
    players: { order: [], by_id: {} },
    instruments: { by_id: {} },
    tracks: { by_id: {} },
    config: {
      auto_count_styles: {
        solo: AutoCountStyle.Roman,
        section: AutoCountStyle.Roman,
      },
    },
    engraves: {
      by_layout_type: {
        [LayoutType.FullScore]: {
          space: 3,

          frame_padding: { top: 35, right: 25, bottom: 35, left: 25 },
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
          minimum_tie_space: 1.0,
          note_space_ratio: 1.41,

          maximum_beam_slant: 0.5,
        },
        [LayoutType.Part]: {
          space: 3,

          frame_padding: { top: 35, right: 25, bottom: 35, left: 25 },
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
          minimum_tie_space: 1.0,
          note_space_ratio: 1.41,

          maximum_beam_slant: 0.5,
        },
      },
    },
    meta: {
      app_version: "0.1.0",
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

export const AppStateProvider: FC<PropsWithChildren> = ({ children }) => {
  const [appState, setAppState] = useState<AppState>(initialAppState);

  useSampler(appState);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    (async () => {
      unlisten = await listen<AppState>("solocompose://state", (e) => {
        setAppState(e.payload);
      });

      await actions.initialize();
    })();

    return () => {
      unlisten?.();
    };
  }, []);

  return <AppContext.Provider value={appState}>{children}</AppContext.Provider>;
};

export const useAppState = () => {
  const context = useContext(AppContext);

  if (!context) {
    throw new Error("useAppState must be used within an AppStateProvider");
  }

  return context;
};
