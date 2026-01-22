use crate::server::Server;
use crate::server::configuration::RequestMethod;
use crate::server::lsp::CanceledError;
use crate::server::lsp::ExecuteUpdateResponseResult;
use crate::server::lsp::PartialSparqlResultNotification;
use crate::server::lsp::SparqlEngine;
use crate::server::sparql_operations::ConnectionError;
use crate::server::sparql_operations::SparqlRequestError;
use crate::sparql::results::RDFTerm;
use crate::sparql::results::SparqlResult;
use futures::lock::Mutex;
use js_sys::JsString;
use lazy_sparql_result_reader::parser::PartialResult;
use lazy_sparql_result_reader::sparql::Head;
use lazy_sparql_result_reader::sparql::Header;
use std::collections::HashMap;
use std::rc::Rc;
use std::str::FromStr;
use urlencoding::encode;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::AbortController;
use web_sys::{Request, RequestInit, Response, WorkerGlobalScope};

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
            let body = format!("send=100&query={}", js_sys::encode_uri_component(query));
            opts.set_body(&JsString::from_str(&body).unwrap());
            let request = Request::new_with_str_and_init(url, &opts).unwrap();
            request
                .headers()
                .set("Content-Type", "application/x-www-form-urlencoded")
                .unwrap();
            if let Some(id) = query_id {
                request.headers().set("Query-Id", id).unwrap();
            }
            request
        }
        _ => {
            opts.set_method("POST");
            opts.set_body(&JsString::from_str(query).unwrap());
            let request = Request::new_with_str_and_init(url, &opts).unwrap();
            request
                .headers()
                .set("Content-Type", "application/sparql-query")
                .unwrap();
            request
        }
    };
    request
        .headers()
        .set("Accept", "application/n-triples")
        .unwrap();

    // Get global worker scope
    let worker_global: WorkerGlobalScope = js_sys::global().unchecked_into();

    // Perform the fetch request and await the response
    let resp_value = JsFuture::from(worker_global.fetch_with_request(&request))
        .await
        .map_err(|err| {
            log::error!("error: {err:?}");
            SparqlRequestError::Connection(ConnectionError {
                status_text: format!("{err:?}"),
                query: query.to_string(),
            })
        })?;

    // Cast the response value to a Response object
    let resp: Response = resp_value.dyn_into().unwrap();

    // Check if the response status is OK (200-299)
    if !resp.ok() {
        return match resp.json() {
            Ok(json) => match JsFuture::from(json).await {
                Ok(js_value) => match serde_wasm_bindgen::from_value(js_value) {
                    Ok(err) => Err(SparqlRequestError::QLeverException(err)),
                    Err(err) => Err(SparqlRequestError::Deserialization(format!(
                        "Could not deserialize error message: {}",
                        err
                    ))),
                },
                Err(err) => Err(SparqlRequestError::Deserialization(format!(
                    "Query failed! Response did not provide a json body but this could not be cast to rust JsValue.\n{:?}",
                    err
                ))),
            },
            Err(err) => Err(SparqlRequestError::Deserialization(format!(
                "Query failed! Response did not provide a json body.\n{err:?}"
            ))),
        };
    }
    // Get the response body as text and await it
    let text = JsFuture::from(resp.text().map_err(|err| {
        SparqlRequestError::Response(format!("Response has no text:\n{:?}", err))
    })?)
    .await
    .map_err(|err| {
        SparqlRequestError::Response(format!("Could not read Response text:\n{:?}", err))
    })?
    .as_string()
    .unwrap();
    log::info!("{}", text);
    // Return the text as a JsValue
    let triples = ntriples_parser::parse(text.as_bytes()).map_err(|_e| {
        SparqlRequestError::Deserialization("Could not read n-triples response".to_string())
    })?;

    let result = SparqlResult::new(
        ["subject", "predicate", "object"]
            .into_iter()
            .map(|var| var.to_string())
            .collect(),
        triples
            .into_iter()
            .map(|triple| {
                HashMap::from_iter([
                    (
                        "subject".to_string(),
                        RDFTerm::Literal {
                            value: String::from_utf8(triple.0.to_vec())
                                .expect("Should be valid utf8"),
                            lang: None,
                            datatype: None,
                        },
                    ),
                    (
                        "predicate".to_string(),
                        RDFTerm::Literal {
                            value: String::from_utf8(triple.1.to_vec())
                                .expect("Should be valid utf8"),
                            lang: None,
                            datatype: None,
                        },
                    ),
                    (
                        "object".to_string(),
                        RDFTerm::Literal {
                            value: String::from_utf8(triple.2.to_vec())
                                .expect("Should be valid utf8"),
                            lang: None,
                            datatype: None,
                        },
                    ),
                ])
            })
            .collect(),
    );
    if lazy {
        let server = server_rc.lock().await;
        let SparqlResult {
            head,
            results,
            prefixes: _prefixes,
        } = result;
        log::info!("lock aquired");
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
    } else {
        Ok(Some(result))
    }
}

