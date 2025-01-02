import { actions, useAppState } from "../../../services/state";
import { Input } from "../../../ui/input";
import { Textarea } from "../../../ui/textarea";

export const Meta = () => {
  const state = useAppState();

  return (
    <div className="p-2 pt-1 overflow-y-auto">
      <Input
        value={state.score.meta.title}
        onChange={actions.score.meta.setTitle}
        label="Title"
      />
      <Input
        value={state.score.meta.subtitle}
        onChange={actions.score.meta.setSubtitle}
        label="Subtitle"
      />
      <Input
        value={state.score.meta.composer}
        onChange={actions.score.meta.setComposer}
        label="Composer"
      />
      <Input
        value={state.score.meta.arranger}
        onChange={actions.score.meta.setArranger}
        label="Arranger"
      />
      <Input
        value={state.score.meta.lyricist}
        onChange={actions.score.meta.setLyricist}
        label="Lyricist"
      />
      <Textarea
        value={state.score.meta.copyright}
        onChange={actions.score.meta.setCopyright}
        label="Copyright"
      />
    </div>
  );
};
