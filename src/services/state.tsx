import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
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
  current_file_path: null,
  ui: {
    tab: Tab.Setup,
    layout_id: 0,
    renderer: {
      zoom: 1,
      pxPerMm: 96 / 25.4,
      pan_x: 0,
      pan_y: 0,
      bounds: { x: 0, y: 0, width: 0, height: 0 },
      debug_rendering: false,
    },
    sequencer: {
      x: 0,
      y: 1008,
      tool: Tool.Select,
      snap: NoteDuration.Eighth,
      flow: null,
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
      },
    },
  },
  preferences: {
    audition: true,
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
export const SamplerContext = createContext<Sampler | null>(null);

type Props = PropsWithChildren<{ isMainWindow?: boolean }>;

export const AppStateProvider: FC<Props> = ({
  children,
  isMainWindow = false,
}) => {
  const [appState, setAppState] = useState<AppState>(initialAppState);
  const [samplerState, setSamplerState] = useState<Sampler>({});

  useSampler(appState, samplerState);

  useEffect(() => {
    let unlistenApp: UnlistenFn | null = null;
    let unlistenSampler: UnlistenFn | null = null;
    let unlistenMenuSave: UnlistenFn | null = null;
    let unlistenMenuSaveAs: UnlistenFn | null = null;
    let unlistenMenuOpen: UnlistenFn | null = null;
    let unlistenMenuOpenRecent: UnlistenFn | null = null;
    let unlistenMenuNew: UnlistenFn | null = null;
    let unlistenMenuPreferences: UnlistenFn | null = null;
    let unlistenRecentsChanged: UnlistenFn | null = null;

    (async () => {
      unlistenApp = await listen<AppState>("soloc://state/general", (e) => {
        setAppState(e.payload);
      });

      unlistenSampler = await listen<Sampler>("soloc://state/sampler", (e) => {
        setSamplerState(e.payload);
      });

      if (isMainWindow) {
        unlistenMenuSave = await listen("soloc://menu/save", () => {
          actions.file.save().catch(console.error);
        });

        unlistenMenuSaveAs = await listen("soloc://menu/save-as", () => {
          actions.file.saveAs().catch(console.error);
        });

        unlistenMenuOpen = await listen("soloc://menu/open", () => {
          actions.file.load().catch(console.error);
        });

        unlistenMenuOpenRecent = await listen<string>(
          "soloc://menu/open-recent",
          (e) => {
            actions.file.loadFromPath(e.payload).catch(console.error);
          },
        );

        unlistenMenuNew = await listen("soloc://menu/new", () => {
          actions.reset().catch(console.error);
        });

        unlistenMenuPreferences = await listen(
          "soloc://menu/preferences",
          () => {
            const webview = new WebviewWindow("preferences", {
              url: "/preferences.html",
              title: "Preferences",
              visible: false,
              titleBarStyle: "overlay",
              acceptFirstMouse: true,
              height: 500,
              width: 750,
              theme: "dark",
            });

            webview.setFocus();
          },
        );

        // Listen for recents changes and refresh menus
        unlistenRecentsChanged = await listen("soloc://recents-changed", () => {
          invoke("refresh_menus").catch(console.error);
        });
      }

      await actions.initialize();

      // Initial menu refresh after initialization - delay to ensure menu is ready
      if (isMainWindow) {
        setTimeout(() => {
          invoke("refresh_menus").catch(console.error);
        }, 100);
      }
    })();

    return () => {
      unlistenApp?.();
      unlistenSampler?.();
      unlistenMenuSave?.();
      unlistenMenuSaveAs?.();
      unlistenMenuOpen?.();
      unlistenMenuOpenRecent?.();
      unlistenMenuNew?.();
      unlistenMenuPreferences?.();
      unlistenRecentsChanged?.();
    };
  }, [isMainWindow]);

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
