import { mdiDelete, mdiMagnify, mdiPlus } from "@mdi/js";
import { FC } from "react";
import { ExpressionMap } from "../../services/types";
import { IconButton } from "../../ui/buttons/icon-button";
import { Input } from "../../ui/input";

type Props = {
  searchTerm: string;
  onSearchTermChange: (value: string) => void;
  expressionMaps: ExpressionMap[];
  selectedExpressionMapId: string | null;
  onSelectExpressionMap: (expressionMapId: string) => void;
  onDeleteExpressionMap: (expressionMapId: string) => void;
  onCreate: () => void;
};

export const ExpressionMapLibraryPane: FC<Props> = ({
  searchTerm,
  onSearchTermChange,
  expressionMaps,
  selectedExpressionMapId,
  onSelectExpressionMap,
  onDeleteExpressionMap,
  onCreate,
}) => {
  return (
    <aside className="flex min-h-0 flex-col overflow-hidden rounded-bl-xl bg-neutral-950/70">
      <div className="flex items-center gap-0.5 bg-neutral-950">
        <Input
          type="text"
          variant="dark"
          leadingIcon={mdiMagnify}
          value={searchTerm}
          onChange={onSearchTermChange}
          placeholder="Search expression maps..."
          className="flex-1"
        />
        <IconButton
          path={mdiPlus}
          variant="interactive"
          onClick={onCreate}
          aria-label="Create"
          title="Create"
        />
      </div>

      <div className="min-h-0 flex-1 overflow-auto pt-0.5">
        {expressionMaps.length === 0 ? (
          <div className="flex h-full w-full items-center justify-center bg-neutral-900 px-6 text-center">
            <div>
              <div className="text-sm text-neutral-200">
                No expression maps match.
              </div>
              <div className="mt-2 text-sm text-neutral-400">
                Adjust the search term or create a new expression map.
              </div>
            </div>
          </div>
        ) : (
          <div className="flex flex-col gap-0.5">
            {expressionMaps.map((expressionMap) => {
              const isSelected = expressionMap.id === selectedExpressionMapId;

              return (
                <div
                  key={expressionMap.id}
                  className={
                    isSelected
                      ? "border-l-4 w-full bg-neutral-800 border-sky-500 transition"
                      : "border-l-4 w-full bg-neutral-900 hover:bg-neutral-800 border-transparent transition"
                  }
                >
                  <div className="flex items-center gap-2 h-9 pr-0.5">
                    <button
                      type="button"
                      onClick={() => onSelectExpressionMap(expressionMap.id)}
                      className="min-w-0 flex-1 text-left pl-3"
                    >
                      <div className="truncate text-sm text-white">
                        {expressionMap.name}
                      </div>
                    </button>
                    {isSelected ? (
                      <IconButton
                        path={mdiDelete}
                        variant="filled"
                        onClick={() => onDeleteExpressionMap(expressionMap.id)}
                        onPointerDown={(event) => {
                          event.stopPropagation();
                        }}
                        aria-label={`Remove expression map ${expressionMap.name}`}
                        title="Remove Expression Map"
                      />
                    ) : null}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </aside>
  );
};
