#[cfg(test)]
mod tests {

    mod test_sum_u32 {

        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // test that literals, that are nested in a outside expressen, are mutated
        #[mutate(conf(local), only(lit_int))]
        fn sum_u32() -> u32 {
            1 + 2
        }
        #[test]
        fn sum_u32_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(sum_u32(), 3);
            })
        }
        #[test]
        fn sum_u32_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(sum_u32(), 4);
            })
        }

        #[test]
        fn sum_u32_active2() {
            MutagenRuntimeConfig::test_with_mutation_id(2, || {
                assert_eq!(sum_u32(), 4);
            })
        }
    }

    mod test_lit_u8_suffixed {

        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        #[mutate(conf(local), only(lit_int))]
        fn lit_u8_suffixed() -> u8 {
            1u8
        }
        #[test]
        fn lit_u8_suffixed_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(lit_u8_suffixed(), 1);
            })
        }
        #[test]
        fn lit_u8_suffixed_active() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(lit_u8_suffixed(), 2);
            })
        }
    }
}
