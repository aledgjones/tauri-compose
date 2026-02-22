import { FC, useMemo } from "react";
import classNames from "classnames";
import { useAppState } from "../../../../../services/state";
import { actions } from "../../../../../services/actions";
import {
  Articulation,
  Tone,
  Accidental,
  NoteDuration,
} from "../../../../../services/types";
import { BravuraIconButton } from "../../../../../ui/buttons/bravura-icon-button";
import { SMuFL } from "../../../../../ui/buttons/smufl";
import { noteDurationToTicks } from "../../../../../services/utils";

type LeftPanelProps = { spanRow: boolean };

const ARTICULATIONS: {
  articulation: Articulation;
  glyph: SMuFL;
  title: string;
  offset?: number;
}[] = [
    {
      articulation: Articulation.Staccatissimo,
      glyph: SMuFL.Staccatissimo,
      title: "Staccatissimo",
      offset: 3,
    },
    {
      articulation: Articulation.Staccato,
      glyph: SMuFL.Staccato,
      title: "Staccato",
    },
    {
      articulation: Articulation.StaccatoTenuto,
      glyph: SMuFL.StaccatoTenuto,
      title: "Staccato Tenuto",
      offset: 3,
    },
    {
      articulation: Articulation.Tenuto,
      glyph: SMuFL.Tenuto,
      title: "Tenuto",
      offset: -1,
    },
  ];

const ACCENTUATIONS: {
  articulation: Articulation;
  glyph: SMuFL;
  title: string;
  offset?: number;
}[] = [
    {
      articulation: Articulation.Accent,
      glyph: SMuFL.Accent,
      title: "Accent",
      offset: 3,
    },
    {
      articulation: Articulation.Marcato,
      glyph: SMuFL.Marcato,
      title: "Marcato",
      offset: 3,
    },
  ];

const STRESSES: {
  articulation: Articulation;
  glyph: SMuFL;
  title: string;
  offset?: number;
}[] = [
    {
      articulation: Articulation.Stressed,
      glyph: SMuFL.Stressed,
      title: "Stressed",
      offset: 3,
    },
    {
      articulation: Articulation.Unstressed,
      glyph: SMuFL.Unstressed,
      title: "Unstressed",
      offset: 3,
    },
  ];

const ACCIDENTALS: {
  accidental: Accidental;
  glyph: SMuFL;
  title: string;
  offset?: number;
}[] = [
    {
      accidental: Accidental.Natural,
      glyph: SMuFL.Natural,
      title: "Natural",
      offset: -2,
    },
    { accidental: Accidental.Flat, glyph: SMuFL.Flat, title: "Flat", offset: 3 },
    {
      accidental: Accidental.Sharp,
      glyph: SMuFL.Sharp,
      title: "Sharp",
      offset: -2,
    },
  ];

const DURATIONS: {
  duration: number;
  glyph: SMuFL;
  title: string;
  offset?: number;
}[] = [
    {
      duration: noteDurationToTicks(NoteDuration.Whole),
      glyph: SMuFL.WholeNote,
      title: "Whole",
      offset: -1,
    },
    {
      duration: noteDurationToTicks(NoteDuration.Half),
      glyph: SMuFL.HalfNote,
      title: "Half",
      offset: 6,
    },
    {
      duration: noteDurationToTicks(NoteDuration.Quarter),
      glyph: SMuFL.QuarterNote,
      title: "Quarter",
      offset: 6,
    },
    {
      duration: noteDurationToTicks(NoteDuration.Eighth),
      glyph: SMuFL.EighthNote,
      title: "Eighth",
      offset: 6,
    },
    {
      duration: noteDurationToTicks(NoteDuration.Sixteenth),
      glyph: SMuFL.SixteenthNote,
      title: "Sixteenth",
      offset: 6,
    },
    {
      duration: noteDurationToTicks(NoteDuration.ThirtySecond),
      glyph: SMuFL.ThirtySecondNote,
      title: "Thirty-Second",
      offset: 7,
    },
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
      className={classNames("flex flex-col gap-0.5", { "row-span-2": spanRow })}
    >
      <div className="bg-neutral-900 grid grid-cols-2 grid-rows-2 gap-0.5">
        {DURATIONS.map(({ duration, glyph, title, offset }) => (
          <BravuraIconButton
            key={duration}
            variant="interactive"
            bravura={glyph}
            title={title}
            offset={offset}
            size={24}
            className="h-12"
            onClick={() => {
              if (selection?.type !== "Tone" || !tone) return;
              actions.score.tone.setDuration(
                selection.track_id,
                selection.tone_id,
                duration,
              );
            }}
            isDisabled={!tone}
            isSelected={tone?.duration === duration}
          />
        ))}
      </div>

      <div className="bg-neutral-900 flex">
        {ACCIDENTALS.map((a) => (
          <BravuraIconButton
            key={a.title}
            variant="interactive"
            bravura={a.glyph}
            title={a.title}
            offset={a.offset}
            onClick={() => {
              if (selection?.type !== "Tone" || !tone) return;
              actions.score.tone.setAccidental(
                selection.track_id,
                selection.tone_id,
                a.accidental,
              );
            }}
            isDisabled={!tone}
            isSelected={tone?.pitch.accidental === a.accidental}
          />
        ))}
      </div>

      <div className="bg-neutral-900 grid grid-cols-2 grid-rows-2 gap-0.5">
        {ARTICULATIONS.map((s, i) => (
          <BravuraIconButton
            key={s.title}
            variant="interactive"
            bravura={s.glyph}
            title={s.title}
            offset={s.offset}
            className={classNames({ "mb-0.5": i == 1 })}
            onClick={() => {
              if (selection?.type !== "Tone" || !tone) return;
              actions.score.tone.setArticulation(
                selection.track_id,
                selection.tone_id,
                s.articulation,
              );
            }}
            isDisabled={!tone}
            isSelected={!!tone?.articulations.includes(s.articulation)}
          />
        ))}
      </div>

      <div className="flex gap-0.5">
        <div className="flex gap-0.5">
          <div className="bg-neutral-900 flex flex-col gap-0.5">
            {STRESSES.map((s) => (
              <BravuraIconButton
                key={s.title}
                variant="interactive"
                bravura={s.glyph}
                title={s.title}
                offset={s.offset}
                onClick={() => {
                  if (selection?.type !== "Tone" || !tone) return;
                  actions.score.tone.setArticulation(
                    selection.track_id,
                    selection.tone_id,
                    s.articulation,
                  );
                }}
                isDisabled={!tone}
                isSelected={!!tone?.articulations.includes(s.articulation)}
              />
            ))}
          </div>

          <div className="bg-neutral-900 flex flex-col gap-0.5">
            {ACCENTUATIONS.map((s) => (
              <BravuraIconButton
                key={s.title}
                variant="interactive"
                bravura={s.glyph}
                title={s.title}
                offset={s.offset}
                onClick={() => {
                  if (selection?.type !== "Tone" || !tone) return;
                  actions.score.tone.setArticulation(
                    selection.track_id,
                    selection.tone_id,
                    s.articulation,
                  );
                }}
                isDisabled={!tone}
                isSelected={!!tone?.articulations.includes(s.articulation)}
              />
            ))}
          </div>
        </div>
      </div>

      <div className="bg-neutral-900 flex-1" />
    </div>
  );
};
