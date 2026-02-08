<h1 align="center">
  lazy-sparql-result-reader
</h1>

<div align="center">
    <a href="https://www.npmjs.com/package/lazy-sparql-result-reader">
        <img alt="npm" src="https://img.shields.io/npm/v/lazy-sparql-result-reader" />
    </a>
    <a href="https://crates.io/crates/lazy-sparql-result-reader">
        <img alt="crates.io" src="https://img.shields.io/crates/v/lazy-sparql-result-reader.svg" />
    </a>
</div>

A fast SPARQL results parser for JavaScript and TypeScript, compiled from Rust via WebAssembly.  
It reads streamed SPARQL query results and calls a callback for each parsed batch of bindings.

---

## Features

- Processes streaming SPARQL results efficiently.
- Calls a JavaScript callback for each batch of parsed bindings.
- Written in Rust for speed and reliability.
- Fully compatible with TypeScript.

---


## Usage Example

### 1. Install dependencies

```bash
npm install  lazy-sparql-result-reader vite @vitejs/plugin-wasm
```

### 2. Configure Vite for WASM

Create or update vite.config.ts:

```bash
import { defineConfig } from 'vite';
import wasm from '@vitejs/plugin-wasm';

export default defineConfig({
  plugins: [wasm()],
});
```

This allows Vite to correctly load WebAssembly modules.

### 3. Use the parser in your app

```ts
import init, { read } from "lazy-sparql-result-reader?init";

// Initialize the WASM module
await init();

fetch("https://qlever.dev/api/wikidata", {
  method: 'POST',
  headers: {
    "Accept": "application/sparql-results+json",
    "Content-Type": "application/x-www-form-urlencoded;charset=UTF-8",
  },
  body: new URLSearchParams({
    query: "SELECT * {?s ?p ?o} LIMIT 1000",
  })
})
.then(async response => {
  if (!response.ok) {
    throw new Error(`SPARQL request failed: ${response.status}`);
  }
  const stream = response.body; // ReadableStream of the SPARQL JSON results
  if (!stream) throw new Error("Response has no body stream");
  // Parse the streamed results with the WASM parser
  await read(stream, 100, (bindings) => {
    console.log("Received batch of bindings:", bindings);
  });
})
.catch((err) => {
  console.error("Error fetching or parsing SPARQL results:", err);
});
```

## Notes

- The **first callback invocation** contains the **SPARQL head**, i.e., the variable names in the result set.
- Subsequent callback invocations contain batches of bindings as they are parsed.
- `batch_size` controls how many bindings are buffered before each callback.
- Ensure your environment supports **ReadableStream** (modern browsers or Node.js >= 18).

This setup allows your JS/TS application to process **streaming SPARQL results** efficiently,  
with immediate access to the head and incremental batches of bindings.

## License

This project is licensed under the **MIT** License.

You are free to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the software, under the conditions of the MIT License.

For full details, see the [LICENSE](./LICENSE) file.
