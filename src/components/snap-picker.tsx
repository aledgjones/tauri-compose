import { FC } from "react";
import { Select } from "../ui/select";
import { NoteDuration } from "../services/types";
import { mdiFormatHorizontalAlignCenter } from "@mdi/js";

const SNAP_OPTIONS = [
  {
    label: "Quarter",
    value: NoteDuration.Quarter,
  },
  {
    label: "Eighth",
    value: NoteDuration.Eighth,
  },
  {
    label: "Sixteenth",
    value: NoteDuration.Sixteenth,
  },
  {
    label: "Thirty-Second",
    value: NoteDuration.ThirtySecond,
  },
];

type SnapPickerProps = {
  variant: "standard" | "filled";
  label?: string;
  value: NoteDuration;
  onChange: (value: NoteDuration) => void;
};

export const SnapPicker: FC<SnapPickerProps> = ({
  variant,
  label,
  value,
  onChange,
}) => {
  return (
    <Select
      icon={mdiFormatHorizontalAlignCenter}
      label={label}
      variant={variant}
      value={value}
      options={SNAP_OPTIONS}
      onChange={onChange}
    />
  );
};
