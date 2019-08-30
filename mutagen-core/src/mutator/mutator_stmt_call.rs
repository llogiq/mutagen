//! Mutator for binary operation `+`.

use std::convert::TryFrom;
use std::ops::Deref;

use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Expr, Stmt};

use crate::comm::Mutation;
use crate::transformer::TransformContext;
use crate::transformer::transform_info::SharedTransformInfo;

use crate::MutagenRuntimeConfig;

pub struct MutatorStmtCall {}

impl MutatorStmtCall {
    pub fn should_run(
        mutator_id: usize,
        runtime: impl Deref<Target = MutagenRuntimeConfig>,
    ) -> bool {
        runtime.covered(mutator_id);
        // should run if mutation is inactive
        !runtime.is_mutation_active(mutator_id)
    }

    pub fn transform(
        s: Stmt,
        transform_info: &SharedTransformInfo,
        context: &TransformContext,
    ) -> Stmt {
        let s = match StmtCall::try_from(s) {
            Ok(s) => s,
            Err(s) => return s,
        };

        let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
            &context,
            "stmt_call".to_owned(),
            format!(
                "{}",
                context
                    .original_stmt
                    .to_token_stream()
                    .to_string()
                    .replace("\n", " ")
            ),
            "".to_owned(),
            s.span,
        ));

        let call = &s.call;

        syn::parse2(quote_spanned! {s.span=>
            if ::mutagen::mutator::MutatorStmtCall::should_run(
                    #mutator_id,
                    ::mutagen::MutagenRuntimeConfig::get_default()
                )
            {
                #call;
            }
        })
        .expect("transformed code invalid")
    }
}

#[derive(Debug, Clone)]
struct StmtCall {
    call: TokenStream,
    span: Span,
}

impl TryFrom<Stmt> for StmtCall {
    type Error = Stmt;
    fn try_from(stmt: Stmt) -> Result<Self, Stmt> {
        match stmt {
            Stmt::Semi(Expr::MethodCall(call), _) => Ok(StmtCall {
                span: call.span(),
                call: call.into_token_stream(),
            }),
            Stmt::Semi(Expr::Call(call), _) => Ok(StmtCall {
                span: call.span(),
                call: call.into_token_stream(),
            }),
            _ => return Err(stmt),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn stmt_inactive() {
        let result = MutatorStmtCall::should_run(1, &MutagenRuntimeConfig::without_mutation());
        assert_eq!(result, true);
    }
    #[test]
    fn stmt_active() {
        let result = MutatorStmtCall::should_run(1, &MutagenRuntimeConfig::with_mutation_id(1));
        assert_eq!(result, false);
    }
}
