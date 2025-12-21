import { useEffect } from "react";
import { sampler } from "../sampler";
import { Expression } from "../sampler/types";
import { actions } from "./actions";
import { AppState, Sampler } from "./types";
import { useAsync } from "./utils";

type SamplerModule = () => Promise<{ default: any }>;

/**
 * Hook that manages sampler initialization and instrument loading (self-contained)
 */
export function useSampler(appState: AppState, samplerState: Sampler) {
  // Load instrument patches when instruments are added
  useAsync(async () => {
    const allInstruments = Object.values(appState.score.instruments.by_id);
    const allPlayers = Object.values(appState.score.players.by_id);

    for (const instrument of allInstruments) {
      const player = allPlayers.find((player) => {
        return player.instrument_ids.includes(instrument.id);
      });

      if (!player) {
        continue;
      }

      const instrumentKey = instrument.id.toString();

      if (sampler.hasInstrument(instrumentKey)) {
        continue;
      }

      const samplerInstance = sampler.createSampler(instrumentKey);

      const patches = await actions.getInstrumentPatches(
        instrument.definition_id,
        player.player_type,
      );

      const patchModules = import.meta.glob("../assets/patches/*/*.json");

      for (const [expression, patchPath] of Object.entries(patches)) {
        const expr = parseInt(expression) as Expression;

        const key = `../assets/patches/${patchPath}`;
        const module = patchModules[key] as SamplerModule | undefined;

        if (module) {
          const data = await module();
          const patchData = data.default;
          await samplerInstance.load(expr, patchData, () => {});
        } else {
          console.warn(`Patch not found: ${key}`);
        }
      }
    }
  }, [appState.score.instruments, appState.score.players]);

  // Update sampler state when samplerState changes
  useEffect(() => {
    Object.entries(samplerState).forEach(([instrumentId, state]) => {
      if (sampler.hasInstrument(instrumentId)) {
        sampler.volume(instrumentId, state.volume / 127.0);
        if (state.mute) {
          sampler.mute(instrumentId);
        } else {
          sampler.unmute(instrumentId);
        }
        if (state.solo) {
          sampler.solo(instrumentId);
        } else {
          sampler.unsolo(instrumentId);
        }
      }
    });
  }, [samplerState]);
}
