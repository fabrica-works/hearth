//! Revision checks of declared reads (format.md, 6.4 and 6.5).
use crate::{
    errors::Schema,
    revision::{self, Documents, Position},
};
use alloc::boxed::Box;
use skein_lib::{List, Map, bytes};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Fault {
    pub schema: Schema,
    pub document: u32,
    pub at: Position,
}
#[derive(Debug)]
pub struct Defaults {
    sound: Box<[Box<[u8]>]>,
}
impl Defaults {
    #[must_use]
    pub fn new() -> Self {
        Self { sound: Box::new([]) }
    }
    #[must_use]
    pub fn sound(&self, task: &[u8]) -> bool {
        self.sound.binary_search(&bytes::copy_of(task)).is_ok()
    }
}
impl Default for Defaults {
    fn default() -> Self {
        Self::new()
    }
}
pub fn declarations(documents: &Documents, faults: &mut List<(Box<[u8]>, Fault)>) -> Defaults {
    let capacity = u32::try_from(documents.entries().len()).expect("listing bound");
    let mut tasks = Map::with_capacity(capacity);
    for entry in documents.entries() {
        if let revision::Class::Declaration { task } = revision::class(&entry.path) {
            tasks.insert(bytes::copy_of(task), ()).expect("listing count");
        }
    }
    let mut sound = List::with_capacity(capacity);
    for (task, ()) in &tasks {
        let Some((document, Some(declaration))) = documents.declaration(task) else {
            continue;
        };
        let before = faults.len();
        for read in &declaration.reads {
            match documents.declaration(&read.task.text) {
                None => push(
                    faults,
                    task,
                    Fault { schema: Schema::Task { task: read.task.text.clone() }, document, at: read.task.at },
                ),
                Some((_, None)) => {}
                Some((_, Some(other))) => {
                    for setting in &read.settings {
                        if !declares(other, &setting.text) {
                            let mut declared =
                                List::with_capacity(u32::try_from(other.settings.len()).expect("setting bound"));
                            for spec in &other.settings {
                                declared.push(spec.name.text.clone()).expect("setting count");
                            }
                            push(
                                faults,
                                task,
                                Fault {
                                    schema: Schema::Unread {
                                        task: read.task.text.clone(),
                                        setting: setting.text.clone(),
                                        declared: declared.into_boxed(),
                                    },
                                    document,
                                    at: setting.at,
                                },
                            );
                        }
                    }
                }
            }
        }
        if before == faults.len() {
            sound.push(bytes::copy_of(task)).expect("listing count");
        }
    }
    Defaults { sound: sound.into_boxed() }
}
fn push(faults: &mut List<(Box<[u8]>, Fault)>, task: &[u8], fault: Fault) {
    if faults.room() == 0 {
        let mut larger = List::with_capacity(faults.capacity().max(1).saturating_mul(2));
        for fault in &*faults {
            larger.push(fault.clone()).expect("doubled capacity");
        }
        *faults = larger;
    }
    faults.push((bytes::copy_of(task), fault)).expect("reserved fault capacity");
}

fn declares(declaration: &revision::declaration::Declaration, setting: &[u8]) -> bool {
    for spec in &declaration.settings {
        if spec.name.text.as_ref() == setting {
            return true;
        }
    }
    false
}
