use mutagen::mutate;

enum LazyAdd {
    Val(u8),
    Lazy(Box<LazyAdd>, Box<LazyAdd>),
}

impl From<u8> for LazyAdd {
    fn from(v: u8) -> Self {
        Self::Val(v)
    }
}

impl std::ops::Add<LazyAdd> for LazyAdd {
    type Output = LazyAdd;
    fn add(self, rhs: LazyAdd) -> LazyAdd {
        LazyAdd::Lazy(Box::new(self), Box::new(rhs))
    }
}

#[cfg_attr(test, mutate)]
impl LazyAdd {
    pub fn eval(self) -> u8 {
        match self {
            Self::Val(v) => v,
            Self::Lazy(l, r) => l.eval() + r.eval(),
        }
    }

    pub fn add_one(self) -> Self {
        self + 1.into()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn add_one_to_zero() {
        assert_eq!(LazyAdd::from(0).add_one().eval(), 1);
    }
}
