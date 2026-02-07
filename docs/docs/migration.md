# Migration Guide

## v2.0.0 (from v1.1.19)

This release flattens the backend configuration structure, removing the nested
`service` object. The change affects configuration files, LSP extension params,
and LSP extension responses.

### Configuration file

The `service` object has been removed. Its fields (`name`, `url`,
`healthCheckUrl`, `engine`) are now top-level properties of each backend entry.

A new optional `additionalData` field is available for attaching arbitrary data
to a backend configuration.

**Before** (`qlue-ls.toml`):

```toml
[backends.wikidata]
default = true
requestMethod = "GET"

[backends.wikidata.service]
name = "Wikidata"
url = "https://query.wikidata.org/sparql"
healthCheckUrl = "https://query.wikidata.org/"

[backends.wikidata.prefixMap]
wd = "http://www.wikidata.org/entity/"
wdt = "http://www.wikidata.org/prop/direct/"
```

**After** (`qlue-ls.toml`):

```toml
[backends.wikidata]
name = "Wikidata"
url = "https://query.wikidata.org/sparql"
healthCheckUrl = "https://query.wikidata.org/"
default = true
requestMethod = "GET"

[backends.wikidata.prefixMap]
wd = "http://www.wikidata.org/entity/"
wdt = "http://www.wikidata.org/prop/direct/"
```

**Before** (`qlue-ls.yml`):

```yaml
backends:
  wikidata:
    service:
      name: Wikidata
      url: https://query.wikidata.org/sparql
      healthCheckUrl: https://query.wikidata.org/
    default: true
    requestMethod: GET
    prefixMap:
      wd: http://www.wikidata.org/entity/
      wdt: http://www.wikidata.org/prop/direct/
```

**After** (`qlue-ls.yml`):

```yaml
backends:
  wikidata:
    name: Wikidata
    url: https://query.wikidata.org/sparql
    healthCheckUrl: https://query.wikidata.org/
    default: true
    requestMethod: GET
    prefixMap:
      wd: http://www.wikidata.org/entity/
      wdt: http://www.wikidata.org/prop/direct/
```

In short: move everything that was under `service` one level up and delete
the `service` key.

---

### `qlueLs/addBackend`

The notification params follow the same flattening. The nested `service`
object is removed and its fields are now top-level.

**Before**:

```ts
interface AddBackendParams {
    service: {
        name: string;
        url: string;
        healthCheckUrl?: string;
        engine?: "QLever" | "GraphDB" | "Virtuoso" | "MillenniumDB" | "Blazegraph" | "Jena";
    };
    requestMethod?: "GET" | "POST";
    default: boolean;
    prefixMap?: Record<string, string>;
    queries?: Record<string, string>;
}
```

**After**:

```ts
interface AddBackendParams {
    name: string;
    url: string;
    healthCheckUrl?: string;
    engine?: "QLever" | "GraphDB" | "Virtuoso" | "MillenniumDB" | "Blazegraph" | "Jena";
    requestMethod?: "GET" | "POST";
    default: boolean;
    prefixMap?: Record<string, string>;
    queries?: Record<string, string>;
    additionalData?: any;
}
```

**Action required**: Remove the `service` wrapper from your `qlueLs/addBackend`
notification params and place `name`, `url`, `healthCheckUrl`, and `engine` at
the top level.

---

### `qlueLs/getBackend`

The response now returns the **full backend configuration** instead of only the
service fields. It also includes an `error` field when no default backend is
configured (previously `result` was simply `null`).

**Before**:

```ts
// result
{
    name: string;
    url: string;
    healthCheckUrl?: string;
    engine?: string;
}
// or null when no default backend
```

**After**:

```ts
// result (when a default backend exists)
{
    name: string;
    url: string;
    healthCheckUrl?: string;
    engine?: string;
    requestMethod?: "GET" | "POST";
    prefixMap: Record<string, string>;
    default: boolean;
    queries: Record<string, string>;
    additionalData?: any;
}

// error (when no default backend is configured)
{
    code: number;
    message: "No default backend is configured.";
}
```

**Action required**: Update your client to handle the new response shape.
If you only used `name` and `url`, no change is needed. If you checked for
`result === null` to detect missing backends, also check the new `error` field.

---

### `qlueLs/listBackends`

The response items have been simplified. Each item now only contains `name`,
`url`, and `default`. The `healthCheckUrl` and `engine` fields are no longer
included in list results.

**Before**:

```ts
// result item
{
    name: string;
    url: string;
    healthCheckUrl?: string;
    engine?: string;
}
```

**After**:

```ts
// result item
{
    name: string;
    url: string;
    default: boolean;
}
```

**Action required**: If your client reads `healthCheckUrl` or `engine` from
list results, use `qlueLs/getBackend` instead for the full backend details.
The new `default` field on each item lets you identify the active backend
without a separate `getBackend` call.

---

### Summary of changes

| Area                    | What changed                                                               |
|-------------------------|----------------------------------------------------------------------------|
| Config file             | `service` object removed; its fields are now top-level                     |
| Config file             | New optional `additionalData` field                                        |
| `qlueLs/addBackend`    | Params flattened (no `service` wrapper)                                    |
| `qlueLs/getBackend`    | Returns full config; new `error` field when no default backend             |
| `qlueLs/listBackends`  | Items narrowed to `name`, `url`, `default`; no `healthCheckUrl`/`engine`   |
