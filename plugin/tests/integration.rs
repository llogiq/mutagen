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

    let or_left = ord < 0x41 || ord > 0x5A;
    if or_left && (ord < 0x71 || ord > 0x7A) {
        // Do something
    }

    let eq = 2 == 3;
    let and = true && false;
    let ne = 2 != 3;

    if ord < 2 {
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

#[mutate]
impl ComplexStruct {
    fn interchange_self(self, other: Self) {

    }

    fn interchange_other_self(one: Self, other: Self) {

    }
}

#[mutate]
fn mutation_prune_ifs() {
    let a = true;
    let b = false;
    let c = true;

    if (a || b) && if b && c { true } else { c == b} {

    }

    if a == b {
        // This should only issue 3 mutations with replace true / replace false / negate
    }
}

#[test]
fn test_simple_interchange() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    assert!(checker.has("exchange x with y", "34"));
    assert!(checker.has("exchange x with z", "34"));
    assert!(checker.has("exchange y with z", "34"));
    assert!(!checker.has("exchange", "52"));
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

    assert!(checker.has_multiple(msgs, "47"));
}

#[test]
fn test_struct_interchange() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let msgs = &[
        "exchange a with c",
        "exchange b with d",
    ];

    assert!(checker.has_multiple(msgs, "53"));
}

#[test]
fn test_complex_interchange() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    assert!(checker.has("exchange a with c", "47"));
    assert!(checker.has("exchange b with d", "47"));
}

#[test]
fn test_self_interchange() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    assert!(checker.has("exchange self with other", "59"));
    assert!(checker.has("exchange one with other", "63"));
}

#[test]
fn test_binop_ors() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let ors = &[
        "REPLACE_WITH_FALSE",
        "REPLACE_WITH_TRUE",
        "REMOVE_RIGHT",
        "NEGATE_LEFT",
        "NEGATE_RIGHT",
    ];

    assert!(checker.has_multiple(ors, "15:19: 15:43"));
}

#[test]
fn test_binop_eq() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let eq_msgs = &[
        "REPLACE_WITH_TRUE",
        "REPLACE_WITH_FALSE",
        "NEGATE_EXPRESSION",
    ];

    assert!(checker.has_multiple(eq_msgs, "20:14: 20:20"));
}

#[test]
fn test_binop_and() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let ands = &[
        "REPLACE_WITH_FALSE",
        "REPLACE_WITH_TRUE",
        "REMOVE_RIGHT",
        "NEGATE_LEFT",
        "NEGATE_RIGHT",
    ];

    assert!(checker.has_multiple(ands, "21:15: 21:28"));
}

#[test]
fn test_binop_ne() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let noneq_msgs = &[
        "REPLACE_WITH_TRUE",
        "REPLACE_WITH_FALSE",
        "NEGATE_EXPRESSION",
    ];

    assert!(checker.has_multiple(noneq_msgs, "22:14: 22:20"));
}

#[test]
fn test_lt() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let lt_msgs = &[
        "REPLACE_WITH_TRUE",
        "REPLACE_WITH_FALSE",
        "NEGATE_EXPRESSION",
        "COMPARISION",
    ];

    assert!(checker.has_multiple(lt_msgs, "24"));
}

#[test]
fn test_binop_eq_and_off_by_one() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let eq_msgs = &[
        "REPLACE_WITH_TRUE",
        "REPLACE_WITH_FALSE",
        "NEGATE_EXPRESSION",
        "ADD_ONE_TO_LITERAL",
        "SUB_ONE_TO_LITERAL",
    ];

    assert!(checker.has_multiple(eq_msgs, "28"));
}

#[test]
fn test_redundant_mutations() {
    let checker = MutationsChecker::new("tests/integration.rs").unwrap();

    let types = &[
        "REPLACE_WITH_TRUE",
        "REPLACE_WITH_FALSE",
        "NEGATE_EXPRESSION",
    ];

    assert!(checker.has_multiple(types, "78:8: 78:14"));
}
