use crate::server::Server;
use crate::server::configuration::RequestMethod;
use crate::server::lsp::CanceledError;
use crate::server::lsp::ExecuteUpdateResponseResult;
use crate::server::lsp::PartialSparqlResultNotification;
use crate::server::lsp::SparqlEngine;
use crate::server::sparql_operations::ConnectionError;
use crate::server::sparql_operations::SparqlRequestError;
use crate::server::sparql_operations::utils::add_limit_offset_to_query;
use crate::sparql::results::RDFTerm;
use crate::sparql::results::SparqlResult;
use futures::lock::Mutex;
use js_sys::JsString;
use lazy_sparql_result_reader::parser::PartialResult;
use lazy_sparql_result_reader::sparql::Head;
use lazy_sparql_result_reader::sparql::Header;
use lazy_sparql_result_reader::sparql::Meta;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use urlencoding::encode;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{AbortController, AbortSignal, Request, RequestInit, Response, WorkerGlobalScope};

const ACCEPT_SPARQL_JSON: &str = "application/sparql-results+json";
const ACCEPT_NTRIPLES: &str = "application/n-triples";
const CONTENT_TYPE_FORM: &str = "application/x-www-form-urlencoded";
const CONTENT_TYPE_SPARQL_QUERY: &str = "application/sparql-query";

/// Install an abort signal on `opts`.
///
/// If `timeout_ms` is set, uses an `AbortSignal::timeout`. Otherwise, if
/// `query_id` is set, registers an `AbortController` with the server state so
/// the request can be canceled by id.
async fn install_abort_signal(
    opts: &RequestInit,
    timeout_ms: Option<u32>,
    query_id: Option<&str>,
    server_rc: &Rc<Mutex<Server>>,
) {
    if let Some(timeout_ms) = timeout_ms {
        opts.set_signal(Some(&AbortSignal::timeout_with_u32(timeout_ms)));
    } else if let Some(query_id) = query_id {
        let controller = AbortController::new().expect("AbortController should be creatable");
        opts.set_signal(Some(&controller.signal()));
        server_rc.lock().await.state.add_running_request(
            query_id.to_string(),
            // NOTE: abort without a custom reason: fetch then rejects with a
            // standard "AbortError" DOMException in every browser. With a
            // custom reason, chromium rejects with the raw reason value
            // instead, which breaks cancellation detection in `fetch`.
            Box::new(move || {
                controller.abort();
            }),
        );
    }
}

/// Send `request` via the worker's fetch and translate any JS error into a
/// `SparqlRequestError`, distinguishing cancellation (`AbortError`) from
/// connection failures.
async fn fetch(request: &Request, query: &str) -> Result<Response, SparqlRequestError> {
    let worker_global: WorkerGlobalScope = js_sys::global().unchecked_into();
    let resp_value = JsFuture::from(worker_global.fetch_with_request(request))
        .await
        .map_err(|err| {
            let was_canceled = err
                .dyn_ref::<web_sys::DomException>()
                .map(|e| e.name() == "AbortError")
                .unwrap_or(false);
            if was_canceled {
                SparqlRequestError::Canceled(CanceledError {
                    query: query.to_string(),
                })
            } else {
                // NOTE: fetch rejects with an `Error` (or `DOMException`);
                // extract its message instead of debug-printing the JsValue.
                let message = err
                    .dyn_ref::<js_sys::Error>()
                    .map(|e| String::from(e.message()))
                    .or_else(|| err.dyn_ref::<web_sys::DomException>().map(|e| e.message()))
                    .or_else(|| err.as_string())
                    .unwrap_or_else(|| format!("{err:?}"));
                SparqlRequestError::Connection(ConnectionError {
                    message,
                    query: query.to_string(),
                })
            }
        })?;
    let resp = resp_value.dyn_into().unwrap();
    Ok(resp)
}

