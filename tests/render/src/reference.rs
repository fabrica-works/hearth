//! Recursive reference from format.md, 7.4 to 7.7; no use of evaluator operations.
use super::expressions::{Plain, Roots, Tree};
use hearth_domain_render::{
    evaluate::{Missing, Value},
    limits::Limits,
};
#[derive(Debug)]
struct Made {
    value: Option<Value>,
    depth: u32,
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
    let mut depth = 0;
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
                    let found = usize::try_from(i).ok().and_then(|i| items.get(i));
                    match found {
                        Some(v) => Some(v.clone()),
                        None => absent(*missing, "index")?,
                    }
                }
                _ => return Err("type"),
            }
        }
        Tree::List(items) => {
            let mut values = vec![];
            depth = 1;
            for item in items {
                let made = run(item, roots, limits, left)?;
                depth = depth.max(made.depth + 1);
                values.push(made.value);
            }
            charge(left, u64::try_from(items.len()).expect("small") * 32)?;
            let values = values.into_iter().collect::<Option<Box<[_]>>>().ok_or("undefined")?;
            if depth > limits.nesting {
                return Err("limit");
            }
            Some(Value::List(values))
        }
        Tree::Defined(v, negated) => Some(Value::Bool(run(v, roots, limits, left)?.value.is_some() != *negated)),
    };
    Ok(Made { value, depth })
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
