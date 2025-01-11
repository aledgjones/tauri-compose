import { window } from "@tauri-apps/api";
import { useAppState } from "../../services/state";
import { useEffect, useMemo, useState } from "preact/hooks";
import { AppBar } from "../../components/app-bar";
import { emit } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import classNames from "classnames";
import { Icon } from "../../ui/icon";
import { mdiChevronRight } from "@mdi/js";
import { Button } from "../../ui/button";

type InstrumentDefinition = {
  id: string;
  path: string[];
};

type InstrumentColumns = [string[], string[], string[]];

type InstrumentItemButtonProps = {
  item: string;
  isSelected: boolean;
  isFinal: boolean;
  onClick: () => void;
};

const InstrumentItemButton = ({
  item,
  isSelected,
  isFinal,
  onClick,
}: InstrumentItemButtonProps) => {
  return (
    <button
      className={classNames(
        "flex items-center rounded-lg p-2 pl-3 w-full text-left text-sm h-[40px]",
        {
          "bg-sky-400 text-black": isSelected,
          "text-white": !isSelected,
        }
      )}
      onClick={onClick}
    >
      <p className="flex-1">{item}</p>
      {!isFinal && <Icon path={mdiChevronRight} />}
    </button>
  );
};

type InstrumentColumnProps = {
  column: string[];
  columnIndex: number;
  selection: InstrumentDefinition;
  onSelect: (path: string[]) => void;
  columns: InstrumentColumns;
};

const InstrumentColumn = ({
  column,
  columnIndex,
  selection,
  onSelect,
  columns,
}: InstrumentColumnProps) => {
  return (
    <div className="bg-neutral-900 rounded-lg flex-1 p-2 overflow-auto">
      {column.map((item) => {
        const isSelected = item === selection.path[columnIndex];
        const isFinal = !(isSelected && columns[columnIndex + 1]?.length > 0);

        return (
          <InstrumentItemButton
            key={item}
            item={item}
            isSelected={isSelected}
            isFinal={isFinal}
            onClick={() => {
              const path = [...selection.path.slice(0, columnIndex), item];
              onSelect(path);
            }}
          />
        );
      })}
    </div>
  );
};

const calculateColumns = (
  instruments: InstrumentDefinition[],
  selection: InstrumentDefinition | null
): InstrumentColumns => {
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
};

export const Main = () => {
  const state = useAppState();
  const [instruments, setInstruments] = useState<InstrumentDefinition[]>([]);
  const [selection, setSelection] = useState<InstrumentDefinition>({
    id: "brass.bass-trombone",
    path: ["Brass", "Bass Trombone"],
  });

  useEffect(() => {
    (async () => {
      const win = window.getCurrentWindow();
      await win.show();
      await win.center();
    })();

    (async () => {
      const result = await invoke<InstrumentDefinition[]>(
        "score_instruments_get_all_definitions"
      );
      setInstruments(result);
      setSelection(result[0]);
    })();
  }, []);

  const columns = useMemo(
    () => calculateColumns(instruments, selection),
    [instruments, selection]
  );

  const handleSelect = (path: string[]) => {
    const selectedInstrument = instruments.find((instrument) =>
      path.every((segment, index) => segment === instrument.path[index])
    );
    setSelection(selectedInstrument!);
  };

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_1fr_36px] grid-cols-[1fr_1fr_1fr] gap-2 pt-0 p-2 overflow-hidden">
      <AppBar
        className="col-span-3"
        title="Instrument Picker"
        filename={state.score.meta.title}
      />
      {columns.map((column, i) => (
        <InstrumentColumn
          key={column[0] ?? `empty-${i}`}
          column={column}
          columnIndex={i}
          selection={selection}
          onSelect={handleSelect}
          columns={columns}
        />
      ))}
      <div className="flex col-span-3 justify-end">
        <Button
          onClick={() => {
            emit("solocompose://select-instrument", {
              instrument_id: selection?.id,
            });
          }}
        >
          Select
        </Button>
      </div>
    </div>
  );
};
