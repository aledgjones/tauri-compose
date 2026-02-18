import { FC, useMemo } from "react";
import classNames from "classnames";
import { useAppState } from "../../../../../services/state";
import { actions } from "../../../../../services/actions";
import { Articulation, Tone } from "../../../../../services/types";
import { BravuraIconButton } from "../../../../../ui/buttons/bravura-icon-button";
import { SMuFL } from "../../../../../ui/buttons/smufl";

type LeftPanelProps = { spanRow: boolean };

const ARTICULATIONS: { articulation: Articulation; glyph: SMuFL; title: string; offset?: number }[] = [
  { articulation: Articulation.Staccato, glyph: SMuFL.Staccato, title: "Staccato" },
  { articulation: Articulation.Tenuto, glyph: SMuFL.Tenuto, title: "Tenuto" },
  { articulation: Articulation.Staccatissimo, glyph: SMuFL.Staccatissimo, title: "Staccatissimo", offset: 4 },
];

export const LeftPanel: FC<LeftPanelProps> = ({ spanRow }) => {
  const state = useAppState();

  const selection = state.ui.sequencer.selection;

  const tone = useMemo<Tone | null>(() => {
    if (!selection || selection.type !== "Tone") {
      return null;
    }

    const track = state.score.tracks.by_id[selection.track_id];
    if (!track) {
      return null;
    }

    const entry = track.entries.by_id[selection.tone_id];
    if (!entry || entry.type !== "Tone") {
      return null;
    }

    return entry as Tone;
  }, [selection, state.score.tracks.by_id]);



  return (
    <div
      className={classNames("bg-neutral-900 flex flex-col items-center", {
        "row-span-2": spanRow,
      })}
    >
      {ARTICULATIONS.map((s) => (
        <BravuraIconButton
          key={s.title}
          variant="interactive"
          bravura={s.glyph}
          title={s.title}
          offset={s.offset}
          onClick={() => {
            if (selection?.type !== "Tone" || !tone) return;
            actions.score.tone.setArticulation(selection.track_id, selection.tone_id, s.articulation);
          }}
          isDisabled={!tone}
          isSelected={!!tone?.articulations.includes(s.articulation)}
        />
      ))}
    </div>
  );
};
