import { window } from "@tauri-apps/api";
import { useAppState } from "../../services/state";
import { useEffect, useMemo, useState } from "preact/hooks";
import { AppBar } from "../../components/app-bar";
import { UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import classNames from "classnames";
import { Icon } from "../../ui/icon";
import { mdiChevronRight } from "@mdi/js";

type InstrumentColumns = [string[], string[], string[]];

type InstrumentIdPath = {
  id: string;
  path: string[];
};

export const Main = () => {
  const state = useAppState();

  const [instruments, setInstruments] = useState<InstrumentIdPath[] | null>(
    null
  );
  const [selection, setSelection] = useState<InstrumentIdPath | null>(null);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    (async () => {
      const win = window.getCurrentWindow();
      unlisten = await win.listen("tauri://blur", async () => {
        await win.close();
      });
      await win.show();
      await win.center();
    })();

    (async () => {
      const result = await invoke<InstrumentIdPath[]>(
        "get_all_instrument_definitions"
      );
      setInstruments(result);
      setSelection(result[0]);
    })();

    return () => {
      unlisten?.();
    };
  }, []);

  const columns: InstrumentColumns = useMemo(() => {
    if (!instruments || !selection) return [[], [], []];

    const [first, second, third] = [
      new Set<string>(),
      new Set<string>(),
      new Set<string>(),
    ];

    instruments.forEach(({ path }) => {
      first.add(path[0]);

      if (path[0] === selection.path[0]) {
        second.add(path[1]);

        if (path[2] && path[1] === selection.path[1]) {
          third.add(path[2]);
        }
      }
    });

    return [Array.from(first), Array.from(second), Array.from(third)];
  }, [instruments, selection]);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_1fr] grid-cols-[1fr_1fr_1fr] gap-2 pt-0 p-2 overflow-hidden">
      <AppBar
        className="col-span-3"
        title="Instrument Picker"
        filename={state.score.meta.title}
      />
      {columns.map((column, i) => {
        return (
          <div key={i} className="bg-neutral-900 rounded-lg flex-1 p-2">
            {column.map((item) => {
              const isSelected = item === selection?.path[i];
              const isFinal = !(
                isSelected &&
                columns[i + 1] &&
                columns[i + 1].length > 0
              );

              return (
                <button
                  className={classNames(
                    "flex items-center rounded-lg p-2 pl-3 w-full text-left text-sm h-[40px]",
                    {
                      "bg-sky-400 text-black": isSelected,
                      "text-white": !isSelected,
                    }
                  )}
                  key={item}
                  onClick={() => {
                    if (!isSelected) {
                      const path = [
                        ...(selection?.path.slice(0, i) || []),
                        item,
                      ];

                      const def = instruments?.find((instrument) =>
                        path.every(
                          (segment, index) => segment === instrument.path[index]
                        )
                      );

                      setSelection(def ?? instruments?.[0] ?? null);
                    }
                  }}
                >
                  <p className="flex-1">{item}</p>
                  {!isFinal && <Icon path={mdiChevronRight} />}
                </button>
              );
            })}
          </div>
        );
      })}
    </div>
  );
};
