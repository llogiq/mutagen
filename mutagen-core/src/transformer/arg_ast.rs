//! A AST type for `#[mutate]` configuration via arguments.
//!
//! The token stream of the input args is parsed into the `ArgAst` type by `ArgAstList::parse_list`.
//!
//! Please refer to the customization documentation about the format of arguments.

use proc_macro2::{Delimiter, TokenStream, TokenTree};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ArgAstList(pub Vec<ArgAst>);

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ArgAst {
    ArgFn(ArgFn),
    ArgEq(ArgEq),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ArgFn {
    pub name: String,
    pub args: ArgAstList,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ArgEq {
    pub name: String,
    pub val: ArgFn,
}

impl ArgAstList {
    pub fn parse_list(input: TokenStream) -> Result<Self, ()> {
        let mut args = Vec::new();

        let mut tt_iter = input.into_iter();
        while let Some(next) = tt_iter.next() {
            let name = if let TokenTree::Ident(next) = next {
                next.to_string()
            } else if let TokenTree::Literal(next) = next {
                next.to_string()
            } else {
                return Err(());
            };

            args.push(ArgAst::parse_single(name, &mut tt_iter)?);
        }

        Ok(Self(args))
    }

    pub fn find_named_arg(&self, name: &str) -> Result<Option<&ArgFn>, ()> {
        let named_args = self
            .0
            .iter()
            .filter(|ast| ast.name() == name)
            .map(|ast| Ok(&ast.expect_eq_ref()?.val))
            .collect::<Result<Vec<&ArgFn>, ()>>()?;
        if named_args.len() > 1 {
            return Err(());
        }
        Ok(named_args.get(0).copied())
    }
}

impl ArgAst {
    fn new_fn(name: String, args: ArgAstList) -> Self {
        Self::ArgFn(ArgFn::new(name, args))
    }
    fn new_eq(name: String, val: ArgFn) -> Self {
        Self::ArgEq(ArgEq::new(name, val))
    }

    pub fn name(&self) -> &str {
        match self {
            ArgAst::ArgFn(ArgFn { name, .. }) => name,
            ArgAst::ArgEq(ArgEq { name, .. }) => name,
        }
    }

    fn parse_single(
        name: String,
        tt_iter: &mut impl Iterator<Item = TokenTree>,
    ) -> Result<Self, ()> {
        match tt_iter.next() {
            None => return Ok(Self::new_fn(name, ArgAstList(vec![]))),

            // parse fn-variant
            Some(TokenTree::Group(g)) => {
                if g.delimiter() != Delimiter::Parenthesis {
                    return Err(());
                }
                let args = ArgAstList::parse_list(g.stream())?;
                tt_expect_comma_or_end(tt_iter)?;
                return Ok(Self::new_fn(name, args));
            }

            // parse eq-variant
            Some(TokenTree::Punct(p)) => {
                if p.as_char() == ',' {
                    return Ok(Self::new_fn(name, ArgAstList(vec![])));
                }
                if p.as_char() != '=' {
                    return Err(());
                }

                let next = tt_iter.next();
                let next = if let Some(TokenTree::Ident(next)) = next {
                    next.to_string()
                } else if let Some(TokenTree::Literal(next)) = next {
                    next.to_string()
                } else {
                    return Err(());
                };

                // parse value, only allow ArgFn values.
                let val = Self::parse_single(next, tt_iter)?.expect_fn()?;
                return Ok(Self::new_eq(name, val));
            }
            _ => return Err(()),
        }
    }

    pub fn expect_fn(self) -> Result<ArgFn, ()> {
        match self {
            ArgAst::ArgFn(f) => Ok(f),
            ArgAst::ArgEq(_) => Err(()),
        }
    }
    pub fn expect_fn_ref(&self) -> Result<&ArgFn, ()> {
        match self {
            ArgAst::ArgFn(f) => Ok(f),
            ArgAst::ArgEq(_) => Err(()),
        }
    }
    pub fn expect_eq_ref(&self) -> Result<&ArgEq, ()> {
        match self {
            ArgAst::ArgFn(_) => Err(()),
            ArgAst::ArgEq(e) => Ok(e),
        }
    }
}

fn tt_expect_comma_or_end(tt_iter: &mut impl Iterator<Item = TokenTree>) -> Result<(), ()> {
    match tt_iter.next() {
        None => {}
        Some(TokenTree::Punct(p)) => {
            if p.as_char() != ',' {
                return Err(());
            }
        }
        _ => return Err(()),
    }
    Ok(())
}

impl ArgFn {
    pub fn new(name: String, args: ArgAstList) -> Self {
        Self { name, args }
    }
}

impl ArgEq {
    pub fn new(name: String, val: ArgFn) -> Self {
        Self { name, val }
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

        let parsed = ArgAstList::parse_list(input);

        assert_eq!(parsed, Ok(ArgAstList(vec![])));
    }

    #[test]
    fn single_arg() {
        let input = TokenStream::from_str("a1").unwrap();

        let parsed = ArgAstList::parse_list(input);

        let expected = ArgAst::new_fn("a1".to_string(), ArgAstList(vec![]));
        assert_eq!(parsed, Ok(ArgAstList(vec![expected])));
    }

    #[test]
    fn single_arg_int() {
        let input = TokenStream::from_str("1").unwrap();

        let parsed = ArgAstList::parse_list(input);

        let expected = ArgAst::new_fn("1".to_string(), ArgAstList(vec![]));
        assert_eq!(parsed, Ok(ArgAstList(vec![expected])));
    }

    #[test]
    fn single_arg_with_args() {
        let input = TokenStream::from_str("a2(x, y, z)").unwrap();

        let parsed = ArgAstList::parse_list(input);

        let mut expected = ArgFn::new("a2".to_string(), ArgAstList(vec![]));
        expected
            .args
            .0
            .push(ArgAst::new_fn("x".to_string(), ArgAstList(vec![])));
        expected
            .args
            .0
            .push(ArgAst::new_fn("y".to_string(), ArgAstList(vec![])));
        expected
            .args
            .0
            .push(ArgAst::new_fn("z".to_string(), ArgAstList(vec![])));
        let expected = ArgAst::ArgFn(expected);
        assert_eq!(parsed, Ok(ArgAstList(vec![expected])));
    }

    #[test]
    fn single_arg_with_trailing_comma() {
        let input = TokenStream::from_str("a2(x,)").unwrap();

        let parsed = ArgAstList::parse_list(input);

        let mut expected = ArgFn::new("a2".to_string(), ArgAstList(vec![]));
        expected
            .args
            .0
            .push(ArgAst::new_fn("x".to_string(), ArgAstList(vec![])));
        let expected = ArgAst::ArgFn(expected);
        assert_eq!(parsed, Ok(ArgAstList(vec![expected])));
    }

    #[test]
    fn single_arg_with_eq_args() {
        let input = TokenStream::from_str("a2(x=a)").unwrap();

        let parsed = ArgAstList::parse_list(input);

        let mut expected = ArgFn::new("a2".to_string(), ArgAstList(vec![]));
        expected.args.0.push(ArgAst::new_eq(
            "x".to_string(),
            ArgFn::new("a".to_owned(), ArgAstList(vec![])),
        ));
        let expected = ArgAst::ArgFn(expected);
        assert_eq!(parsed, Ok(ArgAstList(vec![expected])));
    }

    #[test]
    fn chained_eq_gives_error() {
        let input = TokenStream::from_str("a = b = c").unwrap();
        let parsed = ArgAstList::parse_list(input);
        assert_eq!(parsed, Err(()));
    }

    #[test]
    fn multiple_args() {
        let input = TokenStream::from_str("a2, b5").unwrap();

        let parsed = ArgAstList::parse_list(input);

        let expected1 = ArgAst::new_fn("a2".to_string(), ArgAstList(vec![]));
        let expected2 = ArgAst::new_fn("b5".to_string(), ArgAstList(vec![]));
        assert_eq!(parsed, Ok(ArgAstList(vec![expected1, expected2])));
    }

    #[test]
    fn nested_args() {
        let input = TokenStream::from_str("g55(h3(X))").unwrap();

        let parsed = ArgAstList::parse_list(input);

        let mut expected = ArgFn::new("g55".to_string(), ArgAstList(vec![]));
        let mut expected1 = ArgFn::new("h3".to_string(), ArgAstList(vec![]));
        expected1
            .args
            .0
            .push(ArgAst::new_fn("X".to_string(), ArgAstList(vec![])));
        expected.args.0.push(ArgAst::ArgFn(expected1));
        let expected = ArgAst::ArgFn(expected);
        assert_eq!(parsed, Ok(ArgAstList(vec![expected])));
    }

    #[test]
    fn nested_args_with_trailing_arg() {
        let input = TokenStream::from_str("g55(h3(X), z)").unwrap();

        let parsed = ArgAstList::parse_list(input);

        let mut expected = ArgFn::new("g55".to_string(), ArgAstList(vec![]));
        let mut expected1 = ArgFn::new("h3".to_string(), ArgAstList(vec![]));
        expected1
            .args
            .0
            .push(ArgAst::new_fn("X".to_string(), ArgAstList(vec![])));
        expected.args.0.push(ArgAst::ArgFn(expected1));
        expected
            .args
            .0
            .push(ArgAst::new_fn("z".to_string(), ArgAstList(vec![])));
        let expected = ArgAst::ArgFn(expected);
        assert_eq!(parsed, Ok(ArgAstList(vec![expected])));
    }

    #[test]
    fn single_arg_with_eq_args_nested() {
        let input = TokenStream::from_str("a2(x=a(b))").unwrap();

        let parsed = ArgAstList::parse_list(input);

        let mut expected = ArgFn::new("a2".to_string(), ArgAstList(vec![]));
        let mut expected1 = ArgFn::new("a".to_owned(), ArgAstList(vec![]));
        expected1
            .args
            .0
            .push(ArgAst::new_fn("b".to_owned(), ArgAstList(vec![])));
        expected
            .args
            .0
            .push(ArgAst::new_eq("x".to_string(), expected1));
        let expected = ArgAst::ArgFn(expected);
        assert_eq!(parsed, Ok(ArgAstList(vec![expected])));
    }

    #[test]
    fn list_of_eq_args() {
        let input = TokenStream::from_str("x = a, y = b").unwrap();

        let parsed = ArgAstList::parse_list(input);

        let expected1 = ArgEq::new(
            "x".to_string(),
            ArgFn::new("a".to_owned(), ArgAstList(vec![])),
        );
        let expected2 = ArgEq::new(
            "y".to_string(),
            ArgFn::new("b".to_owned(), ArgAstList(vec![])),
        );

        let expected = ArgAstList(vec![ArgAst::ArgEq(expected1), ArgAst::ArgEq(expected2)]);
        assert_eq!(parsed, Ok(expected));
    }
}
