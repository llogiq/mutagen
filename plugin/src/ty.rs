use std::hash::{Hash, Hasher};
use syn::*;
use crate::pattern::SelfOr;

fn vecd<T, F: Fn(&T, &T) -> bool>(a: impl IntoIterator<Item=T>, b: impl IntoIterator<Item=T>, f: F) -> bool {
    let (mut ai, mut bi) = (a.into_iter(), b.into_iter());
    loop {
        match (ai.next(), bi.next()) {
            (Some(ae), Some(be)) => if !f(&ae, &be) { return false; },
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn optd<T, F: Fn(&T, &T) -> bool>(a: &Option<T>, b: &Option<T>, f: F) -> bool {
    a.as_ref().map_or_else(|| b.is_none(), |aref| b.as_ref().map_or(false,
        |bref| f(aref, bref)))
}

pub fn is_ty_ref_mut(t: &SelfOr<Type>) -> bool {
    if let SelfOr::Other(&Type::Reference(TypeReference { mutability: Some(_), .. })) = *t {
        true
    } else {
        false
    }
}

pub fn self_or_ty_equal(a: &SelfOr<Type>, b: &SelfOr<Type>, isout: bool) -> bool {
    match (a, b) {
        (&SelfOr::JustSelf, &SelfOr::JustSelf) => true,
        (&SelfOr::Other(ref a), &SelfOr::Other(ref b)) => ty_equal(a, b, isout),
        _ => false
    }
}

pub fn ty_equal(a: &Type, b: &Type, inout: bool) -> bool {
    match (a, b) {
        (&Type::Paren(TypeParen { ref elem, .. }), _) => {
            ty_equal(&*elem, b, inout)
        }
        (_, &Type::Paren(TypeParen { ref elem, .. })) => {
            ty_equal(a, &*elem, inout)
        }
        (&Type::Group(TypeGroup { ref elem, .. }), _) => {
            ty_equal(&*elem, b, inout)
        }
        (_, &Type::Group(TypeGroup { ref elem, .. })) => {
            ty_equal(a, &*elem, inout)
        }
        (&Type::Slice(TypeSlice { elem: ref ae, .. }),
         &Type::Slice(TypeSlice { elem: ref be, .. })) => {
            ty_equal(&*ae, &*be, inout)
        }
        (&Type::Array(TypeArray { elem: ref ae, len: ref alen, .. }),
         &Type::Array(TypeArray { elem: ref be, len: ref blen, .. })) => {
            ty_equal(&*ae, &*be, inout) && get_const(alen).map_or(false, |a|
                Some(a) == get_const(blen))
        }
        (&Type::Ptr(TypePtr { const_token: ref ac, elem: ref ae, .. }),
         &Type::Ptr(TypePtr { const_token: ref bc, elem: ref be, .. })) => {
            ac.is_some() == bc.is_some() && ty_equal(&*ae, &*be, inout)
        }
        (&Type::Reference(TypeReference { lifetime: ref alt, mutability: ref amut, elem: ref ae, .. }),
         &Type::Reference(TypeReference { lifetime: ref blt, mutability: ref bmut, elem: ref be, .. })) => {
            amut.is_some() == bmut.is_some() &&
            (if let (&Some(ref al), &Some(ref bl)) = (alt, blt) {
                al.ident == bl.ident
            } else { // lifetime elision rule #2: input == output
                inout && alt.is_none() && blt.is_none()
            }) && ty_equal(&*ae, &be, inout)
        }
        (&Type::BareFn(TypeBareFn {
            lifetimes: ref alt,
            unsafety: ref aunsafe,
            inputs: ref ains,
            variadic: ref avar,
            output: ref aret,
            ..
         }),
         &Type::BareFn(TypeBareFn {
            lifetimes: ref blt,
            unsafety: ref bunsafe,
            inputs: ref bins,
            variadic: ref bvar,
            output: ref bret,
            ..
         })) => {
            (if let (&Some(ref al), &Some(ref bl)) = (alt, blt) {
                vecd(&al.lifetimes, &bl.lifetimes,
                    |a, b| lifetime_def_equal(a, b))
            } else {
                inout && alt.is_none() && blt.is_none()
            }) && aunsafe.is_some() == bunsafe.is_some() &&
                vecd(ains, bins, |a, b| ty_equal(&a.ty, &b.ty, inout)) &&
                avar.is_some() == bvar.is_some() &&
                ret_ty_equal(aret, bret, inout)
        }
        (&Type::Never(_), &Type::Never(_)) => true,
        (&Type::Tuple(TypeTuple { elems: ref ae, .. }),
         &Type::Tuple(TypeTuple { elems: ref be, .. })) => {
             ae.iter().zip(be.iter()).all(|(a, b)| ty_equal(a, b, inout))
        }
        (&Type::Path(TypePath { qself: ref aself, path: ref apath }),
         &Type::Path(TypePath { qself: ref bself, path: ref bpath })) => {
            optd(aself, bself, |a, b| a.position == b.position &&
                ty_equal(&*a.ty, &*b.ty, inout) && a.as_token.is_some() ==
                b.as_token.is_some()) && path_equal(apath, bpath, inout)
        }
        (&Type::TraitObject(TypeTraitObject { bounds: ref ab, .. }),
         &Type::TraitObject(TypeTraitObject { bounds: ref bb, .. })) => {
             vecd(ab, bb, |a, b| ty_param_bound_equal(a, b, inout))
        }
        (&Type::ImplTrait(TypeImplTrait { bounds: ref ab, .. }),
         &Type::ImplTrait(TypeImplTrait { bounds: ref bb, .. })) => {
             vecd(ab, bb, |a, b| ty_param_bound_equal(a, b, inout))
        }
        (&Type::Verbatim(TypeVerbatim { .. }),
         &Type::Verbatim(TypeVerbatim { .. })) => {
             a == b
        }
        _ => false
    }
}

pub fn ty_hash<H: Hasher>(ty: &Type, pos: usize, h: &mut H) {
    match *ty {
        Type::Paren(TypeParen { ref elem, .. }) |
        Type::Group(TypeGroup { ref elem, .. }) => ty_hash(elem, pos, h),
        Type::Slice(TypeSlice { ref elem, .. }) => {
            h.write_u8(0);
            ty_hash(elem, pos, h)
        }
        Type::Array(TypeArray { ref elem, len: ref alen, .. }) => {
            h.write_u8(1);
            ty_hash(elem, pos, h);
            if let Some(u) = get_const(alen) {
                h.write_u64(u)
            }
        }
        Type::Ptr(TypePtr { const_token: ref c, ref elem, .. }) => {
            h.write_u8(if c.is_some() { 3 } else { 2 });
            ty_hash(elem, pos, h);
        }
        Type::Reference(TypeReference { ref lifetime, ref mutability, ref elem, .. }) => {
            h.write_u8(if mutability.is_some() { 5 } else { 4 });
            if let Some(ref l) = lifetime {
                l.hash(h);
            }
            ty_hash(elem, pos, h);
        }
        Type::BareFn(TypeBareFn {
            ref lifetimes,
            ref unsafety,
            ref inputs,
            ref variadic,
            ref output,
            ..
         }) => {
            if let Some(lt) = lifetimes {
                for l in &lt.lifetimes {
                    lifetime_def_hash(l, h)
                }
            }
            h.write_u8(if unsafety.is_some() { 0 } else { 1 });
            for input in inputs {
                ty_hash(&input.ty, pos, h);
            }
            h.write_u8(if variadic.is_some() { 0 } else { 1 });
            ret_ty_hash(output, pos, h);
        }
        Type::Never(_) => h.write_u8(6),
        Type::Tuple(TypeTuple { ref elems, .. }) => {
            h.write_u8(7);
            for elem in elems {
                ty_hash(elem, pos, h);
            }
        }
        Type::Path(TypePath { ref qself, ref path }) => {
            h.write_u8(8);
            //TODO: hash qself
            path_hash(path, pos, h);
        }
        Type::TraitObject(TypeTraitObject { ref bounds, .. }) => {
            h.write_u8(9);
            for bound in bounds {
                ty_param_bound_hash(bound, pos, h);
            }
        }
        Type::ImplTrait(TypeImplTrait { ref bounds, .. }) => {
            h.write_u8(10);
            for bound in bounds {
                ty_param_bound_hash(bound, pos, h);
            }
        }
        Type::Verbatim(TypeVerbatim { .. }) => h.write_u8(11),
        _ => h.write_u8(42)
    }
}

// for now we restrict ourselves to primitive types, just to be sure
static LIFETIME_LESS_PATHS: &[&str] = &[
    "u8", "u16", "u32", "u64", "u128", "usize",
    "i8", "i16", "i32", "i64", "i128", "isize",
    "char", "bool", "Self"]; // Self

fn is_allowlisted_path(path: &Path) -> bool {
    path.segments.iter().last().map_or(false,
        |seg| LIFETIME_LESS_PATHS.iter().any(|m| seg.ident == m))
}

fn path_equal(a: &Path, b: &Path, inout: bool) -> bool {
    let inout_allow = inout || is_allowlisted_path(a);
    vecd(&a.segments, &b.segments,
        |aseg, bseg| path_segment_equal(aseg, bseg, inout_allow))
}

fn path_hash<H: Hasher>(path: &Path, pos: usize, h: &mut H) {
    for seg in &path.segments {
        path_segment_hash(seg, pos, h)
    }
}

fn ret_ty_equal(a: &ReturnType, b: &ReturnType, inout: bool) -> bool {
    match (a, b) {
        (ReturnType::Default, ReturnType::Default) => true,
        (ReturnType::Type(_, ref aty), ReturnType::Type(_, ref bty)) =>
            ty_equal(aty, bty, inout),
        _ => false
    }
}

fn ret_ty_hash<H: Hasher>(r: &ReturnType, pos: usize, h: &mut H) {
    if let ReturnType::Type(_, ref ty) = r {
        ty_hash(ty, pos, h);
    }
}

fn path_segment_equal(a: &PathSegment, b: &PathSegment, inout: bool) -> bool {
    a.ident == b.ident && (match (&a.arguments, &b.arguments) {
        (&PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: ref ca,
            args: ref aargs,
            ..
         }),
         &PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: ref cb,
            args: ref bargs,
            ..
         })) => {
            ca.is_some() == cb.is_some() && vecd(aargs, bargs, |a, b|
                generic_arg_equal(a, b, inout))
        }
        (&PathArguments::Parenthesized(ParenthesizedGenericArguments {
            inputs: ref ia,
            output: ref ra,
            ..
         }),
         &PathArguments::Parenthesized(ParenthesizedGenericArguments {
            inputs: ref ib,
            output: ref rb,
            ..
         })) => {
            vecd(ia, ib, |a, b| ty_equal(a, b, inout)) &&
                ret_ty_equal(ra, rb, inout)
        }
        _ => false
    })
}

