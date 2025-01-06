import { window } from "@tauri-apps/api";
import { useAppState } from "../../services/state";
import { useEffect } from "preact/hooks";
import { AppBar } from "../../components/app-bar";
import { UnlistenFn } from "@tauri-apps/api/event";

export const Main = () => {
  const state = useAppState();

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    (async () => {
      const win = window.getCurrentWindow();
      unlisten = await win.listen("tauri://blur", async () => {
        await win.close();
      });
      await win.show();
      await win.center();
    })();

    return () => {
      unlisten?.();
    };
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_1fr] grid-cols-[1fr_1fr_1fr] gap-2 pt-0 p-2 overflow-hidden">
      <AppBar
        className="col-span-3"
        title="Instrument Picker"
        filename={state.score.meta.title}
      />
      <div className="rounded-lg bg-neutral-900 overflow-auto">
        <p>One</p>
      </div>
      <div className="rounded-lg bg-neutral-900 overflow-auto">
        <p>Two</p>
      </div>
      <div className="rounded-lg bg-neutral-900 overflow-auto">
        <p>Three</p>
      </div>
    </div>
  );
};
