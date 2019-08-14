#[cfg(test)]
mod tests {
    mod test_and {
        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // and-operation using 2 closures
        #[mutate(conf = local)]
        fn and(left: impl Fn() -> bool, right: impl Fn() -> bool) -> bool {
            left() && right()
        }
        #[test]
        fn and_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(and(|| true, || true), true);
                assert_eq!(and(|| false, || true), false);
                assert_eq!(and(|| true, || false), false);
                assert_eq!(and(|| false, || false), false);
            })
        }
        #[test]
        fn and_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(and(|| true, || true), true);
                assert_eq!(and(|| false, || true), true);
                assert_eq!(and(|| true, || false), true);
                assert_eq!(and(|| false, || false), false);
            })
        }
        #[test]
        fn and_short_circuit_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(and(|| false, || panic!()), false);
            })
        }
        #[test]
        #[should_panic]
        fn and_short_circuit_active() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                and(|| false, || panic!());
            })
        }
    }
    mod test_or {
        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // and-operation using 2 closures
        #[mutate(conf = local)]
        fn or(left: impl Fn() -> bool, right: impl Fn() -> bool) -> bool {
            left() || right()
        }
        #[test]
        fn or_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(or(|| true, || true), true);
                assert_eq!(or(|| false, || true), true);
                assert_eq!(or(|| true, || false), true);
                assert_eq!(or(|| false, || false), false);
            })
        }
        #[test]
        fn or_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(or(|| true, || true), true);
                assert_eq!(or(|| false, || true), false);
                assert_eq!(or(|| true, || false), false);
                assert_eq!(or(|| false, || false), false);
            })
        }
        #[test]
        fn or_short_circuit_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(or(|| true, || panic!()), true);
            })
        }
        #[test]
        #[should_panic]
        fn or_short_circuit_active() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                or(|| true, || panic!());
            })
        }
    }
}
