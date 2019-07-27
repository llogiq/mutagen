#[cfg(test)]
mod tests {
    mod test_eq {

        use ::mutagen::mutate;
        use ::mutagen::MutagenRuntimeConfig;

        // simple comparison
        #[mutate(conf(local), only(binop_eq))]
        fn eq(left: i32, right: i32) -> bool {
            left == right
        }
        #[test]
        fn eq_inactive() {
            MutagenRuntimeConfig::test_with_mutation_id(0, || {
                assert_eq!(eq(1, 2), false);
                assert_eq!(eq(3, 3), true);
            })
        }
        #[test]
        fn eq_active1() {
            MutagenRuntimeConfig::test_with_mutation_id(1, || {
                assert_eq!(eq(1, 2), true);
                assert_eq!(eq(3, 3), false);
            })
        }
    }
}
