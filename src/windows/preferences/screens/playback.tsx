import { useMemo } from "react";
import { actions } from "../../../services/actions";
import { useAudioState } from "../../../services/state";
import { Select } from "../../../ui/select";

export const Playback = () => {
  const audio = useAudioState();

  const options = useMemo(() => {
    return audio.devices.map((device) => ({
      label: device.name,
      value: device.name,
    }));
  }, [audio.devices]);

  return (
    <div className="bg-neutral-900">
      <h3 className="p-2 text-sm font-medium text-white bg-neutral-800">
        Device Setup
      </h3>
      <div className="p-4 gap-2 flex flex-col">
        <Select
          variant="filled"
          label="Output Device"
          value={audio.selected_device}
          onChange={async (device_name) => {
            try {
              await actions.audio.setDevice(device_name);
            } catch (error) {
              alert(error);
            }
          }}
          options={options}
          className="w-1/2"
        />
      </div>
    </div>
  );
};
