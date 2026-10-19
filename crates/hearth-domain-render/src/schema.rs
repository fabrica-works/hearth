//! Declared schemas, defaults and target settings (protocol/format.md, 6).
mod debian;
pub mod forms;
pub mod references;
mod targets;
use crate::{
    errors::Schema,
    revision::{self, Documents, Position},
};
use alloc::boxed::Box;
pub use debian::debian;
use skein_lib::{List, Map, bytes};
pub use targets::{Checked, Revision, Validated, unclaimed, validate};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Fault {
    pub schema: Schema,
    pub document: u32,
    pub at: Position,
}
#[derive(Debug, Hash)]
pub struct TaskDefaults {
    layer: Composed,
    values: Box<[DefaultValue]>,
}
#[derive(Debug, Hash)]
struct DefaultValue {
    spec: Spec,
    value: Option<Composed>,
}
#[derive(Debug, Hash)]
pub struct Defaults {
    tasks: Map<Box<[u8]>, TaskDefaults>,
}
impl Defaults {
    #[must_use]
    pub fn new() -> Self {
        Self { tasks: Map::with_capacity(0) }
    }
    #[must_use]
    pub fn sound(&self, task: &[u8]) -> bool {
        self.of(task).is_some()
    }
    #[must_use]
    pub fn of(&self, task: &[u8]) -> Option<&TaskDefaults> {
        self.tasks.get(task)
    }
    #[must_use]
    pub fn layer(&self, task: &[u8]) -> Option<&Composed> {
        Some(&self.of(task)?.layer)
    }
}
impl Default for Defaults {
    fn default() -> Self {
        Self::new()
    }
}
pub fn declarations(
    documents: &Documents,
    listing: &[revision::Entry],
    validator: &mut Validator,
    merge: &mut Merge,
    faults: &mut List<(Box<[u8]>, Fault)>,
) -> Defaults {
    let capacity = u32::try_from(documents.entries().len()).expect("listing bound");
    let mut tasks = Map::with_capacity(capacity);
    for entry in documents.entries() {
        if let revision::Class::Declaration { task } = revision::class(&entry.path) {
            tasks.insert(bytes::copy_of(task), ()).expect("listing count");
        }
    }
    let mut sound = Map::with_capacity(capacity);
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
        let mut references = List::with_capacity(16);
        references::declaration(declaration, document, &mut references);
        crate::exports::check(declaration, document, &mut references);
        for fault in &references {
            let mut fault = fault.clone();
            if let Schema::Unlisted { reader, root: _, task: _, setting: _ } = &mut fault.schema {
                reader.clone_from(task);
            }
            push(faults, task, fault);
        }
        if before == faults.len() {
            let mut local = List::with_capacity(16);
            let defaults = task_defaults(declaration, document, listing, validator, merge, &mut local);
            for fault in &local {
                push(faults, task, fault.clone());
            }
            if local.is_empty() {
                sound.insert(task.clone(), defaults).expect("task count");
            }
        }
    }
    Defaults { tasks: sound }
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

fn declares(declaration: &Declaration, setting: &[u8]) -> bool {
    for spec in &declaration.settings {
        if spec.name.text.as_ref() == setting {
            return true;
        }
    }
    false
}

use crate::{
    compose::{Composed, Composite, Layer, Merge, Provenance},
    evaluate::{Entry as EvaluatedEntry, Value as Evaluated},
    limits::Limits,
    revision::declaration::{Declaration, Form, Spec},
};
use skein_lib::Stack;

