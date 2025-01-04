import { Renderer } from "../../components/renderer";
import { Panel } from "./components/panel";

export const Setup = () => {
  return (
    <div class="w-full h-full px-2 grid grid-rows-[1fr_170px_32px] grid-cols-[360px_1fr_360px] gap-2">
      <Panel className="row-span-2" title="Players" />
      <Renderer />
      <Panel className="row-span-2" title="Flows" />
      <Panel title="Layouts" />
    </div>
  );
};
