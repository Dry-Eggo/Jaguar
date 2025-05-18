use crate::lexer::Span;

#[derive(Debug, Clone)]
pub enum ErrLevel {
    WARNING,
    ERROR,
    INFO,
}
#[derive(Debug, Clone)]
pub struct CompileError {
    pub errmsg: String,
    pub help: Option<String>,
    pub span: Span,
    pub level: ErrLevel,
}

impl CompileError {
    pub fn new(msg: String, help: Option<String>, span: Span, lvl: ErrLevel) -> Self {
        Self {
            errmsg: msg,
            help,
            span,
            level: lvl,
        }
    }
}
