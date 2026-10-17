//! Boot vocabulary across the client boundary.
#[must_use]
pub fn boot(boot: crate::render::parts::boot::Boot) -> crate::records::parts::boot::Boot {
    let crate::render::parts::boot::Boot { params, mutable } = boot;
    crate::records::parts::boot::Boot { params, mutable }
}
