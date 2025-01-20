import { getCurrentWindow } from "@tauri-apps/api/window";
import classNames from "classnames";
import { FC, useEffect, useState } from "react";

type Props = { title: string; filename?: string; className?: string };

export const AppBar: FC<Props> = ({ title, filename, className }) => {
  const [isFocused, setIsFocused] = useState(true);

  useEffect(() => {
    const win = getCurrentWindow();
    win.listen("tauri://blur", () => {
      setIsFocused(false);
    });
    win.listen("tauri://focus", () => {
      setIsFocused(true);
    });
  }, []);

  return (
    <div
      data-tauri-drag-region
      className={classNames(
        "h-[28px] flex-shrink-0 flex items-center px-20 bg-neutral-900",
        className
      )}
    >
      <p
        className={classNames(
          "flex-1 text-center text-xs pointer-events-none font-bold line-clamp-1 break-all",
          {
            "text-white": isFocused,
            "text-neutral-600": !isFocused,
          }
        )}
      >
        {title} -{" "}
        <span className={classNames({ italic: !filename })}>
          {filename || "Untitled"}
        </span>
      </p>
    </div>
  );
};
