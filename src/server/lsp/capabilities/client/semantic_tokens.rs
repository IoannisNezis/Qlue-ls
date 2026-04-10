use serde::Deserialize;

use super::super::{BoolOrEmpty, FullCapability};

/// Capabilities specific to the various semantic token requests.
///
/// https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#semanticTokensClientCapabilities
///
/// @since 3.16.0
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SemanticTokensClientCapabilities {
    /// Whether implementation supports dynamic registration. If this is set to
    /// `true` the client supports the new `(TextDocumentRegistrationOptions &
    /// StaticRegistrationOptions)` return value for the corresponding server
    /// capability as well.
    pub dynamic_registration: Option<bool>,

    /// Which requests the client supports and might send to the server
    /// depending on the server's capability. Please note that clients might not
    /// show semantic tokens or degrade some of the user experience if a range
    /// or full request is advertised by the client but not provided by the
    /// server. If for example the client capability `requests.full` and
    /// `request.range` are both set to true but the server only provides a
    /// range provider the client might not render a minimap correctly or might
    /// even decide to not show any semantic tokens at all.
    pub requests: SemanticTokensRequests,

    /// The token types that the client supports.
    pub token_types: Vec<String>,

    /// The token modifiers that the client supports.
    pub token_modifiers: Vec<String>,

    /// The formats the clients supports.
    pub formats: Vec<TokenFormat>,

    /// Whether the client supports tokens that can overlap each other.
    pub overlapping_token_support: Option<bool>,

    /// Whether the client supports tokens that can span multiple lines.
    pub multiline_token_support: Option<bool>,

    /// Whether the client allows the server to actively cancel a
    /// semantic token request, e.g. supports returning
    /// `ErrorCodes.ServerCancelled`. If a server does the client
    /// needs to retrigger the request.
    ///
    /// @since 3.17.0
    pub server_cancel_support: Option<bool>,

    /// Whether the client uses semantic tokens to augment existing
    /// syntax tokens. If set to `true` client side created syntax
    /// tokens and semantic tokens are both used for colorization. If
    /// set to `false` the client only uses the returned semantic tokens
    /// for colorization.
    ///
    /// If the value is `undefined` then the client behavior is not
    /// specified.
    ///
    /// @since 3.17.0
    pub augments_syntax_tokens: Option<bool>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct SemanticTokensRequests {
    pub range: Option<BoolOrEmpty>,
    pub full: Option<FullCapability>,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum TokenFormat {
    #[serde(rename = "relative")]
    Relative,
}
