import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useMemo, useState } from "react";
import { AppBar } from "../../components/app-bar";
import { useExpressionMapLibraryState } from "../../services/state";

const readInitialExpressionMapId = () => {
  const params = new URLSearchParams(window.location.search);
  return params.get("expression_map_id");
};

export const Main = () => {
  const expressionMapLibrary = useExpressionMapLibraryState();
  const [selectedExpressionMapId, setSelectedExpressionMapId] = useState<
    string | null
  >(() => readInitialExpressionMapId());

  useEffect(() => {
    const show = async () => {
      await getCurrentWindow().show();
    };

    show();
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    listen<{ expression_map_id: string }>(
      "soloc://expression-map/select",
      (event) => {
        setSelectedExpressionMapId(event.payload.expression_map_id);
      },
    ).then((cleanup) => {
      unlisten = cleanup;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  const selectedExpressionMap = useMemo(() => {
    if (selectedExpressionMapId == null) {
      return null;
    }

    return expressionMapLibrary.by_id[selectedExpressionMapId] ?? null;
  }, [expressionMapLibrary.by_id, selectedExpressionMapId]);

  return (
    <div className="grid h-dvh grid-rows-[28px_1fr] gap-0.5 overflow-hidden bg-neutral-950 p-0.75 text-white">
      <AppBar title="Expression Map" />

      <div className="flex min-h-0 flex-col gap-0.5 overflow-auto rounded-b-xl bg-neutral-900 p-5">
        <div className="text-xs uppercase tracking-[0.12em] text-neutral-500">
          Selected Map
        </div>

        {selectedExpressionMap ? (
          <>
            <div className="text-lg text-neutral-50">
              {selectedExpressionMap.name}
            </div>
            <div className="text-sm text-neutral-400">
              {selectedExpressionMap.rules.length} rules in library asset.
            </div>
            <div className="rounded-xl border border-sky-900/60 bg-neutral-950/70 px-4 py-3 text-sm text-neutral-300">
              Story 6.2 keeps this shell intentionally minimal. Story 6.3 will
              expand it into the full Expression Map editor and library
              workflow.
            </div>
          </>
        ) : (
          <div className="rounded-xl border border-neutral-800 bg-neutral-950/70 px-4 py-3 text-sm text-neutral-400">
            No Expression Map is currently selected.
          </div>
        )}
      </div>
    </div>
  );
};
