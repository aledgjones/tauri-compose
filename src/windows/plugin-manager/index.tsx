import { getCurrentWindow } from "@tauri-apps/api/window";
import { FC, useEffect, useState } from "react";
import { AppBar } from "../../components/app-bar";
import { Plugins, PluginsToolbar } from "../main/components/plugins";

export const Main: FC = () => {
  const [searchTerm, setSearchTerm] = useState("");
  const [showActiveOnly, setShowActiveOnly] = useState(false);

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_1fr] gap-0.5 p-0.75 overflow-hidden">
      <AppBar title="Plugin Manager" />

      <div className="flex min-h-0 flex-col gap-0.5 overflow-hidden">
        <PluginsToolbar
          searchTerm={searchTerm}
          onSearchTermChange={setSearchTerm}
          showActiveOnly={showActiveOnly}
          onShowActiveOnlyChange={setShowActiveOnly}
          showRefreshButton={false}
        />
        <Plugins searchTerm={searchTerm} showActiveOnly={showActiveOnly} />
      </div>
    </div>
  );
};
