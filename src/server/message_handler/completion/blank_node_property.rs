use super::{
    context::{CompletionContext, CompletionLocation},
    error::CompletionError,
};
use crate::server::{
    lsp::{textdocument::Range, CompletionList, InsertTextFormat, ItemDefaults},
    message_handler::completion::utils::{fetch_online_completions, get_prefix_declarations},
    Server,
};
use ll_sparql_parser::ast::{AstNode, QueryUnit};
use tera::Context;

static QUERY_TEMPLATE: &str = "predicate_completion.rq";

pub(super) async fn completions(
    server: &Server,
    context: CompletionContext,
) -> Result<CompletionList, CompletionError> {
    if let CompletionLocation::BlankNodeProperty(blank_node_props) = &context.location {
        let query_unit = QueryUnit::cast(context.tree.clone()).ok_or(
            CompletionError::ResolveError("Could not cast root to QueryUnit".to_string()),
        )?;
        let triple = blank_node_props
            .triple()
            .ok_or(CompletionError::ResolveError(
                "Could not find Triple from anchor node".to_string(),
            ))?;
        let subj = triple.subject().ok_or(CompletionError::ResolveError(
            "Triple has no subject".to_string(),
        ))?;
        let props = triple
            .properties_list_path()
            .ok_or(CompletionError::ResolveError(
                "Triple has no property list".to_string(),
            ))?
            .properties();
        let path = &props
            .last()
            .ok_or(CompletionError::ResolveError(
                "Property list is empty".to_string(),
            ))?
            .verb;
        let inject_context = format!(
            "{} {} ?qlue_ls_inner . ?qlue_ls_inner ?qlue_ls_value []",
            subj.text(),
            path.text()
        );
        let prefixes = get_prefix_declarations(server, &context, &triple);
        let mut template_context = Context::new();
        template_context.insert("context", &inject_context);
        template_context.insert("prefixes", &prefixes);
        let items = fetch_online_completions(
            server,
            &query_unit,
            context.backend.as_ref(),
            QUERY_TEMPLATE,
            template_context,
            Range::new(
                context.trigger_textdocument_position.line,
                context.trigger_textdocument_position.character,
                context.trigger_textdocument_position.line,
                context.trigger_textdocument_position.character,
            ),
        )
        .await?;
        Ok(CompletionList {
            is_incomplete: items.len() >= 100,
            item_defaults: Some(ItemDefaults {
                edit_range: None,
                commit_characters: None,
                data: None,
                insert_text_format: Some(InsertTextFormat::PlainText),
            }),
            items,
        })
    } else {
        panic!("object completions requested for non object location");
    }
}
