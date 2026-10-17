//! Recursive reference from format.md, 7.4 to 7.7; no use of evaluator operations.
use super::expressions::{Binary, Plain, Roots, Tree};
use hearth_domain_render::{
    evaluate::{Filter, Missing, Value},
    limits::Limits,
};
#[derive(Clone, Debug)]
struct Made {
    value: Option<Value>,
    depth: u32,
    items: Option<Vec<Made>>,
}
impl Made {
    fn plain(value: Option<Value>) -> Self {
        Self { value, depth: 0, items: None }
    }
}
/// # Errors
/// Returns the first fault specified by format.md, 7.4 to 7.7.
pub fn evaluate(tree: &Tree, roots: &Roots, limits: &Limits) -> Result<Option<Plain>, &'static str> {
    let mut left = limits.made;
    run(tree, roots, limits, &mut left).map(|v| v.value)
}
fn charge(left: &mut u64, n: u64) -> Result<(), &'static str> {
    *left = left.checked_sub(n).ok_or("limit")?;
    Ok(())
}
fn run(tree: &Tree, roots: &Roots, limits: &Limits, left: &mut u64) -> Result<Made, &'static str> {
    run_lookup(tree, roots, limits, left, Missing::Error)
}
fn marked(tree: &Tree, roots: &Roots, limits: &Limits, left: &mut u64) -> Result<Made, &'static str> {
    let missing = if super::expressions::chain(tree) { Missing::Undefined } else { Missing::Error };
    run_lookup(tree, roots, limits, left, missing)
}
fn run_lookup(
    tree: &Tree,
    roots: &Roots,
    limits: &Limits,
    left: &mut u64,
    missing: Missing,
) -> Result<Made, &'static str> {
    let value = match tree {
        Tree::String(v) => {
            charge(left, u64::try_from(v.len()).expect("small"))?;
            Some(Value::String(v.clone().into_boxed_slice()))
        }
        Tree::Int(v) => Some(Value::Int(*v)),
        Tree::Bool(v) => Some(Value::Bool(*v)),
        Tree::Name(n) => Some(match *n {
            b"target" => roots.target.clone(),
            b"debian" => roots.debian.clone(),
            b"settings" => roots.settings.clone(),
            b"tasks" => roots.tasks.clone(),
            b"exports" => roots.exports.clone(),
            _ => roots.bindings.iter().rev().find(|(key, _)| key == n).ok_or("unknown")?.1.clone(),
        }),
        Tree::Key(v, k, _) => key(run_lookup(v, roots, limits, left, missing)?.value, k, missing)?,
        Tree::Item(v, i, _) => {
            let container = run_lookup(v, roots, limits, left, missing)?;
            let index = run(i, roots, limits, left)?.value;
            match (container.value, index) {
                (None, _) => absent(missing, "undefined")?,
                (Some(_), None) => return Err("undefined"),
                (Some(Value::Map(entries)), Some(Value::String(k))) => key(Some(Value::Map(entries)), &k, missing)?,
                (Some(Value::List(items)), Some(Value::Int(i))) => {
                    if let Some(v) = usize::try_from(i).ok().and_then(|i| items.get(i)) {
                        if let Some(made) = container.items {
                            return Ok(made[usize::try_from(i).expect("valid index")].clone());
                        }
                        Some(v.clone())
                    } else {
                        absent(missing, "index")?
                    }
                }
                _ => return Err("type"),
            }
        }
        Tree::List(items) => {
            let mut made = vec![];
            for item in items {
                made.push(run(item, roots, limits, left)?);
            }
            charge(left, u64::try_from(items.len()).expect("small") * 32)?;
            let values = made.iter().map(|v| v.value.clone()).collect::<Option<Box<[_]>>>().ok_or("undefined")?;
            let depth = made.iter().map(|v| v.depth + 1).max().unwrap_or(1);
            if depth > limits.nesting {
                return Err("limit");
            }
            return Ok(Made { value: Some(Value::List(values)), depth, items: Some(made) });
        }
        Tree::Defined(v, negated) => Some(Value::Bool(marked(v, roots, limits, left)?.value.is_some() != *negated)),
        Tree::Not(v) => Some(Value::Bool(!truthy(run(v, roots, limits, left)?.value.as_ref()))),
        Tree::And(a, b) => {
            let a = run(a, roots, limits, left)?;
            return if truthy(a.value.as_ref()) { run(b, roots, limits, left) } else { Ok(a) };
        }
        Tree::Or(a, b) => {
            let a = run(a, roots, limits, left)?;
            return if truthy(a.value.as_ref()) { Ok(a) } else { run(b, roots, limits, left) };
        }
        Tree::Conditional { condition, yes, no } => {
            let condition = run(condition, roots, limits, left)?;
            let result = run(if truthy(condition.value.as_ref()) { yes } else { no }, roots, limits, left)?;
            if result.value.is_none() {
                return Err("undefined");
            }
            return Ok(result);
        }
        Tree::Binary(a, op, b) => {
            let a = run(a, roots, limits, left)?;
            let b = run(b, roots, limits, left)?;
            Some(binary(a.value.ok_or("undefined")?, *op, b.value.ok_or("undefined")?, left)?)
        }
        Tree::Filter(input, filter, arguments) => {
            let input = if *filter == Filter::Default {
                marked(input, roots, limits, left)?
            } else {
                run(input, roots, limits, left)?
            };
            let arguments =
                arguments.iter().map(|argument| run(argument, roots, limits, left)).collect::<Result<Vec<_>, _>>()?;
            return filtered(input, *filter, arguments, limits, left);
        }
        Tree::Output(v) => {
            let value = run(v, roots, limits, left)?.value.ok_or("undefined")?;
            Some(Value::String(match value {
                Value::Bool(v) => v.to_string().into_bytes().into_boxed_slice(),
                Value::Int(v) => v.to_string().into_bytes().into_boxed_slice(),
                Value::String(v) => v,
                Value::List(_) | Value::Map(_) => return Err("output"),
            }))
        }
    };
    Ok(Made::plain(value))
}
fn truthy(value: Option<&Value>) -> bool {
    match value {
        None => false,
        Some(Value::Bool(v)) => *v,
        Some(Value::Int(v)) => *v != 0,
        Some(Value::String(v)) => !v.is_empty(),
        Some(Value::List(v)) => !v.is_empty(),
        Some(Value::Map(v)) => v.iter().any(|e| e.value.is_some()),
    }
}
fn equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::List(a), Value::List(b)) => a.len() == b.len() && a.iter().zip(b).all(|(a, b)| equal(a, b)),
        (Value::Map(a), Value::Map(b)) => {
            let a: Vec<_> = a.iter().filter(|v| v.value.is_some()).collect();
            let b: Vec<_> = b.iter().filter(|v| v.value.is_some()).collect();
            a.len() == b.len()
                && a.iter().zip(b).all(|(a, b)| {
                    a.key == b.key && equal(a.value.as_ref().expect("present"), b.value.as_ref().expect("present"))
                })
        }
        (Value::Bool(_) | Value::Int(_) | Value::String(_) | Value::List(_) | Value::Map(_), _) => false,
    }
}
fn binary(a: Value, op: Binary, b: Value, left: &mut u64) -> Result<Value, &'static str> {
    Ok(match op {
        Binary::Equal => Value::Bool(equal(&a, &b)),
        Binary::NotEqual => Value::Bool(!equal(&a, &b)),
        Binary::Add | Binary::Subtract => {
            let (Value::Int(a), Value::Int(b)) = (a, b) else {
                return Err("type");
            };
            Value::Int((if matches!(op, Binary::Add) { a.checked_add(b) } else { a.checked_sub(b) }).ok_or("overflow")?)
        }
        Binary::Concat => {
            fn text(value: Value) -> Result<Vec<u8>, &'static str> {
                match value {
                    Value::String(v) => Ok(v.into_vec()),
                    Value::Int(v) => Ok(v.to_string().into_bytes()),
                    Value::Bool(_) | Value::List(_) | Value::Map(_) => Err("type"),
                }
            }
            let mut a = text(a)?;
            a.extend(text(b)?);
            charge(left, u64::try_from(a.len()).expect("small"))?;
            Value::String(a.into_boxed_slice())
        }
        Binary::Less | Binary::LessOrEqual | Binary::Greater | Binary::GreaterOrEqual => {
            let order = match (a, b) {
                (Value::Int(a), Value::Int(b)) => a.cmp(&b),
                (Value::String(a), Value::String(b)) => a.cmp(&b),
                _ => return Err("type"),
            };
            Value::Bool(match op {
                Binary::Less => order.is_lt(),
                Binary::LessOrEqual => !order.is_gt(),
                Binary::Greater => order.is_gt(),
                Binary::GreaterOrEqual => !order.is_lt(),
                Binary::Equal
                | Binary::NotEqual
                | Binary::In
                | Binary::NotIn
                | Binary::Concat
                | Binary::Add
                | Binary::Subtract => unreachable!("ordering"),
            })
        }
        Binary::In | Binary::NotIn => {
            let found = match b {
                Value::Map(entries) => {
                    let Value::String(key) = a else {
                        return Err("type");
                    };
                    entries.iter().any(|e| e.key == key && e.value.is_some())
                }
                Value::List(items) => items.iter().any(|v| equal(&a, v)),
                Value::String(text) => {
                    let Value::String(needle) = a else {
                        return Err("type");
                    };
                    needle.is_empty() || text.windows(needle.len()).any(|v| v == needle.as_ref())
                }
                Value::Bool(_) | Value::Int(_) => return Err("type"),
            };
            Value::Bool(found != matches!(op, Binary::NotIn))
        }
    })
}
fn key(value: Option<Value>, key: &[u8], missing: Missing) -> Result<Option<Value>, &'static str> {
    match value {
        None => absent(missing, "undefined"),
        Some(Value::Map(entries)) => match entries.iter().find(|entry| entry.key.as_ref() == key) {
            Some(entry) => Ok(entry.value.clone()),
            None => absent(missing, "key"),
        },
        Some(Value::Bool(_) | Value::Int(_) | Value::String(_) | Value::List(_)) => Err("type"),
    }
}
fn absent(missing: Missing, kind: &'static str) -> Result<Option<Value>, &'static str> {
    match missing {
        Missing::Error => Err(kind),
        Missing::Undefined => Ok(None),
    }
}
fn filtered(
    input: Made,
    filter: Filter,
    arguments: Vec<Made>,
    limits: &Limits,
    left: &mut u64,
) -> Result<Made, &'static str> {
    let mut arguments = arguments.into_iter();
    let first = arguments.next();
    let second = arguments.next();
    if matches!(filter, Filter::Default) && input.value.is_none() {
        return Ok(first.expect("one argument"));
    }
    let value = input.value.as_ref().ok_or("undefined")?;
    let first = first.map(|value| value.value.ok_or("undefined")).transpose()?;
    let second = second.map(|value| value.value.ok_or("undefined")).transpose()?;
    let mut list = None;
    let mut bytes = None;
    match filter {
        Filter::Default => return Ok(input),
        Filter::First | Filter::Last => {
            let Value::List(values) = value else {
                return Err("type");
            };
            let index = if matches!(filter, Filter::First) { 0 } else { values.len().saturating_sub(1) };
            let value = values.get(index).ok_or("empty")?;
            return Ok(match input.items {
                Some(items) => items[index].clone(),
                None => Made::plain(Some(value.clone())),
            });
        }
        Filter::Length => {
            let length = match value {
                Value::String(text) => text.iter().filter(|byte| **byte & 0xc0 != 0x80).count(),
                Value::List(items) => items.len(),
                Value::Map(entries) => entries.iter().filter(|entry| entry.value.is_some()).count(),
                Value::Bool(_) | Value::Int(_) => return Err("type"),
            };
            return Ok(Made::plain(Some(Value::Int(i64::try_from(length).expect("small")))));
        }
        Filter::Join => bytes = Some(join(value, first.expect("one argument"))?),
        Filter::Split => list = Some((split(value, first.expect("one argument"), left)?, 1)),
        Filter::Replace => bytes = Some(replace(value, first.expect("two arguments"), second.expect("two arguments"))?),
        Filter::Items | Filter::Keys | Filter::Values => list = Some(entries(value, filter, left)?),
        Filter::Sort => {
            let Value::List(values) = value else {
                return Err("type");
            };
            charge(left, u64::try_from(values.len()).expect("small") * 32)?;
            let strings = values.iter().all(|value| matches!(value, Value::String(_)));
            let integers = values.iter().all(|value| matches!(value, Value::Int(_)));
            if !strings && !integers {
                return Err("type");
            }
            let mut values = values.to_vec();
            values.sort_by(|a, b| match (a, b) {
                (Value::String(a), Value::String(b)) => a.cmp(b),
                (Value::Int(a), Value::Int(b)) => a.cmp(b),
                _ => unreachable!("homogeneous"),
            });
            list = Some((values.into_iter().map(|value| Made::plain(Some(value))).collect(), 1));
        }
        Filter::Tojson => bytes = Some(json(value)),
        Filter::ShellQuote => {
            let input = text(value.clone())?;
            let mut output = vec![b'\''];
            for byte in input {
                if byte == b'\'' {
                    output.extend_from_slice(b"'\\''");
                } else {
                    output.push(byte);
                }
            }
            output.push(b'\'');
            bytes = Some(output);
        }
        Filter::SystemdEscape | Filter::SystemdEscapePath | Filter::Basename | Filter::Dirname => {
            bytes = Some(path(value, filter)?);
        }
    }
    if let Some(bytes) = bytes {
        charge(left, u64::try_from(bytes.len()).expect("small"))?;
        return Ok(string(bytes));
    }
    let (items, depth) = list.expect("list filter");
    charge(left, u64::try_from(items.len()).expect("small") * 32)?;
    if depth > limits.nesting {
        return Err("limit");
    }
    Ok(Made {
        value: Some(Value::List(items.iter().map(|item| item.value.clone().expect("defined")).collect())),
        depth,
        items: Some(items),
    })
}
fn json(value: &Value) -> Vec<u8> {
    match value {
        Value::Bool(value) => value.to_string().into_bytes(),
        Value::Int(value) => value.to_string().into_bytes(),
        Value::String(text) => {
            let mut result = vec![b'"'];
            for byte in text {
                match byte {
                    b'"' => result.extend_from_slice(b"\\\""),
                    b'\\' => result.extend_from_slice(b"\\\\"),
                    8 => result.extend_from_slice(b"\\b"),
                    12 => result.extend_from_slice(b"\\f"),
                    b'\n' => result.extend_from_slice(b"\\n"),
                    b'\r' => result.extend_from_slice(b"\\r"),
                    b'\t' => result.extend_from_slice(b"\\t"),
                    0..=31 => result.extend(format!("\\u00{byte:02x}").bytes()),
                    _ => result.push(*byte),
                }
            }
            result.push(b'"');
            result
        }
        Value::List(items) => {
            let mut result = vec![b'['];
            result.extend(items.iter().map(json).collect::<Vec<_>>().join(&b","[..]));
            result.push(b']');
            result
        }
        Value::Map(entries) => {
            let mut result = vec![b'{'];
            let mut fields = vec![];
            for entry in entries {
                if let Some(value) = &entry.value {
                    let mut field = json(&Value::String(entry.key.clone()));
                    field.push(b':');
                    field.extend(json(value));
                    fields.push(field);
                }
            }
            result.extend(fields.join(&b","[..]));
            result.push(b'}');
            result
        }
    }
}

