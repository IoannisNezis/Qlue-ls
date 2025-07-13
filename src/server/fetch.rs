use crate::server::configuration::RequestMethod;
use crate::sparql::results::SparqlResult;
use urlencoding::encode;

/// Everything that can go wrong when sending a SPARQL request
/// - `Timeout`: The request took to long
/// - `Connection`: The Http connection could not be established
/// - `Response`: The responst had a non 200 status code
/// - `Deserialization`: The response could not be deserialized
pub(super) enum SparqlRequestError {
    Timeout,
    Connection,
    Response(String),
    Deserialization(String),
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) async fn fetch_sparql_result(
    url: &str,
    query: &str,
    timeout_ms: u32,
    method: RequestMethod,
) -> Result<SparqlResult, SparqlRequestError> {
    use reqwest::Client;
    use std::time::Duration;
    use tokio::time::timeout;

    let request = match method {
        RequestMethod::GET => Client::new()
            .get(format!("{}?query={}", url, encode(query)))
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
            .form(&[("query", query)])
            .send(),
    };

    let duration = Duration::from_millis(timeout_ms as u64);
    let request = timeout(duration, request);

    let response = request
        .await
        .map_err(|_| SparqlRequestError::Timeout)?
        .map_err(|_| SparqlRequestError::Connection)?
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
) -> Result<SparqlResult, SparqlRequestError> {
    use js_sys::JsString;
    use std::str::FromStr;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{AbortSignal, Request, RequestInit, RequestMode, Response, WorkerGlobalScope};

    let opts = RequestInit::new();
    opts.set_signal(Some(&AbortSignal::timeout_with_u32(timeout_ms)));

    let request = match method {
        RequestMethod::GET => {
            opts.set_method("GET");
            Request::new_with_str_and_init(&format!("{url}?query={}", encode(query)), &opts)
                .unwrap()
        }
        RequestMethod::POST => {
            opts.set_method("POST");
            opts.set_body(&JsString::from_str(query).unwrap());
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
            SparqlRequestError::Connection
        })?;

    let end = performance.now();
    log::debug!("Query took {:?}ms", (end - start) as i32,);

    // Cast the response value to a Response object
    let resp: Response = resp_value.dyn_into().unwrap();

    // Check if the response status is OK (200-299)
    if !resp.ok() {
        let status_text = resp.status_text();
        return Err(SparqlRequestError::Response(status_text));
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
