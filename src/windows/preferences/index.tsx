import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useState } from "react";
import { AppBar } from "../../components/app-bar";
import { Menu, MenuItem } from "../../ui/menu";
import { NoteInputAndEditing } from "./screens/note-input-and-editing";
import { DeviceSetup } from "./screens/device-setup";

enum MenuOption {
  Playback = 0,
  PlaybackDeviceSetup,
  NoteInputAndEditing,
  NoteInputAndEditingAuditioning,
}

const options: MenuItem<MenuOption>[] = [
  {
    label: "Playback",
    value: MenuOption.Playback,
    sections: [
      {
        label: "Device Setup",
        value: MenuOption.PlaybackDeviceSetup,
      },
    ],
  },
  {
    label: "Note Input and Editing",
    value: MenuOption.NoteInputAndEditing,
    sections: [
      {
        label: "Auditioning",
        value: MenuOption.NoteInputAndEditingAuditioning,
      },
    ],
  },
];

export const Main = () => {
  const [selection, setSelection] = useState(MenuOption.Playback);

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_1fr] grid-cols-[240px_1fr] gap-0.5 p-0.75 overflow-hidden">
      <AppBar className="col-span-2" title="Settings" />
      <div className="overflow-auto rounded-bl-xl">
        <Menu selection={selection} items={options} onSelect={setSelection} />
      </div>
      <div className="overflow-auto flex flex-col gap-0.5 rounded-br-xl">
        {selection === MenuOption.Playback && <DeviceSetup />}
        {selection === MenuOption.NoteInputAndEditing && (
          <NoteInputAndEditing />
        )}
      </div>
    </div>
  );
};
