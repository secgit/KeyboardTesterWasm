#!/usr/bin/env bash
set -euo pipefail

WASM_FILE="docs/pkg/keyboard_tester_wasm_bg.wasm"
OUTPUT_FILE="docs/pkg/keyboard_tester_wasm_bg_base64.js"

if [[ ! -f "$WASM_FILE" ]]; then
  echo "Missing $WASM_FILE. Run wasm-pack build before embedding." >&2
  exit 1
fi

BASE64_DATA=$(base64 "$WASM_FILE")
cat > "$OUTPUT_FILE" <<"JS"
export const WASM_BASE64 = `
JS
printf '%s
' "$BASE64_DATA" >> "$OUTPUT_FILE"
cat >> "$OUTPUT_FILE" <<"JS"
`;

function decodeBase64(base64) {
  if (typeof atob === "function") {
    return atob(base64);
  }
  if (typeof Buffer === "function") {
    return Buffer.from(base64, "base64").toString("binary");
  }
  throw new Error("No base64 decoder available in this environment.");
}

export function wasmBytes() {
  const sanitized = WASM_BASE64.replace(/\s+/g, "");
  const binaryString = decodeBase64(sanitized);
  const len = binaryString.length;
  const bytes = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}
JS

rm "$WASM_FILE"
echo "Embedded wasm module into $OUTPUT_FILE and removed binary." >&2
