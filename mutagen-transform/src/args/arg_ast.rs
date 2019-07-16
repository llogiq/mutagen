//! A AST type for `#[mutate]` configuration via arguments.
//!
//! The token stream of the input args is parsed into the `ArgAst` type by `ArgAst::parse`.
//!
//! Please refer to the customization documentation about the format of arguments.

use proc_macro2::{Delimiter, TokenStream, TokenTree};

#[derive(Debug, Eq, PartialEq)]
pub struct ArgAst {
    pub arg_name: String,
    pub args: Vec<ArgAst>,
}

impl ArgAst {
    pub fn new_named(arg_name: String) -> Self {
        Self {
            arg_name,
            args: vec![],
        }
    }

    pub fn parse(input: TokenStream) -> Result<Vec<Self>, ()> {
        let mut args = Vec::new();

        let mut tt_iter = input.into_iter();
        while let Some(next) = tt_iter.next() {
            if let TokenTree::Ident(arg_name) = next {
                // register arg-name and get handle to current arg
                let arg_name = arg_name.to_string();
                args.push(Self::new_named(arg_name));
                let new_arg = args.last_mut().unwrap();

                match tt_iter.next() {
                    None => {}
                    Some(TokenTree::Punct(p)) => {
                        if p.as_char() != ',' {
                            return Err(());
                        }
                        continue;
                    }
                    Some(TokenTree::Group(g)) => {
                        if g.delimiter() != Delimiter::Parenthesis {
                            return Err(());
                        }
                        new_arg.args = Self::parse(g.stream())?;

                        match tt_iter.next() {
                            None => {}
                            Some(TokenTree::Punct(p)) => {
                                if p.as_char() != ',' {
                                    return Err(());
                                }
                                continue;
                            }
                            _ => return Err(()),
                        }
                    }
                    _ => return Err(()),
                }
            } else {
                return Err(());
            }
        }

        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proc_macro2::TokenStream;
    use std::str::FromStr;

    #[test]
    fn no_args() {
        let input = TokenStream::new();

        let parsed = ArgAst::parse(input);

        assert_eq!(parsed, Ok(vec![]));
    }

    #[test]
    fn single_arg() {
        let input = TokenStream::from_str("a1").unwrap();

        let parsed = ArgAst::parse(input);

        let expected = ArgAst::new_named("a1".to_string());
        assert_eq!(parsed, Ok(vec![expected]));
    }

    #[test]
    fn single_arg_with_args() {
        let input = TokenStream::from_str("a2(x, y, z)").unwrap();

        let parsed = ArgAst::parse(input);

        let mut expected = ArgAst::new_named("a2".to_string());
        expected.args.push(ArgAst::new_named("x".to_string()));
        expected.args.push(ArgAst::new_named("y".to_string()));
        expected.args.push(ArgAst::new_named("z".to_string()));
        assert_eq!(parsed, Ok(vec![expected]));
    }

    #[test]
    fn multiple_args() {
        let input = TokenStream::from_str("a2, b5").unwrap();

        let parsed = ArgAst::parse(input);

        let expected1 = ArgAst::new_named("a2".to_string());
        let expected2 = ArgAst::new_named("b5".to_string());
        assert_eq!(parsed, Ok(vec![expected1, expected2]));
    }

    #[test]
    fn nested_args() {
        let input = TokenStream::from_str("g55(h3(X))").unwrap();

        let parsed = ArgAst::parse(input);

        let mut expected = ArgAst::new_named("g55".to_string());
        expected.args.push(ArgAst::new_named("h3".to_string()));
        expected.args[0]
            .args
            .push(ArgAst::new_named("X".to_string()));
        assert_eq!(parsed, Ok(vec![expected]));
    }

    #[test]
    fn nested_args_with_trailing_arg() {
        let input = TokenStream::from_str("g55(h3(X), z)").unwrap();

        let parsed = ArgAst::parse(input);

        let mut expected = ArgAst::new_named("g55".to_string());
        expected.args.push(ArgAst::new_named("h3".to_string()));
        expected.args[0]
            .args
            .push(ArgAst::new_named("X".to_string()));
        expected.args.push(ArgAst::new_named("z".to_string()));
        assert_eq!(parsed, Ok(vec![expected]));
    }
}
