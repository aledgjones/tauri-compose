import { getCurrentWindow } from "@tauri-apps/api/window";
import classNames from "classnames";
import { FC, useEffect } from "react";
import { useAppState } from "../services/state";

type Props = { title: string; className?: string };

export const AppBar: FC<Props> = ({ title, className }) => {
  const state = useAppState();

  const filename = state.current_file_path
    ? state.current_file_path.split("/").pop() || "Untitled"
    : "Untitled";

  useEffect(() => {
    const run = async () => {
      const win = getCurrentWindow();
      await win.setTitle(`${title} - ${filename}`);
    };

    run();
  }, [title, filename]);

  return (
    <div
      data-tauri-drag-region
      className={classNames("h-full bg-neutral-900 rounded-t-xl", className)}
    />
  );
};
