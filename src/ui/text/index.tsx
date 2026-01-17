import { FC, Fragment, PropsWithChildren, useMemo } from "react";
import { useTokenization } from "./context";

export { TokenizationProvider } from "./provider";

export const TokenizedText: FC<PropsWithChildren> = ({ children }) => {
  const symbols = useTokenization();

  const tokens = useMemo(() => {
    const str = children?.toString() || "";

    const symRegex = /(\${[^\s}]*})/g;
    const split = str.split(symRegex).filter((entry) => entry);
    return split.map((entry) => {
      if (symbols.has(entry)) {
        return symbols.get(entry) || { render: () => "", sym: false };
      } else {
        return { render: () => entry, sym: false };
      }
    });
  }, [children, symbols]);

  return (
    <>
      {tokens.map((token, i) => {
        if (token.sym) {
          return (
            <span
              key={i}
              className="font-bravura leading-0 text-[1.4em] align-middle"
            >
              {token.render()}
            </span>
          );
        } else {
          return <Fragment key={i}>{token.render()}</Fragment>;
        }
      })}
    </>
  );
};
