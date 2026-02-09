import { useMemo } from "react";
import { actions } from "../../../services/actions";
import { useAudioState } from "../../../services/state";
import { Select } from "../../../ui/select";
import { IconButton } from "../../../ui/icon-button";
import { mdiRefresh } from "@mdi/js";

export const Playback = () => {
  const audio = useAudioState();

  const deviceOptions = useMemo(() => {
    return audio.devices.map((device) => ({
      label: device.name,
      value: device.id,
    }));
  }, [audio.devices]);

  const sampleRateOptions = useMemo(() => {
    const device = audio.devices.find((d) => d.id === audio.active_device_id);

    if (!device) {
      return [];
    }

    return device.sample_rates.map((rate) => ({
      label: rate.toString(),
      value: rate,
    }));
  }, [audio.active_device_id, audio.devices]);
  return (
    <div className="bg-neutral-900">
      <h3 className="p-2 text-sm font-medium text-white bg-neutral-800">
        Device Setup
      </h3>
      <div className="p-4 gap-2 flex flex-col">
        <div className="flex gap-2 items-end">
          <Select
            variant="filled"
            label="Output Device"
            value={audio.active_device_id}
            onChange={async (id) => {
              try {
                if (audio.active_device_id === id) {
                  return;
                }

                await actions.audio.setDevice(id);
              } catch (error) {
                alert(error);
              }
            }}
            options={deviceOptions}
            className="w-1/2"
          />
          <IconButton
            variant="filled"
            path={mdiRefresh}
            onClick={async () => {
              try {
                await actions.audio.refreshDevices();
              } catch (error) {
                alert(error);
              }
            }}
          />
        </div>
        <Select
          variant="filled"
          label="Sample Rate"
          value={audio.current_sample_rate}
          onChange={async (sampleRate) => {
            try {
              await actions.audio.setDeviceSampleRate(sampleRate);
            } catch (error) {
              alert(error);
            }
          }}
          options={sampleRateOptions}
          className="w-1/2"
        />
      </div>
    </div>
  );
};
