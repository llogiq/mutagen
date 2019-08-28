use syn::Stmt;

#[derive(Debug, Default)]
pub struct TransformContext {
    pub fn_name: Option<String>,
    pub original_stmt: Option<Stmt>
}
