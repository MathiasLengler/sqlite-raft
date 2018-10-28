
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
        // TODO: get unique access to a view of a sql user db file
        //  vacuum, other clean up (?)
        //  compress (optional)
        //  define interface for recreating view from data (deserialization)
        //
        //  Open questions:
        //  useful wrapper for Vec<u8>? raft storage API requires it and for testing
        //  impl view manager first? (this should/could be part of its API, unique access...)
        //  serde wrapper with bincode (version tag, compression enabled/disabled)?
        unimplemented!()
    }
}