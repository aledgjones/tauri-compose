import { FunctionalComponent } from "preact";

type Props = {
  title: string;
};

export const Panel: FunctionalComponent<Props> = ({ title }) => {
  return (
    <div className="rounded-lg bg-neutral-900 overflow-hidden h-full">
      <div className="h-[48px] px-4 flex items-center">
        <p className="text-white text-sm">{title}</p>
      </div>
    </div>
  );
};
