//! A recipe's layers and their keys (build.md, 3; store.md, 3.3).
use hearth_lib::{encode::Digest, hash::H};
/// The layers a rootfs recipe builds, bottom-up; the empty layer is no step's.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Layer {
    Packages,
    Software,
    Config,
}
/// Bottom-up, each the parent of the next.
pub const LAYERS: [Layer; 3] = [Layer::Packages, Layer::Software, Layer::Config];
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Keys {
    pub empty: Digest,
    pub packages: Digest,
    pub software: Digest,
    pub config: Digest,
}
impl Keys {
    #[must_use]
    pub const fn of(&self, layer: Layer) -> Digest {
        match layer {
            Layer::Packages => self.packages,
            Layer::Software => self.software,
            Layer::Config => self.config,
        }
    }
}
/// Each layer's key from the resolution's closure digest and the three part digests.
#[must_use]
pub fn keys(closure: Digest, packages: Digest, software: Digest, config: Digest) -> Keys {
    let packages = layer(b"packages", &closure, &packages);
    let software = layer(b"software", &packages, &software);
    let config = layer(b"config", &software, &config);
    Keys { empty: H::new(b"empty", 0).finish(), packages, software, config }
}
fn layer(tag: &[u8], beneath: &Digest, part: &Digest) -> Digest {
    let mut hash = H::new(tag, 2);
    hash.values().digest(beneath);
    hash.values().digest(part);
    hash.finish()
}
