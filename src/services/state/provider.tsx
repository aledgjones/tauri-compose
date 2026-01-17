import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { AppState, Sampler } from "../types";
import { actions } from "../actions";
import { useSampler } from "../use-sampler";
import { FC, PropsWithChildren, useEffect, useState } from "react";
import { AppContext, initialAppState, SamplerContext } from "./context";

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
