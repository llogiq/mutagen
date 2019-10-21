mod test_lt {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // simple comparison
    #[mutate(conf = local(expected_mutations = 3), mutators = only(binop_cmp))]
    fn lt(left: i32, right: i32) -> bool {
        left < right
    }
    #[test]
    fn lt_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(lt(1, 2), true);
            assert_eq!(lt(3, 3), false);
            assert_eq!(lt(5, 4), false);
        })
    }
    // replace with <=
    #[test]
    fn lt_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(lt(1, 2), true);
            assert_eq!(lt(3, 3), true);
            assert_eq!(lt(5, 4), false);
        })
    }
    // replace with >=
    #[test]
    fn lt_active2() {
        MutagenRuntimeConfig::test_with_mutation_id(2, || {
            assert_eq!(lt(1, 2), false);
            assert_eq!(lt(3, 3), true);
            assert_eq!(lt(5, 4), true);
        })
    }
    // replace with >
    #[test]
    fn sum_u32_active3() {
        MutagenRuntimeConfig::test_with_mutation_id(3, || {
            assert_eq!(lt(1, 2), false);
            assert_eq!(lt(3, 3), false);
            assert_eq!(lt(5, 4), true);
        })
    }
}

mod test_le {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // simple comparison
    #[mutate(conf = local(expected_mutations = 3), mutators = only(binop_cmp))]
    fn le(left: i32, right: i32) -> bool {
        left <= right
    }
    #[test]
    fn le_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(le(1, 2), true);
            assert_eq!(le(3, 3), true);
            assert_eq!(le(5, 4), false);
        })
    }
    // replace with <
    #[test]
    fn le_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(le(1, 2), true);
            assert_eq!(le(3, 3), false);
            assert_eq!(le(5, 4), false);
        })
    }
    // replace with >=
    #[test]
    fn le_active2() {
        MutagenRuntimeConfig::test_with_mutation_id(2, || {
            assert_eq!(le(1, 2), false);
            assert_eq!(le(3, 3), true);
            assert_eq!(le(5, 4), true);
        })
    }
    // replace with >
    #[test]
    fn le_active3() {
        MutagenRuntimeConfig::test_with_mutation_id(3, || {
            assert_eq!(le(1, 2), false);
            assert_eq!(le(3, 3), false);
            assert_eq!(le(5, 4), true);
        })
    }
}

mod test_ge {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // simpge comparison
    #[mutate(conf = local(expected_mutations = 3), mutators = only(binop_cmp))]
    fn ge(left: i32, right: i32) -> bool {
        left >= right
    }
    #[test]
    fn ge_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(ge(1, 2), false);
            assert_eq!(ge(3, 3), true);
            assert_eq!(ge(5, 4), true);
        })
    }
    // replace with <
    #[test]
    fn ge_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(ge(1, 2), true);
            assert_eq!(ge(3, 3), false);
            assert_eq!(ge(5, 4), false);
        })
    }
    // replace with <=
    #[test]
    fn ge_active2() {
        MutagenRuntimeConfig::test_with_mutation_id(2, || {
            assert_eq!(ge(1, 2), true);
            assert_eq!(ge(3, 3), true);
            assert_eq!(ge(5, 4), false);
        })
    }
    // replace with >
    #[test]
    fn ge_active3() {
        MutagenRuntimeConfig::test_with_mutation_id(3, || {
            assert_eq!(ge(1, 2), false);
            assert_eq!(ge(3, 3), false);
            assert_eq!(ge(5, 4), true);
        })
    }
}

mod test_gt {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // simple comparison
    #[mutate(conf = local(expected_mutations = 3), mutators = only(binop_cmp))]
    fn gt(left: i32, right: i32) -> bool {
        left > right
    }
    #[test]
    fn gt_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(gt(1, 2), false);
            assert_eq!(gt(3, 3), false);
            assert_eq!(gt(5, 4), true);
        })
    }
    // replace with <
    #[test]
    fn gt_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(gt(1, 2), true);
            assert_eq!(gt(3, 3), false);
            assert_eq!(gt(5, 4), false);
        })
    }
    // replace with <=
    #[test]
    fn gt_active2() {
        MutagenRuntimeConfig::test_with_mutation_id(2, || {
            assert_eq!(gt(1, 2), true);
            assert_eq!(gt(3, 3), true);
            assert_eq!(gt(5, 4), false);
        })
    }
    // replace with >=
    #[test]
    fn gt_active3() {
        MutagenRuntimeConfig::test_with_mutation_id(3, || {
            assert_eq!(gt(1, 2), false);
            assert_eq!(gt(3, 3), true);
            assert_eq!(gt(5, 4), true);
        })
    }
}

mod test_cmp_nocopy {
    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // simple comparison
    #[mutate(conf = local(expected_mutations = 3), mutators = only(binop_cmp))]
    fn max(left: String, right: String) -> String {
        if left > right {
            left
        } else {
            right
        }
    }

    #[test]
    fn max_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(max("a".to_owned(), "b".to_owned()), "b".to_owned());
            assert_eq!(max("b".to_owned(), "a".to_owned()), "b".to_owned());
        })
    }

    #[test]
    fn max_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(max("a".to_owned(), "b".to_owned()), "a".to_owned());
            assert_eq!(max("b".to_owned(), "a".to_owned()), "a".to_owned());
        })
    }
}
