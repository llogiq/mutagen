#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![allow(unused_variables, dead_code, unused)]
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

#[mutate]
fn simple_interchange(x: i32, y: i32, z: i32) -> i32 {
    x + y - z
}

#[mutate]
fn no_interchange(x: i32, y: u32) {}

struct ComplexStruct {
    x: i32,
    y: u32,
}

#[mutate]
fn interchange_tuple((a, b): (i32, u32), c: i32, (d, e): (u32, i32), ComplexStruct{x: f, y: g}: ComplexStruct) {
    // Note that a and f, won't be exchanged with current code
}

#[mutate]
fn interchange_struct(ComplexStruct{x: a, y: b}: ComplexStruct, ComplexStruct{x: c, y: d}: ComplexStruct)
{

}

#[test]
fn test_simple_interchange() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    assert!(checker.has("exchange x with y", "41"));
    assert!(checker.has("exchange x with z", "41"));
    assert!(checker.has("exchange y with z", "41"));
    assert!(!checker.has("exchange", "46"));
}

#[test]
fn test_tuple_interchange() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let msgs = &[
        "exchange a with c",
        "exchange a with e",
        "exchange b with d",
        "exchange c with e",
    ];

    assert!(checker.has_multiple(msgs, "54"));
}

#[test]
fn test_struct_interchange() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let msgs = &[
        "exchange a with c",
        "exchange b with d",
    ];

    assert!(checker.has_multiple(msgs, "60"));
}

#[test]
fn test_complex_interchange() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    assert!(checker.has("exchange a with c", "54"));
    assert!(checker.has("exchange b with d", "54"));
}

#[test]
fn test_binop_ors() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let ors = &[
        "replacing _ || _ with false",
        "replacing _ || _ with true",
        "replacing x || _ with x",
        "replacing x || _ with !x",
        "replacing x || y with x || !y",
    ];

    assert!(checker.has_multiple(ors, "15:9: 15:33"));
    assert!(checker.has_multiple(ors, "15:39: 15:63"));
}

#[test]
fn test_binop_eq() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let eq_msgs = &[
        "replacing _ == _ with false",
        "replacing _ == _ with true",
        "replacing x == y with x != y",
    ];

    assert!(checker.has_multiple(eq_msgs, "19:9: 19:15"));
}

#[test]
fn test_binop_and() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let ands = &[
        "replacing _ && _ with false",
        "replacing _ && _ with true",
        "replacing x && _ with x",
        "replacing x && _ with !x",
        "replacing x && y with x && !y",
    ];

    assert!(checker.has_multiple(ands, "23:9: 23:22"));
}

#[test]
fn test_binop_ne() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let noneq_msgs = &[
        "replacing _ != _ with false",
        "replacing _ != _ with true",
        "replacing x != y with x == y",
    ];

    assert!(checker.has_multiple(noneq_msgs, "27:9: 27:15"));
}

#[test]
fn test_lt() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let lt_msgs = &[
        "replacing _ < _ with false",
        "replacing _ < _ with true",
        "replacing x < y with x > y",
        "replacing x < y with x >= y",
        "replacing x < y with x <= y",
        "replacing x < y with x == y",
        "replacing x < y with x != y",
    ];

    assert!(checker.has_multiple(lt_msgs, "31"));
}

#[test]
fn test_binop_eq_and_off_by_one() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let eq_msgs = &[
        "inverting if condition",
        "replacing if condition with false",
        "replacing if condition with true",
    ];

    assert!(checker.has_multiple(eq_msgs, "35:8: 35:16"));

    let eq_msgs = &["sub one from int constant", "add one to int constant"];

    assert!(checker.has_multiple(eq_msgs, "35:15: 35:16"));
}
