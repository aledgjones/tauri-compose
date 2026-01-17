import { useContext } from "react";
import { TokenizationContext } from "./content";

export const useTokenization = () => {
  const context = useContext(TokenizationContext);
  if (context === undefined) {
    throw new Error(
      "useTokenization must be used within a TokenizationProvider",
    );
  }
  return context;
};