#[derive(Debug)]
pub enum Check<'a> {
    Default,
    Patch,
    Target { environment: &'a [u8], targets: &'a [Box<[u8]>] },
}
#[derive(Debug)]
pub struct Against<'a> {
    pub declaration: &'a Declaration,
    pub defaults: &'a TaskDefaults,
    pub listing: &'a [revision::Entry],
}
#[derive(Debug)]
enum Wanted {
    Type(u32),
    Spec(Spec),
    Settings(Box<[Spec]>),
}
#[derive(Debug, Hash)]
enum Frame {
    List { items: Box<[Composed]>, next: u32, ty: u32, made: List<Evaluated> },
    Map(MapFrame),
}
#[derive(Debug, Hash)]
struct MapFrame {
    from: Provenance,
    root: bool,
    pairs: Box<[crate::compose::Pair]>,
    next: u32,
    fields: Option<Box<[Spec]>>,
    field: u32,
    ty: u32,
    key: Option<Box<[u8]>>,
    made: List<EvaluatedEntry>,
}
enum Advance {
    More,
    Next(Composed, Wanted),
    Done,
}
#[derive(Debug, Hash)]
pub struct Validator {
    stack: Stack<Frame>,
    steps: u64,
    left: u64,
    limited: bool,
    static_file: u64,
}
impl Validator {
    #[must_use]
    pub fn new(limits: &Limits) -> Self {
        Self {
            stack: Stack::with_capacity(limits.depth.saturating_add(1)),
            left: u64::MAX,
            limited: false,
            static_file: u64::from(limits.static_file),
            steps: limits.decoded.saturating_mul(u64::from(limits.document)).saturating_mul(4).saturating_add(4),
        }
    }
    #[must_use]
    pub fn limited(&self) -> bool {
        self.limited
    }
    pub fn settings(
        &mut self,
        settings: &Composed,
        against: &Against<'_>,
        check: &Check<'_>,
        left: &mut u64,
        faults: &mut List<Fault>,
    ) -> Option<Evaluated> {
        self.left = *left;
        self.limited = false;
        let value =
            self.walk(settings.clone(), Wanted::Settings(against.declaration.settings.clone()), against, check, faults);
        *left = self.left;
        value
    }
    fn walk(
        &mut self,
        value: Composed,
        wanted: Wanted,
        against: &Against<'_>,
        check: &Check<'_>,
        faults: &mut List<Fault>,
    ) -> Option<Evaluated> {
        assert!(self.stack.is_empty(), "previous validation completed");
        let mut pending = Some((value, wanted));
        let mut finished = None;
        let mut valid = true;
        for _ in 0..self.steps {
            if let Some((value, wanted)) = pending.take() {
                finished = self.enter(value, wanted, against, check, faults, &mut valid);
                if self.limited {
                    for _ in 0..self.stack.len() {
                        drop(self.stack.pop());
                    }
                    return None;
                }
            }
            let Some(mut frame) = self.stack.pop() else {
                return if valid { finished } else { None };
            };
            match frame.advance(finished.take(), against, check, faults, &mut valid) {
                Advance::Done => {
                    finished = Some(frame.finish());
                    continue;
                }
                Advance::More => {}
                Advance::Next(value, wanted) => pending = Some((value, wanted)),
            }
            self.stack.push(frame).expect("popped frame fits");
        }
        unreachable!("decoded values and schemas bound validation")
    }
    fn enter(
        &mut self,
        value: Composed,
        wanted: Wanted,
        against: &Against<'_>,
        check: &Check<'_>,
        faults: &mut List<Fault>,
        valid: &mut bool,
    ) -> Option<Evaluated> {
        let declaration = against.declaration;
        let root = match &wanted {
            Wanted::Settings(_) => true,
            Wanted::Type(_) | Wanted::Spec(_) => false,
        };
        let (form, spec) = match wanted {
            Wanted::Type(ty) => (
                declaration.types.get(usize::try_from(ty).expect("type index")).expect("declared type").form.clone(),
                None,
            ),
            Wanted::Spec(spec) => (
                declaration
                    .types
                    .get(usize::try_from(spec.ty).expect("type index"))
                    .expect("declared type")
                    .form
                    .clone(),
                Some(spec),
            ),
            Wanted::Settings(fields) => (Form::Object(fields), None),
        };
        if let Some(schema) = type_fault(&value.kind, &form, spec.as_ref()) {
            fault(faults, value.from, schema);
            *valid = false;
            return None;
        }
        if let Some(schema) = forms::fault(&value, &form, check, against.listing, self.static_file) {
            fault(faults, value.from, schema);
            *valid = false;
            return None;
        }
        let charge = storage(&value.kind, &form);
        match self.left.checked_sub(charge) {
            Some(left) => self.left = left,
            None => {
                self.limited = true;
                return None;
            }
        }
        match value.kind {
            Composite::Bool(value) => Some(Evaluated::Bool(value)),
            Composite::Int(value) => Some(Evaluated::Int(value)),
            Composite::String(value) => Some(if let Form::Duration = form {
                Evaluated::Int(forms::seconds(&value).expect("checked duration"))
            } else {
                Evaluated::String(value)
            }),
            Composite::List(items) => {
                let Form::List(ty) = form else { unreachable!("list type checked") };
                let made = List::with_capacity(u32::try_from(items.len()).expect("bounded list"));
                self.stack.push(Frame::List { items, next: 0, ty, made }).expect("decoded depth");
                None
            }
            Composite::Map(pairs) => {
                let (fields, ty, length) = match form {
                    Form::Map(ty) => (None, ty, pairs.len()),
                    Form::Object(fields) => {
                        let length = fields.len();
                        (Some(fields), 0, length)
                    }
                    Form::Bool
                    | Form::Int
                    | Form::String
                    | Form::Path
                    | Form::Mode
                    | Form::Duration
                    | Form::Secret
                    | Form::Target
                    | Form::File
                    | Form::Enum(_)
                    | Form::List(_) => unreachable!("map type checked"),
                };
                let made = List::with_capacity(u32::try_from(length).expect("bounded map"));
                self.stack
                    .push(Frame::Map(MapFrame {
                        from: value.from,
                        root,
                        pairs,
                        next: 0,
                        fields,
                        field: 0,
                        ty,
                        key: None,
                        made,
                    }))
                    .expect("decoded depth");
                None
            }
        }
    }
}
impl Frame {
    fn advance(
        &mut self,
        value: Option<Evaluated>,
        against: &Against<'_>,
        check: &Check<'_>,
        faults: &mut List<Fault>,
        valid: &mut bool,
    ) -> Advance {
        match self {
            Self::List { items, next, ty, made } => {
                if let Some(value) = value {
                    made.push(value).expect("element count");
                }
                if let Some(item) = items.get(usize::try_from(*next).expect("bounded cursor")) {
                    *next = next.saturating_add(1);
                    Advance::Next(item.clone(), Wanted::Type(*ty))
                } else {
                    Advance::Done
                }
            }
            Self::Map(frame) => frame.advance(value, against, check, faults, valid),
        }
    }
    fn finish(self) -> Evaluated {
        match self {
            Self::List { items: _, next: _, ty: _, made } => Evaluated::List(made.into_boxed()),
            Self::Map(MapFrame { from: _, root: _, pairs: _, next: _, fields: _, field: _, ty: _, key: _, made }) => {
                Evaluated::Map(made.into_boxed())
            }
        }
    }
}
impl MapFrame {
    fn advance(
        &mut self,
        value: Option<Evaluated>,
        against: &Against<'_>,
        check: &Check<'_>,
        faults: &mut List<Fault>,
        valid: &mut bool,
    ) -> Advance {
        let Self { from, root, pairs, next, fields, field, ty, key, made } = self;
        if let Some(key) = key.take() {
            made.push(EvaluatedEntry { key, value }).expect("field count");
        }
        let pair = pairs.get(usize::try_from(*next).expect("bounded cursor"));
        let spec = match fields {
            Some(fields) => fields.get(usize::try_from(*field).expect("bounded cursor")),
            None => None,
        };
        match (pair, spec) {
            (Some(pair), Some(spec)) if pair.key.as_ref() < spec.name.text.as_ref() => {
                unknown(faults, pair, fields.as_deref().expect("object fields"), *root);
                *valid = false;
                *next = next.saturating_add(1);
            }
            (Some(pair), Some(spec)) if pair.key == spec.name.text => {
                *key = Some(pair.key.clone());
                *next = next.saturating_add(1);
                *field = field.saturating_add(1);
                return Advance::Next(pair.value.clone(), Wanted::Spec(spec.clone()));
            }
            (Some(_) | None, Some(spec)) => {
                *field = field.saturating_add(1);
                let fill = match check {
                    Check::Default | Check::Target { environment: _, targets: _ } => true,
                    Check::Patch => false,
                };
                if fill {
                    *key = Some(spec.name.text.clone());
                    match default_value(against.defaults, spec) {
                        Some(default) => match &default.value {
                            Some(value) => return Advance::Next(value.clone(), Wanted::Spec(spec.clone())),
                            None => *valid = false,
                        },
                        None if spec.optional => {}
                        None => {
                            let schema = if *root {
                                Schema::Missing { setting: spec.name.text.clone() }
                            } else {
                                Schema::MissingField { field: spec.name.text.clone() }
                            };
                            fault(faults, *from, schema);
                            *valid = false;
                        }
                    }
                }
            }
            (Some(pair), None) => {
                *next = next.saturating_add(1);
                if let Some(fields) = fields {
                    unknown(faults, pair, fields, *root);
                    *valid = false;
                } else if forms::check(crate::errors::Format::Key, &pair.key) {
                    *key = Some(pair.key.clone());
                    return Advance::Next(pair.value.clone(), Wanted::Type(*ty));
                } else {
                    fault(faults, pair.from, Schema::Malformed { format: crate::errors::Format::Key });
                    *valid = false;
                }
            }
            (None, None) => return Advance::Done,
        }
        Advance::More
    }
}
fn unknown(faults: &mut List<Fault>, pair: &crate::compose::Pair, fields: &[Spec], root: bool) {
    let declared = field_names(fields);
    let schema = if root {
        Schema::Undeclared { setting: pair.key.clone(), declared }
    } else {
        Schema::UndeclaredField { field: pair.key.clone(), declared }
    };
    fault(faults, pair.from, schema);
}
fn default_value<'a>(defaults: &'a TaskDefaults, spec: &Spec) -> Option<&'a DefaultValue> {
    for (index, value) in defaults.values.iter().enumerate() {
        if value.spec.at == spec.at {
            return defaults.values.get(index);
        }
    }
    None
}
pub(crate) fn fault(faults: &mut List<Fault>, from: Provenance, schema: Schema) {
    if faults.room() == 0 {
        let mut larger = List::with_capacity(faults.capacity().max(1).saturating_mul(2));
        for fault in &*faults {
            larger.push(fault.clone()).expect("doubled capacity");
        }
        *faults = larger;
    }
    faults.push(Fault { schema, document: from.document, at: from.at }).expect("fault capacity");
}
fn field_names(fields: &[Spec]) -> Box<[Box<[u8]>]> {
    let mut names = List::with_capacity(u32::try_from(fields.len()).expect("field count"));
    for field in fields {
        names.push(field.name.text.clone()).expect("field count");
    }
    names.into_boxed()
}
pub(crate) fn type_fault(kind: &Composite, form: &Form, spec: Option<&Spec>) -> Option<Schema> {
    use crate::errors::TypeName;
    let kind_tag: u8 = match kind {
        Composite::Bool(_) => 0,
        Composite::Int(_) => 1,
        Composite::String(_) => 2,
        Composite::List(_) => 3,
        Composite::Map(_) => 4,
    };
    let (expected, required_kind): (_, u8) = match form {
        Form::Bool => (TypeName::Bool, 0),
        Form::Int => (TypeName::Int, 1),
        Form::String => (TypeName::String, 2),
        Form::Path => (TypeName::Path, 2),
        Form::Mode => (TypeName::Mode, 2),
        Form::Duration => (TypeName::Duration, 2),
        Form::Secret => (TypeName::Secret, 2),
        Form::Target => (TypeName::Target, 2),
        Form::File => (TypeName::File, 2),
        Form::Enum(_) => (TypeName::Enum, 2),
        Form::List(_) => (TypeName::List, 3),
        Form::Map(_) => (TypeName::Map, 4),
        Form::Object(_) => (TypeName::Object, 4),
    };
    if kind_tag != required_kind {
        return Some(Schema::Wrong { expected });
    }
    if let (Composite::String(value), Form::Enum(listed)) = (kind, form) {
        let mut found = false;
        let mut names = List::with_capacity(u32::try_from(listed.len()).expect("enum count"));
        for name in listed {
            found |= name.text == *value;
            names.push(name.text.clone()).expect("enum count");
        }
        if !found {
            return Some(Schema::Enum { value: value.clone(), listed: names.into_boxed() });
        }
    }
    let spec = spec?;
    if let Composite::Int(value) = kind {
        if let Some(min) = &spec.min
            && *value < min.value
        {
            return Some(Schema::Below { min: min.value });
        }
        if let Some(max) = &spec.max
            && *value > max.value
        {
            return Some(Schema::Above { max: max.value });
        }
    }
    let length = match kind {
        Composite::String(value) => value.len(),
        Composite::List(value) => value.len(),
        Composite::Map(value) => value.len(),
        Composite::Bool(_) | Composite::Int(_) => 0,
    };
    if let Some(max) = &spec.max_length
        && u64::try_from(length).expect("bounded length") > max.value.unsigned_abs()
    {
        return Some(Schema::Length { max_length: max.value });
    }
    None
}
fn task_defaults(
    declaration: &Declaration,
    document: u32,
    listing: &[revision::Entry],
    validator: &mut Validator,
    merge: &mut Merge,
    faults: &mut List<Fault>,
) -> TaskDefaults {
    validator.left = u64::MAX;
    validator.limited = false;
    let mut values = List::with_capacity(u32::try_from(declaration.settings.len()).expect("setting count"));
    for spec in &declaration.settings {
        add_default(&mut values, spec, document, merge, validator.stack.capacity(), faults);
    }
    for ty in &declaration.types {
        if let Form::Object(fields) = &ty.form {
            for spec in fields {
                add_default(&mut values, spec, document, merge, validator.stack.capacity(), faults);
            }
        }
    }
    let mut pairs = List::with_capacity(u32::try_from(declaration.settings.len()).expect("setting count"));
    for spec in &declaration.settings {
        if let Some(value) = &spec.default {
            pairs
                .push(revision::Pair { key: spec.name.text.clone(), at: spec.name.at, value: value.clone() })
                .expect("setting count");
        }
    }
    let layer = merge
        .merge(
            None,
            &revision::Value { at: declaration.at, kind: revision::Kind::Map(pairs.into_boxed()) },
            Layer::Defaults,
            document,
        )
        .expect("default map");
    let mut defaults = TaskDefaults { layer, values: values.into_boxed() };
    for index in (0..defaults.values.len()).rev() {
        let value = defaults.values.get(index).expect("counted");
        if let Some(composed) = &value.value {
            let against = Against { declaration, defaults: &defaults, listing };
            if validator
                .walk(composed.clone(), Wanted::Spec(value.spec.clone()), &against, &Check::Default, faults)
                .is_none()
            {
                defaults.values.get_mut(index).expect("counted").value = None;
            }
        }
    }
    defaults
}
fn add_default(
    values: &mut List<DefaultValue>,
    spec: &Spec,
    document: u32,
    merge: &mut Merge,
    depth: u32,
    faults: &mut List<Fault>,
) {
    let Some(value) = &spec.default else {
        return;
    };
    let before = faults.len();
    nulls(value, document, depth, faults);
    let value = if before == faults.len() { merge.merge(None, value, Layer::Defaults, document) } else { None };
    if values.room() == 0 {
        let mut larger = List::with_capacity(values.capacity().max(1).saturating_mul(2));
        for value in &*values {
            larger
                .push(DefaultValue { spec: value.spec.clone(), value: value.value.clone() })
                .expect("doubled capacity");
        }
        *values = larger;
    }
    values.push(DefaultValue { spec: spec.clone(), value }).expect("default capacity");
}

