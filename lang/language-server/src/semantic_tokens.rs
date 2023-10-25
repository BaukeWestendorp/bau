use bau_core::tokenizer::token::TokenKind;
use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensLegend, SemanticTokensParams,
    SemanticTokensResult,
};

pub fn get_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::COMMENT,
            SemanticTokenType::KEYWORD,
            SemanticTokenType::OPERATOR,
            SemanticTokenType::NUMBER,
            SemanticTokenType::STRING,
            SemanticTokenType::TYPE,
            SemanticTokenType::PARAMETER,
            SemanticTokenType::VARIABLE,
        ],
        token_modifiers: vec![],
    }
}

pub fn handle_semantic_tokens_full(
    params: SemanticTokensParams,
) -> RpcResult<Option<SemanticTokensResult>> {
    let file = params.text_document.uri.path();
    let tokens = get_semantic_tokens(file);
    Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    })))
}

fn get_semantic_tokens(file: &str) -> Vec<SemanticToken> {
    let file_content = std::fs::read_to_string(file).unwrap();
    let source = bau_core::parser::source::Source::new(file_content, file.to_string());
    let mut tokenizer = bau_core::tokenizer::Tokenizer::new(source.text());
    let _bau_tokens = tokenizer.tokenize();
    // FIXME: Implement getting semantic tokens from Bau Tokens.
    vec![]
}

fn _bau_token_to_semantic_token_type(
    bau_token_kind: bau_core::tokenizer::token::TokenKind,
) -> Option<u32> {
    match bau_token_kind {
        TokenKind::Comment => Some(0),
        TokenKind::Extend => Some(1),
        TokenKind::Fn => Some(1),
        TokenKind::Let => Some(1),
        TokenKind::If => Some(1),
        TokenKind::Else => Some(1),
        TokenKind::Loop => Some(1),
        TokenKind::Return => Some(1),
        TokenKind::Continue => Some(1),
        TokenKind::Break => Some(1),

        TokenKind::Plus => Some(2),
        TokenKind::Minus => Some(2),
        TokenKind::Asterisk => Some(2),
        TokenKind::Slash => Some(2),
        TokenKind::Percent => Some(2),
        TokenKind::EqualsEquals => Some(2),
        TokenKind::ExclamationMarkEquals => Some(2),
        TokenKind::LessThan => Some(2),
        TokenKind::LessThanEquals => Some(2),
        TokenKind::GreaterThan => Some(2),
        TokenKind::GreaterThanEquals => Some(2),
        TokenKind::AmpersandAmpersand => Some(2),
        TokenKind::PipePipe => Some(2),

        TokenKind::Identifier => Some(7),
        TokenKind::IntLiteral => Some(3),
        TokenKind::FloatLiteral => Some(3),
        TokenKind::StringLiteral => Some(4),
        TokenKind::BoolLiteral => Some(1),

        _ => None,
    }
}
