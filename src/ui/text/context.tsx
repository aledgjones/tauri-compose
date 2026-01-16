import {
  createContext,
  FC,
  PropsWithChildren,
  useContext,
  useMemo,
} from "react";
import { useAppState } from "../../services/state";

type Dictionary = [string, { render: () => string; sym: boolean }][];
type TokenMap = Map<string, { render: () => string; sym: boolean }>;

const TokenizationContext = createContext<TokenMap>(new Map());

export const TokenizationProvider: FC<PropsWithChildren> = ({ children }) => {
  const state = useAppState();

  const symbols = useMemo(() => {
    const meta = state.score.meta;

    /* prettier-ignore-start */
    const userDefinedSymbols: Dictionary = [
      ["${project-title}", { render: () => meta.title, sym: false }],
      ["${project-subtitle}", { render: () => meta.subtitle, sym: false }],
      ["${project-dedication}", { render: () => meta.dedication, sym: false }],
      ["${project-composer}", { render: () => meta.composer, sym: false }],
      ["${project-arranger}", { render: () => meta.arranger, sym: false }],
      ["${project-lyricist}", { render: () => meta.lyricist, sym: false }],
      ["${project-artist}", { render: () => meta.artist, sym: false }],
      ["${project-copyist}", { render: () => meta.copyist, sym: false }],
      ["${project-publisher}", { render: () => meta.publisher, sym: false }],
      ["${project-editor}", { render: () => meta.editor, sym: false }],
      ["${project-copyright}", { render: () => meta.copyright, sym: false }],
      [
        "${project-work-number}",
        { render: () => meta.work_number, sym: false },
      ],
      [
        "${project-composition-year}",
        { render: () => meta.composition_year, sym: false },
      ],
      [
        "${project-composer-dates}",
        { render: () => meta.composer_dates, sym: false },
      ],
      [
        "${project-other-information}",
        { render: () => meta.other_information, sym: false },
      ],
    ];
    /* prettier-ignore-end */

    const staticSymbols: Dictionary = [
      ["${copy}", { render: () => "\u{00A9}", sym: false }],
      ["${sharp}", { render: () => "\u{E262}", sym: true }],
      ["${natural}", { render: () => "\u{E261}", sym: true }],
      ["${flat}", { render: () => "\u{E260}", sym: true }],
      ["${double-sharp}", { render: () => "\u{E263}", sym: true }],
      ["${double-flat}", { render: () => "\u{E264}", sym: true }],
    ];

    return new Map([...userDefinedSymbols, ...staticSymbols]);
  }, [state.score.meta]);

  return (
    <TokenizationContext.Provider value={symbols}>
      {children}
    </TokenizationContext.Provider>
  );
};

export const useTokenization = () => {
  const context = useContext(TokenizationContext);
  if (context === undefined) {
    throw new Error(
      "useTokenization must be used within a TokenizationProvider",
    );
  }
  return context;
};
