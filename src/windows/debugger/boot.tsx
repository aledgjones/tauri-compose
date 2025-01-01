import { FunctionalComponent, render } from "preact";
import { AppStateProvider } from "../../services/state";
import { Main } from ".";

const ProjectSettings: FunctionalComponent = () => {
  return (
    <AppStateProvider>
      <Main />
    </AppStateProvider>
  );
};

render(<ProjectSettings />, document.getElementById("root")!);
