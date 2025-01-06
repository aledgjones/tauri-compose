import { render } from "preact";
import { AppStateProvider } from "../../services/state";
import { Main } from ".";

render(
  <AppStateProvider>
    <Main />
  </AppStateProvider>,
  document.getElementById("root")!
);
