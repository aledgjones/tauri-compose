import {
  mdiBugPlayOutline,
  mdiFolderCogOutline,
  mdiRedo,
  mdiUndo,
} from "@mdi/js";
import { IconButton } from "../../../ui/icon-button";
import classNames from "classnames";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { FC } from "react";

type Props = {
  className?: string;
};

export const Tools: FC<Props> = ({ className }) => {
  return (
    <div className={classNames("flex", className)}>
      <IconButton
        className="mr-2 text-white"
        onClick={() => {
          const webview = new WebviewWindow("project-settings", {
            url: "/project-settings.html",
            title: "Project Settings",
            visible: false,
            titleBarStyle: "overlay",
            acceptFirstMouse: true,
            height: 976,
            width: 750,
            theme: "dark",
          });

          webview.setFocus();
        }}
        variant="standard"
        path={mdiFolderCogOutline}
      />
      <IconButton
        className="mr-2 text-white"
        onClick={async () => {
          const webview = new WebviewWindow("debugger", {
            url: "/debugger.html",
            title: "Debugger",
            visible: false,
            titleBarStyle: "overlay",
            acceptFirstMouse: true,
            theme: "dark",
          });

          await webview.setFocus();
        }}
        variant="standard"
        path={mdiBugPlayOutline}
      />
      <IconButton
        isDisabled
        className="mr-1 text-white"
        onClick={() => {}}
        variant="standard"
        path={mdiUndo}
      />
      <IconButton
        isDisabled
        className="text-white"
        onClick={() => {}}
        variant="standard"
        path={mdiRedo}
      />
    </div>
  );
};
