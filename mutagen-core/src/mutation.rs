use proc_macro2::Span;
use serde::{Deserialize, Serialize};

/// description of a single mutation baked into the code with a given id
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BakedMutation {
    id: u32,
    mutation: Mutation,
}

/// Mutation in source code
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Mutation {
    mutator: String,  // mutator is part of code that is changed
    mutation: String, // description of mutation
    span_str: String,
}

impl Mutation {
    pub fn new(mutator: String, mutation: String, span_str: String) -> Self {
        Self {
            mutator,
            mutation,
            span_str,
        }
    }

    pub fn new_spanned(mutator: String, mutation: String, span: Span) -> Self {
        let start = span.start();
        let end = span.end();
        let source_file = span.unwrap().source_file().path();
        let span_str = format!(
            "{}@{}:{}-{}:{}",
            source_file.display(),
            start.line,
            start.column,
            end.line,
            end.column
        );

        Self::new(mutator, mutation, span_str)
    }

    pub fn with_id(self, id: u32) -> BakedMutation {
        BakedMutation { id, mutation: self }
    }
}

impl BakedMutation {
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn mutator(&self) -> &str {
        &self.mutation.mutator
    }
    pub fn mutation(&self) -> &str {
        &self.mutation.mutation
    }

    /// Generate a string used for logging
    pub fn log_string(&self) -> String {
        format!(
            "{}: {}, {}, {}",
            &self.id,
            &self.mutator(),
            &self.mutation(),
            &self.mutation.span_str
        )
    }
}
