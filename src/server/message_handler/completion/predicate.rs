use indoc::indoc;
use ll_sparql_parser::ast::{AstNode, QueryUnit};

use crate::server::{
    fetch::fetch_sparql_result,
    lsp::{CompletionItem, CompletionItemKind, CompletionTriggerKind, InsertTextFormat},
    Server,
};

use super::{utils::compress_rdf_term, CompletionContext};

static QUERY: &str = indoc! {
"PREFIX dblp: <https://dblp.org/rdf/schema#>
 SELECT ?pred (COUNT(?pred) as ?count)  WHERE {
   ?s ?pred ?o .
 }
 GROUP BY ?pred
 ORDER BY DESC(?count)
 LIMIT 100
"
};

pub(super) async fn completions(
    server: &Server,
    context: CompletionContext,
) -> Vec<CompletionItem> {
    let query_unit = QueryUnit::cast(context.tree).unwrap();
    if context.trigger_kind == CompletionTriggerKind::TriggerCharacter
        && context.trigger_character.map_or(false, |tc| tc == "?")
    {
        log::info!("varibale completion");
        if let Some(select_query) = query_unit.select_query() {
            select_query
                .variables()
                .iter()
                .map(|var| {
                    let var_text = var.var_name();
                    CompletionItem::new(
                        &var_text,
                        "Variable",
                        &format!("{} ", var_text),
                        CompletionItemKind::Variable,
                        InsertTextFormat::PlainText,
                        None,
                    )
                })
                .collect()
        } else {
            vec![]
        }
    } else if let Some(backend) = &server.state.backend {
        match fetch_sparql_result(&backend.url, QUERY).await {
            Ok(result) => result
                .results
                .bindings
                .into_iter()
                .map(|binding| {
                    let pred = binding.get("pred").unwrap();
                    let count = binding.get("count").unwrap();
                    let (value, import_edit) = compress_rdf_term(server, &query_unit, pred);
                    CompletionItem::new(
                        &value,
                        &format!("{} occurences", count),
                        &format!("{} ", value),
                        CompletionItemKind::Value,
                        InsertTextFormat::PlainText,
                        import_edit.map(|edit| vec![edit]),
                    )
                })
                .collect(),
            Err(err) => {
                log::error!("{:?}", err);
                vec![]
            }
        }
    } else {
        vec![]
    }
}
