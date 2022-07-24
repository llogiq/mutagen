mod test_return_non_empty_string {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    #[mutate(conf = local(expected_mutations = 2), mutators = only(lit_str))]
    fn return_non_empty_string() -> String {
        #[allow(unused_parens)]
        let s = "a";
        s.to_string()
    }

    #[test]
    fn inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(return_non_empty_string(), "a".to_string());
        })
    }

    #[test]
    fn active_clear() {
        let _ = MutagenRuntimeConfig::get_default();
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(return_non_empty_string(), "".to_string());
        })
    }

    #[test]
    fn active_set() {
        MutagenRuntimeConfig::test_with_mutation_id(2, || {
            assert_eq!(return_non_empty_string(), "-".to_string());
        })
    }
}

mod test_return_check_equals_a {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    #[mutate(conf = local(expected_mutations = 2), mutators = only(lit_str))]
    fn check_equals_a(input: &str) -> bool {
        "-" == input
    }

    #[test]
    fn inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(check_equals_a("-"), true);
        })
    }

    #[test]
    fn active_clear() {
        let _ = MutagenRuntimeConfig::get_default();
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(check_equals_a("-"), false);
            assert_eq!(check_equals_a(""), true);
        })
    }

    #[test]
    fn active_prepend() {
        MutagenRuntimeConfig::test_with_mutation_id(2, || {
            assert_eq!(check_equals_a("-"), false);
            assert_eq!(check_equals_a("*"), true);
        })
    }
}

mod test_return_empty_string {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    #[mutate(conf = local(expected_mutations = 1), mutators = only(lit_str))]
    fn return_empty_string() -> String {
        #[allow(unused_parens)]
        let s = "";
        s.to_string()
    }

    #[test]
    fn inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(return_empty_string(), "".to_string());
        })
    }

    #[test]
    fn active_set() {
        let _ = MutagenRuntimeConfig::get_default();
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(return_empty_string(), "A".to_string());
        })
    }
}

mod test_return_check_equals_empty_str {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    #[mutate(conf = local(expected_mutations = 1), mutators = only(lit_str))]
    fn check_equals_empty_str(input: &str) -> bool {
        "" == input
    }

    #[test]
    fn inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(check_equals_empty_str(""), true);
        })
    }

    #[test]
    fn active_set() {
        let _ = MutagenRuntimeConfig::get_default();
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(check_equals_empty_str(""), false);
            assert_eq!(check_equals_empty_str("A"), true);
        })
    }
}

mod test_temp_variable {
    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    #[mutate(conf = local(expected_mutations = 1), mutators = only(lit_str))]
    fn a() -> usize {
        #[allow(unused_parens)]
        let x = "";
        x.len()
    }

    #[test]
    fn inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(a(), 0);
        })
    }

    #[test]
    fn active_set() {
        let _ = MutagenRuntimeConfig::get_default();
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(a(), 1);
        })
    }
}

mod test_to_string {
    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    #[mutate(conf = local(expected_mutations = 1), mutators = only(lit_str))]
    fn a() -> &'static str {
        ""
    }

    #[test]
    fn inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(a(), "".to_string());
        })
    }

    #[test]
    fn active_set() {
        let _ = MutagenRuntimeConfig::get_default();
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(a(), "A".to_string());
        })
    }
}

mod test_temporary_variable {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    #[mutate(conf = local(expected_mutations = 1), mutators = only(lit_str))]
    fn a() -> &'static str {
        ""
    }

    #[test]
    fn inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(a(), "");
        })
    }

    #[test]
    fn active_clear() {
        let _ = MutagenRuntimeConfig::get_default();
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(a(), "A");
        })
    }
}
