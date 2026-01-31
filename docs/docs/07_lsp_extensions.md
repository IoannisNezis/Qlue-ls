# :electric_plug: LSP Extensions

Qlue-ls extends the Language Server Protocol with custom methods.
These methods are prefixed with `qlueLs/` and provide SPARQL-specific functionality
that goes beyond standard LSP capabilities.

!!! note

    Your LSP client will not have built-in support for these methods.
    You will need to implement custom handlers in your client to use them.

## Backend Management

Backends represent SPARQL endpoints that the language server can connect to
for completions, hover information, and query execution.

### :heavy_plus_sign: addBackend

Register a SPARQL endpoint with the language server.

*Notification*:

- method: `qlueLs/addBackend`
- params: `AddBackendParams` defined as follows:

```ts
interface AddBackendParams {
    service: BackendService;
    requestMethod?: "GET" | "POST";
    default: boolean;
    prefixMap?: Record<string, string>;
    queries?: Record<string, string>;
}

interface BackendService {
    name: string;
    url: string;
    healthCheckUrl?: string;
    engine?: "QLever" | "GraphDB" | "Virtuoso" | "MillenniumDB" | "Blazegraph" | "Jena";
}
```

### :mag: getBackend

Get information about the currently active default backend.

*Request*:

- method: `qlueLs/getBackend`
- params: none

*Response*:

- result: `BackendService | null`

```ts
interface BackendService {
    name: string;
    url: string;
    healthCheckUrl?: string;
    engine?: "QLever" | "GraphDB" | "Virtuoso" | "MillenniumDB" | "Blazegraph" | "Jena";
}
```

### :clipboard: listBackends

List all registered backends.

*Request*:

- method: `qlueLs/listBackends`
- params: none

*Response*:

- result: `BackendService[]`

### :arrows_counterclockwise: updateDefaultBackend

Change the active default backend.

*Notification*:

- method: `qlueLs/updateDefaultBackend`
- params: `UpdateDefaultBackendParams` defined as follows:

```ts
interface UpdateDefaultBackendParams {
    backendName: string;
}
```

### :satellite: pingBackend

Test connectivity to a SPARQL endpoint.

*Request*:

- method: `qlueLs/pingBackend`
- params: `PingBackendParams` defined as follows:

```ts
interface PingBackendParams {
    backendName?: string;  // If omitted, pings the default backend
}
```

*Response*:

- result: `PingBackendResult` defined as follows:

```ts
interface PingBackendResult {
    available: boolean;
}
```

## Settings

### :gear: defaultSettings

Get the default server settings.

*Request*:

- method: `qlueLs/defaultSettings`
- params: none

*Response*:

- result: `Settings` defined as follows:

```ts
interface Settings {
    format: FormatSettings;
    completion: CompletionSettings;
    prefixes?: PrefixesSettings;
}
```

See [Configuration](03_configuration.md) for details on the settings structure.

### :wrench: changeSettings

Update server settings at runtime.

*Notification*:

- method: `qlueLs/changeSettings`
- params: `Settings`

!!! warning

    This replaces all current settings with the provided values.

## Query Execution

### :arrow_forward: executeOperation

Execute a SPARQL query or update operation against the configured backend.

*Request*:

- method: `qlueLs/executeOperation`
- params: `ExecuteOperationParams` defined as follows:

```ts
interface ExecuteOperationParams {
    textDocument: TextDocumentIdentifier;
    maxResultSize?: number;
    resultOffset?: number;
    queryId?: string;
    lazy?: boolean;       // WASM only: stream results incrementally
    accessToken?: string; // For authenticated endpoints
}
```

*Response*:

- result: `ExecuteOperationResult | null`
- error: `ExecuteOperationError | null`

```ts
type ExecuteOperationResult = 
    | { queryResult: QueryResult }
    | { updateResult: UpdateResult[] };

interface QueryResult {
    timeMs: number;
    result?: SparqlResult;
}

interface ExecuteOperationError {
    code: number;
    message: string;
    data: ExecuteOperationErrorData;
}

type ExecuteOperationErrorData =
    | { type: "QLeverException"; exception: string; query: string; status: "ERROR"; metadata?: ErrorMetadata }
    | { type: "Connection"; /* connection error details */ }
    | { type: "Canceled"; query: string }
    | { type: "InvalidFormat"; query: string; message: string }
    | { type: "Unknown" };
```

### :stop_button: cancelQuery

Cancel a running SPARQL query.

*Notification*:

- method: `qlueLs/cancelQuery`
- params: `CancelQueryParams` defined as follows:

```ts
interface CancelQueryParams {
    queryId: string;
}
```

!!! note

    The `queryId` must match the one provided in the `executeOperation` request.

### :fast_forward: partialResult

Stream partial SPARQL results as they become available.

!!! note

    This is a server-to-client notification, only available in WASM builds
    when `lazy: true` is set in `executeOperation`.

*Notification* (server to client):

- method: `qlueLs/partialResult`
- params: `PartialResult` (partial SPARQL result data)

## Navigation

### :hole: jump

Enable "tab navigation" within SPARQL queries.
The server provides the next (or previous) relevant position in the query.
The LSP client should move the cursor to this position.

*Request*:

- method: `qlueLs/jump`
- params: `JumpParams` defined as follows:

```ts
interface JumpParams extends TextDocumentPositionParams {
    previous?: boolean;
}
```

*Response*:

- result: `JumpResult | null` defined as follows:

```ts
interface JumpResult {
    position: Position;
    insertBefore: string | null;
    insertAfter: string | null;
}
```

### :question: identifyOperationType

Determine if a SPARQL document is a query or update operation.

*Request*:

- method: `qlueLs/identifyOperationType`
- params: `IdentifyOperationTypeParams` defined as follows:

```ts
interface IdentifyOperationTypeParams {
    textDocument: TextDocumentIdentifier;
}
```

*Response*:

- result: `IdentifyOperationTypeResult` defined as follows:

```ts
interface IdentifyOperationTypeResult {
    operationType: OperationType;
}

type OperationType = "Query" | "Update" | "Unknown";
```
