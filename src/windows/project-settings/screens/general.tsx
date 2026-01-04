import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import { Input } from "../../../ui/input";
import { Textarea } from "../../../ui/textarea";

export const Meta = () => {
  const state = useAppState();

  return (
    <div className="bg-neutral-900">
      <h3 className="p-2 text-sm font-medium text-white bg-neutral-800">
        Project Information
      </h3>
      <div className="p-4 gap-2 flex flex-col">
        <Input
          type="text"
          value={state.score.meta.title}
          onChange={actions.score.meta.setTitle}
          label="Title"
        />
        <Input
          type="text"
          value={state.score.meta.subtitle}
          onChange={actions.score.meta.setSubtitle}
          label="Subtitle"
        />
        <Input
          type="text"
          value={state.score.meta.dedication}
          onChange={actions.score.meta.setDedication}
          label="Dedication"
        />
        <Input
          type="text"
          value={state.score.meta.composer}
          onChange={actions.score.meta.setComposer}
          label="Composer"
        />
        <Input
          type="text"
          value={state.score.meta.arranger}
          onChange={actions.score.meta.setArranger}
          label="Arranger"
        />
        <Input
          type="text"
          value={state.score.meta.lyricist}
          onChange={actions.score.meta.setLyricist}
          label="Lyricist"
        />
        <Input
          type="text"
          value={state.score.meta.artist}
          onChange={actions.score.meta.setArtist}
          label="Artist"
        />
        <Input
          type="text"
          value={state.score.meta.copyist}
          onChange={actions.score.meta.setCopyist}
          label="Copyist"
        />
        <Input
          type="text"
          value={state.score.meta.publisher}
          onChange={actions.score.meta.setPublisher}
          label="Publisher"
        />
        <Input
          type="text"
          value={state.score.meta.editor}
          onChange={actions.score.meta.setEditor}
          label="Editor"
        />
        <Textarea
          lines={3}
          value={state.score.meta.copyright}
          onChange={actions.score.meta.setCopyright}
          label="Copyright"
        />
        <div className="flex gap-2">
          <Input
            type="text"
            className="flex-1"
            value={state.score.meta.work_number}
            onChange={actions.score.meta.setWorkNumber}
            label="Work Number"
          />
          <Input
            type="text"
            className="flex-1"
            value={state.score.meta.composition_year}
            onChange={actions.score.meta.setCompositionYear}
            label="Composition Year"
          />
        </div>
        <Input
          type="text"
          value={state.score.meta.composer_dates}
          onChange={actions.score.meta.setComposerDates}
          label="Composer Dates"
        />
        <Textarea
          lines={3}
          value={state.score.meta.other_information}
          onChange={actions.score.meta.setOtherInformation}
          label="Other Information"
        />
      </div>
    </div>
  );
};
