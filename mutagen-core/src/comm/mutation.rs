use std::ops::Deref;
use std::path::{Path, PathBuf};

use proc_macro2::Span;
use serde::{Deserialize, Serialize};

use crate::transformer::TransformContext;

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
    impl_name: Option<String>,
    fn_name: Option<String>,
    mutator: String, // mutator is part of code that is changed
    original_code: String,
    mutated_code: String,
    source_file: PathBuf,
    location_in_file: String,
}

impl Mutation {
    pub fn new_spanned(
        context: &TransformContext,
        mutator: String,
        original_code: String,
        mutated_code: String,
        span: Span,
    ) -> Self {
        let impl_name = context.impl_name.clone();
        let fn_name = context.fn_name.clone();
        let start = span.start();
        let end = span.end();
        let source_file = span.unwrap().source_file().path();
        let location_in_file = format!(
            "{}:{}-{}:{}",
            start.line, start.column, end.line, end.column
        );

        Self {
            impl_name,
            fn_name,
            mutator,
            original_code,
            mutated_code,
            source_file,
            location_in_file,
        }
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

    pub fn context_description_in_brackets(&self) -> String {
        match (&self.fn_name, &self.impl_name) {
            (None, None) => format!(""),
            (Some(fn_name), None) => format!("(fn {})", fn_name),
            (None, Some(impl_name)) => format!("(impl {})", impl_name),
            (Some(fn_name), Some(impl_name)) => format!("(fn {}::{})", impl_name, fn_name),
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
    pub fn context_description_in_brackets(&self) -> String {
        self.mutation.context_description_in_brackets()
    }
}

impl AsRef<Mutation> for BakedMutation {
    fn as_ref(&self) -> &Mutation {
        &self.mutation
    }
}
