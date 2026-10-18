//! The independent engine adapter and the differences listed by render.md, 9.5.
use hearth_domain_render::{
    evaluate::{self, Operand, Value as HearthValue},
    limits,
};
use hearth_render_world::{expressions::Roots, fixtures::oracle as inputs};
use minijinja::{AutoEscape, Environment, Error, ErrorKind, UndefinedBehavior, Value, value::ValueKind};
use std::{borrow::Cow, fmt};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    Text(Cow<'static, str>),
    Fails,
}
pub struct Difference {
    pub id: &'static str,
    pub template: &'static str,
    pub hearth: Outcome,
    pub oracle: Outcome,
    pub reason: &'static str,
}
pub const DIFFERENCES: &[Difference] = &[
    Difference {
        id: "string sort",
        template: "{{ ['a','B'] | sort | join(',') }}",
        hearth: Outcome::Text(Cow::Borrowed("B,a")),
        oracle: Outcome::Text(Cow::Borrowed("a,B")),
        reason: "Hearth sorts bytes; the oracle folds case.",
    },
    Difference {
        id: "string addition",
        template: "{{ 'a' + 'b' }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("ab")),
        reason: "Addition takes integers in format.md 7.4.",
    },
    Difference {
        id: "boolean concatenation",
        template: "{{ true ~ 'x' }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("truex")),
        reason: "Concatenation takes strings or integers.",
    },
    Difference {
        id: "boolean equality",
        template: "{{ 1 == true }}",
        hearth: Outcome::Text(Cow::Borrowed("false")),
        oracle: Outcome::Text(Cow::Borrowed("true")),
        reason: "Equality keeps the types distinct.",
    },
    Difference {
        id: "empty first",
        template: "{{ [] | first | default('empty') }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("empty")),
        reason: "An empty list gives Empty, never undefined.",
    },
    Difference {
        id: "empty last",
        template: "{{ [] | last | default('empty') }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("empty")),
        reason: "An empty list gives Empty, never undefined.",
    },
    Difference {
        id: "string first",
        template: "{{ 'ab' | first }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("a")),
        reason: "First takes a list.",
    },
    Difference {
        id: "string last",
        template: "{{ 'ab' | last }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("b")),
        reason: "Last takes a list.",
    },
    Difference {
        id: "JSON HTML characters",
        template: "{{ \"<>&'\" | tojson }}",
        hearth: Outcome::Text(Cow::Borrowed("\"<>&'\"")),
        oracle: Outcome::Text(Cow::Borrowed("\"\\u003c\\u003e\\u0026\\u0027\"")),
        reason: "Format.md 7.5 preserves these bytes in JSON.",
    },
    Difference {
        id: "concatenation precedence",
        template: "{{ 'v' ~ 1 + 1 }}",
        hearth: Outcome::Text(Cow::Borrowed("v2")),
        oracle: Outcome::Fails,
        reason: "Format.md 7.4 puts concatenation below addition.",
    },
    Difference {
        id: "defined precedence",
        template: "{{ settings.a == settings.b is defined }}",
        hearth: Outcome::Text(Cow::Borrowed("true")),
        oracle: Outcome::Text(Cow::Borrowed("false")),
        reason: "The defined test follows a comparison in 7.4.",
    },
    Difference {
        id: "missing chain defined",
        template: "{{ settings.absent.child is defined }}",
        hearth: Outcome::Text(Cow::Borrowed("false")),
        oracle: Outcome::Fails,
        reason: "A chain stops at its first missing link in 7.6.",
    },
    Difference {
        id: "missing chain default",
        template: "{{ settings.absent.child | default('empty') }}",
        hearth: Outcome::Text(Cow::Borrowed("empty")),
        oracle: Outcome::Fails,
        reason: "A chain stops at its first missing link in 7.6.",
    },
    Difference {
        id: "optional if",
        template: "{% if settings.absent %}yes{% else %}no{% endif %}",
        hearth: Outcome::Text(Cow::Borrowed("no")),
        oracle: Outcome::Fails,
        reason: "An absent optional is false as a condition.",
    },
    Difference {
        id: "optional elif",
        template: "{% if false %}yes{% elif settings.absent %}yes{% else %}no{% endif %}",
        hearth: Outcome::Text(Cow::Borrowed("no")),
        oracle: Outcome::Fails,
        reason: "An absent optional is false as a condition.",
    },
    Difference {
        id: "optional not",
        template: "{{ not settings.absent }}",
        hearth: Outcome::Text(Cow::Borrowed("true")),
        oracle: Outcome::Fails,
        reason: "An absent optional is false as a condition.",
    },
    Difference {
        id: "optional and",
        template: "{% if settings.absent and true %}yes{% else %}no{% endif %}",
        hearth: Outcome::Text(Cow::Borrowed("no")),
        oracle: Outcome::Fails,
        reason: "An absent optional is false as a condition.",
    },
    Difference {
        id: "optional or",
        template: "{{ settings.absent or 'empty' }}",
        hearth: Outcome::Text(Cow::Borrowed("empty")),
        oracle: Outcome::Fails,
        reason: "An absent optional is false as a condition.",
    },
    Difference {
        id: "optional conditional",
        template: "{{ 'yes' if settings.absent else 'no' }}",
        hearth: Outcome::Text(Cow::Borrowed("no")),
        oracle: Outcome::Fails,
        reason: "An absent optional is false as a condition.",
    },
    Difference {
        id: "addition overflow",
        template: "{{ 9223372036854775807 + 1 }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("9223372036854775808")),
        reason: "Arithmetic is checked at i64 in 7.4.",
    },
    Difference {
        id: "subtraction overflow",
        template: "{{ -9223372036854775808 - 1 }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("-9223372036854775809")),
        reason: "Arithmetic is checked at i64 in 7.4.",
    },
    Difference {
        id: "list output",
        template: "{{ [1,2] }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("[1, 2]")),
        reason: "Output requires a scalar in 7.4.",
    },
    Difference {
        id: "negative index",
        template: "{{ [1,2][-1] }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("2")),
        reason: "Negative indices are outside a list in 7.4.",
    },
    Difference {
        id: "list key",
        template: "{{ [1,2]['key'] | default('empty') }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("empty")),
        reason: "A key into a list is Type in 7.4.",
    },
    Difference {
        id: "map index",
        template: "{{ settings[1] | default('empty') }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("empty")),
        reason: "An integer into a map is Type in 7.4.",
    },
    Difference {
        id: "empty split",
        template: "{{ 'a' | split('') | join('|') }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("|a|")),
        reason: "A split separator is nonempty in 7.5.",
    },
    Difference {
        id: "empty replace",
        template: "{{ 'a' | replace('', '-') }}",
        hearth: Outcome::Fails,
        oracle: Outcome::Text(Cow::Borrowed("-a-")),
        reason: "A replacement needle is nonempty in 7.5.",
    },
];
#[derive(Debug)]
pub struct Mismatch {
    pub template: String,
    pub roots: Box<Roots>,
    pub hearth: Outcome,
    pub oracle: Outcome,
}
impl fmt::Display for Mismatch {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            out,
            "template {:?}, roots {:?}: hearth {:?}, oracle {:?}",
            self.template, self.roots, self.hearth, self.oracle
        )
    }
}
pub struct Oracle {
    environment: Environment<'static>,
}
impl Default for Oracle {
    fn default() -> Self {
        Self::new()
    }
}
impl Oracle {
    #[must_use]
    pub fn new() -> Self {
        let mut environment = Environment::new();
        environment.set_trim_blocks(true);
        environment.set_lstrip_blocks(true);
        environment.set_keep_trailing_newline(true);
        environment.set_undefined_behavior(UndefinedBehavior::Strict);
        environment.set_auto_escape_callback(|_| AutoEscape::None);
        environment.add_filter("keys", |value: Value| members(&value, Projection::Keys));
        environment.add_filter("values", |value: Value| members(&value, Projection::Values));
        environment.add_filter("shell_quote", |value: Value| -> Result<String, Error> {
            Ok(format!("'{}'", as_text(&value)?.replace('\'', "'\\''")))
        });
        environment
            .add_filter("systemd_escape", |value: Value| -> Result<String, Error> { Ok(escape(as_text(&value)?)) });
        environment.add_filter("systemd_escape_path", |value: Value| path(as_text(&value)?, PathFilter::Escape));
        environment.add_filter("basename", |value: Value| path(as_text(&value)?, PathFilter::Basename));
        environment.add_filter("dirname", |value: Value| path(as_text(&value)?, PathFilter::Dirname));
        Self { environment }
    }
    #[must_use]
    pub fn outcomes(&self, template: &str, roots: &Roots) -> (Outcome, Outcome) {
        let hearth = hearth(template, roots);
        let oracle = outcome(self.environment.render_str(template, context_of(roots)).ok());
        (hearth, oracle)
    }
    /// # Errors
    /// Names both results and the full inputs when the engines disagree.
    pub fn compare(&self, template: &str, roots: &Roots) -> Result<(), Mismatch> {
        let (hearth, oracle) = self.outcomes(template, roots);
        if hearth == oracle {
            Ok(())
        } else {
            Err(Mismatch { template: template.to_owned(), roots: Box::new(roots.clone()), hearth, oracle })
        }
    }
    /// # Errors
    /// Names a row whose pinned outcomes have changed.
    pub fn check_differences(&self) -> Result<(), String> {
        let roots = inputs::roots();
        for row in DIFFERENCES {
            let results = self.outcomes(row.template, &roots);
            if results != (row.hearth.clone(), row.oracle.clone()) || row.hearth == row.oracle || row.reason.is_empty()
            {
                return Err(format!(
                    "{}: {:?}, expected {:?}; {}",
                    row.id,
                    results,
                    (&row.hearth, &row.oracle),
                    row.reason
                ));
            }
        }
        Ok(())
    }
}
fn outcome(text: Option<String>) -> Outcome {
    text.map_or(Outcome::Fails, |text| Outcome::Text(Cow::Owned(text)))
}
fn hearth(source: &str, roots: &Roots) -> Outcome {
    let Ok(compiled) =
        hearth_protocol_template::template::compile(source.as_bytes(), &hearth_protocol_template::limits::SHIPPED)
    else {
        return Outcome::Fails;
    };
    let template = hearth_protocol_builder::translate::instructions::template(compiled);
    let binding = |name| roots.bindings.iter().find(|(key, _)| *key == name).map(|(_, value)| Operand::Value(value));
    let element = binding(b"key".as_slice()).zip(binding(b"value".as_slice()));
    let bytes = evaluate::template::run(
        &template,
        &roots.context(),
        element,
        &limits::SHIPPED,
        u64::from(limits::SHIPPED.rendered_file),
    );
    outcome(bytes.ok().map(|bytes| String::from_utf8(bytes.into()).expect("UTF-8 template")))
}
#[must_use]
pub fn context_of(roots: &Roots) -> Value {
    let mut values: Vec<_> = [
        ("target", &roots.target),
        ("debian", &roots.debian),
        ("settings", &roots.settings),
        ("tasks", &roots.tasks),
        ("exports", &roots.exports),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), convert(value)))
    .collect();
    values.extend(
        roots
            .bindings
            .iter()
            .map(|(name, value)| (std::str::from_utf8(name).expect("binding name").to_owned(), convert(value))),
    );
    values.into_iter().collect()
}
fn convert(value: &HearthValue) -> Value {
    match value {
        HearthValue::Bool(value) => Value::from(*value),
        HearthValue::Int(value) => Value::from(*value),
        HearthValue::String(value) => Value::from(std::str::from_utf8(value).expect("generated UTF-8")),
        HearthValue::List(values) => values.iter().map(convert).collect(),
        HearthValue::Map(entries) => entries
            .iter()
            .filter_map(|entry| {
                entry.value.as_ref().map(|value| (std::str::from_utf8(&entry.key).expect("key"), convert(value)))
            })
            .collect(),
    }
}
#[derive(Clone, Copy)]
enum Projection {
    Keys,
    Values,
}
fn members(value: &Value, projection: Projection) -> Result<Value, Error> {
    if value.kind() != ValueKind::Map {
        return Err(Error::new(ErrorKind::InvalidOperation, "expected map"));
    }
    value
        .try_iter()?
        .map(|key| match projection {
            Projection::Keys => Ok(key),
            Projection::Values => value.get_item(&key),
        })
        .collect()
}
#[derive(Clone, Copy)]
enum PathFilter {
    Basename,
    Dirname,
    Escape,
}
fn as_text(value: &Value) -> Result<&str, Error> {
    value.as_str().ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "expected string"))
}
fn path(value: &str, filter: PathFilter) -> Result<String, Error> {
    if value != "/"
        && (value.is_empty()
            || value
                .strip_prefix('/')
                .unwrap_or(value)
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == ".."))
    {
        return Err(Error::new(ErrorKind::InvalidOperation, "expected path"));
    }
    Ok(match filter {
        PathFilter::Basename if value != "/" => value.rsplit('/').next().expect("segment").to_owned(),
        PathFilter::Dirname if value != "/" => match value.rsplit_once('/') {
            Some(("", _)) => "/".to_owned(),
            Some((parent, _)) => parent.to_owned(),
            None => ".".to_owned(),
        },
        PathFilter::Basename | PathFilter::Dirname => value.to_owned(),
        PathFilter::Escape => {
            let Some(relative) = value.strip_prefix('/') else {
                return Err(Error::new(ErrorKind::InvalidOperation, "expected absolute path"));
            };
            if relative.is_empty() { "-".to_owned() } else { escape(relative) }
        }
    })
}
fn escape(value: &str) -> String {
    let mut escaped = String::new();
    for (index, byte) in value.bytes().enumerate() {
        if byte == b'/' {
            escaped.push('-');
        } else if byte.is_ascii_alphanumeric() || b":_".contains(&byte) || (byte == b'.' && index != 0) {
            escaped.push(char::from(byte));
        } else {
            use fmt::Write;
            write!(escaped, "\\x{byte:02x}").expect("String write");
        }
    }
    escaped
}
