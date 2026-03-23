import classNames from "classnames";
import { FC } from "react";
import { Icon } from "../../../../../ui/icon";
import { PlaybackReadinessPresentation } from "./playback-readiness-presentation";

type PlaybackReadinessBadgeProps = {
  presentation: PlaybackReadinessPresentation;
  className?: string;
};

export const PlaybackReadinessBadge: FC<PlaybackReadinessBadgeProps> = ({
  presentation,
  className,
}) => {
  return (
    <div
      role="status"
      aria-label={`Playback status: ${presentation.label}`}
      className={classNames(
        "inline-flex shrink-0 items-center gap-1.5 rounded-full border pl-1.5 pr-2.5 py-1 text-[11px] uppercase tracking-[0.16em]",
        presentation.tone_class_name,
        className,
      )}
    >
      <Icon
        path={presentation.icon}
        size={0.7}
        className={presentation.icon_class_name}
      />
      <span>{presentation.label}</span>
    </div>
  );
};
