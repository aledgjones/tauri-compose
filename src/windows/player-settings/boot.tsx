import { createRoot } from "react-dom/client";
import { AppStateProvider } from "../../services/state";
import { Main } from ".";

createRoot(document.getElementById("root")!).render(
  <AppStateProvider>
    <Main />
  </AppStateProvider>
);
