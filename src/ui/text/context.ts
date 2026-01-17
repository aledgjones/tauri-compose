import { createContext, useContext } from "react";

export type TokenMap = Map<string, { render: () => string; sym: boolean }>;

export const TokenizationContext = createContext<TokenMap>(new Map());

export const useTokenization = () => {
  const context = useContext(TokenizationContext);
  if (context === undefined) {
    throw new Error(
      "useTokenization must be used within a TokenizationProvider",
    );
  }
  return context;
};
