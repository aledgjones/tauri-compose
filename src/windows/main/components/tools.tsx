import {
  mdiBugPlayOutline,
  mdiFolderCogOutline,
  mdiRedo,
  mdiUndo,
} from "@mdi/js";
import { IconButton } from "../../../ui/icon-button";
import { FunctionComponent } from "preact";
import classNames from "classnames";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

type Props = {
  className?: string;
};

export const Tools: FunctionComponent<Props> = ({ className }) => {
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
            hiddenTitle: true,
            acceptFirstMouse: true,
            height: 976,
            width: 750,
          });

          webview.setFocus();
        }}
        variant="filled"
        path={mdiFolderCogOutline}
      />
      <IconButton
        className="mr-2 text-white"
        onClick={() => {
          const webview = new WebviewWindow("debugger", {
            url: "/debugger.html",
            title: "Debugger",
            visible: false,
            titleBarStyle: "overlay",
            hiddenTitle: true,
            acceptFirstMouse: true,
          });

          webview.setFocus();
        }}
        variant="filled"
        path={mdiBugPlayOutline}
      />
      <IconButton
        isDisabled
        className="mr-1 text-white"
        onClick={() => {}}
        variant="filled"
        path={mdiUndo}
      />
      <IconButton
        isDisabled
        className="text-white"
        onClick={() => {}}
        variant="filled"
        path={mdiRedo}
      />
    </div>
  );
};
