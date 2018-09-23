use raft::eraftpb::ConfState;
use model::core::CoreId;
use rusqlite::types::ToSql;
use rusqlite::Row;

pub struct SqliteConfState {
    nodes: Vec<NodeId>,
    learners: Vec<NodeId>,
}

impl SqliteConfState {}

impl From<ConfState> for SqliteConfState {
    fn from(mut conf_state: ConfState) -> Self {
        SqliteConfState {
            nodes: conf_state.take_nodes().into_iter().map(Into::into).collect(),
            learners: conf_state.take_learners().into_iter().map(Into::into).collect(),
        }
    }
}

impl From<SqliteConfState> for ConfState {
    fn from(sqlite_conf_state: SqliteConfState) -> Self {
        let mut conf_state = ConfState::new();
        conf_state.set_nodes(sqlite_conf_state.nodes.into_iter().map(Into::into).collect());
        conf_state.set_learners(sqlite_conf_state.learners.into_iter().map(Into::into).collect());
        conf_state
    }
}


#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
struct NodeId(i64);

impl From<i64> for NodeId {
    fn from(id: i64) -> Self {
        NodeId(id)
    }
}

impl From<NodeId> for i64 {
    fn from(node_id: NodeId) -> Self {
        node_id.0
    }
}

// TODO: try from
impl From<u64> for NodeId {
    fn from(id: u64) -> Self {
        NodeId(id as i64)
    }
}

impl From<NodeId> for u64 {
    fn from(node_id: NodeId) -> Self {
        node_id.0 as u64
    }
}

enum SqliteNode {
    Normal(NodeId),
    Learner(NodeId),
}

enum NodeType {
    Normal = 0,
    Learner = 1,
}

impl SqliteNode {
    fn as_row_tuple(&self) -> (i64, i64) {
        let (node_id, node_type): (&NodeId, i64) = match self {
            SqliteNode::Normal(node_id) => (node_id, 0),
            SqliteNode::Learner(node_id) => (node_id, 1),
        };

        (node_id.clone().into(), node_type)
    }

    pub fn named_params<'a>(node_id: &'a i64, node_type: &'a i64, core_id: &'a CoreId) -> [(&'static str, &'a ToSql); 3] {
        [
            (":node_id", node_id),
            (":node_type", node_type),
            core_id.as_named_param(),
        ]
    }

    fn from_row(row: &Row) -> Self {
        // TODO: get_checked

        let node_id: i64 = row.get("node_id");
        let node_type: i64 = row.get("node_type");

        match node_type {
            0 => SqliteNode::Normal(node_id.into()),
            1 => SqliteNode::Learner(node_id.into()),
            _ => panic!("Unexpected node_type."),
        }
    }

    // TODO: query
    // TODO: insert
    // TODO: delete
}

impl From<SqliteConfState> for Vec<SqliteNode> {
    fn from(sqlite_conf_state: SqliteConfState) -> Self {
        unimplemented!()
    }
}

impl From<Vec<SqliteNode>> for SqliteConfState {
    fn from(sqlite_nodes: Vec<SqliteNode>) -> Self {
        unimplemented!()
    }
}
