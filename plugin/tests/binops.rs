#![cfg_attr(test, feature(plugin))]
#![cfg_attr(test, plugin(mutagen_plugin))]
extern crate mutagen;

mod common;

use common::*;

#[cfg_attr(test, mutate)]
#[allow(unused)]
fn mutated_function() {
    let ord = 0x60;

    if (ord < 0x41 || ord > 0x5A) && (ord < 0x71 || ord > 0x7A) {
        // Do something
    }

    if (2 == 3) {
        // Do something
    }

    if (true && false) {
        // Do something
    }

    if (2 != 3) {
        // Do something
    }

    if (ord < 2) {
        // Do something
    }

    if ord == 2 {
        // Some
    }
}

#[test]
fn test_binop_ors() {
    let checker = MutationsChecker::new("tests/binops.rs").unwrap();

    let ors = &[
        "replacing _ || _ with false",
        "replacing _ || _ with true",
        "replacing x || _ with x",
        "replacing x || _ with !x",
        "replacing x || y with x || !y",
    ];

    assert!(checker.has_multiple(ors, "14:9: 14:33"));
    assert!(checker.has_multiple(ors, "14:39: 14:63"));
}

#[test]
fn test_binop_eq() {
    let checker = MutationsChecker::new("tests/binops.rs").unwrap();

    let eq_msgs = &[
        "replacing _ == _ with false",
        "replacing _ == _ with true",
        "replacing x == y with x != y",
    ];

    assert!(checker.has_multiple(eq_msgs, "18:9: 18:15"));
}

#[test]
fn test_binop_and() {
    let checker = MutationsChecker::new("tests/binops.rs").unwrap();

    let ands = &[
        "replacing _ && _ with false",
        "replacing _ && _ with true",
        "replacing x && _ with x",
        "replacing x && _ with !x",
        "replacing x && y with x && !y",
    ];

    assert!(checker.has_multiple(ands, "22:9: 22:22"));
}

#[test]
fn test_binop_ne() {
    let checker = MutationsChecker::new("tests/binops.rs").unwrap();

    let noneq_msgs = &[
        "replacing _ != _ with false",
        "replacing _ != _ with true",
        "replacing x != y with x == y",
    ];

    assert!(checker.has_multiple(noneq_msgs, "26:9: 26:15"));
}

#[test]
fn test_lt() {
    let checker = MutationsChecker::new("tests/binops.rs").unwrap();

    let lt_msgs = &[
        "replacing _ < _ with false",
        "replacing _ < _ with true",
        "replacing x < y with x > y",
        "replacing x < y with x >= y",
        "replacing x < y with x <= y",
        "replacing x < y with x == y",
        "replacing x < y with x != y",
    ];

    assert!(checker.has_multiple(lt_msgs, "30"));
}

#[test]
fn test_binop_eq_and_off_by_one() {
    let checker = MutationsChecker::new("tests/binops.rs").unwrap();

    let eq_msgs = &[
        "inverting if condition",
        "replacing if condition with false",
        "replacing if condition with true",
    ];

    assert!(checker.has_multiple(eq_msgs, "34:8: 34:16"));

    let eq_msgs = &["sub one to int constant", "add one to int constant"];

    assert!(checker.has_multiple(eq_msgs, "34:15: 34:16"));
}
