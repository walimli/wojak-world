import { render } from "preact";
import "98.css";
import { App } from "./app";
import "./styles.css";

const root = document.getElementById("app");
if (root) {
  render(<App />, root);
}
