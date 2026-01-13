import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import { AppBar } from "../../components/app-bar";
import { Menu, MenuItem } from "../../ui/menu";
import { Meta } from "./screens/general";
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
  const [selection, setSelection] = useState(MenuOption.General);

  const Content = screens.get(selection) || NotFound;

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_1fr] grid-cols-[240px_1fr] gap-0.5 p-0.75 overflow-hidden">
      <AppBar className="col-span-2" title="Project Settings" />
      <div className="overflow-auto rounded-bl-xl">
        <Menu selection={selection} items={options} onSelect={setSelection} />
      </div>
      <div className="overflow-auto flex flex-col gap-0.5 rounded-br-xl">
        <Content />
      </div>
    </div>
  );
};
