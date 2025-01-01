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
    <div className="bg-neutral-950 flex flex-col">
      <AppBar title="Project Settings" filename={state.score.meta.title} />
      <div class="h-full p-2 overflow-auto">
        <div className="flex-1 bg-neutral-900 rounded-lg p-2 pt-1 mb-2">
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
        <div className="bg-neutral-900 rounded-lg p-2 mb-2">
          <p className="text-xs text-neutral-500">created</p>
          <p className="text-sm text-white">{state.score.meta.created}</p>
        </div>
      </div>
    </div>
  );
};
