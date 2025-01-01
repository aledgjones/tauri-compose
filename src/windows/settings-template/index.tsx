import { window } from "@tauri-apps/api";
import { useAppState } from "../../services/state";
import { useEffect } from "preact/hooks";
import { AppBar } from "../../ui/app-bar";
import { Panel } from "../main/screens/setup/components/panel";

export const Main = () => {
  const state = useAppState();

  useEffect(() => {
    const show = async () => {
      const win = window.getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 flex flex-col">
      <AppBar title="Project Settings" filename={state.score.meta.title} />
      <div class="h-full p-2">
        <div class="h-full grid grid-cols-[240px_1fr] gap-2">
          <div className="bg-neutral-900 rounded-lg"></div>
          <div className="bg-neutral-900 rounded-lg"></div>
        </div>
      </div>
    </div>
  );
};