fn path_segment_hash<H: Hasher>(seg: &PathSegment, pos: usize, h: &mut H) {
    seg.ident.hash(h);
    match seg.arguments {
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            ref colon2_token, ref args, ..
        }) => {
            h.write_u8(if colon2_token.is_some() { 1 } else { 0 });
            for arg in args {
                generic_arg_hash(arg, pos, h);
            }
        }
        PathArguments::Parenthesized(ParenthesizedGenericArguments {
            ref inputs, ref output, ..
        }) => {
            h.write_u8(2);
            for input in inputs {
                ty_hash(input, pos, h);
            }
            ret_ty_hash(output, pos, h);
        }
        _ => h.write_u8(3)
    }
}

fn generic_arg_equal(a: &GenericArgument, b: &GenericArgument, inout: bool)
-> bool {
    use syn::GenericArgument::*;

    match (a, b) {
        (&Lifetime(ref al), &Lifetime(ref bl)) => al.ident == bl.ident,
        (&Type(ref at), &Type(ref bt)) => ty_equal(at, bt, inout),
        (&Binding(ref ab), &Binding(ref bb)) => {
            ab.ident == bb.ident && ty_equal(&ab.ty, &bb.ty, inout)
        }
        (&Constraint(ref ac), &Constraint(ref bc)) => {
            constraint_equal(ac, bc, inout)
        }
        (&Const(ref ae), &Const(ref be)) => expr_equal(ae, be, inout),
        _ => false
    }
}

