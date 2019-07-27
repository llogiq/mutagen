//! Mutator for comparison operations `<`, `<=`, `=>`, `>`

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_quote, BinOp, Expr, ExprBinary};

use crate::transform_info::SharedTransformInfo;
use crate::transformer::ExprTransformerOutput;
use crate::Mutation;

use crate::MutagenRuntimeConfig;

pub struct MutatorBinopCmp {}

impl MutatorBinopCmp {
    pub fn run<L: PartialOrd<R>, R>(
        mutator_id: u32,
        left: L,
        right: R,
        original_op: BinopCmp,
        runtime: MutagenRuntimeConfig,
    ) -> bool {
        let mutations = MutationBinopCmp::possible_mutations(original_op);
        if let Some(m) = runtime.get_mutation(mutator_id, &mutations) {
            m.mutate(left, right)
        } else {
            original_op.cmp(left, right)
        }
    }

    pub fn transform(e: Expr, transform_info: &SharedTransformInfo) -> ExprTransformerOutput {
        match e {
            Expr::Binary(ExprBinary {
                left,
                right,
                op,
                attrs,
            }) => {
                let (op, tt) = match op {
                    BinOp::Lt(t) => (BinopCmp::Lt, t.into_token_stream()),
                    BinOp::Le(t) => (BinopCmp::Le, t.into_token_stream()),
                    BinOp::Ge(t) => (BinopCmp::Ge, t.into_token_stream()),
                    BinOp::Gt(t) => (BinopCmp::Gt, t.into_token_stream()),
                    _ => {
                        return ExprTransformerOutput::unchanged(Expr::Binary(ExprBinary {
                            left,
                            right,
                            op,
                            attrs,
                        }))
                    }
                };

                let mutator_id = transform_info.add_mutations(
                    MutationBinopCmp::possible_mutations(op)
                        .into_iter()
                        .map(|m| m.to_mutation(op, tt.span())),
                );

                let expr = parse_quote! {
                    ::mutagen::mutator::MutatorBinopCmp::run::<_, _>(
                            #mutator_id,
                            #left,
                            #right,
                            #op,
                            ::mutagen::MutagenRuntimeConfig::get_default()
                        )
                };
                ExprTransformerOutput::changed(expr, tt.span())
            }
            _ => ExprTransformerOutput::unchanged(e),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct MutationBinopCmp {
    op: BinopCmp,
}

impl MutationBinopCmp {
    fn possible_mutations(original_op: BinopCmp) -> Vec<Self> {
        [BinopCmp::Lt, BinopCmp::Le, BinopCmp::Ge, BinopCmp::Gt]
            .into_iter()
            .copied()
            .filter(|&op| op != original_op)
            .map(|op| MutationBinopCmp { op })
            .collect()
    }

    fn mutate<L: PartialOrd<R>, R>(self, left: L, right: R) -> bool {
        self.op.cmp(left, right)
    }

    fn to_mutation(self, original_op: BinopCmp, span: Span) -> Mutation {
        Mutation::new_spanned(
            "binop_cmp".to_owned(),
            format!("replace `{}` with `{}`", original_op, self.op),
            span,
        )
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BinopCmp {
    Lt,
    Le,
    Ge,
    Gt,
}

impl BinopCmp {
    fn cmp<L: PartialOrd<R>, R>(self, left: L, right: R) -> bool {
        match self {
            BinopCmp::Lt => left < right,
            BinopCmp::Le => left <= right,
            BinopCmp::Ge => left >= right,
            BinopCmp::Gt => left > right,
        }
    }
}

impl ToTokens for BinopCmp {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote!(::mutagen::mutator::mutator_binop_cmp::BinopCmp::));
        tokens.extend(match self {
            BinopCmp::Lt => quote!(Lt),
            BinopCmp::Le => quote!(Le),
            BinopCmp::Ge => quote!(Ge),
            BinopCmp::Gt => quote!(Gt),
        })
    }
}

use std::fmt;

impl fmt::Display for BinopCmp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BinopCmp::Lt => write!(f, "<"),
            BinopCmp::Le => write!(f, "<="),
            BinopCmp::Ge => write!(f, ">="),
            BinopCmp::Gt => write!(f, ">"),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn possible_mutations_le() {
        assert_eq!(
            MutationBinopCmp::possible_mutations(BinopCmp::Le),
            vec![
                MutationBinopCmp { op: BinopCmp::Lt },
                MutationBinopCmp { op: BinopCmp::Ge },
                MutationBinopCmp { op: BinopCmp::Gt },
            ]
        )
    }

    #[test]
    fn possible_mutations_gt() {
        assert_eq!(
            MutationBinopCmp::possible_mutations(BinopCmp::Gt),
            vec![
                MutationBinopCmp { op: BinopCmp::Lt },
                MutationBinopCmp { op: BinopCmp::Le },
                MutationBinopCmp { op: BinopCmp::Ge },
            ]
        )
    }

    #[test]
    fn cmp_lt() {
        assert_eq!(BinopCmp::Lt.cmp(1, 2), true);
        assert_eq!(BinopCmp::Lt.cmp(3, 3), false);
        assert_eq!(BinopCmp::Lt.cmp(5, 4), false);
    }

    #[test]
    fn cmp_le() {
        assert_eq!(BinopCmp::Le.cmp(1, 2), true);
        assert_eq!(BinopCmp::Le.cmp(3, 3), true);
        assert_eq!(BinopCmp::Le.cmp(5, 4), false);
    }

    #[test]
    fn cmp_ge() {
        assert_eq!(BinopCmp::Ge.cmp(1, 2), false);
        assert_eq!(BinopCmp::Ge.cmp(3, 3), true);
        assert_eq!(BinopCmp::Ge.cmp(5, 4), true);
    }

    #[test]
    fn cmp_gt() {
        assert_eq!(BinopCmp::Gt.cmp(1, 2), false);
        assert_eq!(BinopCmp::Gt.cmp(3, 3), false);
        assert_eq!(BinopCmp::Gt.cmp(5, 4), true);
    }

    use crate::MutagenRuntimeConfig;

    #[test]
    fn mutator_cmp_gt_inactive() {
        assert_eq!(
            MutatorBinopCmp::run(
                1,
                1,
                2,
                BinopCmp::Gt,
                MutagenRuntimeConfig::with_mutation_id(0)
            ),
            false
        );
        assert_eq!(
            MutatorBinopCmp::run(
                1,
                5,
                4,
                BinopCmp::Gt,
                MutagenRuntimeConfig::with_mutation_id(0)
            ),
            true
        );
    }
    #[test]
    fn mutator_cmp_gt_active1() {
        assert_eq!(
            MutatorBinopCmp::run(
                1,
                1,
                2,
                BinopCmp::Gt,
                MutagenRuntimeConfig::with_mutation_id(1)
            ),
            true
        );
        assert_eq!(
            MutatorBinopCmp::run(
                1,
                3,
                3,
                BinopCmp::Gt,
                MutagenRuntimeConfig::with_mutation_id(1)
            ),
            false
        );
    }

}
