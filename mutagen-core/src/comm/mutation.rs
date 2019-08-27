use std::ops::Deref;
use std::path::{Path, PathBuf};

use proc_macro2::Span;
use serde::{Deserialize, Serialize};

/// description of a single mutation baked into the code with a given id
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BakedMutation {
    id: usize,
    // id of the mutator that generates this mutation
    mutator_id: usize,
    mutation: Mutation,
}

/// Mutation in source code
#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Mutation {
    fn_name: Option<String>,
    mutator: String, // mutator is part of code that is changed
    original_code: String,
    mutated_code: String,
    source_file: PathBuf,
    location_in_file: String,
}

impl Mutation {
    pub fn new(
        fn_name: Option<String>,
        mutator: String,
        original_code: String,
        mutated_code: String,
        source_file: PathBuf,
        location_in_file: String,
    ) -> Self {
        Self {
            fn_name,
            mutator,
            original_code,
            mutated_code,
            source_file,
            location_in_file,
        }
    }

    pub fn new_spanned(
        fn_name: Option<String>,
        mutator: String,
        original_code: String,
        mutated_code: String,
        span: Span,
    ) -> Self {
        let start = span.start();
        let end = span.end();
        let source_file = span.unwrap().source_file().path();
        let location_in_file = format!(
            "{}:{}-{}:{}",
            start.line, start.column, end.line, end.column
        );

        Self::new(
            fn_name,
            mutator,
            original_code,
            mutated_code,
            source_file,
            location_in_file,
        )
    }

    pub fn with_id(self, id: usize, mutator_id: usize) -> BakedMutation {
        BakedMutation {
            id,
            mutator_id,
            mutation: self,
        }
    }

    /// construct a string representation of the mutation
    pub fn mutation_description(&self) -> String {
        if self.mutated_code.is_empty() {
            format!("remove `{}`", &self.original_code)
        } else if self.original_code.is_empty() {
            format!("insert `{}`", &self.mutated_code)
        } else {
            format!(
                "replace `{}` with `{}`",
                &self.original_code, &self.mutated_code,
            )
        }
    }
}

impl BakedMutation {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn mutator_id(&self) -> usize {
        self.mutator_id
    }

    pub fn mutator_name(&self) -> &str {
        self.mutation.mutator.deref()
    }

    pub fn fn_name(&self) -> Option<&str> {
        // TODO: use Option::deref instead
        self.mutation.fn_name.as_ref().map(String::deref)
    }

    pub fn original_code(&self) -> &str {
        self.mutation.original_code.deref()
    }

    pub fn mutated_code(&self) -> &str {
        self.mutation.mutated_code.deref()
    }

    pub fn source_file(&self) -> &Path {
        self.mutation.source_file.deref()
    }
    pub fn location_in_file(&self) -> &str {
        self.mutation.location_in_file.deref()
    }
    pub fn mutation_description(&self) -> String {
        self.mutation.mutation_description()
    }
}

impl AsRef<Mutation> for BakedMutation {
    fn as_ref(&self) -> &Mutation {
        &self.mutation
    }
}
