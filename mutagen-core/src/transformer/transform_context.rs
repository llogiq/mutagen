use super::ast_inspect;

#[derive(Debug, Default)]
pub struct TransformContext {
    pub impl_name: Option<String>,
    pub fn_name: Option<String>,
    pub original_stmt: Option<syn::Stmt>,
    pub original_expr: Option<syn::Expr>,
}

impl TransformContext {
    pub fn is_num_expr(&self) -> bool {
        self.original_expr
            .as_ref()
            .map(|e| ast_inspect::is_num_expr(e))
            .unwrap_or(false)
    }
}
