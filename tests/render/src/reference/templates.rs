//! Recursive statement semantics, without the compiler or template runner.
use crate::{expressions::Roots, templates::Tree};
use hearth_domain_render::{evaluate::Value, limits::Limits};
struct Run<'a> {
    limits: &'a Limits,
    iterations: u32,
    output: Vec<u8>,
}
/// # Errors
/// Returns the first fault specified by the template's statement semantics.
pub fn run(tree: &Tree, roots: &Roots, limits: &Limits) -> Result<Vec<u8>, &'static str> {
    let mut roots = roots.clone();
    roots.bindings.clear();
    let mut run = Run { limits, iterations: limits.iterations, output: vec![] };
    let mut made = limits.made;
    run.visit(tree, &mut roots, &mut made, 0)?;
    Ok(run.output)
}
impl Run<'_> {
    fn visit(&mut self, tree: &Tree, roots: &mut Roots, left: &mut u64, depth: u32) -> Result<(), &'static str> {
        match tree {
            Tree::Text(bytes) => self.write(bytes)?,
            Tree::Output(value) => {
                let value = super::expression(value, roots, self.limits, &mut { *left })?.value.ok_or("undefined")?;
                let bytes = match value {
                    Value::Bool(value) => value.to_string().into_bytes(),
                    Value::Int(value) => value.to_string().into_bytes(),
                    Value::String(value) => value.into_vec(),
                    Value::List(_) | Value::Map(_) => return Err("output"),
                };
                self.write(&bytes)?;
            }
            Tree::Sequence(children) => {
                for child in children {
                    self.visit(child, roots, left, depth)?;
                }
            }
            Tree::Branch { arms, otherwise } => {
                for (condition, body) in arms {
                    if super::truthy(super::expression(condition, roots, self.limits, &mut { *left })?.value.as_ref()) {
                        return self.visit(body, roots, left, depth);
                    }
                }
                self.visit(otherwise, roots, left, depth)?;
            }
            Tree::Set { value } => {
                let value = super::expression(value, roots, self.limits, left)?.value.ok_or("undefined")?;
                roots.bindings.push((b"x", value));
            }
            Tree::For { pair, iterable, body } => {
                let mut budget = *left;
                let value = super::expression(iterable, roots, self.limits, &mut budget)?.value.ok_or("undefined")?;
                let (values, keys): (Vec<_>, bool) = match value {
                    Value::List(values) => (values.into_vec(), false),
                    Value::Map(entries) => (
                        entries
                            .into_iter()
                            .filter(|entry| entry.value.is_some())
                            .map(|entry| Value::String(entry.key))
                            .collect(),
                        true,
                    ),
                    Value::Bool(_) | Value::Int(_) | Value::String(_) => return Err("type"),
                };
                for (index, value) in values.iter().enumerate() {
                    let mut scoped = roots.clone();
                    let mut budget = budget;
                    if keys {
                        super::charge(&mut budget, u64::try_from(value_len(value)).expect("small"))?;
                    }
                    self.iterations = self.iterations.checked_sub(1).ok_or("limit")?;
                    if *pair {
                        let Value::List(items) = value else { return Err("type") };
                        let [first, second] = items.as_ref() else { return Err("type") };
                        scoped.bindings.push((b"x", first.clone()));
                        scoped.bindings.push((b"y", second.clone()));
                    } else {
                        scoped.bindings.push((b"x", value.clone()));
                    }
                    scoped.bindings.push((b"loop", crate::templates::loop_value(index, values.len())));
                    if depth >= self.limits.statements {
                        return Err("limit");
                    }
                    self.visit(body, &mut scoped, &mut budget, depth + 1)?;
                }
            }
        }
        Ok(())
    }
    fn write(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        if self.output.len().saturating_add(bytes.len())
            > usize::try_from(self.limits.rendered_file).expect("file limit")
        {
            return Err("limit");
        }
        self.output.extend_from_slice(bytes);
        Ok(())
    }
}
fn value_len(value: &Value) -> usize {
    let Value::String(bytes) = value else { unreachable!("map keys") };
    bytes.len()
}
