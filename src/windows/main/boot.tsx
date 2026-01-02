import { createRoot } from "react-dom/client";
import { AppStateProvider } from "../../services/state";
import { Main } from ".";
import { TokenizationProvider } from "../../ui/text/context";

createRoot(document.getElementById("root")!).render(
  <AppStateProvider>
    <TokenizationProvider>
      <Main />
    </TokenizationProvider>
  </AppStateProvider>,
);
