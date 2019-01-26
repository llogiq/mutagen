use syn::*;
use core::hash::{Hash, Hasher};
use crate::ty::{self_or_ty_equal, ty_hash};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum BindingMode {
    Value,
    ValueMut,
    Ref,
    RefMut
}

#[derive(PartialEq, Eq, Hash)]
pub enum SelfOr<'t, T> {
    JustSelf,
    Other(&'t T),
}

impl<'t, T> Clone for SelfOr<'t, T> {
    fn clone(&self) -> Self {
        match *self {
            SelfOr::JustSelf => SelfOr::JustSelf,
            SelfOr::Other(ref t) => SelfOr::Other(t),
        }
    }
}

impl<'t, T> From<&'t T> for SelfOr<'t, T> {
    fn from(t: &'t T) -> Self {
        SelfOr::Other(t)
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum TyOcc<'a> {
    Field(&'a Ident),
    Index(usize),
    IndexFromEnd(usize),
    Deref,
    DerefMut,
}

pub struct ArgTy<'t>(pub BindingMode, pub SelfOr<'t, Type>, pub usize, pub Vec<TyOcc<'t>>);

impl<'t> PartialEq for ArgTy<'t> {
    fn eq(&self, other: &ArgTy<'t>) -> bool {
        self.0 == other.0 && self_or_ty_equal(&self.1, &other.1, self.2 == other.2) && self.3 == other.3
    }
}

impl<'t> Eq for ArgTy<'t> { }

impl<'t> Hash for ArgTy<'t> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        if let SelfOr::Other(ref t) = self.1 {
            ty_hash(t, self.2, state);
        }
        self.3.hash(state);
    }
}

impl<'t> ArgTy<'t> {
    fn new(is_ref: bool, is_mut: bool, ty: SelfOr<'t, Type>, pos: usize, occ: Vec<TyOcc<'t>>) -> ArgTy<'t> {
        ArgTy(
            match (is_ref, is_mut) {
                (false, false) => BindingMode::Value,
                (false, true) => BindingMode::ValueMut,
                (true, false) => BindingMode::Ref,
                (true, true) => BindingMode::RefMut,
            },
            ty,
            pos,
            occ
        )
    }
}

pub fn destructure_fn_arg<'a>(
    arg: &'a FnArg,
    occs: &mut Vec<TyOcc<'a>>,
    pos: usize,
    argdefs: &mut Vec<(SelfOr<'a, Ident>, ArgTy<'a>)>)
{
    match *arg {
        FnArg::SelfRef(ArgSelfRef {
            ref lifetime,
            ref mutability,
            ..
        }) => {
            argdefs.push((SelfOr::JustSelf, ArgTy::new(true, mutability.is_some(), SelfOr::JustSelf, pos, occs.clone())))
        }
        FnArg::SelfValue(ArgSelf {
            ref mutability,
            ..
        }) => {
            argdefs.push((SelfOr::JustSelf, ArgTy::new(false, mutability.is_some(), SelfOr::JustSelf, pos, occs.clone())))
        }
        FnArg::Captured(ArgCaptured {
            ref pat,
            ref ty,
            ..
        }) => {
            destructure_pat(pat, ty.into(), occs, pos, argdefs);
        }
        _ => {} // we cannot do anything with inferred or ignored args
    }
}

fn destructure_pat<'a>(
    pat: &'a Pat,
    ty: SelfOr<'a, Type>,
    occs: &mut Vec<TyOcc<'a>>,
    pos: usize,
    argdefs: &mut Vec<(SelfOr<'a, Ident>, ArgTy<'a>)>)
{
    match pat {
        Pat::Ident(PatIdent {
            ref by_ref,
            ref mutability,
            ref ident,
            ref subpat
        }) => {
            argdefs.push((SelfOr::Other(ident), ArgTy::new(by_ref.is_some(), mutability.is_some(), ty.clone(), pos, occs.clone())));
            if let Some((_, ref p)) = *subpat {
                destructure_pat(&p, ty, occs, pos, argdefs);
            }
        }
        Pat::Struct(PatStruct { ref path, ref fields, .. }) => {
            for field in fields {
                if let Member::Named(ref ident) = field.member {
                    occs.push(TyOcc::Field(ident));
                    destructure_pat(&field.pat, ty.clone(), occs, pos, argdefs);
                    occs.pop();
                }
            }
        }
        Pat::TupleStruct(PatTupleStruct { ref path, pat: PatTuple { ref front, ref back, .. } }) => {
            for (i, p) in front.iter().enumerate() {
                occs.push(TyOcc::Index(i));
                destructure_pat(p, ty.clone(), occs, pos, argdefs);
                occs.pop();
            }
            let len = back.len();
            for (i, p) in back.iter().enumerate() {
                occs.push(TyOcc::IndexFromEnd(len - i));
                destructure_pat(p, ty.clone(), occs, pos, argdefs);
                occs.pop();
            }
        }
        Pat::Tuple(PatTuple { ref front, ref back, .. }) => {
            let len = back.len();
            if let SelfOr::Other(Type::Tuple(TypeTuple { ref elems, .. })) = ty {
                let tys : Vec<&Type> = elems.iter().collect();
                let ty_len = tys.len();
                for (p, ty) in front.iter().zip(tys.iter()) {
                    destructure_pat(p, SelfOr::Other(ty), &mut vec![], pos, argdefs);
                }
                for (p, ty) in back.iter().zip(&tys[ty_len - len..]) {
                    destructure_pat(p, SelfOr::Other(ty), &mut vec![], pos, argdefs);
                }
            } else { // perhaps a type alias?
                for (i, p) in front.iter().enumerate() {
                    occs.push(TyOcc::Index(i));
                    destructure_pat(p, ty.clone(), occs, pos, argdefs);
                    occs.pop();
                }
                for (i, p) in back.iter().enumerate() {
                    occs.push(TyOcc::IndexFromEnd(len - i));
                    destructure_pat(p, ty.clone(), occs, pos, argdefs);
                    occs.pop();
                }
            }
        },
        Pat::Box(PatBox { ref pat, .. }) => {
            occs.push(TyOcc::Deref);
            destructure_pat(&*pat, ty, occs, pos, argdefs);
            occs.pop();
        }
        Pat::Ref(PatRef { ref mutability, ref pat, .. }) => {
            occs.push(if mutability.is_some() { TyOcc::DerefMut } else { TyOcc::Deref });
            destructure_pat(&*pat, ty, occs, pos, argdefs);
            occs.pop();
        }
        //TODO Pat::Range(PatRange { ref lo: Box<Expr>, ref hi: Box<Expr>, .. }),

        //TODO `[a, b, i.., y, z]` Pat::Slice(PatSlice {
        //    pub front: Punctuated<Pat, Token![,]>,
        //    pub middle: Option<Box<Pat>>,
        //    pub dot2_token: Option<Token![..]>,
        //    pub comma_token: Option<Token![,]>,
        //    pub back: Punctuated<Pat, Token![,]>,
        //}),
        _ => {}
    }
}
