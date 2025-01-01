import { render } from "preact";
import { AppStateProvider } from "../services/state";
import { App } from ".";

render(
  <AppStateProvider>
    <App />
  </AppStateProvider>,
  document.getElementById("root")!
);