fn generic_arg_hash<H: Hasher>(g: &GenericArgument, pos: usize, h: &mut H) {
    use syn::GenericArgument::*;

    match *g {
        Lifetime(ref lifetime) => {
            h.write_u8(0);
            lifetime.hash(h)
        }
        Type(ref ty) => {
            h.write_u8(1);
            ty_hash(ty, pos, h)
        }
        Binding(ref b) => {
            h.write_u8(2);
            b.ident.hash(h);
            ty_hash(&b.ty, pos, h)
        }
        Constraint(ref c) => {
            h.write_u8(3);
            constraint_hash(c, pos, h)
        }
        Const(ref c) => {
            h.write_u8(4);
            c.hash(h)
        }
        _ => h.write_u8(5)
    }
}

fn constraint_equal(a: &Constraint, b: &Constraint, inout: bool) -> bool {
    a.ident == b.ident && vecd(&a.bounds, &b.bounds,
        |a, b| ty_param_bound_equal(a, b, inout))
}

fn constraint_hash<H: Hasher>(c: &Constraint, pos: usize, h: &mut H) {
    c.ident.hash(h);
    for bound in &c.bounds {
        ty_param_bound_hash(bound, pos, h);
    }
}

fn ty_param_bound_equal(a: &TypeParamBound, b: &TypeParamBound, inout: bool)
-> bool {
    match (a, b) {
        (&TypeParamBound::Trait(ref at), &TypeParamBound::Trait(ref bt)) =>
            trait_bound_equal(at, bt, inout),
        (&TypeParamBound::Lifetime(ref al),
         &TypeParamBound::Lifetime(ref bl)) => lifetime_equal(al, bl),
        _ => false
    }
}

