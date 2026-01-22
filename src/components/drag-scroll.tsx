import classNames from "classnames";
import {
  FC,
  PointerEvent as ReactPointerEvent,
  PropsWithChildren,
  useCallback,
  useRef,
  useEffect,
  UIEvent,
} from "react";

const allow = (target: HTMLElement, ignore?: string) => {
  if (ignore) {
    const nodes = document.getElementsByClassName(ignore);
    for (let i = 0; i < nodes.length; i++) {
      const node = nodes.item(i);
      if (node && (node === target || node.contains(target))) {
        return false;
      }
    }
    return true;
  } else {
    return true;
  }
};

type Props = PropsWithChildren<{
  className?: string;
  onScroll?: (x: number, y: number) => void;
  x?: number;
  y?: number;
}>;

export const DragScroll: FC<Props> = ({
  children,
  className,
  onScroll,
  x = 0,
  y = 0,
}) => {
  const divRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (divRef.current) {
      divRef.current.scrollLeft = x;
      divRef.current.scrollTop = y;
    }
    // only set on mount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleScroll = useCallback(
    (e: UIEvent<HTMLDivElement>) => {
      if (onScroll) {
        onScroll(e.currentTarget.scrollLeft, e.currentTarget.scrollTop);
      }
    },
    [onScroll],
  );

  const handlePointerDown = useCallback(
    (e: ReactPointerEvent<HTMLDivElement>) => {
      if (!divRef.current) {
        return;
      }

      let isDragging = true;

      const startX = e.clientX;
      const startY = e.clientY;
      const scrollLeft = divRef.current.scrollLeft;
      const scrollTop = divRef.current.scrollTop;
      const allowX = Boolean(allow(e.target as HTMLElement, "ignore-x"));
      const allowY = Boolean(allow(e.target as HTMLElement, "ignore-y"));

      const handlePointerMove = (e: PointerEvent) => {
        if (!isDragging || !divRef.current) {
          return;
        }

        e.preventDefault();

        const x = e.clientX - startX;
        const y = e.clientY - startY;

        if (allowX) {
          divRef.current.scrollLeft = scrollLeft - x;
        }

        if (allowY) {
          divRef.current.scrollTop = scrollTop - y;
        }
      };

      const handlePointerUp = () => {
        document.removeEventListener("pointermove", handlePointerMove);
        document.removeEventListener("pointerup", handlePointerUp);
        isDragging = false;
      };

      document.addEventListener("pointermove", handlePointerMove);
      document.addEventListener("pointerup", handlePointerUp);
    },
    [],
  );

  return (
    <div
      ref={divRef}
      style={{ overflow: "overlay" }}
      className={classNames("touch-none", className)}
      onPointerDown={handlePointerDown}
      onScroll={handleScroll}
    >
      {children}
    </div>
  );
};
