mod test_eq {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // simple comparison
    #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_eq))]
    fn eq(left: i32, right: i32) -> bool {
        left == right
    }
    #[test]
    fn eq_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(eq(1, 2), false);
            assert_eq!(eq(3, 3), true);
        })
    }
    #[test]
    fn eq_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(eq(1, 2), true);
            assert_eq!(eq(3, 3), false);
        })
    }
}
mod test_ne {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // simple comparison
    #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_eq))]
    fn ne(left: i32, right: i32) -> bool {
        left != right
    }
    #[test]
    fn ne_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(ne(1, 2), true);
            assert_eq!(ne(3, 3), false);
        })
    }
    #[test]
    fn ne_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(ne(1, 2), false);
            assert_eq!(ne(3, 3), true);
        })
    }
}

mod eq_but_not_copy {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    #[derive(PartialEq, Eq)]
    struct EqButNotCopy;

    #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_eq))]
    fn eq(x: &EqButNotCopy, y: &EqButNotCopy) -> bool {
        *x == *y
    }
    #[test]
    fn eq_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert!(eq(&EqButNotCopy, &EqButNotCopy));
        })
    }
    #[test]
    fn eq_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert!(!eq(&EqButNotCopy, &EqButNotCopy));
        })
    }
}

mod divides {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_eq))]
    fn divides(x: u32, y: u32) -> bool {
        x % y == 0u32
    }

    #[test]
    fn divides_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert!(divides(2, 2));
            assert!(!divides(3, 4));
        })
    }
    #[test]
    fn divides_active() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert!(!divides(2, 2));
            assert!(divides(3, 4));
        })
    }
}
