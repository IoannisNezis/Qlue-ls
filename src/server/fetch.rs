use std::str::FromStr;

use js_sys::JsString;
use sparql::results::SparqlResult;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

use super::lsp::errors::{ErrorCode, ResponseError};

pub(crate) async fn fetch_sparql_result(
    url: &str,
    query: &str,
) -> Result<SparqlResult, ResponseError> {
    let opts = RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&JsString::from_str(query).unwrap());
    let request = Request::new_with_str_and_init(url, &opts).map_err(|err| {
        ResponseError::new(
            ErrorCode::InternalError,
            &format!("Could not init request:\n{:?}", err),
        )
    })?;
    request
        .headers()
        .set("Content-Type", "application/sparql-query")
        .map_err(|err| {
            ResponseError::new(
                ErrorCode::InternalError,
                &format!("Could not set Header:\n{:?}", err),
            )
        })?;

    // Get global worker scope
    let worker_global = js_sys::global().unchecked_into::<web_sys::WorkerGlobalScope>();

    // Perform the fetch request and await the response
    let resp_value = JsFuture::from(worker_global.fetch_with_request(&request))
        .await
        .map_err(|err| {
            ResponseError::new(
                ErrorCode::InternalError,
                &format!("SPARQL request failed:\n{:?}", err),
            )
        })?;

    // Cast the response value to a Response object
    let resp: Response = resp_value.dyn_into().map_err(|err| {
        ResponseError::new(
            ErrorCode::InternalError,
            &format!("Could not cast reponse:\n{:?}", err),
        )
    })?;

    // Check if the response status is OK (200-299)
    if !resp.ok() {
        let status = resp.status();
        let status_text = resp.status_text();
        return Err(ResponseError::new(
            ErrorCode::InternalError,
            &format!(
                "SPARQL request failed:\nHTTP error: {} {}",
                status, status_text
            ),
        ));
    }

    // Get the response body as text and await it
    let text = JsFuture::from(resp.text().map_err(|err| {
        ResponseError::new(
            ErrorCode::InternalError,
            &format!("Response has no text:\n{:?}", err),
        )
    })?)
    .await
    .map_err(|err| {
        ResponseError::new(
            ErrorCode::InternalError,
            &format!("Could not read Response text:\n{:?}", err),
        )
    })?
    .as_string()
    .unwrap();
    // Return the text as a JsValue
    serde_json::from_str(&text).map_err(|err| {
        ResponseError::new(
            ErrorCode::InternalError,
            &format!("Failed to parse SPARQL response:\n{}", err),
        )
    })
}
