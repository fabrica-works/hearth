//! Boot vocabulary across the client boundary.
#[must_use]
pub fn boot(boot: crate::render::parts::boot::Boot) -> crate::records::parts::boot::Boot {
    let crate::render::parts::boot::Boot { params, mutable } = boot;
    crate::records::parts::boot::Boot { params, mutable }
}

#[must_use]
pub fn change(value: crate::render::parts::boot::Change) -> crate::records::parts::boot::Change {
    let crate::render::parts::boot::Change { before, after } = value;
    crate::records::parts::boot::Change { before: boot(before), after: boot(after) }
}