/// Read a non-2xx response body and turn it into the most specific
/// `SparqlRequestError` available: a `QLeverException` when the body is the
/// expected JSON shape, otherwise an `Http` error carrying the status code
/// and the raw body.
async fn parse_error_response(resp: Response) -> SparqlRequestError {
    let status = resp.status();
    let status_text = resp.status_text();
    let body = match read_reponse_body_as_text(resp).await {
        Ok(body) => body,
        Err(err) => return err,
    };
    match serde_json::from_str(&body) {
        Ok(err) => SparqlRequestError::QLeverException(err),
        Err(_) => SparqlRequestError::Http(crate::server::sparql_operations::HttpError {
            status,
            status_text,
            body,
        }),
    }
}

async fn read_reponse_body_as_text(response: Response) -> Result<String, SparqlRequestError> {
    JsFuture::from(response.text().map_err(|err| {
        SparqlRequestError::Deserialization(format!("Response has no text:\n{err:?}"))
    })?)
    .await
    .map_err(|err| {
        SparqlRequestError::Deserialization(format!("Could not read Response text:\n{err:?}"))
    })?
    .as_string()
    .ok_or(SparqlRequestError::Deserialization(
        "Could not read response body as utf-8 string".to_string(),
    ))
}

fn set_header(request: &Request, name: &str, value: &str) {
    request.headers().set(name, value).unwrap();
}

/// Build the `Request` for a SELECT/ASK query, honoring the request method and
/// engine-specific quirks (QLever uses a urlencoded body with optional
/// `Query-Id` header).
fn build_query_request(
    url: &str,
    query: &str,
    method: &RequestMethod,
    limit: Option<usize>,
    offset: usize,
    engine: &Option<SparqlEngine>,
    query_id: Option<&str>,
    opts: &RequestInit,
) -> Request {
    let request = match (method, engine) {
        (RequestMethod::GET, _) => {
            opts.set_method("GET");
            Request::new_with_str_and_init(&format!("{url}?query={}", encode(query)), opts).unwrap()
        }
        (RequestMethod::POST, Some(SparqlEngine::QLever)) => {
            opts.set_method("POST");
            // NOTE: QLever provides the "send" parameter.
            // It causes the Engine to only send and therfor produce n results,
            // even if the result size is larger then n.
            // We set this send parameter to the minimal n required.
            // Maybe QLever will also provide a "offset" parameter in the future.
            let mut fields = Vec::with_capacity(2);
            if let Some(limit) = limit {
                fields.push(format!("send={}", limit + offset));
            }
            fields.push(format!("query={}", js_sys::encode_uri_component(query)));
            let body: String = fields.join("&");
            opts.set_body(&JsString::from_str(&body).expect("Request body should be valid."));
            let request = Request::new_with_str_and_init(url, opts).unwrap();
            set_header(&request, "Content-Type", CONTENT_TYPE_FORM);
            if let Some(id) = query_id {
                set_header(&request, "Query-Id", id);
            }
            request
        }
        (RequestMethod::POST, _) => {
            opts.set_method("POST");
            opts.set_body(&JsString::from_str(query).unwrap());
            let request = Request::new_with_str_and_init(url, opts).unwrap();
            set_header(&request, "Content-Type", CONTENT_TYPE_SPARQL_QUERY);
            request
        }
    };
    set_header(&request, "Accept", ACCEPT_SPARQL_JSON);
    request
}

/// Stream a lazy SPARQL JSON response, forwarding each parsed chunk to the
/// client as a `PartialSparqlResultNotification`. Returns `Ok(None)` once the
/// stream is fully consumed.
async fn stream_lazy_query_results(
    resp: Response,
    server_rc: Rc<Mutex<Server>>,
    query: &str,
    limit: Option<usize>,
    offset: usize,
) -> Result<usize, SparqlRequestError> {
    let result = lazy_sparql_result_reader::read(
        resp.body().unwrap(),
        // INFO: `limit` is the window size (rows after `offset`), so the read
        // window is `limit` itself; cap the batch size at it.
        limit.map(|limit| 1000.min(limit)).unwrap_or(1000),
        limit,
        offset,
        async |mut partial_result: PartialResult| {
            let server = server_rc.lock().await;
            compress_result_uris(&server, &mut partial_result);
            if let Err(err) =
                server.send_message(PartialSparqlResultNotification::new(partial_result))
            {
                tracing::error!("Could not send Partial-Sparql-Result-Notification:\n{err:?}");
            }
        },
    )
    .await;

    use lazy_sparql_result_reader::SparqlResultReaderError;
    match result {
        Ok(n) => Ok(n),
        Err(SparqlResultReaderError::Canceled) => {
            Err(SparqlRequestError::Canceled(CanceledError {
                query: query.to_string(),
            }))
        }
        Err(
            err @ (SparqlResultReaderError::CorruptStream
            | SparqlResultReaderError::JsonParseError(_)),
        ) => Err(SparqlRequestError::Deserialization(format!("{err:?}"))),
    }
}

