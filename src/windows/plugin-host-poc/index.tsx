import { FC } from "react";

export const Main: FC = () => {
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-neutral-900 text-neutral-100">
      <div className="relative flex-1 overflow-hidden bg-neutral-900">
        <div className="absolute inset-0 flex items-center justify-center px-8 text-center text-sm text-neutral-500">
          If the backend validates `SOLOC_PLUGIN_EDITOR_POC_VST3_PATH`, it
          attaches a native AppKit subview into this window content area.
        </div>
      </div>
    </div>
  );
};
