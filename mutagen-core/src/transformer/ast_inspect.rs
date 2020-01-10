//! a collection of functions for extracting information from ast-types.

/// check if an expression has numeric type.
///
/// This is implemented via a heuristic. An expression has an numeric type if:
/// * it is a numeric literal
/// * it is an binary arithmetic- or bit-operation that has an integer expression on the left side
/// * it is an unary operation with an numeric expression
/// * it is a reference to a numeric expression. This lets us count `*&1` as numeric expression.
pub fn is_num_expr(e: &syn::Expr) -> bool {
    match e {
        syn::Expr::Lit(expr) => match expr.lit {
            syn::Lit::Int(_) => return true,
            syn::Lit::Byte(_) => return true,
            syn::Lit::Float(_) => return true,
            _ => {}
        },
        syn::Expr::Binary(expr) => match expr.op {
            syn::BinOp::Add(_) => return is_num_expr(&expr.left),
            syn::BinOp::Sub(_) => return is_num_expr(&expr.left),
            syn::BinOp::Mul(_) => return is_num_expr(&expr.left),
            syn::BinOp::Div(_) => return is_num_expr(&expr.left),
            syn::BinOp::Rem(_) => return is_num_expr(&expr.left),
            syn::BinOp::BitAnd(_) => return is_num_expr(&expr.left),
            syn::BinOp::BitOr(_) => return is_num_expr(&expr.left),
            syn::BinOp::BitXor(_) => return is_num_expr(&expr.left),
            syn::BinOp::Shl(_) => return is_num_expr(&expr.left),
            syn::BinOp::Shr(_) => return is_num_expr(&expr.left),
            _ => {}
        },
        syn::Expr::Unary(expr) => return is_num_expr(&expr.expr),
        syn::Expr::Reference(expr) => return is_num_expr(&expr.expr),
        syn::Expr::Paren(expr) => return is_num_expr(&expr.expr),
        _ => {}
    };
    return false;
}

#[cfg(test)]
mod tests {

    use super::*;
    use syn::parse_quote;

    #[test]
    fn num_expr_lit_int() {
        let tt = parse_quote! {1};

        assert!(is_num_expr(&tt));
    }

    #[test]
    fn num_expr_add_of_lit_int() {
        let tt = parse_quote! {1 + 2};

        assert!(is_num_expr(&tt));
    }

    #[test]
    fn num_expr_neg_one() {
        let tt = parse_quote! {-1};

        assert!(is_num_expr(&tt));
    }

    #[test]
    fn num_expr_bit_not_one() {
        let tt = parse_quote! {!1};

        assert!(is_num_expr(&tt));
    }

    #[test]
    fn num_expr_variable() {
        let tt = parse_quote! {x};

        assert!(!is_num_expr(&tt));
    }

    #[test]
    fn num_expr_multiple_plus_lit_int() {
        let tt = parse_quote! {1+2+3};

        assert!(is_num_expr(&tt));
    }

    #[test]
    fn num_expr_multiple_plus_left_is_var() {
        let tt = parse_quote! {x+2+3};

        assert!(!is_num_expr(&tt));
    }

    #[test]
    fn num_expr_deref_ref_lit_int() {
        let tt = parse_quote! {*&1};

        assert!(is_num_expr(&tt));
    }

    #[test]
    fn num_expr_lit_float() {
        let tt = parse_quote! {1.5};

        assert!(is_num_expr(&tt));
    }

    #[test]
    fn num_expr_lit_byte() {
        let tt = parse_quote! {b'a'};

        assert!(is_num_expr(&tt));
    }

    #[test]
    fn num_expr_lit_str() {
        let tt = parse_quote! {"a"};

        assert!(!is_num_expr(&tt));
    }

    #[test]
    fn num_expr_lit_bool() {
        let tt = parse_quote! {true};

        assert!(!is_num_expr(&tt));
    }

    #[test]
    fn num_expr_bool_and() {
        let tt = parse_quote! {true && false};

        assert!(!is_num_expr(&tt));
    }

    #[test]
    fn num_expr_bitand_vars() {
        let tt = parse_quote! {x & y};

        assert!(!is_num_expr(&tt));
    }

    #[test]
    fn num_expr_bitand_lit_int() {
        let tt = parse_quote! {1 & 2};

        assert!(is_num_expr(&tt));
    }

    #[test]
    fn num_expr_shl_lit_int() {
        let tt = parse_quote! {1 << 3};

        assert!(is_num_expr(&tt));
    }

    #[test]
    fn num_expr_shr_lit_int() {
        let tt = parse_quote! {1 >> 3};

        assert!(is_num_expr(&tt));
    }

    #[test]
    fn num_expr_not_shift() {
        let tt = parse_quote! {!(1 >> 3)};

        assert!(is_num_expr(&tt), format!("{:#?}", tt));
    }
}
