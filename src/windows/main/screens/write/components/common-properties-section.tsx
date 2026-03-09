import { FC } from "react";
import { Switch } from "../../../../../ui/switch";
import { PanelSection, PanelSectionColumn } from "./properties-panel";

type Props = {
  hidden: boolean;
  onHiddenChange: (hidden: boolean) => void;
};

export const CommonPropertiesSection: FC<Props> = ({
  hidden,
  onHiddenChange,
}) => {
  return (
    <>
      <PanelSection title="Common">
        <PanelSectionColumn>
          <Switch
            small
            label="Hidden"
            value={hidden}
            onChange={onHiddenChange}
          />
        </PanelSectionColumn>
      </PanelSection>
    </>
  );
};
