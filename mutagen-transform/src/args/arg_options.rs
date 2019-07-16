use proc_macro2::TokenStream;

use super::arg_ast::ArgAst;

pub struct ArgOptions {
    pub conf: Conf,
    pub transformers: Transformers,
    pub transformer_confs: Vec<TransformerConf>,
}

pub enum Transformers {
    All,
    Only(TransformerList),
    Not(TransformerList),
}

pub enum Conf {
    Global,
    Local,
}

pub struct TransformerList {
    pub transformers: Vec<String>,
}

pub struct TransformerConf {
    pub name: String,
    pub conf: Vec<String>,
}

impl Default for ArgOptions {
    fn default() -> Self {
        Self {
            conf: Conf::Global,
            transformers: Transformers::All,
            transformer_confs: Vec::new(),
        }
    }
}

impl ArgOptions {
    pub fn parse(args: TokenStream) -> Result<Self, ()> {
        let mut options: Self = Default::default();

        let ast = ArgAst::parse(args)?;
        for arg in ast {
            match &*arg.arg_name {
                "conf" => {
                    options.conf = Conf::parse(arg.args)?;
                }
                "only" => {
                    options.transformers = Transformers::parse_only(arg.args)?;
                }
                "not" => {
                    options.transformers = Transformers::parse_not(arg.args)?;
                }
                m => {
                    options
                        .transformer_confs
                        .push(TransformerConf::parse(m, arg.args)?);
                }
            }
        }

        Ok(options)
    }
}

impl Conf {
    fn parse(ast: Vec<ArgAst>) -> Result<Self, ()> {
        if ast.is_empty() {
            return Err(());
        }
        if ast.len() > 1 {
            return Err(());
        }
        match &*ast[0].arg_name {
            "local" => Ok(Conf::Local),
            "global" => Ok(Conf::Global),
            _ => Err(()),
        }
    }
}

impl TransformerList {
    fn parse(ast: Vec<ArgAst>) -> Result<Self, ()> {
        let mut transformers = Vec::new();

        for transformer in ast {
            if !transformer.args.is_empty() {
                return Err(());
            }
            transformers.push(transformer.arg_name);
        }

        Ok(Self { transformers })
    }
}

impl Transformers {
    fn parse_only(ast: Vec<ArgAst>) -> Result<Self, ()> {
        Ok(Transformers::Only(TransformerList::parse(ast)?))
    }

    fn parse_not(ast: Vec<ArgAst>) -> Result<Self, ()> {
        Ok(Transformers::Not(TransformerList::parse(ast)?))
    }
}

impl TransformerConf {
    fn parse(_transformer: &str, _ast: Vec<ArgAst>) -> Result<Self, ()> {
        Err(())
    }
}
