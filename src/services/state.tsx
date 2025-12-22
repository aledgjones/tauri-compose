import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  Align,
  AppState,
  AutoCountStyle,
  BottomPanelTab,
  BracketingApproach,
  BracketStyle,
  FontFamily,
  Justify,
  LayoutType,
  NoteDuration,
  Sampler,
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
    write: {
      bottom_panel_minimized: false,
      bottom_panel_tab: BottomPanelTab.Sequencer,
    },
    sequence: {
      bottom_panel_minimized: false,
      bottom_panel_tab: BottomPanelTab.Sequencer,
    },
    sequencer: {
      x: 0,
      y: 1008,
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
        solo: AutoCountStyle.Arabic,
        section: AutoCountStyle.Roman,
      },
    },
    engraves: {
      by_layout_type: {
        [LayoutType.FullScore]: {
          space: 1.5,

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

export const SamplerContext = createContext<Sampler | null>(null);

export const AppStateProvider: FC<PropsWithChildren> = ({ children }) => {
  const [appState, setAppState] = useState<AppState>(initialAppState);
  const [samplerState, setSamplerState] = useState<Sampler>({});

  useSampler(appState, samplerState);

  useEffect(() => {
    let unlistenApp: UnlistenFn | null = null;
    let unlistenSampler: UnlistenFn | null = null;
    let unlistenMenuSaveAs: UnlistenFn | null = null;
    let unlistenMenuOpen: UnlistenFn | null = null;
    let unlistenMenuNew: UnlistenFn | null = null;

    (async () => {
      unlistenApp = await listen<AppState>("soloc://state/general", (e) => {
        setAppState(e.payload);
      });

      unlistenSampler = await listen<Sampler>("soloc://state/sampler", (e) => {
        setSamplerState(e.payload);
      });

      unlistenMenuSaveAs = await listen("soloc://menu/save-as", () => {
        actions.save.project().catch(console.error);
      });

      unlistenMenuOpen = await listen("soloc://menu/open", () => {
        actions.load.project().catch(console.error);
      });

      unlistenMenuNew = await listen("soloc://menu/new", () => {
        actions.reset().catch(console.error);
      });

      await actions.initialize();
    })();

    return () => {
      unlistenApp?.();
      unlistenSampler?.();
      unlistenMenuSaveAs?.();
      unlistenMenuOpen?.();
      unlistenMenuNew?.();
    };
  }, []);

  return (
    <AppContext.Provider value={appState}>
      <SamplerContext.Provider value={samplerState}>
        {children}
      </SamplerContext.Provider>
    </AppContext.Provider>
  );
};

export const useAppState = () => {
  const context = useContext(AppContext);

  if (!context) {
    throw new Error("useAppState must be used within an AppStateProvider");
  }

  return context;
};

export const useSamplerState = () => {
  const context = useContext(SamplerContext);

  if (!context) {
    throw new Error("useSamplerState must be used within an AppStateProvider");
  }

  return context;
};
