pub struct SnapshotDataBuilder {
    idx: u64
}

impl SnapshotDataBuilder {
    pub fn new(idx: u64) -> SnapshotDataBuilder {
        SnapshotDataBuilder {
            idx
        }
    }

    pub fn build() -> Vec<u8> {
        // TODO
        unimplemented!()
    }
}