import init, { initSync } from "./pkg/keyboard_tester_wasm.js";
import { wasmBytes } from "./pkg/keyboard_tester_wasm_bg_base64.js";

try {
  initSync(wasmBytes());
} catch (error) {
  console.warn("Falling back to async initialization due to:", error);
  init({ module_or_path: wasmBytes() }).catch((err) => {
    console.error("Failed to initialize keyboard tester WebAssembly module", err);
  });
}