fn string(text: Vec<u8>) -> Made {
    Made::plain(Some(Value::String(text.into_boxed_slice())))
}
fn text(value: Value) -> Result<Vec<u8>, &'static str> {
    match value {
        Value::String(text) => Ok(text.into_vec()),
        Value::Bool(_) | Value::Int(_) | Value::List(_) | Value::Map(_) => Err("type"),
    }
}

fn join(value: &Value, separator: Value) -> Result<Vec<u8>, &'static str> {
    let Value::List(values) = value else {
        return Err("type");
    };
    let separator = text(separator)?;
    let values = values
        .iter()
        .map(|value| match value {
            Value::String(text) => Ok(text.to_vec()),
            Value::Int(value) => Ok(value.to_string().into_bytes()),
            Value::Bool(_) | Value::List(_) | Value::Map(_) => Err("type"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(values.join(separator.as_slice()))
}

fn split(value: &Value, separator: Value, left: &mut u64) -> Result<Vec<Made>, &'static str> {
    let input = text(value.clone())?;
    let separator = text(separator)?;
    if separator.is_empty() {
        return Err("empty");
    }
    let mut pieces = vec![];
    let mut from = 0;
    while let Some(index) = input[from..].windows(separator.len()).position(|part| part == separator) {
        pieces.push(string(input[from..from + index].to_vec()));
        from += index + separator.len();
    }
    pieces.push(string(input[from..].to_vec()));
    charge(
        left,
        pieces
            .iter()
            .map(|piece| match piece.value.as_ref().expect("defined") {
                Value::String(text) => u64::try_from(text.len()).expect("small"),
                Value::Bool(_) | Value::Int(_) | Value::List(_) | Value::Map(_) => unreachable!("string"),
            })
            .sum(),
    )?;
    Ok(pieces)
}

fn replace(value: &Value, pattern: Value, replacement: Value) -> Result<Vec<u8>, &'static str> {
    let input = text(value.clone())?;
    let pattern = text(pattern)?;
    let replacement = text(replacement)?;
    if pattern.is_empty() {
        return Err("empty");
    }
    let mut output = vec![];
    let mut from = 0;
    while let Some(index) = input[from..].windows(pattern.len()).position(|part| part == pattern) {
        output.extend_from_slice(&input[from..from + index]);
        output.extend_from_slice(&replacement);
        from += index + pattern.len();
    }
    output.extend_from_slice(&input[from..]);
    Ok(output)
}

