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