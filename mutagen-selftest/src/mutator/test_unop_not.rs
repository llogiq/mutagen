#[cfg(test)]
mod tests {

    mod test_boolnot {

        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // simple function that negates the input
        #[mutate(conf = local(expected_mutations = 1), mutators = only(unop_not))]
        fn boolnot(x: bool) -> bool {
            !x
        }
        #[test]
        fn boolnot_inactive() {
            MutagenRuntimeConfig::test_without_mutation(|| {
                assert_eq!(boolnot(false), true);
                assert_eq!(boolnot(true), false);
            })
        }
        #[test]
        fn boolnot_active() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(boolnot(false), false);
                assert_eq!(boolnot(true), true);
            })
        }
    }

    mod test_optimistic_incorrect {

        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;
        use ::mutagen_core::optimistic::{TypeWithNotOtherOutput, TypeWithNotTarget};

        // strings cannot be subtracted, the mutation that changes `+` into `-` should panic
        #[mutate(conf = local(expected_mutations = 1), mutators = only(unop_not))]
        fn optimistic_incorrect(x: TypeWithNotOtherOutput) -> TypeWithNotTarget {
            !x
        }
        #[test]
        fn optimistic_incorrect_inactive() {
            MutagenRuntimeConfig::test_without_mutation(|| {
                assert_eq!(
                    optimistic_incorrect(TypeWithNotOtherOutput()),
                    TypeWithNotTarget()
                );
            })
        }
        #[test]
        #[should_panic]
        fn optimistic_incorrect_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                optimistic_incorrect(TypeWithNotOtherOutput());
            })
        }
    }

    mod test_double_negation {
        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // double negation
        #[mutate(conf = local(expected_mutations = 2), mutators = only(unop_not))]
        fn double_negation(x: bool) -> bool {
            !!x
        }
        #[test]
        fn double_negation_inactive() {
            MutagenRuntimeConfig::test_without_mutation(|| {
                assert_eq!(double_negation(true), true);
                assert_eq!(double_negation(false), false);
            })
        }
        #[test]
        fn double_negation_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(double_negation(true), false);
                assert_eq!(double_negation(false), true);
            })
        }
        #[test]
        fn double_negation_active2() {
            MutagenRuntimeConfig::test_with_mutation_id(2, || {
                assert_eq!(double_negation(true), false);
                assert_eq!(double_negation(false), true);
            })
        }
    }
}
