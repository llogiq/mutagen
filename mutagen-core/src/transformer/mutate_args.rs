//! parse arguments for the `#[mutate]` attribute and gather all information necessary to transform the source code.
//!
//! Please refer to the customization documentation about the format of arguments.

use super::arg_ast::{ArgAstList, ArgFn};
use proc_macro2::TokenStream;

#[derive(PartialEq, Eq, Debug)]
pub struct ArgOptions {
    pub conf: Conf,
    pub transformers: Transformers,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Transformers {
    All,
    Only(TransformerList),
    Not(TransformerList),
}

#[derive(PartialEq, Eq, Debug)]
pub enum Conf {
    Global,
    Local(LocalConf),
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct LocalConf {
    pub expected_mutations: Option<usize>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct TransformerList {
    pub transformers: Vec<String>,
}

impl Default for ArgOptions {
    fn default() -> Self {
        Self {
            conf: Conf::Global,
            transformers: Transformers::All,
        }
    }
}

impl ArgOptions {
    pub fn parse(args: TokenStream) -> Result<Self, ()> {
        let mut options: Self = Default::default();

        let ast = ArgAstList::parse_list(args)?;
        if let Some(conf) = ast.find_named_arg("conf")? {
            options.conf = Conf::parse(conf)?;
        }
        if let Some(transformers_arg) = ast.find_named_arg("mutators")? {
            match &*transformers_arg.name {
                "only" => {
                    options.transformers = Transformers::parse_only(&transformers_arg.args)?;
                }
                "not" => {
                    options.transformers = Transformers::parse_not(&transformers_arg.args)?;
                }
                _ => return Err(()),
            }
        }
        Ok(options)
    }
}

impl Conf {
    fn parse(conf: &ArgFn) -> Result<Self, ()> {
        match &*conf.name {
            "local" => {
                let expected_mutations = conf.args.find_named_arg("expected_mutations")?;
                let expected_mutations = expected_mutations
                    .map(|arg| arg.name.parse::<usize>())
                    .transpose()
                    .map_err(|_| ())?;
                Ok(Conf::Local(LocalConf { expected_mutations }))
            }
            "global" => Ok(Conf::Global),
            _ => Err(()),
        }
    }
}

impl TransformerList {
    fn parse(ast: &ArgAstList) -> Result<Self, ()> {
        let transformers = ast
            .0
            .iter()
            .map(|t| {
                let t = t.expect_fn_ref()?;
                if !t.args.0.is_empty() {
                    return Err(());
                }
                Ok(t.name.clone())
            })
            .collect::<Result<Vec<_>, ()>>()?;
        Ok(Self { transformers })
    }
}

impl Transformers {
    fn parse_only(ast: &ArgAstList) -> Result<Self, ()> {
        Ok(Transformers::Only(TransformerList::parse(ast)?))
    }

    fn parse_not(ast: &ArgAstList) -> Result<Self, ()> {
        Ok(Transformers::Not(TransformerList::parse(ast)?))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    #[test]
    fn config_for_empty_args() {
        let input = TokenStream::new();

        let parsed = ArgOptions::parse(input);

        let expected = Ok(ArgOptions::default());

        assert_eq!(expected, parsed);
    }

    #[test]
    fn config_local() {
        let input = TokenStream::from_str("conf = local").unwrap();

        let parsed = ArgOptions::parse(input);

        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();

        let expected_conf_local = Conf::Local(LocalConf::default());
        assert_eq!(parsed.conf, expected_conf_local);
        assert_eq!(parsed.transformers, Transformers::All);
    }

    #[test]
    fn config_local_single_mutator() {
        let input = TokenStream::from_str("conf = local, mutators = only(binop_add)").unwrap();

        let parsed = ArgOptions::parse(input);

        assert!(parsed.is_ok());
        let parsed = parsed.unwrap();

        let expected_conf_local = Conf::Local(LocalConf::default());
        assert_eq!(parsed.conf, expected_conf_local);

        let expected_transformers = Transformers::Only(TransformerList {
            transformers: vec!["binop_add".to_owned()],
        });
        assert_eq!(parsed.transformers, expected_transformers);
    }
}
