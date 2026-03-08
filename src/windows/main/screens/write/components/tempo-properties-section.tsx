import { FC, ReactNode, useCallback, useEffect, useState } from "react";
import { actions } from "../../../../../services/actions";
import { NoteDuration, Tempo, UniqueId } from "../../../../../services/types";
import { TextInput } from "../../../../../ui/input/text-input";
import { NumberInput } from "../../../../../ui/input/number-input";
import { Switch } from "../../../../../ui/switch";
import { BravuraIconButton } from "../../../../../ui/buttons/bravura-icon-button";
import { SMuFL } from "../../../../../ui/buttons/smufl";

type Props = {
  flowId: UniqueId;
  tempo: Tempo;
};

const BEAT_TYPE_OPTIONS = [
  {
    value: NoteDuration.Whole,
    glyph: SMuFL.WholeNote,
    title: "Whole note",
    offset: -2,
  },
  {
    value: NoteDuration.Half,
    glyph: SMuFL.HalfNote,
    title: "Half note",
    offset: 6,
  },
  {
    value: NoteDuration.Quarter,
    glyph: SMuFL.QuarterNote,
    title: "Quarter note",
    offset: 6,
  },
  {
    value: NoteDuration.Eighth,
    glyph: SMuFL.EighthNote,
    title: "Eighth note",
    offset: 6,
  },
  {
    value: NoteDuration.Sixteenth,
    glyph: SMuFL.SixteenthNote,
    title: "Sixteenth note",
    offset: 6,
  },
  {
    value: NoteDuration.ThirtySecond,
    glyph: SMuFL.ThirtySecondNote,
    title: "Thirty-second note",
    offset: 9,
  },
] as const;

const clampBpm = (value: number) => {
  return Math.max(20, Math.min(400, Math.round(value)));
};

const Section: FC<{ title: string; children: ReactNode }> = ({
  title,
  children,
}) => {
  return (
    <section className="flex min-w-[18rem] flex-col bg-neutral-900">
      <h3 className="text-xs text-white p-3 bg-neutral-800">{title}</h3>
      <div className="p-3 gap-5 flex">{children}</div>
    </section>
  );
};

const SectionColumn: FC<{ children: ReactNode }> = ({ children }) => {
  return <div className="flex flex-col gap-3 w-75">{children}</div>;
};

export const TempoPropertiesSection: FC<Props> = ({ flowId, tempo }) => {
  const [draftTempo, setDraftTempo] = useState(tempo);
  const [textDraft, setTextDraft] = useState(tempo.text);

  useEffect(() => {
    setDraftTempo(tempo);
    setTextDraft(tempo.text);
  }, [tempo]);

  const submitTempo = useCallback(
    (patch: Partial<Tempo>) => {
      setDraftTempo((currentTempo) => {
        const nextTempo = { ...currentTempo, ...patch };

        void actions.score.tempo.update(
          flowId,
          nextTempo.id,
          nextTempo.text,
          nextTempo.beat_type,
          nextTempo.dots,
          nextTempo.bpm,
          nextTempo.hidden,
          nextTempo.show_text,
          nextTempo.show_metronome,
          nextTempo.parenthesized,
        );

        return nextTempo;
      });
    },
    [flowId],
  );

  const commitText = () => {
    if (textDraft === draftTempo.text) {
      return;
    }

    submitTempo({ text: textDraft });
  };

  return (
    <div
      className="flex min-w-max gap-0.5"
      data-testid="tempo-properties-section"
    >
      <Section title="Tempo">
        <SectionColumn>
          <TextInput
            type="text"
            label="Text"
            value={textDraft}
            onChange={setTextDraft}
            onBlur={commitText}
            onEnter={commitText}
            variant="default"
            placeholder="Tempo text"
          />
          <Switch
            small
            label="Show text"
            value={draftTempo.show_text}
            onChange={(show_text) => submitTempo({ show_text })}
          />
        </SectionColumn>
        <SectionColumn>
          <div>
            <p className="text-xs mb-1 text-neutral-400">Beat type</p>
            <div className="flex">
              {BEAT_TYPE_OPTIONS.map((option) => (
                <BravuraIconButton
                  key={option.value}
                  variant="filled"
                  size={24}
                  bravura={option.glyph}
                  title={option.title}
                  offset={option.offset}
                  onClick={() => submitTempo({ beat_type: option.value })}
                  isSelected={draftTempo.beat_type === option.value}
                  className="flex-1"
                />
              ))}
            </div>
          </div>
          <NumberInput
            type="number"
            label="Beats Per Minute"
            value={draftTempo.bpm}
            onChange={(value) => submitTempo({ bpm: clampBpm(value) })}
            step={1}
            decimalPlaces={0}
            units="BPM"
            variant="default"
          />
          <Switch
            small
            label="Show metronome"
            value={draftTempo.show_metronome}
            onChange={(show_metronome) => submitTempo({ show_metronome })}
          />
          <Switch
            small
            label="Parenthesized"
            value={draftTempo.parenthesized}
            onChange={(parenthesized) => submitTempo({ parenthesized })}
          />
        </SectionColumn>
      </Section>
    </div>
  );
};
