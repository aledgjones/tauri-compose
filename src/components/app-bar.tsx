import classNames from "classnames";
import { FC } from "react";
import { useAppState } from "../services/state";

type Props = { title: string; className?: string; showFilename?: boolean };

export const AppBar: FC<Props> = ({
  title,
  className,
  showFilename = false,
}) => {
  const state = useAppState();
  const isWindowFocused = state.is_window_focused;

  const filename = state.current_file_path
    ? state.current_file_path.split("/").pop() || "Untitled"
    : "Untitled";

  return (
    <div
      data-tauri-drag-region
      className={classNames(
        "h-full overflow-hidden rounded-t-xl bg-neutral-900 z-200",
        className,
      )}
    >
      <div className="pointer-events-none select-none h-full flex min-w-0 items-center justify-center">
        <span
          data-testid="app-bar-custom-title"
          className={classNames(
            "truncate text-xs font-semibold transition-opacity duration-150",
            isWindowFocused ? "text-neutral-100/80" : "text-neutral-100/35",
          )}
        >
          {title}
        </span>

        {showFilename && (
          <>
            <span
              className={classNames(
                "mx-2 h-1 w-1 rounded-full transition-opacity duration-150",
                isWindowFocused ? "bg-neutral-500/80" : "bg-neutral-500/35",
              )}
            />

            <span
              data-testid="app-bar-filename"
              className={classNames(
                "truncate text-xs font-semibold transition-opacity duration-150",
                isWindowFocused ? "text-neutral-300/75" : "text-neutral-300/35",
              )}
            >
              {filename}
            </span>
          </>
        )}
      </div>
    </div>
  );
};
