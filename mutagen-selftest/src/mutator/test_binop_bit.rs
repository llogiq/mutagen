mod test_and_i32 {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // simple function that sums two values
    #[mutate(conf = local(expected_mutations = 2), mutators = only(binop_bit))]
    fn and_u32() ->u32 {
        0b10 & 0b11
    }
    #[test]
    fn and_u32_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(and_u32(), 0b10);
        })
    }
    #[test]
    fn sum_u32_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(and_u32(), 0b11);
        })
    }
    #[test]
    fn sum_u32_active2() {
        MutagenRuntimeConfig::test_with_mutation_id(2, || {
            assert_eq!(and_u32(), 0b01);
        })
    }
}
