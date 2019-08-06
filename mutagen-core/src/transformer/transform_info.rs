use lazy_static::lazy_static;
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};

use crate::{mutagen_file::get_mutations_file, BakedMutation, Mutation};

lazy_static! {
    pub static ref GLOBAL_TRANSFORM_INFO: SharedTransformInfo = Default::default();
}

#[derive(Default)]
pub struct SharedTransformInfo(Arc<Mutex<MutagenTransformInfo>>);

/// Contains information about all mutations inserted into the code under test
///
/// This struct is used to collect the mutations during transformation. After running all transformers, this struct contains all mutators and their mutaitons inserted into the code
#[derive(Debug)]
pub struct MutagenTransformInfo {
    mutations: Vec<BakedMutation>,
    mutagen_file: Option<File>,
}

impl Default for MutagenTransformInfo {
    fn default() -> Self {
        Self {
            mutations: vec![],
            mutagen_file: None,
        }
    }
}

impl MutagenTransformInfo {
    pub fn with_default_mutagen_file(&mut self) {
        // open file only once
        if self.mutagen_file.is_none() {
            let mutagen_filepath = get_mutations_file().unwrap();
            let mutagen_dir = mutagen_filepath.parent().unwrap();
            if !mutagen_dir.exists() {
                create_dir_all(&mutagen_dir).unwrap();
            }
            let mutagen_file = File::create(mutagen_filepath.clone())
                .unwrap_or_else(|_| panic!("unable to open file {:?}", mutagen_filepath));

            self.mutagen_file = Some(mutagen_file);
        }
    }

    /// add a mutation and return the id used for it, also writes the mutation to the global file.
    pub fn add_mutation(&mut self, mutation: Mutation) -> u32 {
        let mut_id = 1 + self.mutations.len() as u32;
        let mutation = mutation.with_id(mut_id);

        // write the mutation if file was configured
        if let Some(mutagen_file) = &mut self.mutagen_file {
            let mut w = BufWriter::new(mutagen_file);
            serde_json::to_writer(&mut w, &mutation).expect("unable to write to mutagen file");
            // write newline
            writeln!(&mut w).expect("unable to write to mutagen file");
        }

        // add mutation to list
        self.mutations.push(mutation);

        // return next mutation id
        mut_id
    }
}

impl SharedTransformInfo {
    pub fn add_mutation(&self, mutation: Mutation) -> u32 {
        self.add_mutations(vec![mutation])
    }

    pub fn add_mutations(&self, mutations: impl IntoIterator<Item = Mutation>) -> u32 {
        let mut transform_info = self.0.lock().unwrap();

        // add all mutations within a single lock and return the first id
        let mut mutation_id = None;
        for mutation in mutations.into_iter() {
            let id = transform_info.add_mutation(mutation);
            mutation_id.get_or_insert(id);
        }
        mutation_id.expect("mutations list empty")
    }

    pub fn clone_shared(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn with_default_mutagen_file(&self) {
        self.0.lock().unwrap().with_default_mutagen_file()
    }
}
