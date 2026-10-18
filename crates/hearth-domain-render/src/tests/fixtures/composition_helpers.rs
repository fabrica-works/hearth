use crate::compose::{Present, Settings};
pub fn task_of(settings: &Settings) -> &Present {
    let mut task = None;
    for present in &settings.tasks {
        if !present.implicit {
            assert!(task.is_none(), "one named task");
            task = Some(present);
        }
    }
    task.expect("named task")
}

pub fn host_cases() -> [(&'static [u8], crate::errors::Host); 3] {
    use crate::errors::Host;
    [
        (b"absent-lab", Host::Unknown),
        (b"rhi-prod", Host::Elsewhere { environment: Box::from(b"prod".as_slice()) }),
        (b"chat-lab", Host::Container),
    ]
}

pub fn composition_rules(documents: &crate::revision::Documents) -> Box<[crate::errors::Error]> {
    let mut errors = crate::errors::Collector::new(&crate::limits::SHIPPED);
    crate::compose::rules(documents, &mut errors);
    errors.finish().0
}
pub fn composed_settings(
    target: &crate::compose::Target,
    documents: &crate::revision::Documents,
    defaults: &crate::schema::Defaults,
) -> Settings {
    crate::compose::compose(target, documents, defaults, &mut crate::compose::Merge::new(&crate::limits::SHIPPED))
}
pub fn discover(documents: &crate::revision::Documents) -> crate::compose::Discovery {
    crate::compose::targets(
        documents,
        &crate::limits::SHIPPED,
        &mut crate::errors::Collector::new(&crate::limits::SHIPPED),
    )
    .expect("targets")
}
pub fn composition_merge() -> crate::compose::Merge {
    crate::compose::Merge::new(&crate::limits::SHIPPED)
}
pub fn composition_collector() -> crate::errors::Collector {
    crate::errors::Collector::new(&crate::limits::SHIPPED)
}

pub fn target_tasks(
    targets: &[crate::compose::Target],
    documents: &crate::revision::Documents,
) -> Box<[Box<[crate::compose::Index]>]> {
    let mut tasks = skein_lib::List::with_capacity(u32::try_from(targets.len()).expect("target count"));
    for target in targets {
        tasks.push(crate::compose::indexes(target, documents)).expect("target count");
    }
    tasks.into_boxed()
}
