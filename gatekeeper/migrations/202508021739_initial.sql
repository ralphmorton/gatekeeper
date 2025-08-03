CREATE TABLE nodes (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    node TEXT NOT NULL,
    superadmin INTEGER NOT NULL,
    created TEXT NOT NULL
);

CREATE UNIQUE INDEX ix_nodes_name ON nodes(name);
CREATE UNIQUE INDEX ix_nodes_node ON nodes(node);

CREATE TABLE roles (
    id INTEGER PRIMARY KEY,
    role TEXT NOT NULL,
    created TEXT NOT NULL
);

CREATE UNIQUE INDEX ix_roles_role ON roles(role);

CREATE TABLE node_roles (
    id INTEGER PRIMARY KEY,
    node_id INTEGER NOT NULL,
    role_id INTEGER NOT NULL,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX ix_node_roles_node_role ON node_roles (node_id, role_id);
