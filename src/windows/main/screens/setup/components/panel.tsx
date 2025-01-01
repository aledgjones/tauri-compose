import { FunctionalComponent } from "preact";

type Props = {
  title: string;
};

export const Panel: FunctionalComponent<Props> = ({ title }) => {
  return (
    <div className="bg-neutral-900 rounded-lg overflow-hidden h-full">
      <div className="h-[48px] px-4 flex items-center">
        <p className="text-white text-sm">{title}</p>
      </div>
    </div>
  );
};
