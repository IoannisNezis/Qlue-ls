use crate::server::Server;
use crate::server::configuration::RequestMethod;
use crate::server::lsp::ExecuteUpdateResponseResult;
use crate::server::lsp::SparqlEngine;
use crate::server::sparql_operations::ConnectionError;
use crate::server::sparql_operations::SparqlRequestError;
use crate::sparql::results::SparqlResult;
use futures::lock::Mutex;
use reqwest::Client;
use std::rc::Rc;
use std::time::Duration;
use tokio::time::timeout;
use urlencoding::encode;

pub(crate) async fn execute_query(
    _server_rc: Rc<Mutex<Server>>,
    url: String,
    mut query: String,
    _query_id: Option<&str>,
    _engine: Option<SparqlEngine>,
    timeout_ms: Option<u32>,
    method: RequestMethod,
    limit: Option<usize>,
    offset: usize,
    lazy: bool,
) -> Result<Option<SparqlResult>, SparqlRequestError> {
    if lazy {
        log::warn!("Lazy Query execution is not implemented for non wasm targets");
    }
    if let Some(new_query) = add_limit_offset_to_query(&query, limit, offset) {
        query = new_query;
    }

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

    // FIXME: Proper timout / cancel solution for native target
    let duration = Duration::from_millis(timeout_ms.unwrap_or(5000) as u64);
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

    let result = response
        .json::<SparqlResult>()
        .await
        .map_err(|err| SparqlRequestError::Deserialization(err.to_string()))?;
    Ok(Some(result))
}

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

pub(crate) async fn execute_construct_query(
    _server_rc: Rc<Mutex<Server>>,
    _url: &str,
    _query: &str,
    _query_id: Option<&str>,
    _engine: Option<SparqlEngine>,
    _lazy: bool,
) -> Result<Option<SparqlResult>, SparqlRequestError> {
    todo!()
}

pub(crate) async fn execute_update(
    _server_rc: Rc<Mutex<Server>>,
    _url: &str,
    _query: &str,
    _query_id: Option<&str>,
    _access_token: Option<&str>,
) -> Result<Vec<ExecuteUpdateResponseResult>, SparqlRequestError> {
    todo!()
}

fn add_limit_offset_to_query(query: &str, limit: Option<usize>, offset: usize) -> Option<String> {
    if limit.is_none() && offset == 0 {
        return None;
    }
    use ll_sparql_parser::{
        ast::{AstNode, QueryUnit},
        parse_query,
    };
    let syntax_tree = QueryUnit::cast(parse_query(query))?;
    let select_query = syntax_tree.select_query()?;
    Some(format!(
        "{}{}{}",
        &query[0..select_query.syntax().text_range().start().into()],
        format!(
            "SELECT * WHERE {{\n{}\n}}\n{}OFFSET {}",
            select_query.text(),
            limit.map_or(String::new(), |limit| format!("LIMIT {limit}\n")),
            offset
        ),
        &query[select_query.syntax().text_range().end().into()..]
    ))
}
