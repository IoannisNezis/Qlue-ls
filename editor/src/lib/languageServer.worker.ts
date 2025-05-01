import init, { init_language_server, listen } from 'qlue-ls';
import { BrowserMessageReader, BrowserMessageWriter } from "vscode-languageserver/browser";
import wasmUrl from 'qlue-ls/qlue_ls_bg.wasm?url'

init({
        module_or_path: wasmUrl
}).then(() => {
        // Connection Language-Client <-> Worker
        const editorReader = new BrowserMessageReader(self);
        const editorWriter = new BrowserMessageWriter(self);

        // Connection Worker <-> Language Server(WASM)
        const wasmInputStream = new TransformStream();
        const wasmOutputStream = new TransformStream();
        const wasmReader = wasmOutputStream.readable.getReader();
        const wasmWriter = wasmInputStream.writable.getWriter();

        // Initialize & start language server
        const server = init_language_server(wasmOutputStream.writable.getWriter());
        listen(server, wasmInputStream.readable.getReader());

        // Language Client -> Language Server
        editorReader.listen((data) => {
                // console.log(data);
                wasmWriter.write(JSON.stringify(data));
        });

        // Forward Language Serverne> Language Client
        (async () => {
                while (true) {
                        const { value, done } = await wasmReader.read();
                        if (done) break;
                        // console.log(JSON.parse(value));
                        editorWriter.write(JSON.parse(value));
                }
        })();

        self.postMessage({ type: "ready" });
});
export { }
