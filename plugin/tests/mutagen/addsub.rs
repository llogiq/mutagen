#![feature(plugin)]
#![plugin(mutagen_plugin)]
#![feature(custom_attribute)]
extern crate mutagen;

use std::ops::Add;

#[mutate]
fn mutated_function() {
    2 + 3;

    let p1 = Point{x: 2, y: 3};
    let p2 = Point{x: 6, y: 1};
    let sum = p1 + p2;

    assert_eq!(sum.x, 8);
    assert_eq!(sum.y, 4);
}

struct Point {
    x: i32,
    y: i32,
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

fn main() {}