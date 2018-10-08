use std::env;
use std::fs::File;
use std::io::{Write, BufWriter, Result};
use std::path::Path;

static BINOP_PAIRS: &[[&str; 6]] = &[
    ["Add", "add", "Sub", "sub", "+", "-"],
    ["Mul", "mul", "Div", "div", "*", "/"],
    ["Shl", "shl", "Shr", "shr", "<<", ">>"],
    ["BitAnd", "bitand", "BitOr", "bitor", "&", "|"],
//    ["BitXor", "bitxor", "BitOr", "bitor", "^"], TODO: allow multi-mutations
//    ["BitAnd", "bitand", "BitXor", "bitxor"],
];

fn write_binop_arm(out: &mut Write,
                   o_trait: &str,
                   o_fn: &str,
                   mut_trait: &str,
                   o_sym: &str,
                   mut_sym: &str,
           shift: bool) -> Result<()> {
    if shift {
        writeln!(out, r#"
            BinOpKind::{0} => {{
                let left = p.fold_expr(original_left);
                let right = p.fold_expr(original_right);

                let (n, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        Mutation::new(MutationType::OPORTUNISTIC_BINARY, "(opportunistically) replacing x {3} y with x {4} y"),
                    ]
                );
                quote_expr!(p.cx(), {{
                    let (left, right) = ($left, $right);
                    if false {{ left {3} right }} else {{
                        ::mutagen::{0}{2}::{1}(left, right, $n, &$sym[$flag], $mask)
                    }}
                }})
            }}"#, o_trait, o_fn, mut_trait, o_sym, mut_sym)
    } else {
        writeln!(out, r#"
            BinOpKind::{0} => {{
                let left = p.fold_expr(original_left);
                let right = p.fold_expr(original_right);

                let (n, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        Mutation::new(MutationType::OPORTUNISTIC_BINARY, "(opportunistically) replacing x {3} y with x {4} y"),
                    ]
                );
                quote_expr!(p.cx(),
                    ::mutagen::{0}{2}::{1}($left, $right, $n, &$sym[$flag], $mask))
            }}"#, o_trait, o_fn, mut_trait, o_sym, mut_sym)
    }
}

fn write_opassign_arm(out: &mut Write,
                      o_trait: &str,
                      o_fn: &str,
                      mut_trait: &str,
                      o_sym: &str,
                      mut_sym: &str) -> Result<()> {
     writeln!(out, r#"
            BinOpKind::{0} => {{
                let (n, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        Mutation::new(MutationType::OPORTUNISTIC_UNARY, "(opportunistically) replacing x {3}= y with x {4}= y"),
                    ]
                );
                quote_expr!(p.cx(), {{
                    ::mutagen::{0}{2}Assign::{1}_assign(&mut $left, $right, $n, &$sym[$flag], $mask)
                }})
            }}"#, o_trait, o_fn, mut_trait, o_sym, mut_sym)
}

