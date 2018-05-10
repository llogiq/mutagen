use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{BufReader, BufRead};
use std::fs::File;

lazy_static!{
    static ref MUTAGEN_LOOP_COUNT: HashMap<LoopId, usize> = {
        let mut h = HashMap::new();

        File::open("target/mutagen/loops.txt").map(|file| {
            let reader = BufReader::new(file);

            for l in reader.lines() {
                let line = l.unwrap_or(String::from(""));
                let splits: Vec<&str> = line.split(",").collect();

                if splits.len() != 2 {
                    continue;
                }

                splits[0].parse::<usize>()
                    .map(|lid| {
                       let bound = splits[1].parse().unwrap_or(0usize);

                        h.insert(LoopId::new(lid), bound);
                    });
            }
        });

        h
    };
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct LoopId(usize);

impl LoopId {
    pub fn new(id: usize) -> Self {
        LoopId(id)
    }

    pub fn next(&self) -> Self {
        LoopId(self.0 + 1)
    }

    pub fn id(&self) -> usize {
        self.0
    }
}

impl fmt::Display for LoopId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// LoopStep is used to record or impose a bounded limit to a specific loop, which is identified by an id.
pub trait LoopStep {
    fn step(&mut self);
}

/// LoopCount tracks and report the maximum bound for this specific loop.
pub struct LoopCount<'a> {
    /// identifies a specific loop
    id: LoopId,
    /// count keeps the track of the current execution of this loop
    count: usize,
    /// reference to a static allocated AtomicUsize which keeps track of
    atomic: &'a AtomicUsize,
}

impl<'a> LoopStep for LoopCount<'a> {
    /// step is called on each iteration of a loop
    fn step(&mut self) {
        self.count = self.count.saturating_add(1);
    }
}

impl<'a> Drop for LoopCount<'a> {
    fn drop(&mut self) {
        // TODO: llogiq suggested to use atomic max as the following code is not accurate due to TOCTOU
        let current = self.atomic.load(Ordering::SeqCst);

        if self.count > current {
            self.atomic.store(self.count, Ordering::SeqCst);

            OpenOptions::new()
                .create(true)
                .append(true)
                .truncate(false)
                .open("target/mutagen/loops.txt")
                .and_then(|mut f| {
                    f.write_all(format!("{},{}\n", self.id, self.count).as_str().as_ref())
                });
        }
    }
}

impl<'a> LoopCount<'a> {
    /// Creates the LoopCounter on recording mode and it won't impose a maximum bound
    pub fn new(id: LoopId, atomic: &'a AtomicUsize) -> Self {
        LoopCount {
            id,
            count: 0,
            atomic,
        }
    }
}

/// LoopBound keeps track of the amount of iterations of a loop and it will stop when a limit is
/// reached. This limit is calculated from previous executions of the tests with mutation count == 0.
/// Then we add some base amount of iterations and we multiply by a factor to give some margin.
/// Finally, if the limit is reached, it exits the process.
pub struct LoopBound {
    /// Keeps track of the amount of iterations on the current loop
    count: usize,
    /// Bound of the loop
    bound: usize,
}

impl LoopBound {
    /// Creates a `LoopStep` with a maximum bound
    pub fn new(id: LoopId) -> Self {
        let count = MUTAGEN_LOOP_COUNT.get(&id).unwrap_or(&0);
        let bound = BoundCalculator::calculate(*count);

        LoopBound {
            count: 0,
            bound,
        }
    }
}

impl LoopStep for LoopBound {
    /// step is called on each iteration of a loop
    fn step(&mut self) {
        self.count = self.count.saturating_add(1);

        if self.bound == self.count {
            ::std::process::exit(-2);
        }
    }
}

struct BoundCalculator {}

impl BoundCalculator {
    fn calculate(count: usize) -> usize {
        let base_value = 1000;
        let factor = 2.0;

        (factor * (base_value + count) as f32) as usize
    }
}