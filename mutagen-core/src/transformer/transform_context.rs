#[derive(Debug, Default)]
pub struct TransformContext {
    pub impl_name: Option<String>,
    pub fn_name: Option<String>,
    pub original_stmt: Option<syn::Stmt>,
    pub original_expr: Option<syn::Expr>,
}
