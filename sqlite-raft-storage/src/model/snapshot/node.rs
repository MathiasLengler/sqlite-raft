use error::Result;
use model::core::CoreId;
use model::core::CoreTx;
use raft::eraftpb::ConfState;
use rusqlite::Result as RusqliteResult;
use rusqlite::Row;
use rusqlite::types::ToSql;

pub struct SqliteConfState {
    nodes: Vec<NodeId>,
    learners: Vec<NodeId>,
}

impl SqliteConfState {
    pub fn query(core_tx: &CoreTx) -> Result<SqliteConfState> {
        let sqlite_nodes = SqliteNode::query_all(&core_tx)?;
        Ok(sqlite_nodes.into())
    }

    pub fn insert_or_replace(&self, core_tx: &CoreTx) -> Result<()> {
        SqliteNode::delete_all(&core_tx)?;

        let sqlite_nodes: Vec<SqliteNode> = self.into();

        SqliteNode::insert_all(&core_tx, &sqlite_nodes)?;

        Ok(())
    }
}

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

struct SqliteNode {
    node_id: NodeId,
    node_type: NodeType,
}

impl SqliteNode {
    const SQL_QUERY: &'static str =
        include_str!("../../../res/sql/node/query.sql");
    const SQL_INSERT: &'static str =
        include_str!("../../../res/sql/node/insert.sql");
    const SQL_DELETE: &'static str =
        include_str!("../../../res/sql/node/delete.sql");

    fn as_row_tuple(&self) -> (i64, i64) {
        (self.node_id.into(), self.node_type.into())
    }

    pub fn named_params<'a>(node_id: &'a i64, node_type: &'a i64, core_id: &'a CoreId) -> [(&'static str, &'a dyn ToSql); 3] {
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

        SqliteNode {
            node_id: node_id.into(),
            node_type: node_type.into(),
        }
    }

    pub fn query_all(core_tx: &CoreTx) -> Result<Vec<SqliteNode>> {
        let mut stmt = core_tx.tx().prepare(Self::SQL_QUERY)?;

        let rows = stmt.query_map_named(
            &[core_tx.core_id().as_named_param()],
            Self::from_row,
        )?;

        Ok(rows.collect::<RusqliteResult<Vec<_>>>()?)
    }

    pub fn insert(&self, core_tx: &CoreTx) -> Result<()> {
        let (node_id, node_type) = self.as_row_tuple();
        core_tx.tx().execute_named(Self::SQL_INSERT, &Self::named_params(&node_id, &node_type, &core_tx.core_id()))?;
        Ok(())
    }

    pub fn insert_all(core_tx: &CoreTx, nodes: &[Self]) -> Result<()> {
        for node in nodes {
            node.insert(&core_tx)?;
        }
        Ok(())
    }

    pub fn delete_all(core_tx: &CoreTx) -> Result<()> {
        core_tx.tx().execute_named(Self::SQL_DELETE, &[core_tx.core_id().as_named_param()])?;
        Ok(())
    }
}

impl<'a> From<&'a SqliteConfState> for Vec<SqliteNode> {
    fn from(sqlite_conf_state: &'a SqliteConfState) -> Self {
        sqlite_conf_state.nodes.iter().map(|node_id| SqliteNode {
            node_id: *node_id,
            node_type: NodeType::Normal,
        }).chain(sqlite_conf_state.learners.iter().map(|node_id| SqliteNode {
            node_id: *node_id,
            node_type: NodeType::Learner,
        })).collect()
    }
}

impl From<Vec<SqliteNode>> for SqliteConfState {
    fn from(sqlite_nodes: Vec<SqliteNode>) -> Self {
        SqliteConfState {
            nodes: sqlite_nodes.iter().filter_map(|sqlite_node| match sqlite_node.node_type {
                NodeType::Normal => Some(sqlite_node.node_id),
                NodeType::Learner => None,
            }).collect(),
            learners: sqlite_nodes.iter().filter_map(|sqlite_node| match sqlite_node.node_type {
                NodeType::Normal => None,
                NodeType::Learner => Some(sqlite_node.node_id),
            }).collect(),
        }
    }
}


#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
enum NodeType {
    Normal = 0,
    Learner = 1,
}

impl From<i64> for NodeType {
    fn from(i: i64) -> Self {
        match i {
            0 => NodeType::Normal,
            1 => NodeType::Learner,
            _ => panic!("Unexpected node_type."),
        }
    }
}

impl From<NodeType> for i64 {
    fn from(node_type: NodeType) -> Self {
        match node_type {
            NodeType::Normal => 0,
            NodeType::Learner => 1,
        }
    }
}