pub(crate) async fn execute_update(
    server_rc: Rc<Mutex<Server>>,
    url: &str,
    query: &str,
    query_id: Option<&str>,
    access_token: Option<&str>,
) -> Result<Vec<ExecuteUpdateResponseResult>, SparqlRequestError> {
    use js_sys::JsString;
    use std::str::FromStr;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{AbortController, Request, RequestInit, Response, WorkerGlobalScope};

    let opts = RequestInit::new();
    if let Some(query_id) = query_id {
        let controller = AbortController::new().expect("AbortController should be creatable");

        opts.set_signal(Some(&controller.signal()));
        server_rc.lock().await.state.add_running_request(
            query_id.to_string(),
            Box::new(move || {
                controller.abort_with_reason(&JsValue::from_str("Query was canceled"));
            }),
        );
    }
    opts.set_method("POST");
    let body = format!("update={}", js_sys::encode_uri_component(query));
    opts.set_body(&JsString::from_str(&body).unwrap());
    let request = Request::new_with_str_and_init(url, &opts).unwrap();
    request
        .headers()
        .set("Content-Type", "application/x-www-form-urlencoded")
        .unwrap();
    if let Some(access_token) = access_token {
        request
            .headers()
            .set("Authorization", &format!("Bearer {access_token}"))
            .unwrap();
    }
    request
        .headers()
        .set("Accept", "application/sparql-results+json")
        .unwrap();

    let worker_global: WorkerGlobalScope = js_sys::global().unchecked_into();

    // Perform the fetch request and await the response
    let resp_value = JsFuture::from(worker_global.fetch_with_request(&request))
        .await
        .map_err(|err| {
            let was_canceled = err
                .dyn_ref::<web_sys::DomException>()
                .map(|e| e.name() == "AbortError")
                .unwrap_or(false);
            if was_canceled {
                SparqlRequestError::_Canceled(CanceledError {
                    query: query.to_string(),
                })
            } else {
                SparqlRequestError::Connection(ConnectionError {
                    status_text: format!("{err:?}"),
                    query: query.to_string(),
                })
            }
        })?;

    // Cast the response value to a Response object
    let resp: Response = resp_value.dyn_into().unwrap();

    // Check if the response status is OK (200-299)
    if !resp.ok() {
        return match resp.json() {
            Ok(json) => match JsFuture::from(json).await {
                Ok(js_value) => match serde_wasm_bindgen::from_value(js_value) {
                    Ok(err) => Err(SparqlRequestError::QLeverException(err)),
                    Err(err) => Err(SparqlRequestError::Deserialization(format!(
                        "Could not deserialize error message: {}",
                        err
                    ))),
                },
                Err(err) => Err(SparqlRequestError::Deserialization(format!(
                    "Query failed! Response did not provide a json body but this could not be cast to rust JsValue.\n{:?}",
                    err
                ))),
            },
            Err(err) => Err(SparqlRequestError::Deserialization(format!(
                "Query failed! Response did not provide a json body.\n{err:?}"
            ))),
        };
    }

    let text = read_reponse_body_as_text(resp).await?;

    Ok(serde_json::from_str(&text)
        .map_err(|err| SparqlRequestError::Deserialization(err.to_string()))?)
}

