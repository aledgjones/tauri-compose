import { useContext } from "react";
import { AppContext, SamplerContext } from "./context";

export const useAppState = () => {
  const context = useContext(AppContext);

  if (!context) {
    throw new Error("useAppState must be used within an AppStateProvider");
  }

  return context;
};

export const useSamplerState = () => {
  const context = useContext(SamplerContext);

  if (!context) {
    throw new Error("useSamplerState must be used within an AppStateProvider");
  }

  return context;
};
