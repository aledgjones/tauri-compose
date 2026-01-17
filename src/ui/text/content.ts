import { createContext } from "react";

export type TokenMap = Map<string, { render: () => string; sym: boolean }>;

export const TokenizationContext = createContext<TokenMap>(new Map());
