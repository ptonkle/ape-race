// The page draws what the core returns and decides nothing.
// Every call goes through the shim, which returns {"ok": ...} or {"error": "..."}.
import init, { load } from "./pkg/tool_shim.js";

const el = (id) => document.getElementById(id);

function call(fn, ...args) {
  return JSON.parse(fn(...args));
}

async function start() {
  await init();
  const text = await (await fetch("data/strings.json")).text();
  const result = call(load, text);
  if (result.error) {
    el("message").textContent = result.error;
  } else {
    document.title = result.ok.title;
    el("title").textContent = result.ok.title;
    el("intro").textContent = result.ok.intro;
  }
  document.body.dataset.ready = "true";
}

start();
