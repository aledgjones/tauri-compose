import { FC } from "react";
import { actions } from "../../../../../services/actions";
import { Clef, UniqueId } from "../../../../../services/types";
import { CommonPropertiesSection } from "./common-properties-section";

type Props = {
  flowId: UniqueId;
  staveId: UniqueId;
  clef: Clef;
};

export const ClefPropertiesSection: FC<Props> = ({ flowId, staveId, clef }) => {
  return (
    <>
      <CommonPropertiesSection
        hidden={clef.hidden}
        onHiddenChange={(hidden) => {
          void actions.score.clef.setHidden(flowId, staveId, clef.id, hidden);
        }}
      />
    </>
  );
};