pub(crate) async fn execute_construct_query(
    server_rc: Rc<Mutex<Server>>,
    url: &str,
    query: &str,
    query_id: Option<&str>,
    engine: Option<SparqlEngine>,
    lazy: bool,
) -> Result<Option<SparqlResult>, SparqlRequestError> {
    let opts = RequestInit::new();

    let request = match engine {
        Some(SparqlEngine::QLever) => {
            opts.set_method("POST");
            // INFO: `send=100` tells QLever how many rows to send back.
            let body = format!("send=100&query={}", js_sys::encode_uri_component(query));
            opts.set_body(&JsString::from_str(&body).unwrap());
            let request = Request::new_with_str_and_init(url, &opts).unwrap();
            set_header(&request, "Content-Type", CONTENT_TYPE_FORM);
            if let Some(id) = query_id {
                set_header(&request, "Query-Id", id);
            }
            request
        }
        _ => {
            opts.set_method("POST");
            opts.set_body(&JsString::from_str(query).unwrap());
            let request = Request::new_with_str_and_init(url, &opts).unwrap();
            set_header(&request, "Content-Type", CONTENT_TYPE_SPARQL_QUERY);
            request
        }
    };
    set_header(&request, "Accept", ACCEPT_NTRIPLES);

    let resp = fetch(&request, query).await?;
    if !resp.ok() {
        return Err(parse_error_response(resp).await);
    }

    let text = read_reponse_body_as_text(resp).await?;
    let (triples, _errors) = ntriples_parser::parse(text.as_bytes()).map_err(|_e| {
        SparqlRequestError::Deserialization("Could not read n-triples response".to_string())
    })?;

    let result = SparqlResult::new(
        ["subject", "predicate", "object"]
            .into_iter()
            .map(str::to_string)
            .collect(),
        triples
            .into_iter()
            .map(|triple| {
                HashMap::from_iter([
                    ("subject".to_string(), triple_term(triple.0)),
                    ("predicate".to_string(), triple_term(triple.1)),
                    ("object".to_string(), triple_term(triple.2)),
                ])
            })
            .collect(),
    );

    if !lazy {
        return Ok(Some(result));
    }

    let server = server_rc.lock().await;
    let SparqlResult {
        head,
        results,
        prefixes: _prefixes,
    } = result;
    server
        .send_message(PartialSparqlResultNotification::new(PartialResult::Header(
            Header {
                head: Head { vars: head.vars },
            },
        )))
        .expect("Response should be sendable");
    server
        .send_message(PartialSparqlResultNotification::new(
            PartialResult::Bindings(
                results
                    .bindings
                    .into_iter()
                    .map(|binding| {
                        lazy_sparql_result_reader::sparql::Binding(HashMap::from_iter(
                            binding.into_iter().map(|(key, value)| (key, value.into())),
                        ))
                    })
                    .collect(),
            ),
        ))
        .expect("Response should be sendable");
    Ok(None)
}

fn triple_term(bytes: impl AsRef<[u8]>) -> RDFTerm {
    RDFTerm::Literal {
        value: String::from_utf8(bytes.as_ref().to_vec()).expect("Should be valid utf8"),
        lang: None,
        datatype: None,
    }
}