fn entries(value: &Value, filter: Filter, left: &mut u64) -> Result<(Vec<Made>, u32), &'static str> {
    let Value::Map(entries) = value else {
        return Err("type");
    };
    let mut items = vec![];
    for entry in entries {
        let Some(value) = &entry.value else {
            continue;
        };
        if !matches!(filter, Filter::Values) {
            charge(left, u64::try_from(entry.key.len()).expect("small"))?;
        }
        items.push(match filter {
            Filter::Keys => string(entry.key.to_vec()),
            Filter::Values => Made::plain(Some(value.clone())),
            Filter::Items => {
                charge(left, 64)?;
                let pair = vec![string(entry.key.to_vec()), Made::plain(Some(value.clone()))];
                Made {
                    value: Some(Value::List(pair.iter().map(|value| value.value.clone().expect("defined")).collect())),
                    depth: 1,
                    items: Some(pair),
                }
            }
            Filter::Default
            | Filter::Join
            | Filter::Split
            | Filter::First
            | Filter::Last
            | Filter::Length
            | Filter::Replace
            | Filter::Sort
            | Filter::Tojson
            | Filter::ShellQuote
            | Filter::SystemdEscape
            | Filter::SystemdEscapePath
            | Filter::Basename
            | Filter::Dirname => unreachable!("map filter"),
        });
    }
    let depth = if matches!(filter, Filter::Items) && !items.is_empty() { 2 } else { 1 };
    Ok((items, depth))
}

