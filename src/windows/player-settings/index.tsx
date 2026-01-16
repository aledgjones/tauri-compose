import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import { AppBar } from "../../components/app-bar";
import { Menu, MenuItem } from "../../ui/menu";
import { General } from "./screens/general";

enum MenuOption {
  General = 0,
}

const options: MenuItem<MenuOption>[] = [
  {
    label: "General",
    value: MenuOption.General,
  },
];

export const Main = () => {
  const [selection, setSelection] = useState(MenuOption.General);

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_1fr] grid-cols-[240px_1fr] gap-0.5 p-0.75 overflow-hidden">
      <AppBar className="col-span-2" title="Player Settings" />
      <div className="overflow-auto rounded-bl-xl">
        <Menu selection={selection} items={options} onSelect={setSelection} />
      </div>
      <div className="overflow-auto flex flex-col gap-0.5 rounded-br-xl">
        {selection === MenuOption.General && <General />}
      </div>
    </div>
  );
};
