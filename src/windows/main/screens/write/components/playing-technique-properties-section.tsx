import { FC } from "react";
import { actions } from "../../../../../services/actions";
import { PlayingTechnique, UniqueId } from "../../../../../services/types";
import { CommonPropertiesSection } from "./common-properties-section";

type Props = {
  flowId: UniqueId;
  staveId: UniqueId;
  playingTechnique: PlayingTechnique;
};

export const PlayingTechniquePropertiesSection: FC<Props> = ({
  flowId,
  staveId,
  playingTechnique,
}) => {
  return (
    <>
      <CommonPropertiesSection
        hidden={playingTechnique.hidden}
        onHiddenChange={(hidden) => {
          void actions.score.playingTechnique.update(
            flowId,
            staveId,
            playingTechnique.id,
            hidden,
          );
        }}
      />
    </>
  );
};
