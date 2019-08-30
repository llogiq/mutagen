#[cfg(test)]
mod tests {

    mod test_sum_u32 {

        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // test that literals, that are nested in a outside expressen, are mutated
        #[mutate(conf = local(expected_mutations = 4), mutators = only(lit_int))]
        fn sum_u32() -> u32 {
            1 + 2
        }
        #[test]
        fn sum_u32_inactive() {
            MutagenRuntimeConfig::test_without_mutation(|| {
                assert_eq!(sum_u32(), 3);
            })
        }
        // first literal -1
        #[test]
        fn sum_u32_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(sum_u32(), 4);
            })
        }

        // second literal +1
        #[test]
        fn sum_u32_active2() {
            MutagenRuntimeConfig::test_with_mutation_id(2, || {
                assert_eq!(sum_u32(), 2);
            })
        }
        // second literal -1
        #[test]
        fn sum_u32_active3() {
            MutagenRuntimeConfig::test_with_mutation_id(3, || {
                assert_eq!(sum_u32(), 4);
            })
        }
        // first literal -1
        #[test]
        fn sum_u32_active4() {
            MutagenRuntimeConfig::test_with_mutation_id(4, || {
                assert_eq!(sum_u32(), 2);
            })
        }
    }

    mod test_lit_u8_suffixed {

        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        #[mutate(conf = local(expected_mutations = 2), mutators = only(lit_int))]
        fn lit_u8_suffixed() -> u8 {
            1u8
        }
        #[test]
        fn lit_u8_suffixed_inactive() {
            MutagenRuntimeConfig::test_without_mutation(|| {
                assert_eq!(lit_u8_suffixed(), 1);
            })
        }
        // literal +1
        #[test]
        fn lit_u8_suffixed_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(lit_u8_suffixed(), 2);
            })
        }
        // literal -1
        #[test]
        fn lit_u8_suffixed_active2() {
            MutagenRuntimeConfig::test_with_mutation_id(2, || {
                assert_eq!(lit_u8_suffixed(), 0);
            })
        }
    }
    mod test_lit_u8_overflown_literal {

        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        #[mutate(conf = local(expected_mutations = 2), mutators = only(lit_int))]
        fn lit_u8_overflown_literal() -> u8 {
            255
        }
        #[test]
        fn lit_u8_overflown_literal_inactive() {
            MutagenRuntimeConfig::test_without_mutation(|| {
                assert_eq!(lit_u8_overflown_literal(), 255);
            })
        }
        // literal +1 -> wraps around
        #[test]
        fn lit_u8_overflown_literal_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(lit_u8_overflown_literal(), 0);
            })
        }
        // literal -1
        #[test]
        fn lit_u8_overflown_literal_active2() {
            MutagenRuntimeConfig::test_with_mutation_id(2, || {
                assert_eq!(lit_u8_overflown_literal(), 254);
            })
        }
    }

    mod const_not_mutated {

        use ::mutagen::mutate;

        #[mutate(conf = local(expected_mutations = 0), mutators = only(lit_int))]
        const X: i32 = 5;

        #[test]
        fn x_is_5() {
            assert_eq!(X, 5)
        }
    }
    mod const_fn_not_mutated {

        use ::mutagen::mutate;

        #[mutate(conf = local(expected_mutations = 0), mutators = only(lit_int))]
        const fn x() -> i32 {
            5
        }

        #[test]
        fn x_is_5() {
            assert_eq!(x(), 5)
        }
    }

    mod array_expr_size_not_mutated {

        use ::mutagen::mutate;

        #[mutate(conf = local(expected_mutations = 0), mutators = only(lit_int))]
        fn x() -> Vec<()> {
            [(); 5].to_vec()
        }

        #[test]
        fn x_is_vec5() {
            assert_eq!(x().len(), 5)
        }
    }

    mod array_returntype_size_not_mutated {

        use ::mutagen::mutate;

        #[mutate(conf = local(expected_mutations = 0), mutators = only(lit_int))]
        fn x() -> Option<[();5]> {
            None
        }

        #[test]
        fn x_is_none() {
            assert_eq!(x(), None)
        }
    }

    mod tuple_index_access_not_mutated {

        use ::mutagen::mutate;

        #[mutate(conf = local(expected_mutations = 0), mutators = only(lit_int))]
        fn x() -> &'static str {
            ((),"").1
        }

        #[test]
        fn x_is_emptystr() {
            assert_eq!(x(), "")
        }
    }
}
