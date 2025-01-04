import { Renderer } from "../../components/renderer";
import { Panel } from "../../components/panel";
import { mdiAccountCogOutline, mdiFileCogOutline, mdiPlus } from "@mdi/js";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export const Setup = () => {
  return (
    <div class="w-full h-full px-2 grid grid-rows-[1fr_170px_32px] grid-cols-[360px_1fr_360px] gap-2">
      <Panel
        className="row-span-2"
        title="Players"
        icons={[
          {
            path: mdiAccountCogOutline,
            onClick: () => {
              const webview = new WebviewWindow("project-settings", {
                url: "/player-settings.html",
                title: "Player Settings",
                visible: false,
                titleBarStyle: "overlay",
                hiddenTitle: true,
                acceptFirstMouse: true,
                height: 500,
                width: 750,
              });

              webview.setFocus();
            },
          },
          {
            path: mdiPlus,
            onClick: () => {},
          },
        ]}
      />
      <Renderer />
      <Panel
        className="row-span-2"
        title="Layouts"
        icons={[
          {
            path: mdiFileCogOutline,
            onClick: () => {},
          },
          {
            path: mdiPlus,
            onClick: () => {},
          },
        ]}
      />
      <Panel title="Flows" />
    </div>
  );
};
