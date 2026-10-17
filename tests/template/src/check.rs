use hearth_protocol_template::{Expression, Op};
pub fn invariants(expression: &Expression, text: &[u8]) {
    let mut depth = 0_u32;
    let mut deepest = 0;
    for operation in &expression.operations {
        assert!(usize::try_from(operation.at).expect("bounded by fixture or checked text") < text.len());
        match &operation.op {
            Op::String(_) | Op::Integer(_) | Op::Boolean(_) | Op::Name { .. } => depth += 1,
        }
        deepest = deepest.max(depth);
    }
    assert_eq!(depth, 1);
    assert_eq!(expression.values, deepest);
}
