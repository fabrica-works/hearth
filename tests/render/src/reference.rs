//! Recursive reference from format.md, 7.4 to 7.7; no use of evaluator operations.
use super::expressions::{Binary, Plain, Roots, Tree};
use hearth_domain_render::{
    evaluate::{Missing, Value},
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
    let value = match tree {
        Tree::String(v) => {
            charge(left, u64::try_from(v.len()).expect("small"))?;
            Some(Value::String(v.clone().into_boxed_slice()))
        }
        Tree::Int(v) => Some(Value::Int(*v)),
        Tree::Bool(v) => Some(Value::Bool(*v)),
        Tree::Name(n) => Some(match *n {
            b"target" => roots.values[0].clone(),
            b"debian" => roots.values[1].clone(),
            b"settings" => roots.values[2].clone(),
            b"tasks" => roots.values[3].clone(),
            b"exports" => roots.values[4].clone(),
            _ => roots.bindings.iter().rev().find(|(key, _)| key == n).ok_or("unknown")?.1.clone(),
        }),
        Tree::Key(v, k, missing) => key(run(v, roots, limits, left)?.value, k, *missing)?,
        Tree::Item(v, i, missing) => {
            let container = run(v, roots, limits, left)?;
            let index = run(i, roots, limits, left)?.value;
            match (container.value, index) {
                (None, _) => absent(*missing, "undefined")?,
                (Some(_), None) => return Err("undefined"),
                (Some(Value::Map(entries)), Some(Value::String(k))) => key(Some(Value::Map(entries)), &k, *missing)?,
                (Some(Value::List(items)), Some(Value::Int(i))) => {
                    if let Some(v) = usize::try_from(i).ok().and_then(|i| items.get(i)) {
                        if let Some(made) = container.items {
                            return Ok(made[usize::try_from(i).expect("valid index")].clone());
                        }
                        Some(v.clone())
                    } else {
                        absent(*missing, "index")?
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
        Tree::Defined(v, negated) => Some(Value::Bool(run(v, roots, limits, left)?.value.is_some() != *negated)),
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
