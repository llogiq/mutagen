use std::fs::File;
use std::io::{Write, BufWriter, Result};

fn write_binop(out: &mut Write, o_trait: &str, o_fn: &str, mut_trait: &str, mut_fn: &str) ->
    Result<()> {
    writeln!(out, "
pub trait {0}{2}<Rhs = Self> {{
    type Output;
    fn {1}(self, rhs: Rhs, mutation_count: usize) -> Self::Output;
}}

impl<T, Rhs> {0}{2}<Rhs> for T
where T: {0}<Rhs> {{
    type Output = <T as {0}<Rhs>>::Output;
    default fn {1}(self, rhs: Rhs, _mutation_count: usize) -> Self::Output {{
        {0}::{1}(self, rhs)
    }}
}}

impl<T, Rhs> {0}{2}<Rhs> for T
where T: {0}<Rhs>,
      T: {2}<Rhs>,
     <T as {2}<Rhs>>::Output: Into<<T as {0}<Rhs>>::Output> {{
    fn {1}(self, rhs: Rhs, mutation_count: usize) -> Self::Output {{
        if super::now(mutation_count) {{
            {2}::{3}(self, rhs).into()
        }} else {{
            {0}::{1}(self, rhs)
        }}
    }}
}}

pub trait {0}{2}Assign<Rhs=Self> {{
    fn {1}_assign(&mut self, rhs: Rhs, mutation_count: usize);
}}

impl<T, R> {0}{2}Assign<R> for T where T: {0}Assign<R> {{
    default fn {1}_assign(&mut self, rhs: R, _mutation_count: usize) {{
        {0}Assign::{1}_assign(self, rhs);
    }}
}}

impl<T, R> {0}{2}Assign<R> for T
where T: {0}Assign<R>,
      T: {2}Assign<R> {{
    fn {1}_assign(&mut self, rhs: R, mutation_count: usize) {{
        if super::now(mutation_count) {{
            {2}Assign::{3}_assign(self, rhs);
        }} else {{
            {0}Assign::{1}_assign(self, rhs);
        }}
    }}
}}
", o_trait, o_fn, mut_trait, mut_fn)
}

static BINOP_PAIRS: &[[&str; 4]] = &[
    ["Add", "add", "Sub", "sub"],
    ["Mul", "mul", "Div", "div"],
    ["Shl", "shl", "Shr", "shr"],
    ["BitAnd", "bitand", "BitOr", "bitor"],
    ["BitXor", "bitxor", "BitOr", "bitor"],
    ["BitAnd", "bitand", "BitXor", "bitxor"],
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

fn main() {
    write_ops().unwrap();
}
