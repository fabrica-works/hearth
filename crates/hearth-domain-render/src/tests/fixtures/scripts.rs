use super::{composition::at, declaration, kinds};
use crate::{
    kinds::scripts,
    parts::config,
    revision::{
        self,
        declaration::{Declaration, Fragment, Script},
    },
    schema,
};
pub const DEFAULT_SCRIPT_ORDER: u16 = 500;
pub const CUSTOMIZE: &[u8] = b"tasks/motd/customize.sh";
pub const CUSTOMIZE_TEMPLATE: &[u8] = b"tasks/motd/customize.sh.j2";
pub const SOFTWARE: &[u8] = b"tasks/motd/software.sh";
pub fn script_declaration(fragment: Fragment) -> Declaration {
    let mut declared = declaration(&[], &[]);
    declared.scripts = Box::new([Script { fragment, at: at(3, 5), order: DEFAULT_SCRIPT_ORDER }]);
    declared
}
pub fn script_plan(
    declared: &Declaration,
    paths: &[&[u8]],
) -> Result<Box<[scripts::Source]>, Box<[crate::errors::Error]>> {
    let mut listing = skein_lib::List::with_capacity(u32::try_from(paths.len()).expect("fixture count"));
    for path in paths {
        listing.push(kinds::source_listing(path, revision::Mode::File, 20)).expect("fixture count");
    }
    let mut faults = skein_lib::List::with_capacity(crate::limits::SHIPPED.expansion);
    let sources = scripts::plan(b"motd", declared, listing.as_slice(), &mut faults);
    if faults.is_empty() { Ok(sources) } else { Err(faults.into_boxed()) }
}
pub fn rendered_scripts(tasks: &[(&[u8], u16)], template: bool) -> config::Config {
    let mut listing = skein_lib::List::with_capacity(u32::try_from(tasks.len()).expect("fixture count"));
    let mut sources = skein_lib::List::with_capacity(u32::try_from(tasks.len()).expect("fixture count"));
    let mut plans = skein_lib::List::with_capacity(u32::try_from(tasks.len()).expect("fixture count"));
    let mut checked = skein_lib::List::with_capacity(u32::try_from(tasks.len()).expect("fixture count"));
    for &(task, order) in tasks {
        let suffix: &[u8] = if template { b"/customize.sh.j2" } else { b"/customize.sh" };
        let mut path = skein_lib::Writer::new(b"tasks/".len().saturating_add(task.len()).saturating_add(suffix.len()));
        path.put(b"tasks/").expect("room");
        path.put(task).expect("room");
        path.put(suffix).expect("room");
        let path = path.finish();
        listing.push(kinds::source_listing(&path, revision::Mode::File, 20)).expect("fixture count");
        sources.push(scripts::Source { task: task.into(), path, order }).expect("fixture count");
        plans.push(crate::kinds::files::Plan { task: task.into(), installs: Box::new([]) }).expect("fixture count");
        let roots = super::Roots::x1_prod();
        checked
            .push(schema::Checked { name: task.into(), settings: roots.settings, tasks: roots.tasks })
            .expect("fixture count");
    }
    let mut side = kinds::source_side(&declaration(&[], &[]), listing.as_slice());
    let mut prepared = super::template_prepared(plans.into_boxed(), crate::entries::Expansion { tasks: Box::new([]) });
    prepared.installation.scripts = sources.into_boxed();
    side.rendering = Some(crate::call::templates::Target::new(
        super::meta(),
        Box::new([]),
        hearth_lib::encode::Digest([1; 32]),
        schema::Validated { tasks: checked.into_boxed() },
        super::Roots::x1_prod().debian,
        prepared,
        crate::limits::SHIPPED.expansion,
    ));
    side.ready = true;
    side.files = true;
    let crate::vocabulary::Answer::Spec(spec) = super::answer(&mut side) else { panic!("script spec") };
    for part in spec.parts {
        if let crate::parts::PartRecord::Config(config) = part.record {
            return config;
        }
    }
    panic!("config part")
}

pub fn missing_script() -> Box<[crate::errors::Error]> {
    script_plan(&script_declaration(Fragment::Customize), &[]).expect_err("missing script")
}
pub const HELLO_SHA256: hearth_lib::encode::Digest = hearth_lib::encode::Digest([
    44, 242, 77, 186, 95, 176, 163, 14, 38, 232, 59, 42, 197, 185, 226, 158, 27, 22, 30, 92, 31, 167, 66, 94, 115, 4,
    51, 98, 147, 139, 152, 36,
]);
