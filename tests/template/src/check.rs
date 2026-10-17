use hearth_protocol_template::{Does, Expression, Op, Template};
pub fn invariants(expression: &Expression, text: &[u8]) {
    let mut depths = vec![None; expression.operations.len() + 1];
    depths[0] = Some(0_u32);
    let mut deepest = 0;
    for (index, operation) in expression.operations.iter().enumerate() {
        assert!(usize::try_from(operation.at).expect("bounded text") < text.len());
        let mut depth = depths[index].expect("every emitted operation is reachable");
        deepest = deepest.max(depth);
        match &operation.op {
            Op::String(_) | Op::Integer(_) | Op::Boolean(_) | Op::Name { .. } => depth += 1,
            Op::Key { .. } | Op::Not | Op::Defined | Op::NotDefined => assert!(depth >= 1),
            Op::Item { .. }
            | Op::Equal
            | Op::NotEqual
            | Op::Less
            | Op::LessOrEqual
            | Op::Greater
            | Op::GreaterOrEqual
            | Op::In
            | Op::NotIn
            | Op::Concat
            | Op::Add
            | Op::Subtract => {
                assert!(depth >= 2);
                depth -= 1;
            }
            Op::List { length } => {
                assert!(depth >= *length);
                depth = depth - length + 1;
            }
            Op::Filter(filter) => {
                assert!(depth > filter.arity());
                depth -= filter.arity();
            }
            Op::JumpIfFalseKeeping { over } | Op::JumpIfTrueKeeping { over } => {
                assert!(depth >= 1);
                arrive(&mut depths, index, *over, depth);
                depth -= 1;
            }
            Op::JumpIfFalse { over } => {
                assert!(depth >= 1);
                depth -= 1;
                arrive(&mut depths, index, *over, depth);
            }
            Op::Jump { over } => {
                arrive(&mut depths, index, *over, depth);
                continue;
            }
        }
        deepest = deepest.max(depth);
        arrive(&mut depths, index, 0, depth);
    }
    assert_eq!(depths.last(), Some(&Some(1)));
    assert_eq!(expression.values, deepest);
}
fn arrive(depths: &mut [Option<u32>], index: usize, over: u32, depth: u32) {
    let target = index + 1 + usize::try_from(over).expect("jump fits");
    assert!(target > index && target < depths.len());
    if let Some(previous) = depths[target] {
        assert_eq!(previous, depth, "target {target}");
    }
    depths[target] = Some(depth);
}

pub fn template(compiled: &Template, text: &[u8]) {
    let lines: Vec<_> = std::iter::once(0)
        .chain(
            text.iter()
                .enumerate()
                .filter(|(_, b)| **b == b'\n')
                .map(|(at, _)| u32::try_from(at + 1).expect("validated template length")),
        )
        .collect();
    assert_eq!(&*compiled.lines, lines);
    for (index, instruction) in compiled.instructions.iter().enumerate() {
        assert!((instruction.at as usize) < text.len());
        match &instruction.does {
            Does::Text(bytes) => assert!(!bytes.is_empty()),
            Does::Output(e) | Does::Set { value: e, .. } => invariants(e, text),
            Does::Branch { condition, over } => {
                invariants(condition, text);
                assert!(index + 1 + *over as usize <= compiled.instructions.len());
            }
            Does::Jump { over } => assert!(index + 1 + *over as usize <= compiled.instructions.len()),
            Does::Enter { iterable, over, .. } => {
                invariants(iterable, text);
                assert!(index + 1 + *over as usize <= compiled.instructions.len());
            }
            Does::Advance { back } => {
                let enter = index - *back as usize - 1;
                let Does::Enter { over, .. } = &compiled.instructions[enter].does else {
                    panic!("advance has its enter")
                };
                assert_eq!(enter + 1 + *over as usize, index + 1);
            }
        }
    }
}
