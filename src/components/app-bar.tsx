import { getCurrentWindow } from "@tauri-apps/api/window";
import classNames from "classnames";
import { FC, useEffect } from "react";

type Props = { title: string; filename?: string; className?: string };

export const AppBar: FC<Props> = ({ title, filename, className }) => {
  useEffect(() => {
    const run = async () => {
      const win = getCurrentWindow();
      await win.setTitle(`${title} - ${filename || "Untitled"}`);
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
