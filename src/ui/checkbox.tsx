import { FunctionalComponent } from "preact";
import { mdiCheck } from "@mdi/js";
import { Icon } from "./icon";

type Props = {
  value: boolean;
  onChange: (value: boolean) => void;
  className?: string;
};

const Checkbox: FunctionalComponent<Props> = ({
  value,
  onChange,
  className,
}) => {
  const handleChange = (event: Event) => {
    onChange((event.target as HTMLInputElement).checked);
  };

  return (
    <div className={`relative h-[36px] w-[36px] ${className}`}>
      <div
        className={`
          appearance-none
          border
          border-neutral-800
          checked:bg-sky-500
          checked:border-sky-500
          focus:outline-none
          focus:ring-2
          focus:ring-sky-500
          h-5
          w-5
          transition
          duration-200
        `}
      >
        {value && (
          <Icon
            className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2"
            path={mdiCheck}
            size={0.8}
            color="white"
          />
        )}
      </div>
    </div>
  );
};

export default Checkbox;
