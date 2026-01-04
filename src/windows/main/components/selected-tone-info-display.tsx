import { FC } from "react";
import { useSelectedToneInfo } from "../hooks/use-selected-tone-info";
import { TokenizedText } from "../../../ui/text";

export const SelectedToneInfoDisplay: FC = () => {
  const selectedToneInfo = useSelectedToneInfo();

  return (
    <p className="text-white text-xs bg-neutral-900 h-full flex items-center px-2 rounded-bl-xl truncate">
      {selectedToneInfo ? (
        <TokenizedText>
          {`${selectedToneInfo.noteName}, ${selectedToneInfo.instrumentName}, ${selectedToneInfo.trackLabel}`}
        </TokenizedText>
      ) : (
        "No selection"
      )}
    </p>
  );
};
