import { render } from "preact";
import { AppStateProvider } from "../../services/state";
import { Main } from ".";
import { TokenizationProvider } from "../../ui/text/context";

render(
  <AppStateProvider>
    <TokenizationProvider>
      <Main />
    </TokenizationProvider>
  </AppStateProvider>,
  document.getElementById("root")!
);
