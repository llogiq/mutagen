#[cfg(test)]
mod tests {

    mod test_simple_true {

        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        #[mutate(conf = local(expected_mutations = 1), mutators = only(lit_bool))]
        fn simple_true() -> bool {
            true
        }
        #[test]
        fn simple_true_inactive() {
            MutagenRuntimeConfig::test_without_mutation(|| {
                assert_eq!(simple_true(), true);
            })
        }
        #[test]
        fn simple_true_active() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(simple_true(), false);
            })
        }
    }

    mod test_simple_false {

        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // constant false
        #[mutate(conf = local(expected_mutations = 1), mutators = only(lit_bool))]
        fn simple_false() -> bool {
            false
        }
        #[test]
        fn simple_false_inactive() {
            MutagenRuntimeConfig::test_without_mutation(|| {
                assert_eq!(simple_false(), false);
            })
        }
        #[test]
        fn simple_false_active() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(simple_false(), true);
            })
        }
    }
}
