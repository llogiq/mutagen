use lazy_static::lazy_static;
use std::fs::{create_dir_all, File};
use std::iter;
use std::sync::{Arc, Mutex, MutexGuard};

use super::mutate_args::LocalConf;
use crate::comm;
use crate::comm::{BakedMutation, Mutation};

lazy_static! {
    static ref GLOBAL_TRANSFORM_INFO: SharedTransformInfo = Default::default();
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
    expected_mutations: Option<u32>,
}

impl Default for MutagenTransformInfo {
    fn default() -> Self {
        Self {
            mutations: vec![],
            mutagen_file: None,
            expected_mutations: None,
        }
    }
}

impl MutagenTransformInfo {
    pub fn with_default_mutagen_file(&mut self) {
        // open file only once
        if self.mutagen_file.is_none() {
            let mutagen_filepath = comm::get_mutations_file().unwrap();
            let mutagen_dir = mutagen_filepath.parent().unwrap();
            if !mutagen_dir.exists() {
                create_dir_all(&mutagen_dir).unwrap();
            }
            let mutagen_file = File::create(&mutagen_filepath)
                .unwrap_or_else(|_| panic!("unable to open file {:?}", &mutagen_filepath));

            self.mutagen_file = Some(mutagen_file);
        }
    }

    /// add a mutation and return the id used for it, also writes the mutation to the global file.
    pub fn add_mutation(&mut self, mutation: Mutation, mutator_id: u32) -> u32 {
        let mut_id = 1 + self.mutations.len() as u32;
        let mutation = mutation.with_id(mut_id, mutator_id);

        // write the mutation if file was configured
        if let Some(mutagen_file) = &mut self.mutagen_file {
            comm::append_item(mutagen_file, &mutation).expect("unable to write to mutagen file");
        }

        // add mutation to list
        self.mutations.push(mutation);

        // return next mutation id
        mut_id
    }

    pub fn get_num_mutations(&self) -> u32 {
        self.mutations.len() as u32
    }

    pub fn get_next_mutation_id(&self) -> u32 {
        self.mutations.len() as u32 + 1
    }

    pub fn check_mutations(&mut self) {
        if let Some(expected_mutations) = self.expected_mutations {
            let actual_mutations = self.mutations.len() as u32;
            if expected_mutations != actual_mutations {
                panic!(
                    "expected {} mutations but inserted {}",
                    expected_mutations, actual_mutations
                );
            }
        }
    }
}

impl SharedTransformInfo {
    fn lock_tranform_info(&self) -> MutexGuard<MutagenTransformInfo> {
        self.0.lock().unwrap()
    }

    fn new(transform_info: MutagenTransformInfo) -> SharedTransformInfo {
        SharedTransformInfo(Arc::new(Mutex::new(transform_info)))
    }

    pub fn global_info() -> Self {
        GLOBAL_TRANSFORM_INFO
            .lock_tranform_info()
            .with_default_mutagen_file();
        GLOBAL_TRANSFORM_INFO.clone_shared()
    }

    pub fn local_info(conf: LocalConf) -> Self {
        let mut transform_info = MutagenTransformInfo::default();
        if let Some(n) = conf.expected_mutations {
            transform_info.expected_mutations = Some(n);
        }
        Self::new(transform_info)
    }

    pub fn add_mutation(&self, mutation: Mutation) -> u32 {
        self.add_mutations(iter::once(mutation))
    }

    pub fn add_mutations(&self, mutations: impl IntoIterator<Item = Mutation>) -> u32 {
        let mut transform_info = self.lock_tranform_info();

        let mutator_id = transform_info.get_next_mutation_id();

        // add all mutations within a single lock and return the first id
        for mutation in mutations.into_iter() {
            transform_info.add_mutation(mutation, mutator_id);
        }
        mutator_id
    }

    pub fn clone_shared(&self) -> Self {
        Self(Arc::clone(&self.0))
    }

    pub fn get_num_mutations(&self) -> u32 {
        self.lock_tranform_info().get_num_mutations()
    }

    pub fn check_mutations(&self) {
        self.lock_tranform_info().check_mutations()
    }
}
