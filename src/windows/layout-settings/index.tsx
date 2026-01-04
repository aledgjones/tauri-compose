import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppState } from "../../services/state";
import { FC, useEffect, useState } from "react";
import { AppBar } from "../../components/app-bar";
import { Menu, MenuItem } from "../../ui/menu";
import { NotFound } from "../../components/not-found";
import { LayoutType } from "../../services/types";

import { BracketsAndBraces } from "./screens/brackets-and-braces";
import { Barlines } from "./screens/barlines";
import { NoteSpacing } from "./screens/note-spacing";
import { PageSettings } from "./screens/page-settings";

enum MenuOption {
  Barlines = 0,
  BarlinesSystemicBarlines,
  BracketsAndBraces,
  BracketsAndBracesBrackets,
  BracketsAndBracesBracketGroups,
  NoteSpacing,
  PageSettings,
  PageSettingStaveSize,
  PageSettingPageMargins,
  PageSettingMusicFrameMargins,
}

const options: MenuItem<MenuOption>[] = [
  {
    label: "Barlines",
    value: MenuOption.Barlines,
    sections: [
      {
        label: "Systemic Barlines",
        value: MenuOption.BarlinesSystemicBarlines,
      },
    ],
  },
  {
    label: "Brackets and Braces",
    value: MenuOption.BracketsAndBraces,
    sections: [
      { label: "Brackets", value: MenuOption.BracketsAndBracesBrackets },
      {
        label: "Bracketed Groups",
        value: MenuOption.BracketsAndBracesBracketGroups,
      },
    ],
  },
  {
    label: "Note Spacing",
    value: MenuOption.NoteSpacing,
  },
  {
    label: "Page Settings",
    value: MenuOption.PageSettings,
    sections: [
      { label: "Space Size", value: MenuOption.PageSettingStaveSize },
      { label: "Page Margins", value: MenuOption.PageSettingPageMargins },
      {
        label: "Music Frame Margins",
        value: MenuOption.PageSettingMusicFrameMargins,
      },
    ],
  },
];

const screens = new Map([
  [MenuOption.Barlines, Barlines],
  [MenuOption.BracketsAndBraces, BracketsAndBraces],
  [MenuOption.NoteSpacing, NoteSpacing],
  [MenuOption.PageSettings, PageSettings],
]);

export const Main = () => {
  const state = useAppState();
  const [selection, setSelection] = useState(MenuOption.Barlines);

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
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_1fr] grid-cols-[240px_1fr] overflow-hidden gap-0.5 p-0.75">
      <AppBar
        className="col-span-2"
        title="Layout Settings"
        filename={state.score.meta.title}
      />
      <div className="bg-neutral-900 overflow-auto rounded-bl-xl">
        <Menu selection={selection} items={options} onSelect={setSelection} />
      </div>
      <div className="overflow-auto flex flex-col gap-0.5 rounded-br-xl">
        <Content layoutType={LayoutType.FullScore} />
      </div>
    </div>
  );
};
