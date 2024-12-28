import { Main } from "./screens/main";
import { AppStateProvider } from "./services/state";

function App() {
  return (
    <AppStateProvider>
      <Main />
    </AppStateProvider>
  );
}

export default App;
