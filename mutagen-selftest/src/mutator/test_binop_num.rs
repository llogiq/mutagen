mod test_sum_i32 {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // simple function that sums two values
    #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_num))]
    fn sum_i32() -> i32 {
        5 + 1
    }
    #[test]
    fn sum_u32_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(sum_i32(), 6);
        })
    }
    #[test]
    fn sum_u32_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(sum_i32(), 4);
        })
    }
}

mod test_sum_u32 {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // simple function that sums 2 u32 values
    #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_num))]
    fn sum_u32() -> u32 {
        5 + 1
    }
    #[test]
    fn sum_u32_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(sum_u32(), 6);
        })
    }
    #[test]
    fn sum_u32_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(sum_u32(), 4);
        })
    }
}

mod test_str_add {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // strings cannot be subtracted, the mutation that changes `+` into `-` should panic
    #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_num))]
    fn str_add() -> String {
        "a".to_string() + "b"
    }
    #[test]
    fn str_add_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(&str_add(), "ab");
        })
    }
    #[test]
    #[should_panic]
    fn str_add_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            str_add();
        })
    }
}

mod test_multiple_adds {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // sum of multiple values without brackets
    #[mutate(conf = local(expected_mutations = 2), mutators = only(binop_num))]
    pub fn multiple_adds() -> usize {
        5 + 4 + 1
    }

    #[test]
    fn multiple_adds_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(multiple_adds(), 10);
        })
    }
    #[test]
    fn multiple_adds_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(multiple_adds(), 2);
        })
    }
    #[test]
    fn multiple_adds_active2() {
        MutagenRuntimeConfig::test_with_mutation_id(2, || {
            assert_eq!(multiple_adds(), 8);
        })
    }
}

mod test_sub_f64 {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // sum of multiple values without brackets
    #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_num))]
    pub fn sub_f64() -> f64 {
        1.0 - 2.0
    }

    #[test]
    fn sub_f64_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(sub_f64(), -1.0);
        })
    }
    #[test]
    fn sub_f64_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(sub_f64(), 3.0);
        })
    }
}

mod test_mul_2_3 {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // multiplication of two integers
    #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_num))]
    pub fn mul_2_3() -> u32 {
        2 * 3
    }

    #[test]
    fn mul_2_3_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(mul_2_3(), 6);
        })
    }
    #[test]
    fn mul_2_3_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(mul_2_3(), 0);
        })
    }
}

mod test_div_4_4 {

    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // division of two integers
    #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_num))]
    pub fn div_4_4() -> u32 {
        4 / 4
    }

    #[test]
    fn div_4_4_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(div_4_4(), 1);
        })
    }
    #[test]
    fn div_4_4_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(div_4_4(), 16);
        })
    }
}

mod test_add_after_if {
    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // addition after an if expression
    #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_num))]
    pub fn add_after_if(bit: bool) -> i8 {
        (if bit { 1 } else { 0 }) + 2
    }

    #[test]
    fn add_after_if_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(add_after_if(true), 3);
            assert_eq!(add_after_if(false), 2);
        })
    }
    #[test]
    fn add_after_if_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(add_after_if(true), -1);
            assert_eq!(add_after_if(false), -2);
        })
    }
}

mod test_add_after_block {
    use ::mutagen::mutate;
    use ::mutagen::MutagenRuntimeConfig;

    // addition after a block expression
    #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_num))]
    pub fn add_after_block() -> i8 {
        ({
            "";
            1
        }) + 2
    }

    #[test]
    fn add_after_block_inactive() {
        MutagenRuntimeConfig::test_without_mutation(|| {
            assert_eq!(add_after_block(), 3);
        })
    }
    #[test]
    fn add_after_block_active1() {
        MutagenRuntimeConfig::test_with_mutation_id(1, || {
            assert_eq!(add_after_block(), -1);
        })
    }
}

// TODO: fix this issue
// mod test_add_i64_with_tempvar {

//     use ::mutagen::mutate;
//     use ::mutagen::MutagenRuntimeConfig;

//     // add two numbers, the first one is a temporary variable
//     #[mutate(conf = local(expected_mutations = 1), mutators = only(binop_num))]
//     pub fn add_with_tempvar() -> i64 {
//         let x = 2;
//         x + 1
//     }

//     #[test]
//     fn add_with_tempvar_inactive() {
//         MutagenRuntimeConfig::test_without_mutation(|| {
//             assert_eq!(add_with_tempvar(), 3);
//         })
//     }
//     #[test]
//     fn add_with_tempvar_active1() {
//         MutagenRuntimeConfig::test_with_mutation_id(1, || {
//             assert_eq!(add_with_tempvar(), 1);
//         })
//     }
// }
