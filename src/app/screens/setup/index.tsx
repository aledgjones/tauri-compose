import { Panel } from "./components/panel";

export const Setup = () => {
  return (
    <div class="px-2 pb-2 h-full grid grid-rows-[1fr_32px] gap-2">
      <div class="grid grid-cols-[360px_1fr_360px] gap-2">
        <Panel title="Players" />
        <div class="grid grid-rows-[auto_170px] gap-2">
          <div className="bg-neutral-900 rounded-lg"></div>
          <Panel title="Flows" />
        </div>
        <Panel title="Layouts" />
      </div>
      <div></div>
    </div>
  );
};
