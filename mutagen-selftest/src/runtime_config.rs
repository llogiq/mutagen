#[cfg(test)]
mod tests {

    use ::mutagen::MutagenRuntimeConfig;

    #[test]
    fn with_mutation_id_1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(MutagenRuntimeConfig::get_default().mutation_id, 1);
        })
    }
    #[test]
    fn with_mutation_id_0() {
        MutagenRuntimeConfig::test_with_mutation_id(0, || {
            assert_eq!(MutagenRuntimeConfig::get_default().mutation_id, 0);
        })
    }
}
