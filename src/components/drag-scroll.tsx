import classNames from "classnames";
import { FC, PropsWithChildren, useRef, useState } from "react";

type Props = PropsWithChildren<{ className?: string }>;

export const DragScroll: FC<Props> = ({ children, className }) => {
  const divRef = useRef<HTMLDivElement>(null);

  const [isDragging, setIsDragging] = useState(false);

  const handlePointerDown = (e: any) => {
    if (!divRef.current) {
      return;
    }

    setIsDragging(true);

    const startX = e.clientX;
    const startY = e.clientY;
    const scrollLeft = divRef.current.scrollLeft;
    const scrollTop = divRef.current.scrollTop;

    const handlePointerMove = (e: any) => {
      if (!divRef.current) {
        return;
      }

      e.preventDefault();
      const x = e.clientX - startX;
      const y = e.clientY - startY;
      divRef.current.scrollLeft = scrollLeft - x;
      divRef.current.scrollTop = scrollTop - y;
    };

    const handlePointerUp = () => {
      document.removeEventListener("pointermove", handlePointerMove);
      document.removeEventListener("pointerup", handlePointerUp);
      setIsDragging(false);
      console.log("UP");
    };

    document.addEventListener("pointermove", handlePointerMove);
    document.addEventListener("pointerup", handlePointerUp);
  };

  return (
    <div
      ref={divRef}
      className={classNames("overflow-auto touch-none", className)}
      onPointerDown={handlePointerDown}
    >
      {children}
    </div>
  );
};
