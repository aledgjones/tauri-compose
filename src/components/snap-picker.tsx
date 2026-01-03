import { FC } from "react";
import { Select } from "../ui/select";
import { NoteDuration } from "../services/types";
import { mdiFormatHorizontalAlignCenter } from "@mdi/js";

const SNAP_OPTIONS = [
  {
    label: "Quarter",
    value: NoteDuration.Quarter,
    icon: mdiFormatHorizontalAlignCenter,
  },
  {
    label: "Eighth",
    value: NoteDuration.Eighth,
    icon: mdiFormatHorizontalAlignCenter,
  },
  {
    label: "Sixteenth",
    value: NoteDuration.Sixteenth,
    icon: mdiFormatHorizontalAlignCenter,
  },
  {
    label: "Thirty-Second",
    value: NoteDuration.ThirtySecond,
    icon: mdiFormatHorizontalAlignCenter,
  },
];

type SnapPickerProps = {
  label?: string;
  value: NoteDuration;
  onChange: (value: NoteDuration) => void;
};

export const SnapPicker: FC<SnapPickerProps> = ({ label, value, onChange }) => {
  return (
    <Select
      label={label}
      variant="standard"
      value={value}
      options={SNAP_OPTIONS}
      onChange={onChange}
    />
  );
};
