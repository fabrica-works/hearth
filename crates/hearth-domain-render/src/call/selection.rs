//! Copy the selected target before mutating its call's state.
impl super::Side {
    pub(super) fn answer_target(&self, index: u32) -> super::Target {
        self.targets.get(usize::try_from(index).expect("bounded cursor")).expect("answer cursor within targets").clone()
    }
}
