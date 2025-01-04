import { window } from "@tauri-apps/api";
import { useAppState } from "../../services/state";
import { useEffect, useState } from "preact/hooks";
import { AppBar } from "../../components/app-bar";
import { Menu, MenuItem } from "../../ui/menu";
import { Meta } from "./screens/meta";
import { NotFound } from "../../components/not-found";

enum MenuOption {
  General = 0,
}

const options: MenuItem<MenuOption>[] = [
  {
    label: "General",
    value: MenuOption.General,
  },
];

const screens = new Map([[MenuOption.General, Meta]]);

export const Main = () => {
  const state = useAppState();
  const [selection, setSelection] = useState(MenuOption.General);

  const Content = screens.get(selection) || NotFound;

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
      <div className="rounded-lg bg-neutral-900 overflow-auto">
        <Menu selection={selection} items={options} onSelect={setSelection} />
      </div>
      <div className="rounded-lg bg-neutral-900 overflow-auto">
        <Content />
      </div>
    </div>
  );
};
