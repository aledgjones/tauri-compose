import { FC } from "react";
import { actions } from "../../../services/actions";
import { useAppState } from "../../../services/state";
import { LayoutType } from "../../../services/types";
import { Input } from "../../../ui/input";

type Props = { layoutType: LayoutType };

export const PageSettings: FC<Props> = ({ layoutType }) => {
  const state = useAppState();

  return (
    <div className="gap-2 flex flex-col">
      <div className="bg-neutral-900">
        <h3 className="p-2 text-sm font-medium text-white bg-neutral-800">
          Space Size
        </h3>
        <div className="p-4">
          <Input
            type="number"
            label="Space size"
            value={state.score.engraves.by_layout_type[layoutType].space}
            onChange={(value: number) =>
              actions.score.engrave.setSpace(layoutType, value)
            }
            min={0}
            step={0.1}
            decimalPlaces={1}
            units="mm"
            className="w-48"
          />
        </div>
      </div>

      <div className="bg-neutral-900">
        <h3 className="p-2 text-sm font-medium text-white bg-neutral-800">
          Page Margins
        </h3>
        <div className="p-4">
          <Input
            label="Top"
            type="number"
            value={
              state.score.engraves.by_layout_type[layoutType].page_padding.top
            }
            onChange={(value: number) =>
              actions.score.engrave.setPagePaddingTop(layoutType, value)
            }
            min={0}
            step={1}
            decimalPlaces={0}
            units="mm"
            className="w-48"
          />

          <Input
            label="Left"
            type="number"
            value={
              state.score.engraves.by_layout_type[layoutType].page_padding.left
            }
            onChange={(value: number) =>
              actions.score.engrave.setPagePaddingLeft(layoutType, value)
            }
            min={0}
            step={1}
            decimalPlaces={0}
            units="mm"
            className="w-48"
          />

          <Input
            label="Bottom"
            type="number"
            value={
              state.score.engraves.by_layout_type[layoutType].page_padding
                .bottom
            }
            onChange={(value: number) =>
              actions.score.engrave.setPagePaddingBottom(layoutType, value)
            }
            min={0}
            step={1}
            decimalPlaces={0}
            units="mm"
            className="w-48"
          />

          <Input
            label="Right"
            type="number"
            value={
              state.score.engraves.by_layout_type[layoutType].page_padding.right
            }
            onChange={(value: number) =>
              actions.score.engrave.setPagePaddingRight(layoutType, value)
            }
            min={0}
            step={1}
            decimalPlaces={0}
            units="mm"
            className="w-48"
          />
        </div>
      </div>

      <div className="bg-neutral-900">
        <h3 className="p-2 text-sm font-medium text-white bg-neutral-800">
          Music Frame Margins
        </h3>
        <div className="p-4">
          <Input
            label="Top"
            type="number"
            value={
              state.score.engraves.by_layout_type[layoutType].frame_padding.top
            }
            onChange={(value: number) =>
              actions.score.engrave.setFramePaddingTop(layoutType, value)
            }
            min={0}
            step={1}
            decimalPlaces={0}
            units="mm"
            className="w-48"
          />

          <Input
            label="Bottom"
            type="number"
            value={
              state.score.engraves.by_layout_type[layoutType].frame_padding
                .bottom
            }
            onChange={(value: number) =>
              actions.score.engrave.setFramePaddingBottom(layoutType, value)
            }
            min={0}
            step={1}
            decimalPlaces={0}
            units="mm"
            className="w-48"
          />
        </div>
      </div>
    </div>
  );
};