fn path(value: &Value, filter: Filter) -> Result<Vec<u8>, &'static str> {
    let mut input = text(value.clone())?;
    if !matches!(filter, Filter::SystemdEscape) {
        let absolute = input.first() == Some(&b'/');
        let segments = if absolute { &input[1..] } else { &input[..] };
        if input != b"/"
            && (input.is_empty()
                || segments.split(|byte| *byte == b'/').any(|part| part.is_empty() || part == b"." || part == b".."))
        {
            return Err("path");
        }
        if matches!(filter, Filter::SystemdEscapePath) && !absolute {
            return Err("path");
        }
    }
    Ok(match filter {
        Filter::Basename if input != b"/" => input.rsplit(|byte| *byte == b'/').next().expect("one segment").to_vec(),
        Filter::Dirname if input != b"/" => match input.iter().rposition(|byte| *byte == b'/') {
            Some(0) => b"/".to_vec(),
            Some(index) => input[..index].to_vec(),
            None => b".".to_vec(),
        },
        Filter::Basename | Filter::Dirname => input,
        Filter::SystemdEscape | Filter::SystemdEscapePath => {
            if matches!(filter, Filter::SystemdEscapePath) && input != b"/" {
                input.remove(0);
            }
            let mut output = vec![];
            for (index, byte) in input.into_iter().enumerate() {
                if byte == b'/' {
                    output.push(b'-');
                } else if byte.is_ascii_alphanumeric() || b":_".contains(&byte) || (byte == b'.' && index != 0) {
                    output.push(byte);
                } else {
                    output.extend(format!("\\x{byte:02x}").bytes());
                }
            }
            output
        }
        Filter::Default
        | Filter::Join
        | Filter::Split
        | Filter::First
        | Filter::Last
        | Filter::Length
        | Filter::Replace
        | Filter::Items
        | Filter::Keys
        | Filter::Values
        | Filter::Sort
        | Filter::Tojson
        | Filter::ShellQuote => unreachable!("text filter"),
    })
}
