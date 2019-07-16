//! parse arguments for the `#[mutate]` attribute and gather all information necessary to transform the source code.
//!
//! Please refer to the customization documentation about the format of arguments.

use proc_macro2::TokenStream;

use crate::transform_info::{SharedTransformInfo, GLOBAL_TRANSFORM_INFO};
use crate::transformer::MutagenTransformerBundle;

mod arg_ast;
pub mod arg_options;

use arg_options::{ArgOptions, Conf};

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

        let transformer_bundle =
            MutagenTransformerBundle::new(options.transformers, &transform_info);

        MutagenArgs {
            transformer_bundle,
            transform_info,
        }
    }
}
