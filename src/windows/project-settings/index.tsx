import { window } from "@tauri-apps/api";
import { actions, useAppState } from "../../services/state";
import { useEffect } from "preact/hooks";
import { AppBar } from "../../ui/app-bar";
import { Input } from "../../ui/input";

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
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_1fr] grid-cols-[240px_1fr] gap-2 pt-0 p-2 overflow-hidden">
      <AppBar
        className="col-span-2"
        title="Project Settings"
        filename={state.score.meta.title}
      />
      <div className="bg-neutral-900 rounded-lg overflow-auto"></div>
      <div className="bg-neutral-900 rounded-lg p-2 pt-1 overflow-y-auto">
        <Input
          value={state.score.meta.title}
          onChange={actions.score.meta.setTitle}
          label="Title"
        />
        <Input
          value={state.score.meta.subtitle}
          onChange={actions.score.meta.setTitle}
          label="Subtitle"
        />
        <Input
          value={state.score.meta.composer}
          onChange={actions.score.meta.setTitle}
          label="Composer"
        />
        <Input
          value={state.score.meta.arranger}
          onChange={actions.score.meta.setTitle}
          label="Arranger"
        />
        <Input
          value={state.score.meta.lyricist}
          onChange={actions.score.meta.setTitle}
          label="Lyricist"
        />
        <Input
          value={state.score.meta.copyright}
          onChange={actions.score.meta.setTitle}
          label="Copyright"
        />
      </div>
    </div>
  );
};
