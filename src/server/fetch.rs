use crate::server::configuration::RequestMethod;
use crate::server::lsp::QLeverException;
use crate::sparql::results::SparqlResult;
use ll_sparql_parser::ast::{AstNode, QueryUnit};
use ll_sparql_parser::parse_query;
use serde::{Deserialize, Serialize};
use urlencoding::encode;

/// Everything that can go wrong when sending a SPARQL request
/// - `Timeout`: The request took to long
/// - `Connection`: The Http connection could not be established
/// - `Response`: The responst had a non 200 status code
/// - `Deserialization`: The response could not be deserialized
#[derive(Debug)]
pub(super) enum SparqlRequestError {
    Timeout,
    Connection(ConnectionError),
    Response(String),
    Deserialization(String),
    QLeverException(QLeverException),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionError {
    pub query: String,
    pub status_text: String,
}

#[derive(Debug)]
pub struct Window {
    window_size: u32,
    window_offset: u32,
}

impl Window {
    pub fn new(window_size: u32, window_offset: u32) -> Self {
        Self {
            window_size,
            window_offset,
        }
    }

    fn rewrite(&self, query: &str) -> Option<String> {
        let syntax_tree = QueryUnit::cast(parse_query(query))?;
        let select_query = syntax_tree.select_query()?;
        Some(format!(
            "{}{}{}",
            &query[0..select_query.syntax().text_range().start().into()],
            format!(
                "SELECT * WHERE {{\n{}\n}}\nLIMIT {}\nOFFSET {}",
                select_query.text(),
                self.window_size,
                self.window_offset
            ),
            &query[select_query.syntax().text_range().end().into()..]
        ))
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn fetch_sparql_result(
    url: &str,
    query: &str,
    timeout_ms: u32,
    method: RequestMethod,
    window: Option<Window>,
) -> Result<SparqlResult, SparqlRequestError> {
    use reqwest::Client;
    use std::time::Duration;
    use tokio::time::timeout;

    let query = window
        .and_then(|window| window.rewrite(query))
        .unwrap_or(query.to_string());

    let request = match method {
        RequestMethod::GET => Client::new()
            .get(format!("{}?query={}", url, encode(&query)))
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded;charset=UTF-8",
            )
            .header("Accept", "application/sparql-results+json")
            .header("User-Agent", "qlue-ls/1.0")
            .send(),
        RequestMethod::POST => Client::new()
            .post(url)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded;charset=UTF-8",
            )
            .header("Accept", "application/sparql-results+json")
            .header("User-Agent", "qlue-ls/1.0")
            .form(&[("query", &query)])
            .send(),
    };

    let duration = Duration::from_millis(timeout_ms as u64);
    let request = timeout(duration, request);

    let response = request
        .await
        .map_err(|_| SparqlRequestError::Timeout)?
        .map_err(|err| {
            SparqlRequestError::Connection(ConnectionError {
                status_text: err.to_string(),
                query,
            })
        })?
        .error_for_status()
        .map_err(|err| {
            log::debug!("Error: {:?}", err.status());
            SparqlRequestError::Response("failed".to_string())
        })?;

    response
        .json::<SparqlResult>()
        .await
        .map_err(|err| SparqlRequestError::Deserialization(err.to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn check_server_availability(url: &str) -> bool {
    use reqwest::Client;
    let response = Client::new().get(url).send();
    response.await.is_ok_and(|res| res.status() == 200)
    // let opts = RequestInit::new();
    // opts.set_method("GET");
    // opts.set_mode(RequestMode::Cors);
    // let request = Request::new_with_str_and_init(url, &opts).expect("Failed to create request");
    // let resp_value = match JsFuture::from(worker_global.fetch_with_request(&request)).await {
    //     Ok(resp) => resp,
    //     Err(_) => return false,
    // };
    // let resp: Response = resp_value.dyn_into().unwrap();
    // resp.ok()
}

#[cfg(target_arch = "wasm32")]
pub(crate) async fn fetch_sparql_result(
    url: &str,
    query: &str,
    timeout_ms: u32,
    method: RequestMethod,
    window: Option<Window>,
) -> Result<SparqlResult, SparqlRequestError> {
    use js_sys::JsString;
    use std::str::FromStr;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{AbortSignal, Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

    let query = window
        .and_then(|window| window.rewrite(query))
        .unwrap_or(query.to_string());

    let opts = RequestInit::new();
    opts.set_signal(Some(&AbortSignal::timeout_with_u32(timeout_ms)));

    let request = match method {
        RequestMethod::GET => {
            opts.set_method("GET");
            Request::new_with_str_and_init(&format!("{url}?query={}", encode(&query)), &opts)
                .unwrap()
        }
        RequestMethod::POST => {
            opts.set_method("POST");
            opts.set_body(&JsString::from_str(&query).unwrap());
            Request::new_with_str_and_init(url, &opts).unwrap()
        }
    };
    let headers = request.headers();
    if method == RequestMethod::POST {
        headers
            .set("Content-Type", "application/sparql-query")
            .unwrap();
    }
    headers
        .set("Accept", "application/sparql-results+json")
        .unwrap();
    // headers.set("User-Agent", "qlue-ls/1.0").unwrap();

    // Get global worker scope
    let worker_global: WorkerGlobalScope = js_sys::global().unchecked_into();

    let performance = worker_global
        .performance()
        .expect("performance should be available");

    let start = performance.now();

    // Perform the fetch request and await the response
    let resp_value = JsFuture::from(worker_global.fetch_with_request(&request))
        .await
        .map_err(|err| {
            log::error!("{err:?}");
            SparqlRequestError::Connection(ConnectionError {
                status_text: format!("{err:?}"),
                query,
            })
        })?;

    let end = performance.now();
    log::debug!("Query took {:?}ms", (end - start) as i32,);

    // Cast the response value to a Response object
    let resp: Response = resp_value.dyn_into().unwrap();

    // Check if the response status is OK (200-299)
    if !resp.ok() {
        return match resp.json() {
            Ok(json) => {
                match JsFuture::from(json).await {
                    Ok(js_value) => match serde_wasm_bindgen::from_value(js_value) {
                        Ok(err) =>  Err(SparqlRequestError::QLeverException(err)),
                        Err(err) => {
                            Err(SparqlRequestError::Deserialization(format!(
                                "Could not deserialize error message: {}",
                                err
                            )))
                        }
                    },
                    Err(err) => {
                        Err(SparqlRequestError::Deserialization(
                            format!("Query failed! Response did not provide a json body but this could not be cast to rust JsValue.\n{:?}", err),
                        ))
                    }
                }
            }
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
    // Return the text as a JsValue
    serde_json::from_str(&text).map_err(|err| SparqlRequestError::Deserialization(err.to_string()))
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(test)]
mod test {
    use indoc::indoc;

    use crate::server::fetch::Window;

    #[test]
    fn window_rewrite_query() {
        let window = Window {
            window_size: 100,
            window_offset: 20,
        };
        let query = indoc! {
            "Prefix ab: <ab>
             Select * WHERE {
               ?a ?b ?c
             }
             Limit 1000
             VALUES ?x {42}
            "
        };
        assert_eq!(
            window.rewrite(&query).expect("Should add request window"),
            indoc! {
            "Prefix ab: <ab>
             SELECT * WHERE {
             Select * WHERE {
               ?a ?b ?c
             }
             Limit 1000
             }
             LIMIT 100
             OFFSET 20
             VALUES ?x {42}
            "
            }
        );
    }
}
