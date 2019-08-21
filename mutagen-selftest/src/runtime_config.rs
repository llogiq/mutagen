#[cfg(test)]
mod tests {

    use ::mutagen::MutagenRuntimeConfig;

    #[test]
    fn with_mutation_id_1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(MutagenRuntimeConfig::get_default().mutation_id(), Some(1));
        })
    }
    #[test]
    fn without_mutation() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(MutagenRuntimeConfig::get_default().mutation_id(), None);
        })
    }
    #[test]
    #[should_panic]
    fn with_mutation_id_0() {
        MutagenRuntimeConfig::with_mutation_id(0);
    }
}
