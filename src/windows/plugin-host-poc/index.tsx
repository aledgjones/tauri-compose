import { FC } from "react";

export const Main: FC = () => {
  return (
    <div className="flex h-screen w-screen overflow-hidden bg-neutral-900 text-neutral-100">
      <div className="relative flex-1 overflow-hidden bg-neutral-900">
        <div className="absolute inset-0 flex items-center justify-center px-8 text-center text-sm text-neutral-500">
          If the backend resolves a valid VST3 bundle, it attaches that
          plugin&apos;s native editor view into this window as an experimental
          host preview.
        </div>
      </div>
    </div>
  );
};
