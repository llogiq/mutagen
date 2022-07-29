//! tests that ensure that no mutation is triggered in certain cases

mod static_const {

    use ::mutagen::mutate;

    #[mutate(conf = local(expected_mutations = 0))]
    const X: i32 = 5;

    #[test]
    fn x_is_5() {
        assert_eq!(X, 5)
    }
}
mod const_fn {

    use ::mutagen::mutate;

    #[mutate(conf = local(expected_mutations = 0))]
    const fn x() -> i32 {
        5
    }

    #[test]
    fn x_is_5() {
        assert_eq!(x(), 5)
    }
}

mod const_method {

    use ::mutagen::mutate;

    struct X;

    #[mutate(conf = local(expected_mutations = 0))]
    impl X {
        const fn x() -> i32 {
            5
        }
    }

    #[test]
    fn x_is_5() {
        assert_eq!(X::x(), 5)
    }
}

mod array_expr_size {

    use ::mutagen::mutate;

    #[mutate(conf = local(expected_mutations = 0))]
    fn x() -> Vec<()> {
        [(); 5].to_vec()
    }

    #[test]
    fn x_is_vec5() {
        assert_eq!(x().len(), 5)
    }
}

mod array_returntype_size {

    use ::mutagen::mutate;

    #[mutate(conf = local(expected_mutations = 0))]
    fn x() -> Option<[(); 5]> {
        None
    }

    #[test]
    fn x_is_none() {
        assert_eq!(x(), None)
    }
}

mod tuple_index_access {

    use ::mutagen::mutate;

    #[mutate(conf = local(expected_mutations = 0), mutators = not(lit_str))]
    fn x() -> &'static str {
        ((), "").1
    }

    #[test]
    fn x_is_emptystr() {
        assert_eq!(x(), "")
    }
}
mod int_as_pattern {

    use ::mutagen::mutate;

    #[mutate(conf = local(expected_mutations = 0), mutators = not(lit_str))]
    fn x(i: i8) -> &'static str {
        match i {
            0 => "zero",
            1..=127 => "positive",
            _ => "negative",
        }
    }

    #[test]
    fn x_zero() {
        assert_eq!(x(0), "zero")
    }
    #[test]
    fn x_one_positive() {
        assert_eq!(x(1), "positive")
    }
    #[test]
    fn x_minus_one_negative() {
        assert_eq!(x(-1), "negative")
    }
}

mod unsafe_fn {
    use ::mutagen::mutate;

    #[mutate(conf = local(expected_mutations = 0))]
    unsafe fn x() -> u8 {
        5
    }

    #[test]
    fn x_is_5() {
        unsafe { assert_eq!(x(), 5) }
    }
}

mod unsafe_method {

    use ::mutagen::mutate;

    struct X;

    #[mutate(conf = local(expected_mutations = 0))]
    impl X {
        unsafe fn x() -> i32 {
            5
        }
    }

    #[test]
    fn x_is_5() {
        assert_eq!(unsafe { X::x() }, 5)
    }
}

mod unsafe_block {

    use ::mutagen::mutate;

    #[mutate(conf = local(expected_mutations = 0))]
    fn x() -> u8 {
        // this is a dummy-unsafe-block with something that *could* be mutated but should not
        #[allow(unused_unsafe)]
        unsafe {
            5
        }
    }

    #[test]
    fn x_is_5() {
        assert_eq!(x(), 5)
    }
}
