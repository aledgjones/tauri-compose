import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  AppState,
  AudioState,
  PluginInventoryState,
  SamplerState,
  Tempo,
  TransportState,
  UniqueId,
} from "../types";
import { actions } from "../actions";
import {
  FC,
  PropsWithChildren,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import {
  AppContext,
  initialAppState,
  initialAudioState,
  AudioContext,
  initialPluginInventoryState,
  PluginInventoryContext,
  initialSamplerState,
  SamplerContext,
  initialTransportState,
  TransportContext,
} from "./context";

type Props = PropsWithChildren<{ isMainWindow?: boolean }>;

export const AppStateProvider: FC<Props> = ({
  children,
  isMainWindow = false,
}) => {
  const [appState, setAppState] = useState<AppState>(initialAppState);
  const [audioState, setAudioState] = useState<AudioState>(initialAudioState);
  const [pluginInventoryState, setPluginInventoryState] =
    useState<PluginInventoryState>(initialPluginInventoryState);
  const [samplerState, setSamplerState] =
    useState<SamplerState>(initialSamplerState);
  const [transportState, setTransportState] = useState<TransportState>(
    initialTransportState,
  );
  const [isInitialized, setIsInitialized] = useState(false);
  const lastPluginInventoryErrorSignatureRef = useRef("");

  // Transport polling: poll atomics via lightweight command instead of event listener.
  // This keeps the transport thread completely silent — no app.emit() during playback.
  const [isPolling, setIsPolling] = useState(false);

  useEffect(() => {
    if (!isPolling) return;

    let rafId: number;
    let active = true;

    const poll = () => {
      actions.transport.getState().then((state) => {
        setTransportState(state);
        if (state.playing && active) {
          rafId = requestAnimationFrame(poll);
        } else {
          setIsPolling(false);
        }
      });
    };

    rafId = requestAnimationFrame(poll);

    return () => {
      active = false;
      cancelAnimationFrame(rafId);
    };
  }, [isPolling]);

  const activeFlowId = useMemo<UniqueId | null>(() => {
    return appState.ui.flow_id ?? appState.score.flows.order[0] ?? null;
  }, [appState.ui.flow_id, appState.score.flows.order]);

  const transportTempoSyncKey = useMemo(() => {
    if (activeFlowId === null) {
      return "no-flow";
    }

    const flow = appState.score.flows.by_id[activeFlowId];
    if (!flow) {
      return `missing-flow:${activeFlowId}`;
    }

    const masterTrack = appState.score.tracks.by_id[flow.master_id];
    if (!masterTrack) {
      return `missing-master:${flow.master_id}`;
    }

    const tempos = Object.values(masterTrack.entries.by_id)
      .filter((entry): entry is Tempo => entry.type === "Tempo")
      .sort((left, right) => left.tick - right.tick)
      .map((tempo) => `${tempo.tick}:${tempo.bpm}`)
      .join("|");

    return `${activeFlowId}:${tempos}`;
  }, [activeFlowId, appState.score.flows.by_id, appState.score.tracks.by_id]);

  useEffect(() => {
    if (!isInitialized || isPolling) return;

    actions.transport.getState().then(setTransportState);
  }, [isInitialized, isPolling, transportTempoSyncKey]);

  // Wrap transport actions to start/stop polling
  useEffect(() => {
    const originalPlay = actions.transport.play;
    const originalPause = actions.transport.pause;
    const originalSeek = actions.transport.seek;

    actions.transport.play = () => {
      const result = originalPlay();
      result.then(() => setIsPolling(true));
      return result;
    };

    actions.transport.pause = () => {
      const result = originalPause();
      result.then(() => {
        setIsPolling(false);
        actions.transport.getState().then(setTransportState);
      });
      return result;
    };

    actions.transport.seek = (tick: number) => {
      const result = originalSeek(tick);
      result.then(() => {
        actions.transport.getState().then(setTransportState);
      });
      return result;
    };

    return () => {
      actions.transport.play = originalPlay;
      actions.transport.pause = originalPause;
      actions.transport.seek = originalSeek;
    };
  }, []);

  useEffect(() => {
    if (pluginInventoryState.errors.length === 0) {
      lastPluginInventoryErrorSignatureRef.current = "";
      return;
    }

    const signature = pluginInventoryState.errors
      .map((error) => `${error.kind}:${error.message}`)
      .join("\n");

    if (signature === lastPluginInventoryErrorSignatureRef.current) {
      return;
    }

    lastPluginInventoryErrorSignatureRef.current = signature;
    alert(
      `Plugin inventory issue\n\n${pluginInventoryState.errors
        .map((error) => error.message)
        .join("\n\n")}`,
    );
  }, [pluginInventoryState.errors]);

  useEffect(() => {
    let unlistenApp: UnlistenFn | null = null;
    let unlistenAudio: UnlistenFn | null = null;
    let unlistenPluginInventory: UnlistenFn | null = null;
    let unlistenSampler: UnlistenFn | null = null;
    let unlistenMenuSave: UnlistenFn | null = null;
    let unlistenMenuSaveAs: UnlistenFn | null = null;
    let unlistenMenuOpen: UnlistenFn | null = null;
    let unlistenMenuOpenRecent: UnlistenFn | null = null;
    let unlistenMenuNew: UnlistenFn | null = null;
    let unlistenMenuPluginManager: UnlistenFn | null = null;
    let unlistenMenuPreferences: UnlistenFn | null = null;
    let unlistenRecentsChanged: UnlistenFn | null = null;

    (async () => {
      unlistenApp = await listen<AppState>("soloc://state/general", (e) => {
        setAppState(e.payload);
      });

      unlistenAudio = await listen<AudioState>("soloc://state/audio", (e) => {
        setAudioState(e.payload);
      });

      unlistenPluginInventory = await listen<PluginInventoryState>(
        "soloc://state/plugin_inventory",
        (e) => {
          setPluginInventoryState(e.payload);
        },
      );

      unlistenSampler = await listen<SamplerState>(
        "soloc://state/sampler",
        (e) => {
          setSamplerState(e.payload);
        },
      );

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

        unlistenMenuPluginManager = await listen(
          "soloc://menu/plugin-manager",
          () => {
            actions.openPluginManager().catch(console.error);
          },
        );

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
      setIsInitialized(true);

      // Initial menu refresh after initialization - delay to ensure menu is ready
      if (isMainWindow) {
        setTimeout(() => {
          invoke("refresh_menus").catch(console.error);
        }, 100);
      }
    })();

    return () => {
      unlistenApp?.();
      unlistenAudio?.();
      unlistenPluginInventory?.();
      unlistenSampler?.();
      unlistenMenuSave?.();
      unlistenMenuSaveAs?.();
      unlistenMenuOpen?.();
      unlistenMenuOpenRecent?.();
      unlistenMenuNew?.();
      unlistenMenuPluginManager?.();
      unlistenMenuPreferences?.();
      unlistenRecentsChanged?.();
    };
  }, [isMainWindow]);

  return (
    <AppContext.Provider value={appState}>
      <AudioContext.Provider value={audioState}>
        <PluginInventoryContext.Provider value={pluginInventoryState}>
          <SamplerContext.Provider value={samplerState}>
            <TransportContext.Provider value={transportState}>
              {children}
            </TransportContext.Provider>
          </SamplerContext.Provider>
        </PluginInventoryContext.Provider>
      </AudioContext.Provider>
    </AppContext.Provider>
  );
};
