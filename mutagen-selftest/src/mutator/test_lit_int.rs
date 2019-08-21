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

        #[mutate(conf = local, mutators = only(lit_int))]
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
}
