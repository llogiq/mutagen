#[cfg(test)]
mod tests {

    mod test_boolnot {

        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // simple test that sums 2 u32 values. Unfortunately, the tag `u32` is necessary
        #[mutate(conf(local), only(unop_not))]
        fn boolnot(x: bool) -> bool {
            !x
        }
        #[test]
        fn boolnot_false_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(boolnot(false), true);
            })
        }
        #[test]
        fn boolnot_false_active() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(boolnot(false), false);
            })
        }
        #[test]
        fn boolnot_true_active() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(boolnot(true), true);
            })
        }
    }

    mod test_optimistic_incorrect {

        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;
        use ::mutagen_core::optimistic::{TypeWithNotOtherOutput, TypeWithNotTarget};

        // strings cannot be subtracted, the mutation that changes `+` into `-` should panic
        #[mutate(conf(local), only(unop_not))]
        fn optimistic_incorrect(x: TypeWithNotOtherOutput) -> TypeWithNotTarget {
            !x
        }
        #[test]
        fn optimistic_incorrect_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
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
}
