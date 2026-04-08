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
  if (presentation.kind !== "needs-attention") {
    return null;
  }

  return (
    <span
      role="img"
      aria-label={`Playback ${presentation.label.toLowerCase()}`}
      title={presentation.hover_title ?? undefined}
      className={classNames(
        "inline-flex shrink-0 items-center",
        presentation.tone_class_name,
        className,
      )}
    >
      <Icon
        path={presentation.icon}
        size={0.8}
        className={presentation.icon_class_name}
      />
    </span>
  );
};
