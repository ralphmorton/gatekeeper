use bincode::{Decode, Encode};
use clap::Subcommand;

use crate::db;

pub const ALPN: &[u8] = b"gatekeeper";

#[derive(Clone, Debug)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

#[derive(Clone, Debug, Decode, Encode, Subcommand)]
pub enum Cmd {
    /// List roles
    Roles,
    /// List nodes
    Nodes,
    /// List roles granted to the specified node
    NodeRoles {
        /// Node public key
        node: String,
    },
    /// Create a new node
    CreateNode {
        /// Node name (unique)
        name: String,
        /// Node public key (unique)
        node: String,
        /// Grant superadmin access to node?
        superadmin: bool,
    },
    /// Delete a node
    DeleteNode {
        /// Node public key
        node: String,
    },
    /// Grant a role to a node
    GrantRole {
        /// Node public key
        node: String,
        /// Role
        role: String,
    },
    /// Revoke a previously granted role
    RevokeRole {
        /// Node public key
        node: String,
        /// Role
        role: String,
    },
}

#[derive(Clone, Debug, Decode, Encode)]
pub struct Node {
    pub name: String,
    pub node: String,
    pub superadmin: bool,
}

impl From<db::Node> for Node {
    fn from(value: db::Node) -> Self {
        Self {
            name: value.name,
            node: value.node,
            superadmin: value.superadmin,
        }
    }
}
