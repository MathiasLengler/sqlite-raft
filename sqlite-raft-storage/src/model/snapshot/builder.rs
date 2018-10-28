pub struct SnapshotDataBuilder {
    idx: u64
}

impl SnapshotDataBuilder {
    fn new(idx: u64) -> SnapshotDataBuilder {
        SnapshotDataBuilder {
            idx
        }
    }

    fn build() -> Vec<u8> {
        // TODO
        unimplemented!()
    }
}