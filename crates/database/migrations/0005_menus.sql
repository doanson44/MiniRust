CREATE TABLE menus (
    id BINARY(16) NOT NULL,
    parent_id BINARY(16) NULL,
    name VARCHAR(200) NOT NULL,
    path VARCHAR(512) NOT NULL,
    icon VARCHAR(100) NULL,
    required_role VARCHAR(32) NOT NULL DEFAULT 'user',
    sort_order INT NOT NULL DEFAULT 0,
    is_active TINYINT(1) NOT NULL DEFAULT 1,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL,
    PRIMARY KEY (id),
    KEY ix_menus_parent (parent_id),
    KEY ix_menus_role_active_order (required_role, is_active, sort_order),
    CONSTRAINT fk_menus_parent
        FOREIGN KEY (parent_id) REFERENCES menus (id)
        ON DELETE SET NULL
) ENGINE=InnoDB;

INSERT INTO menus
    (id, parent_id, name, path, icon, required_role, sort_order, is_active, created_at, updated_at)
VALUES
    (UNHEX(REPLACE('01a0f1be-1215-77a2-ba65-aafb21c8e53e', '-', '')), NULL, 'Dashboard', '/app', 'layout-dashboard', 'user', 10, 1, 0, 0),
    (UNHEX(REPLACE('01a0f1be-1215-7469-88cb-19f9e99563c8', '-', '')), NULL, 'Profile', '/profile', 'user', 'user', 20, 1, 0, 0),
    (UNHEX(REPLACE('01a0f1be-1215-7330-ac8a-d28c0fbd52ee', '-', '')), NULL, 'Administration', '/admin', 'shield', 'admin', 90, 1, 0, 0);
