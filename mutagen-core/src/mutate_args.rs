//! parse arguments for the `#[mutate]` attribute and gather all information necessary to transform the source code.
//!
//! Please refer to the customization documentation about the format of arguments.

use proc_macro2::TokenStream;

mod arg_ast;
mod arg_options;

use arg_options::{ArgOptions, Conf, Transformers};

use crate::transform_info::{SharedTransformInfo, GLOBAL_TRANSFORM_INFO};
use crate::transformer::{MutagenTransformer, MutagenTransformerBundle};

pub struct MutagenArgs {
    pub transformer_bundle: MutagenTransformerBundle,
    pub transform_info: SharedTransformInfo,
}

impl MutagenArgs {
    /// parse the arguments of the `#[mutate]` attribute
    pub fn args_from_attr(args: TokenStream) -> MutagenArgs {
        let options = ArgOptions::parse(args).expect("invalid options");

        // WIP: better error messages if format is not valid

        let transform_info: SharedTransformInfo = match options.conf {
            Conf::Global => {
                let transform_info = GLOBAL_TRANSFORM_INFO.clone_shared();
                transform_info.with_default_mutagen_file();
                transform_info
            }
            Conf::Local => Default::default(),
        };

        let transformers = match options.transformers {
            Transformers::All => MutagenTransformerBundle::all_transformers(),
            Transformers::Only(list) => {
                let mut transformers = list.transformers;
                transformers.sort_by_key(|t| MutagenTransformerBundle::transformer_order()[t]);
                transformers
            }
            Transformers::Not(list) => {
                let mut transformers = MutagenTransformerBundle::all_transformers();
                for l in &list.transformers {
                    transformers.remove_item(l);
                }
                transformers
            }
        };

        let mut expr_transformers = Vec::new();
        for t in &transformers {
            let t = MutagenTransformerBundle::mk_transformer(t, &[], transform_info.clone_shared());
            match t {
                MutagenTransformer::Expr(t) => expr_transformers.push(t),
            }
        }

        let transformer_bundle =MutagenTransformerBundle::new(expr_transformers);

        MutagenArgs {
            transformer_bundle,
            transform_info,
        }
    }

}
