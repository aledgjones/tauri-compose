import { mdiBugPlayOutline, mdiFolderCog, mdiRedo, mdiUndo } from "@mdi/js";
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
        className="mr-2"
        onClick={() => {
          const webview = new WebviewWindow("project-settings", {
            url: "/project-settings.html",
            title: "Project Settings",
            visible: false,
            titleBarStyle: "overlay",
            hiddenTitle: true,
            acceptFirstMouse: true,
            height: 500,
            width: 750,
            // resizable: false,
          });

          webview.setFocus();
        }}
        variant="filled"
        path={mdiFolderCog}
      />
      <IconButton
        className="mr-2"
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
        className="mr-1"
        onClick={() => {}}
        variant="filled"
        path={mdiUndo}
      />
      <IconButton onClick={() => {}} variant="filled" path={mdiRedo} />
    </div>
  );
};
