use bau::tokenizer::token::TokenKind;
use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensLegend, SemanticTokensParams,
    SemanticTokensResult,
};

pub fn get_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::COMMENT,   // 0
            SemanticTokenType::KEYWORD,   // 1
            SemanticTokenType::OPERATOR,  // 2
            SemanticTokenType::NUMBER,    // 3
            SemanticTokenType::STRING,    // 4
            SemanticTokenType::TYPE,      // 5
            SemanticTokenType::PARAMETER, // 6
            SemanticTokenType::VARIABLE,  // 7
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
    let source = bau::source::Source::new(&file_content);
    let mut tokenizer = bau::tokenizer::Tokenizer::new(source.text());
    let bau_tokens = tokenizer.tokenize();
    let mut semantic_tokens = Vec::new();

    let mut prev_line = 0;
    let mut prev_token_start = 0;

    for bau_token in bau_tokens.iter() {
        let token_length = bau_token.range().span.len();
        let token_type = match bau_token_to_semantic_token_type(bau_token.kind()) {
            Some(token_type) => token_type,
            None => continue,
        };

        let line = bau_token.range().coords.line as u32;
        let column = bau_token.range().coords.column as u32;

        let delta_line = line - prev_line;
        let delta_start = if prev_line == line {
            column - prev_token_start
        } else {
            column
        };

        semantic_tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length: token_length as u32,
            token_type,
            token_modifiers_bitset: 0,
        });

        prev_line = line;
        prev_token_start = column;
    }
    semantic_tokens
}

fn bau_token_to_semantic_token_type(bau_token_kind: TokenKind) -> Option<u32> {
    match bau_token_kind {
        // Keywords
        TokenKind::Fn => Some(1),
        TokenKind::Extend => Some(1),
        TokenKind::Let => Some(1),
        TokenKind::If => Some(1),
        TokenKind::Else => Some(1),
        TokenKind::Loop => Some(1),
        TokenKind::Return => Some(1),
        TokenKind::Continue => Some(1),
        TokenKind::Break => Some(1),

        // Literals
        TokenKind::StringLiteral => Some(4),
        TokenKind::IntLiteral => Some(3),
        TokenKind::FloatLiteral => Some(3),
        TokenKind::BoolLiteral => Some(3),

        // Identifiers
        TokenKind::Identifier => Some(7),

        // Operators
        TokenKind::Plus => Some(2),
        TokenKind::Minus => Some(2),
        TokenKind::Asterisk => Some(2),
        TokenKind::Slash => Some(2),
        TokenKind::Percent => Some(2),
        TokenKind::ExclamationMark => Some(2),
        TokenKind::LessThan => Some(2),
        TokenKind::GreaterThan => Some(2),

        // Compound operators
        TokenKind::PlusEquals => Some(2),
        TokenKind::MinusEquals => Some(2),
        TokenKind::AsteriskEquals => Some(2),
        TokenKind::SlashEquals => Some(2),
        TokenKind::PercentEquals => Some(2),

        TokenKind::EqualsEquals => Some(2),
        TokenKind::ExclamationMarkEquals => Some(2),
        TokenKind::LessThanEquals => Some(2),
        TokenKind::GreaterThanEquals => Some(2),
        TokenKind::AmpersandAmpersand => Some(2),
        TokenKind::PipePipe => Some(2),

        // Punctuation
        TokenKind::Equals => None,
        TokenKind::Arrow => None,
        TokenKind::ParenOpen => None,
        TokenKind::ParenClose => None,
        TokenKind::BraceOpen => None,
        TokenKind::BraceClose => None,
        TokenKind::SquareOpen => None,
        TokenKind::SquareClose => None,
        TokenKind::Semicolon => None,
        TokenKind::Period => None,
        TokenKind::Comma => None,

        // Misc
        TokenKind::Comment => None,
        TokenKind::Whitespace => None,
        TokenKind::EndOfFile => None,
        TokenKind::Invalid => None,
    }
}