fn ty_param_bound_hash<H: Hasher>(bound: &TypeParamBound, pos: usize, h: &mut H) {
    match *bound {
        TypeParamBound::Trait(ref t) => {
            h.write_u8(0);
            trait_bound_hash(t, pos, h)
        }
        TypeParamBound::Lifetime(ref l) => {
            h.write_u8(1);
            l.hash(h)
        }
        _ => h.write_u8(2)
    }
}

fn trait_bound_equal(a: &TraitBound, b: &TraitBound, inout: bool) -> bool {
    a.modifier == b.modifier && optd(&a.lifetimes, &b.lifetimes,
        |a, b| vecd(&a.lifetimes, &b.lifetimes,
            |a, b| lifetime_def_equal(a, b))) &&
        path_equal(&a.path, &b.path, inout)
}

fn trait_bound_hash<H: Hasher>(t: &TraitBound, pos: usize, h: &mut H) {
    t.modifier.hash(h);
    for bound_lifetimes in &t.lifetimes {
        for lifetime_def in &bound_lifetimes.lifetimes {
            lifetime_def_hash(lifetime_def, h);
        }
    }
    path_hash(&t.path, pos, h);
}

fn lifetime_def_equal(a: &LifetimeDef, b: &LifetimeDef) -> bool {
    lifetime_equal(&a.lifetime, &b.lifetime) && vecd(&a.bounds, &b.bounds,
        |a, b| lifetime_equal(a, b))
}

fn lifetime_def_hash<H: Hasher>(l: &LifetimeDef, h: &mut H) {
    l.lifetime.hash(h);
    for bound in &l.bounds {
        bound.hash(h);
    }
}

fn lifetime_equal(a: &Lifetime, b: &Lifetime) -> bool {
    a == b
}

fn expr_equal(a: &Expr, b: &Expr, inout: bool) -> bool {
    a == b // TODO test this
}

fn get_const(e: &Expr) -> Option<u64> {
    if let Expr::Lit(ExprLit { lit: Lit::Int(ref int), ..}) = *e {
        return Some(int.value());
    }
    None
}
