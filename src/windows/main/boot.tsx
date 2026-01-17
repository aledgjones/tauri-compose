import { createRoot } from "react-dom/client";
import { AppStateProvider } from "../../services/state";
import { Main } from ".";
import { TokenizationProvider } from "../../ui/text";

createRoot(document.getElementById("root")!).render(
  <AppStateProvider isMainWindow={true}>
    <TokenizationProvider>
      <Main />
    </TokenizationProvider>
  </AppStateProvider>,
);
