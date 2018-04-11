use std::fs::File;
use std::io::{Write, BufWriter, Result};

fn write_binop(out: &mut Write, o_trait: &str, o_fn: &str, mut_trait: &str, mut_fn: &str) ->
    Result<()> {
    writeln!(out, "
pub trait {0}{2}<Rhs = Self> {{
    type Output;
    fn {1}(self, rhs: Rhs, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self::Output;
}}

impl<T, Rhs> {0}{2}<Rhs> for T
where T: {0}<Rhs> {{
    type Output = <T as {0}<Rhs>>::Output;
    default fn {1}(self, rhs: Rhs, _mutation_count: usize, _cov: &AtomicUsize, _mask: usize) -> Self::Output {{
        {0}::{1}(self, rhs)
    }}
}}

impl<T, Rhs> {0}{2}<Rhs> for T
where T: {0}<Rhs>,
      T: {2}<Rhs>,
     <T as {2}<Rhs>>::Output: Into<<T as {0}<Rhs>>::Output> {{
    fn {1}(self, rhs: Rhs, mutation_count: usize, coverage: &AtomicUsize, mask: usize) -> Self::Output {{
    super::report_coverage(mutation_count..(mutation_count + 1), coverage, mask);
        if super::now(mutation_count) {{
            {2}::{3}(self, rhs).into()
        }} else {{
            {0}::{1}(self, rhs)
        }}
    }}
}}

pub trait {0}{2}Assign<Rhs=Self> {{
    fn {1}_assign(&mut self, rhs: Rhs, mutation_count: usize, coverage: &AtomicUsize, mask: usize);
}}

impl<T, R> {0}{2}Assign<R> for T where T: {0}Assign<R> {{
    default fn {1}_assign(&mut self, rhs: R, _mutation_count: usize, _coverage: &AtomicUsize, _mask: usize) {{
        {0}Assign::{1}_assign(self, rhs);
    }}
}}

impl<T, R> {0}{2}Assign<R> for T
where T: {0}Assign<R>,
      T: {2}Assign<R> {{
    fn {1}_assign(&mut self, rhs: R, mutation_count: usize, coverage: &AtomicUsize, mask: usize) {{
    super::report_coverage(mutation_count..(mutation_count + 1), coverage, mask);
        if super::now(mutation_count) {{
            {2}Assign::{3}_assign(self, rhs);
        }} else {{
            {0}Assign::{1}_assign(self, rhs);
        }}
    }}
}}
", o_trait, o_fn, mut_trait, mut_fn)
}

fn write_binop_arm(out: &mut Write,
                   o_trait: &str,
                   o_fn: &str,
                   mut_trait: &str,
                   o_sym: &str,
                   mut_sym: &str,
           shift: bool) -> Result<()> {
    if shift {
        writeln!(out, "
            BinOpKind::{0} => {{
                let (n, current, sym, flag, mask) = p.add_mutations(
                    span,
                    &[\"(opportunistically) replacing x {3} y with x {4} y\"]
                );
                quote_expr!(p.cx(), {{
                    let (left, right) = ($left, $right);
                    if false {{ left {3} right }} else {{
                        ::mutagen::{0}{2}::{1}(left, right, $n, &$sym[$flag], $mask)
                    }}
                }})
            }}", o_trait, o_fn, mut_trait, o_sym, mut_sym)
    } else {
        writeln!(out, "
            BinOpKind::{0} => {{
                let (n, current, sym, flag, mask) = p.add_mutations(
                    span,
                    &[\"(opportunistically) replacing x {3} y with x {4} y\"]
                );
                quote_expr!(p.cx(),
                    ::mutagen::{0}{2}::{1}($left, $right, $n, &$sym[$flag], $mask))
            }}", o_trait, o_fn, mut_trait, o_sym, mut_sym)
    }
}

fn write_opassign_arm(out: &mut Write,
                      o_trait: &str,
                      o_fn: &str,
                      mut_trait: &str,
                      o_sym: &str,
                      mut_sym: &str) -> Result<()> {
     writeln!(out, "
            BinOpKind::{0} => {{
                let (n, current, sym, flag, mask) = p.add_mutations(
                    span,
                    &[\"(opportunistically) replacing x {3}= y with x {4}= y\"]
                );
                quote_expr!(p.cx(), {{
                    ::mutagen::{0}{2}Assign::{1}_assign(&mut $left, $right, $n, &$sym[$flag], $mask)
                }})
            }}", o_trait, o_fn, mut_trait, o_sym, mut_sym)
}

static BINOP_PAIRS: &[[&str; 6]] = &[
    ["Add", "add", "Sub", "sub", "+", "-"],
    ["Mul", "mul", "Div", "div", "*", "/"],
//    ["Shl", "shl", "Shr", "shr", "<<", ">>"],
    ["BitAnd", "bitand", "BitOr", "bitor", "&", "|"],
//    ["BitXor", "bitxor", "BitOr", "bitor", "^"], TODO: allow multi-mutations
//    ["BitAnd", "bitand", "BitXor", "bitxor"],
];

fn write_unop(out: &mut Write, op_trait: &str, op_fn: &str) -> Result<()> {
    writeln!(out, "
pub trait May{0} {{
    type Output;
    fn {1}(self, mutation_count: usize) -> Self::Output;
}}

impl<T> May{0} for T where T: {0} {{
    type Output = <T as {0}>::Output;
    default fn {1}(self, _mutation_count: usize) -> Self::Output {{
        {0}::{1}(self)
    }}
}}

impl<T> May{0} for T where T: {0}, T: Into<<T as {0}>::Output> {{
    fn {1}(self, mutation_count: usize) -> Self::Output {{
        if super::now(mutation_count) {{ self.into() }} else {{ {0}::{1}(self) }}
    }}
}}
", op_trait, op_fn)
}

fn write_ops() -> Result<()> {
    let mut f = File::create("src/ops.rs")?;
    let mut out = BufWriter::new(&mut f);
    writeln!(out, "use std::ops::*;
use std::sync::atomic::AtomicUsize;
")?;
    for names in BINOP_PAIRS.iter() {
        write_binop(&mut out, names[0], names[1], names[2], names[3])?;
        write_binop(&mut out, names[2], names[3], names[0], names[1])?;
    }
    for &(ref op_trait, ref op_fn) in [("Not", "not"), ("Neg", "neg")].iter() {
        write_unop(&mut out, op_trait, op_fn)?;
    }
    writeln!(out, "
pub trait MayClone<T> {{
    fn may_clone(&self) -> bool;
    fn clone(&self) -> Self;
}}

impl<T> MayClone<T> for T {{
    default fn may_clone(&self) -> bool {{ false }}
    default fn clone(&self) -> Self {{ unimplemented!() }}
}}

impl<T: Clone> MayClone<T> for T {{
    fn may_clone(&self) -> bool {{ true }}
    fn clone(&self) -> Self {{ self.clone() }}
}}")?;
    out.flush()
}

fn write_plugin() -> Result<()> {
    let mut f = File::create("plugin/src/binop.rs")?;
    let mut out = BufWriter::new(&mut f);
    write!(out, "use super::MutatorPlugin;
use syntax::ast::{{Attribute, BinOp, BinOpKind, Expr, ExprKind, NodeId, ThinVec}};
use syntax::codemap::Span;
use syntax::ptr::P;

pub fn fold_binop(p: &mut MutatorPlugin, id: NodeId, op: BinOp, left: P<Expr>, right: P<Expr>, span: Span, attrs: ThinVec<Attribute>) -> P<Expr> {{
    match op.node {{
        BinOpKind::And => {{
            let (n, current, sym, flag, op) = p.add_mutations(span,
                    &[
                        \"replacing _ && _ with false\",
                        \"replacing _ && _ with true\",
                        \"replacing x && _ with x\",
                        \"replacing x && _ with !x\",
                        \"replacing x && y with x && !y\",
                    ],
                );
            quote_expr!(p.cx(), {{
                ::mutagen::report_coverage($n..$current, &$sym[$flag], $op);
                (match ($left, ::mutagen::diff($n)) {{
                        (_, 0) => false,
                        (_, 1) => true,
                        (x, 2) => x,
                        (x, 3) => !x,
                        (x, n) => x && ($right) == (n != 4),
                }})
            }})
        }}
        BinOpKind::Or => {{
            let (n, current, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        \"replacing _ || _ with false\",
                        \"replacing _ || _ with true\",
                        \"replacing x || _ with x\",
                        \"replacing x || _ with !x\",
                        \"replacing x || y with x || !y\",
                    ],
                );
            quote_expr!(p.cx(), {{
                ::mutagen::report_coverage($n..$current, &$sym[$flag], $mask);
                (match ($left, ::mutagen::diff($n)) {{
                    (_, 0) => false,
                    (_, 1) => true,
                    (x, 2) => x,
                    (x, 3) => !x,
                    (x, n) => x || ($right) == (n != 4),
                }})
            }})
        }}
        BinOpKind::Eq => {{
            let (n, current, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        \"replacing _ == _ with true\",
                        \"replacing _ == _ with false\",
                        \"replacing x == y with x != y\",
                    ],
                );
            quote_expr!(p.cx(), {{
                ::mutagen::report_coverage($n..$current, &$sym[$flag], $mask);
                ::mutagen::eq($left, $right, $n)
            }})
        }}
        BinOpKind::Ne => {{
            let (n, current, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        \"replacing _ != _ with true\",
                        \"replacing _ != _ with false\",
                        \"replacing x != y with x == y\",
                    ],
                );
            quote_expr!(p.cx(), {{
                ::mutagen::report_coverage($n..$current, &$sym[$flag], $mask);
                ::mutagen::ne($left, $right, $n)
            }})
        }}
        BinOpKind::Gt => {{
            let (n, current, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        \"replacing _ > _ with false\",
                        \"replacing _ > _ with true\",
                        \"replacing x > y with x < y\",
                        \"replacing x > y with x <= y\",
                        \"replacing x > y with x >= y\",
                        \"replacing x > y with x == y\",
                        \"replacing x > y with x != y\",
                    ],
                );
            quote_expr!(p.cx(), {{
                ::mutagen::report_coverage($n..$current, &$sym[$flag], $mask);
                ::mutagen::gt($left, $right, $n)
            }})
        }}
        BinOpKind::Lt => {{
            let (n, current, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        \"replacing _ < _ with false\",
                        \"replacing _ < _ with true\",
                        \"replacing x < y with x > y\",
                        \"replacing x < y with x >= y\",
                        \"replacing x < y with x <= y\",
                        \"replacing x < y with x == y\",
                        \"replacing x < y with x != y\",
                    ],
                );
            quote_expr!(p.cx(), {{
                ::mutagen::report_coverage($n..$current, &$sym[$flag], $mask);
                ::mutagen::gt($right, $left, $n)
            }})
        }}
        BinOpKind::Ge => {{
            let (n, current, sym, flag, mask) = p.add_mutations(
                    span,
                    &[
                        \"replacing _ >= _ with false\",
                        \"replacing _ >= _ with true\",
                        \"replacing x >= y with x < y\",
                        \"replacing x >= y with x <= y\",
                        \"replacing x >= y with x > y\",
                        \"replacing x >= y with x == y\",
                        \"replacing x >= y with x != y\",
                    ],
                );
            quote_expr!(p.cx(), {{
                ::mutagen::report_coverage($n..$current, &$sym[$flag], $mask);
                ::mutagen::ge($left, $right, $n)
            }})
        }}
        BinOpKind::Le => {{
            let (n, current, sym, flag, mask) = p.add_mutations(
                span,
                &[
                    \"replacing _ <= _ with false\",
                    \"replacing _ <= _ with true\",
                    \"replacing x <= y with x > y\",
                    \"replacing x <= y with x >= y\",
                    \"replacing x <= y with x < y\",
                    \"replacing x <= y with x == y\",
                    \"replacing x <= y with x != y\",
                ],
            );
            quote_expr!(p.cx(), {{
                ::mutagen::report_coverage($n..$current, &$sym[$flag], $mask);
                ::mutagen::ge($right, $left, $n)
            }})
        }}")?;
        for names in BINOP_PAIRS.iter() {
            write_binop_arm(&mut out, names[0], names[1], names[2], names[4], names[5], names[0].starts_with("Sh"))?;
            write_binop_arm(&mut out, names[2], names[3], names[0], names[5], names[4], names[0].starts_with("Sh"))?;
        }
        write!(out, "_ => P(Expr {{
                id,
                node: ExprKind::Binary(op, left, right),
                span,
                attrs,
            }})
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
    write_ops().unwrap();
    write_plugin().unwrap();
}