pub(crate) async fn execute_query(
    server_rc: Rc<Mutex<Server>>,
    url: String,
    query: String,
    query_id: Option<&str>,
    engine: Option<SparqlEngine>,
    timeout_ms: Option<u32>,
    method: RequestMethod,
    limit: Option<usize>,
    offset: usize,
    lazy: bool,
) -> Result<Option<SparqlResult>, SparqlRequestError> {
    use js_sys::JsString;
    use std::str::FromStr;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{AbortSignal, Request, RequestInit, Response, WorkerGlobalScope};

    use lazy_sparql_result_reader::parser::PartialResult;

    let opts = RequestInit::new();
    if let Some(timeout_ms) = timeout_ms {
        opts.set_signal(Some(&AbortSignal::timeout_with_u32(timeout_ms)));
    } else if let Some(query_id) = query_id {
        let controller = AbortController::new().expect("AbortController should be creatable");

        opts.set_signal(Some(&controller.signal()));
        server_rc.lock().await.state.add_running_request(
            query_id.to_string(),
            Box::new(move || {
                controller.abort_with_reason(&JsValue::from_str("Query was canceled"));
            }),
        );
    }

    let request = match (&method, engine) {
        (RequestMethod::GET, _) => {
            opts.set_method("GET");
            Request::new_with_str_and_init(&format!("{url}?query={}", encode(&query)), &opts)
                .unwrap()
        }
        (RequestMethod::POST, Some(SparqlEngine::QLever)) => {
            opts.set_method("POST");
            // FIXME: Here the send limit is hardcoded to 10000
            // this is due to the internal batching of QLever
            // A lower send limit causes QLever not imediatly sending the result.
            let body = format!("send=10000&query={}", js_sys::encode_uri_component(&query));
            opts.set_body(&JsString::from_str(&body).unwrap());
            let request = Request::new_with_str_and_init(&url, &opts).unwrap();
            request
                .headers()
                .set("Content-Type", "application/x-www-form-urlencoded")
                .unwrap();
            if let Some(id) = query_id {
                request.headers().set("Query-Id", id).unwrap();
            }
            request
        }
        (RequestMethod::POST, _) => {
            opts.set_method("POST");
            opts.set_body(&JsString::from_str(&query).unwrap());
            let request = Request::new_with_str_and_init(&url, &opts).unwrap();
            request
                .headers()
                .set("Content-Type", "application/sparql-query")
                .unwrap();
            request
        }
    };
    request
        .headers()
        .set("Accept", "application/sparql-results+json")
        .unwrap();

    // Currently blocked by CORS...
    // request.headers().set("User-Agent", "qlue-ls/1.0").unwrap();

    // Get global worker scope
    let worker_global: WorkerGlobalScope = js_sys::global().unchecked_into();

    // Perform the fetch request and await the response
    let resp_value = JsFuture::from(worker_global.fetch_with_request(&request))
        .await
        .map_err(|err| {
            let was_canceled = err
                .dyn_ref::<web_sys::DomException>()
                .map(|e| e.name() == "AbortError")
                .unwrap_or(false);
            if was_canceled {
                SparqlRequestError::_Canceled(CanceledError {
                    query: query.to_string(),
                })
            } else {
                SparqlRequestError::Connection(ConnectionError {
                    status_text: format!("{err:?}"),
                    query: query.to_string(),
                })
            }
        })?;

    // Cast the response value to a Response object
    let resp: Response = resp_value.dyn_into().unwrap();

    // Check if the response status is OK (200-299)
    if !resp.ok() {
        return match resp.json() {
            Ok(json) => match JsFuture::from(json).await {
                Ok(js_value) => match serde_wasm_bindgen::from_value(js_value) {
                    Ok(err) => Err(SparqlRequestError::QLeverException(err)),
                    Err(err) => Err(SparqlRequestError::Deserialization(format!(
                        "Could not deserialize error message: {}",
                        err
                    ))),
                },
                Err(err) => Err(SparqlRequestError::Deserialization(format!(
                    "Query failed! Response did not provide a json body but this could not be cast to rust JsValue.\n{:?}",
                    err
                ))),
            },
            Err(err) => Err(SparqlRequestError::Deserialization(format!(
                "Query failed! Response did not provide a json body.\n{err:?}"
            ))),
        };
    }
    if lazy {
        if let Err(err) = lazy_sparql_result_reader::read(
            resp.body().unwrap(),
            1000,
            limit,
            offset,
            async |mut partial_result: PartialResult| {
                let server = server_rc.lock().await;
                compress_result_uris(&*server, &mut partial_result);
                if let Err(err) =
                    server.send_message(PartialSparqlResultNotification::new(partial_result))
                {
                    log::error!(
                        "Could not send Partial-Sparql-Result-Notification:\n{:?}",
                        err
                    );
                }
            },
        )
        .await
        {
            match err {
                lazy_sparql_result_reader::SparqlResultReaderError::CorruptStream
                | lazy_sparql_result_reader::SparqlResultReaderError::JsonParseError(_) => {
                    Err(SparqlRequestError::Deserialization(format!("{err:?}")))
                }
                lazy_sparql_result_reader::SparqlResultReaderError::Canceled => {
                    Err(SparqlRequestError::_Canceled(CanceledError {
                        query: query.to_string(),
                    }))
                }
            }
        } else {
            Ok(None)
        }
    } else {
        // Get the response body as text and await it
        let text = read_reponse_body_as_text(resp).await?;
        // Return the text as a JsValue
        let result = serde_json::from_str(&text)
            .map_err(|err| SparqlRequestError::Deserialization(err.to_string()))?;
        Ok(Some(result))
    }
}

async fn read_reponse_body_as_text(response: Response) -> Result<String, SparqlRequestError> {
    JsFuture::from(
        response.text().map_err(|err| {
            SparqlRequestError::Response(format!("Response has no text:\n{:?}", err))
        })?,
    )
    .await
    .map_err(|err| {
        SparqlRequestError::Response(format!("Could not read Response text:\n{:?}", err))
    })?
    .as_string()
    .ok_or(SparqlRequestError::Response(
        "Could not read response body as utf-8 string".to_string(),
    ))
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
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

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
