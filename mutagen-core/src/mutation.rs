use proc_macro2::Span;
use serde::{Deserialize, Serialize};

/// description of a single mutation baked into the code with a given id
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BakedMutation {
    id: u32,
    // id of the mutator that generates this mutation
    mutator_id: u32,
    mutation: Mutation,
}

/// Mutation in source code
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Mutation {
    mutator: String, // mutator is part of code that is changed
    original_code: String,
    mutated_code: String,
    location: String,
}

impl Mutation {
    pub fn new(
        mutator: String,
        original_code: String,
        mutated_code: String,
        location: String,
    ) -> Self {
        Self {
            mutator,
            original_code,
            mutated_code,
            location,
        }
    }

    pub fn new_spanned(
        mutator: String,
        original_code: String,
        mutated_code: String,
        span: Span,
    ) -> Self {
        let start = span.start();
        let end = span.end();
        let source_file = span.unwrap().source_file().path();
        let location = format!(
            "{}@{}:{}-{}:{}",
            source_file.display(),
            start.line,
            start.column,
            end.line,
            end.column
        );

        Self::new(mutator, original_code, mutated_code, location)
    }

    pub fn with_id(self, id: u32, mutator_id: u32) -> BakedMutation {
        BakedMutation {
            id,
            mutator_id,
            mutation: self,
        }
    }
}

impl BakedMutation {
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Generate a string used for logging
    pub fn log_string(&self) -> String {
        let mutation_description = if self.mutation.mutated_code.is_empty() {
            format!("remove `{}`", &self.mutation.original_code)
        } else if self.mutation.original_code.is_empty() {
            format!("insert `{}`", &self.mutation.mutated_code)
        } else {
            format!(
                "replace `{}` with `{}`",
                &self.mutation.original_code, &self.mutation.mutated_code,
            )
        };
        format!(
            "{}: {}, {}, {}",
            &self.id, &self.mutation.mutator, mutation_description, &self.mutation.location
        )
    }
}

impl AsRef<Mutation> for BakedMutation {
    fn as_ref(&self) -> &Mutation {
        &self.mutation
    }
}