fn write_plugin(out_dir: &str) -> Result<()> {
    let dest = Path::new(out_dir).join("plugin_ops.rs");
    let mut f = File::create(&dest)?;
    let mut out = BufWriter::new(&mut f);
    write!(out, r#"use super::MutatorPlugin;
use syntax::ast::{{Attribute, BinOp, BinOpKind, Expr, ExprKind, NodeId}};
use syntax::source_map::Span;
use syntax::ptr::P;
use super::{{MutationType, Mutation}};
use syntax::fold::Folder;
use syntax::ThinVec;

pub fn fold_binop(p: &mut MutatorPlugin, id: NodeId, op: BinOp, original_left: P<Expr>, original_right: P<Expr>, span: Span, attrs: ThinVec<Attribute>) -> P<Expr> {{
    match op.node {{
        BinOpKind::And => {{
            // avoid restrictions that would lead to a false evaluation, we will already replace
            // the and with a false expression
            p.set_restrictions(MutationType::REPLACE_WITH_FALSE);

            let left = p.fold_expr(original_left);
            let right = p.fold_expr(original_right);

            let (n, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        Mutation::new(MutationType::REPLACE_WITH_FALSE, "replacing _ && _ with false"),
                        Mutation::new(MutationType::REPLACE_WITH_TRUE, "replacing _ && _ with true"),
                        Mutation::new(MutationType::REMOVE_RIGHT, "replacing x && _ with x"),
                        Mutation::new(MutationType::NEGATE_LEFT, "replacing x && _ with !x"),
                        Mutation::new(MutationType::NEGATE_RIGHT, "replacing x && y with x && !y"),
                    ],
                );
            quote_expr!(p.cx(), {{
                (match ($left, ::mutagen::diff($n, 5, &$sym[$flag], $mask)) {{
                        (_, 0) => false,
                        (_, 1) => true,
                        (x, 2) => x,
                        (x, 3) => !x,
                        (x, n) => x && ($right) == (n != 4),
                }})
            }})
        }}
        BinOpKind::Or => {{
            p.set_restrictions(MutationType::REPLACE_WITH_TRUE);

            let left = p.fold_expr(original_left);
            let right = p.fold_expr(original_right);

            let (n, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        Mutation::new(MutationType::REPLACE_WITH_FALSE, "replacing _ || _ with false"),
                        Mutation::new(MutationType::REPLACE_WITH_TRUE, "replacing _ || _ with true"),
                        Mutation::new(MutationType::REMOVE_RIGHT, "replacing x || _ with x"),
                        Mutation::new(MutationType::NEGATE_LEFT, "replacing x || _ with !x"),
                        Mutation::new(MutationType::NEGATE_RIGHT, "replacing x || y with x || !y"),
                    ],
                );

            quote_expr!(p.cx(), {{
                (match ($left, ::mutagen::diff($n, 5, &$sym[$flag], $mask)) {{
                    (_, 0) => false,
                    (_, 1) => true,
                    (x, 2) => x,
                    (x, 3) => !x,
                    (x, n) => x || ($right) == (n != 4),
                }})
            }})
        }}
        BinOpKind::Eq => {{
            let left = p.fold_expr(original_left);
            let right = p.fold_expr(original_right);

            let (n, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        Mutation::new(MutationType::REPLACE_WITH_TRUE, "replacing _ == _ with true"),
                        Mutation::new(MutationType::REPLACE_WITH_FALSE, "replacing _ == _ with false"),
                        Mutation::new(MutationType::NEGATE_EXPRESSION, "replacing x == y with x != y"),
                    ],
                );
            quote_expr!(p.cx(), ::mutagen::eq(&$left, &$right, $n, &$sym[$flag], $mask))
        }}
        BinOpKind::Ne => {{
            let left = p.fold_expr(original_left);
            let right = p.fold_expr(original_right);

            let (n, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        Mutation::new(MutationType::REPLACE_WITH_TRUE, "replacing _ != _ with true"),
                        Mutation::new(MutationType::REPLACE_WITH_FALSE, "replacing _ != _ with false"),
                        Mutation::new(MutationType::NEGATE_EXPRESSION, "replacing x != y with x == y"),
                    ],
                );
            quote_expr!(p.cx(), {{
                ::mutagen::ne(&$left, &$right, $n, &$sym[$flag], $mask)
            }})
        }}
        BinOpKind::Gt => {{
            let left = p.fold_expr(original_left);
            let right = p.fold_expr(original_right);

            let (n, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        Mutation::new(MutationType::REPLACE_WITH_FALSE, "replacing _ > _ with false"),
                        Mutation::new(MutationType::REPLACE_WITH_TRUE, "replacing _ > _ with true"),
                        Mutation::new(MutationType::COMPARISON, "replacing x > y with x < y"),
                        Mutation::new(MutationType::NEGATE_EXPRESSION, "replacing x > y with x <= y"),
                        Mutation::new(MutationType::COMPARISON, "replacing x > y with x >= y"),
                        Mutation::new(MutationType::COMPARISON, "replacing x > y with x == y"),
                        Mutation::new(MutationType::COMPARISON, "replacing x > y with x != y"),
                    ],
                );
            quote_expr!(p.cx(), {{
                ::mutagen::gt(&$left, &$right, $n, &$sym[$flag], $mask)
            }})
        }}
        BinOpKind::Lt => {{
            let left = p.fold_expr(original_left);
            let right = p.fold_expr(original_right);

            let (n, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        Mutation::new(MutationType::REPLACE_WITH_FALSE, "replacing _ < _ with false"),
                        Mutation::new(MutationType::REPLACE_WITH_TRUE, "replacing _ < _ with true"),
                        Mutation::new(MutationType::COMPARISON, "replacing x < y with x > y"),
                        Mutation::new(MutationType::NEGATE_EXPRESSION, "replacing x < y with x >= y"),
                        Mutation::new(MutationType::COMPARISON, "replacing x < y with x <= y"),
                        Mutation::new(MutationType::COMPARISON, "replacing x < y with x == y"),
                        Mutation::new(MutationType::COMPARISON, "replacing x < y with x != y"),
                    ],
                );
            quote_expr!(p.cx(), {{
                ::mutagen::gt(&$right, &$left, $n, &$sym[$flag], $mask)
            }})
        }}
        BinOpKind::Ge => {{
            let left = p.fold_expr(original_left);
            let right = p.fold_expr(original_right);

            let (n, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        Mutation::new(MutationType::REPLACE_WITH_FALSE, "replacing _ >= _ with false"),
                        Mutation::new(MutationType::REPLACE_WITH_TRUE, "replacing _ >= _ with true"),
                        Mutation::new(MutationType::NEGATE_EXPRESSION, "replacing x >= y with x < y"),
                        Mutation::new(MutationType::COMPARISON, "replacing x >= y with x <= y"),
                        Mutation::new(MutationType::COMPARISON, "replacing x >= y with x > y"),
                        Mutation::new(MutationType::COMPARISON, "replacing x >= y with x == y"),
                        Mutation::new(MutationType::COMPARISON, "replacing x >= y with x != y"),
                    ],
                );
            quote_expr!(p.cx(), {{
                ::mutagen::ge(&$left, &$right, $n, &$sym[$flag], $mask)
            }})
        }}
        BinOpKind::Le => {{
            let left = p.fold_expr(original_left);
            let right = p.fold_expr(original_right);

            let (n, sym, flag, mask) = p.add_mutations(
                span,
                &[
                    Mutation::new(MutationType::REPLACE_WITH_FALSE, "replacing _ <= _ with false"),
                    Mutation::new(MutationType::REPLACE_WITH_TRUE, "replacing _ <= _ with true"),
                    Mutation::new(MutationType::NEGATE_EXPRESSION, "replacing x <= y with x > y"),
                    Mutation::new(MutationType::COMPARISON, "replacing x <= y with x >= y"),
                    Mutation::new(MutationType::COMPARISON, "replacing x <= y with x < y"),
                    Mutation::new(MutationType::COMPARISON, "replacing x <= y with x == y"),
                    Mutation::new(MutationType::COMPARISON, "replacing x <= y with x != y"),
                ],
            );
            quote_expr!(p.cx(), {{
                ::mutagen::ge(&$right, &$left, $n, &$sym[$flag], $mask)
            }})
        }}"#)?;
    for names in BINOP_PAIRS.iter() {
        write_binop_arm(&mut out, names[0], names[1], names[2], names[4], names[5], names[0].starts_with("Sh"))?;
        write_binop_arm(&mut out, names[2], names[3], names[0], names[5], names[4], names[0].starts_with("Sh"))?;
    }
    write!(out, "_ => {{
            let left = p.fold_expr(original_left);
            let right = p.fold_expr(original_right);

            let e = P(Expr {{
                id,
                node: ExprKind::Binary(op, left, right),
                span,
                attrs,
            }});

            e
        }}
    }}
}}
")?;
    write!(out, "pub fn fold_assignop(p: &mut MutatorPlugin,
        id: NodeId,
        op: BinOp,
        left: P<Expr>,
        right: P<Expr>,
        span: Span,
        attrs: ThinVec<Attribute>) -> P<Expr> {{
    match op.node {{")?;
    for names in BINOP_PAIRS.iter() {
        //                 BufWriter Add       add       Sub       +         -
        write_opassign_arm(&mut out, names[0], names[1], names[2], names[4], names[5])?;
        write_opassign_arm(&mut out, names[2], names[3], names[0], names[5], names[4])?;
    }
    write!(out, "_ => P(Expr {{
                id,
                node: ExprKind::AssignOp(op, left, right),
                span,
                attrs,
            }})
    }}
}}
")?;

    out.flush()
}

fn main() {
    write_plugin(&env::var("OUT_DIR").unwrap()).unwrap();
}
