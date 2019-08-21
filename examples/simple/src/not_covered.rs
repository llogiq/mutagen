use mutagen::mutate;

#[cfg_attr(test, mutate)]
pub fn simple_assert_not_covered() {
    1 < 3;
}

// There are no tests since the function above is supposed to be no covered by tests
