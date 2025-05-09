import init, { init_language_server, listen } from 'qlue-ls';
import wasmUrl from 'qlue-ls/qlue_ls_bg.wasm?url'

init({
        module_or_path: wasmUrl
}).then(() => {
        // Connection Worker <-> Language Server(WASM)
        const wasmInputStream = new TransformStream();
        const wasmOutputStream = new TransformStream();
        const wasmReader = wasmOutputStream.readable.getReader();
        const wasmWriter = wasmInputStream.writable.getWriter();

        // Initialize & start language server
        const server = init_language_server(wasmOutputStream.writable.getWriter());
        listen(server, wasmInputStream.readable.getReader());

        // Language Client -> Language Server
        self.onmessage = function(message) {
                // console.log(data);
                wasmWriter.write(JSON.stringify(message.data));
        };
        // Language Server -> Language Client
        (async () => {
                while (true) {
                        const { value, done } = await wasmReader.read();
                        if (done) break;
                        // console.log(JSON.parse(value));
                        self.postMessage(JSON.parse(value));
                }
        })();

        self.postMessage({ type: "ready" });
});
export { }
