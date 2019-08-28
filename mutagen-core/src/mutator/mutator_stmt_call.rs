//! Mutator for binary operation `+`.

use std::ops::Deref;

use quote::quote_spanned;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Expr, Stmt};

use crate::comm::Mutation;
use crate::transformer::transform_context::TransformContext;
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
        e: Stmt,
        transform_info: &SharedTransformInfo,
        context: &TransformContext,
    ) -> Stmt {
        let call: Box<dyn ToTokens> = match e {
            Stmt::Semi(Expr::MethodCall(call), _) => Box::new(call),
            Stmt::Semi(Expr::Call(call), _) => Box::new(call),
            _ => return e,
        };

        let mutator_id = transform_info.add_mutation(Mutation::new_spanned(
            context.fn_name.clone(),
            "stmt_methodcall".to_owned(),
            format!(
                "{}",
                context
                    .original_stmt
                    .to_token_stream()
                    .to_string()
                    .replace("\n", " ")
            ),
            "".to_owned(),
            call.span(),
        ));

        syn::parse2(quote_spanned! {call.span()=>
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
