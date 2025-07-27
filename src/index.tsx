/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";
import { ThemeProvider } from "./theme";

render(() => (
  <ThemeProvider>
    <App />
  </ThemeProvider>
), document.getElementById("root") as HTMLElement);
