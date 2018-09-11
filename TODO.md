implement the protobuf layer inside sqlite-commands crate because of visibility.

inject generated protobuf code inside this crate.

implement skeleton of RaftNode and RaftSqliteClientAPI

decide where/how to implement the "Node communication trait" for the grpc RaftNode service