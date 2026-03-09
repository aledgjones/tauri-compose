import { FC } from "react";
import { actions } from "../../../../../services/actions";
import { KeySignature, UniqueId } from "../../../../../services/types";
import { CommonPropertiesSection } from "./common-properties-section";

type Props = {
  flowId: UniqueId;
  keySignature: KeySignature;
};

export const KeySignaturePropertiesSection: FC<Props> = ({
  flowId,
  keySignature,
}) => {
  return (
    <>
      <CommonPropertiesSection
        hidden={keySignature.hidden}
        onHiddenChange={(hidden) => {
          void actions.score.keySignature.setHidden(
            flowId,
            keySignature.id,
            hidden,
          );
        }}
      />
    </>
  );
};
