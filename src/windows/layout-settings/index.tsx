import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppState } from "../../services/state";
import { FC, useEffect, useState } from "react";
import { AppBar } from "../../components/app-bar";
import { Menu, MenuItem } from "../../ui/menu";
import { BracketsAndBraces } from "./screens/brackets-and-braces";
import { NotFound } from "../../components/not-found";
import { LayoutType } from "../../services/types";

enum MenuOption {
  BracketsAndBraces,
  NoteSpacing,
  PageSettings,
  PageSettingStaveSize,
  PageSettingPageMargins,
}

const options: MenuItem<MenuOption>[] = [
  {
    label: "Brackets and Braces",
    value: MenuOption.BracketsAndBraces,
  },
  {
    label: "Note Spacing",
    value: MenuOption.NoteSpacing,
  },
  {
    label: "Page Settings",
    value: MenuOption.PageSettings,
    sections: [
      { label: "Stave Size", value: MenuOption.PageSettingStaveSize },
      { label: "Page Margins", value: MenuOption.PageSettingPageMargins },
    ],
  },
];

const screens = new Map([[MenuOption.BracketsAndBraces, BracketsAndBraces]]);

export const Main = () => {
  const state = useAppState();
  const [selection, setSelection] = useState(MenuOption.PageSettings);

  const Content: FC<{ layoutType: LayoutType }> =
    screens.get(selection) || NotFound;

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_1fr] grid-cols-[240px_1fr] overflow-hidden gap-2">
      <AppBar
        className="col-span-2"
        title="Layout Settings"
        filename={state.score.meta.title}
      />
      <div className="bg-neutral-900 overflow-auto ml-2 mb-2">
        <Menu selection={selection} items={options} onSelect={setSelection} />
      </div>
      <div className=" bg-neutral-900 overflow-auto mr-2 mb-2">
        <Content layoutType={LayoutType.FullScore} />
      </div>
    </div>
  );
};
