import { invoke } from "@tauri-apps/api/core";
import { FunctionComponent } from "preact";
import { useEffect, useRef } from "preact/hooks";

export const Renderer: FunctionComponent = () => {
  const img = useRef<HTMLImageElement>(null);

  useEffect(() => {
    const render = async () => {
      if (img.current === null) {
        return;
      }

      const t1 = performance.now();
      const bytes: any = await invoke("render");
      const uint8Array = new Uint8Array(bytes);
      console.log(uint8Array);
      const blob = new Blob([uint8Array], { type: "image/png" });
      const data = URL.createObjectURL(blob);
      console.log(data);
      img.current.src = data;
      console.log("Rendered in", performance.now() - t1, "ms");
    };

    render();
  }, []);

  return (
    <div className="rounded-lg bg-neutral-900 text-white">
      <img ref={img} />
    </div>
  );
};
