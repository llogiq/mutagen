mod test_vecpush {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    /// create a vector and push a single value to it.
    #[mutate(conf = local(expected_mutations = 1), mutators = only(stmt_call))]
    fn vecpush() -> Vec<i32> {
        let mut x = Vec::new();
        x.push(1);
        x
    }
    #[test]
    fn vecpush_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| assert_eq!(vecpush(), vec![1]))
    }
    #[test]
    fn vecpush_active() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || assert_eq!(vecpush(), Vec::<i32>::new()))
    }
}

mod test_set_to_1 {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    /// sets the given reference to 1
    fn set_to_1_fn(x: &mut i32) {
        *x = 1;
    }

    /// returns `1`, by calling the function `set_to_one`
    #[mutate(conf = local(expected_mutations = 1), mutators = only(stmt_call))]
    fn set_to_1() -> i32 {
        let mut x = 0;
        set_to_1_fn(&mut x);
        x
    }
    #[test]
    fn set_to_1_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| assert_eq!(set_to_1(), 1))
    }
    #[test]
    fn set_to_1_active() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || assert_eq!(set_to_1(), 0))
    }
}