pub(crate) async fn execute_update(
    server_rc: Rc<Mutex<Server>>,
    url: &str,
    query: &str,
    query_id: Option<&str>,
    access_token: Option<&str>,
) -> Result<ExecuteUpdateResponseResult, SparqlRequestError> {
    let opts = RequestInit::new();
    install_abort_signal(&opts, None, query_id, &server_rc).await;
    opts.set_method("POST");
    let body = format!("update={}", js_sys::encode_uri_component(query));
    opts.set_body(&JsString::from_str(&body).unwrap());

    let request = Request::new_with_str_and_init(url, &opts).unwrap();
    set_header(&request, "Content-Type", CONTENT_TYPE_FORM);
    if let Some(access_token) = access_token {
        set_header(&request, "Authorization", &format!("Bearer {access_token}"));
    }
    set_header(&request, "Accept", ACCEPT_SPARQL_JSON);

    let resp = fetch(&request, query).await?;
    if !resp.ok() {
        return Err(parse_error_response(resp).await);
    }

    let text = read_reponse_body_as_text(resp).await?;
    serde_json::from_str(&text).map_err(|err| SparqlRequestError::Deserialization(err.to_string()))
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn execute_query(
    server_rc: Rc<Mutex<Server>>,
    url: String,
    mut query: String,
    query_id: Option<&str>,
    engine: Option<SparqlEngine>,
    timeout_ms: Option<u32>,
    method: RequestMethod,
    limit: Option<usize>,
    offset: usize,
    lazy: bool,
) -> Result<Option<SparqlResult>, SparqlRequestError> {
    let opts = RequestInit::new();
    install_abort_signal(&opts, timeout_ms, query_id, &server_rc).await;

    // NOTE: Non-lazy POST execution paginates by rewriting the query;
    // the lazy reader handles limit/offset itself, so we leave the query alone.
    if !lazy && let Some(new_query) = add_limit_offset_to_query(&query, limit, offset) {
        query = new_query;
    }
    let request = build_query_request(
        &url, &query, &method, limit, offset, &engine, query_id, &opts,
    );

    let resp = fetch(&request, &query).await?;
    if !resp.ok() {
        return Err(parse_error_response(resp).await);
    }

    if lazy {
        let count =
            stream_lazy_query_results(resp, server_rc.clone(), &query, limit, offset).await?;
        // INFO: QLever's response includes a trailing `meta` block that the
        // streaming reader already forwards. Other engines do not, so we
        // synthesize one from the parser's count to give the client the total.
        if engine.is_none_or(|engine| engine != SparqlEngine::QLever) {
            let server = server_rc.lock().await;
            if let Err(err) = server.send_message(PartialSparqlResultNotification::new(
                PartialResult::Meta(Meta {
                    query_time_ms: None,
                    result_size_total: count as u64,
                }),
            )) {
                tracing::error!("Could not send Partial-Sparql-Result-Notification:\n{err:?}");
            }
        }
        Ok(None)
    } else {
        let text = read_reponse_body_as_text(resp).await?;
        let result = serde_json::from_str(&text)
            .map_err(|err| SparqlRequestError::Deserialization(err.to_string()))?;
        Ok(Some(result))
    }
}

fn compress_result_uris(server: &Server, partial_result: &mut PartialResult) {
    use lazy_sparql_result_reader::sparql::RDFValue;
    if let PartialResult::Bindings(bindings) = partial_result {
        for binding in bindings.iter_mut() {
            for (_, rdf_term) in binding.0.iter_mut() {
                if let RDFValue::Uri { value, curie } = rdf_term {
                    *curie = server
                        .state
                        .get_default_converter()
                        .and_then(|converer| converer.compress(value).ok());
                }
            }
        }
    }
}

pub(crate) async fn check_server_availability(url: &str) -> bool {
    use web_sys::RequestMode;

    let worker_global: WorkerGlobalScope = js_sys::global().unchecked_into();
    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::Cors);
    let request = Request::new_with_str_and_init(url, &opts).expect("Failed to create request");
    let resp_value = match JsFuture::from(worker_global.fetch_with_request(&request)).await {
        Ok(resp) => resp,
        Err(_) => return false,
    };
    let resp: Response = resp_value.dyn_into().unwrap();
    resp.ok()
}