#[derive(Debug)]
enum NullFrame<'a> {
    List(core::slice::Iter<'a, revision::Value>),
    Map(core::slice::Iter<'a, revision::Pair>),
}
fn nulls(value: &revision::Value, document: u32, depth: u32, faults: &mut List<Fault>) {
    let mut stack = Stack::with_capacity(depth);
    let mut pending = Some(value);
    for _ in 0..u64::from(crate::limits::SHIPPED.document).saturating_mul(3).saturating_add(3) {
        if let Some(value) = pending.take() {
            match &value.kind {
                revision::Kind::Null => {
                    fault(faults, Provenance { layer: Layer::Defaults, document, at: value.at }, Schema::Null);
                }
                revision::Kind::List(items) => stack.push(NullFrame::List(items.iter())).expect("decoded depth"),
                revision::Kind::Map(pairs) => stack.push(NullFrame::Map(pairs.iter())).expect("decoded depth"),
                revision::Kind::Bool(_) | revision::Kind::Int(_) | revision::Kind::String(_) => {}
            }
        }
        let Some(mut frame) = stack.pop() else {
            return;
        };
        pending = match &mut frame {
            NullFrame::List(items) => items.next(),
            NullFrame::Map(pairs) => match pairs.next() {
                Some(pair) => Some(&pair.value),
                None => None,
            },
        };
        if pending.is_some() {
            stack.push(frame).expect("popped frame fits");
        }
    }
    unreachable!("decoded document bounds the null scan")
}

fn entry_size(key: &[u8]) -> u64 {
    crate::evaluate::ELEMENT.saturating_add(u64::try_from(key.len()).expect("bounded key"))
}
fn storage(value: &Composite, form: &Form) -> u64 {
    match (value, form) {
        (Composite::String(bytes), _) => u64::try_from(bytes.len()).expect("bounded bytes"),
        (Composite::List(items), _) => {
            u64::try_from(items.len()).expect("bounded items").saturating_mul(crate::evaluate::ELEMENT)
        }
        (Composite::Map(pairs), Form::Map(_)) => {
            let mut size = 0_u64;
            for pair in pairs {
                size = size.saturating_add(entry_size(&pair.key));
            }
            size
        }
        (Composite::Map(_), Form::Object(fields)) => {
            let mut size = 0_u64;
            for spec in fields {
                size = size.saturating_add(entry_size(&spec.name.text));
            }
            size
        }
        (Composite::Bool(_) | Composite::Int(_) | Composite::Map(_), _) => 0,
    }
}
