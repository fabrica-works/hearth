use alloc::boxed::Box;
pub fn finish_render(
    render: &mut crate::Render,
    env: &skein_lib::Env<crate::limits::Limits>,
    out: &mut skein_lib::Queue<crate::vocabulary::Output>,
) -> Box<[crate::vocabulary::Answer]> {
    let mut answers = skein_lib::List::with_capacity(env.limits.errors.saturating_add(10));
    for _ in 0..env.limits.errors.saturating_add(20) {
        if !render.work_pending() {
            break;
        }
        crate::resume(render, env, out);
        match out.pop() {
            Some(crate::vocabulary::Output::Answer(answer)) => {
                answers.push(answer).expect("answer capacity");
            }
            Some(crate::vocabulary::Output::Request(crate::vocabulary::Request::Find { request, .. })) => {
                crate::step(render, env, crate::vocabulary::Event::Absent { request }, out);
            }
            Some(crate::vocabulary::Output::Request(_)) | None => {}
        }
    }
    answers.into_boxed()
}

pub const KEY_A: hearth_lib::encode::Digest = hearth_lib::encode::Digest([1; 32]);
pub const KEY_B: hearth_lib::encode::Digest = hearth_lib::encode::Digest([2; 32]);
pub fn finding_side() -> crate::call::Side {
    use crate::{diff::Made, keys::Resolution};
    let mut side = crate::call::Side::new(b"/repo".as_slice().into(), Box::new([]), &crate::limits::SHIPPED);
    side.ready = true;
    side.files = true;
    side.made = skein_lib::Map::with_capacity(3);
    for (name, key) in [(b"c".as_slice(), KEY_A), (b"a", KEY_A), (b"b", KEY_B)] {
        let mut spec = super::spec(name, &[]);
        spec.resolution = Some(Box::new(Resolution::Needed { key }));
        side.made.insert(name.into(), Made::Spec(Box::new(spec))).expect("fixture targets");
    }
    side
}
pub fn key_inputs() -> crate::parts::packages::Inputs {
    use hearth_lib::encode::Item;
    crate::parts::packages::Inputs {
        sources: Box::new([Item::Map(0)]),
        request: Box::new([Item::List(1), Item::Bytes(b"hello".as_slice().into())]),
        variant: Box::new([Item::Bytes(b"apt".as_slice().into())]),
        apt: Box::new([Item::Map(0)]),
    }
}
