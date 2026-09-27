CREATE TABLE users (
    id BINARY(16) NOT NULL,
    email VARCHAR(320) NOT NULL,
    created_at BIGINT NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_users_email (email)
) ENGINE=InnoDB;

CREATE TABLE user_roles (
    user_id BINARY(16) NOT NULL,
    role VARCHAR(32) NOT NULL,
    PRIMARY KEY (user_id, role),
    CONSTRAINT fk_user_roles_user
        FOREIGN KEY (user_id) REFERENCES users (id)
        ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE TABLE user_entitlements (
    user_id BINARY(16) NOT NULL,
    entitlement VARCHAR(32) NOT NULL,
    active TINYINT(1) NOT NULL DEFAULT 1,
    expires_at BIGINT NULL,
    PRIMARY KEY (user_id, entitlement),
    CONSTRAINT fk_user_entitlements_user
        FOREIGN KEY (user_id) REFERENCES users (id)
        ON DELETE CASCADE
) ENGINE=InnoDB;

CREATE TABLE auth_challenges (
    id BINARY(16) NOT NULL,
    email VARCHAR(320) NOT NULL,
    purpose VARCHAR(32) NOT NULL,
    code_hash BINARY(32) NOT NULL,
    attempts TINYINT UNSIGNED NOT NULL DEFAULT 0,
    max_attempts TINYINT UNSIGNED NOT NULL,
    expires_at BIGINT NOT NULL,
    created_at BIGINT NOT NULL,
    consumed_at BIGINT NULL,
    PRIMARY KEY (id),
    KEY ix_auth_challenges_email_purpose_created (email, purpose, created_at)
) ENGINE=InnoDB;

CREATE TABLE auth_sessions (
    id BINARY(16) NOT NULL,
    user_id BINARY(16) NOT NULL,
    token_hash BINARY(32) NOT NULL,
    created_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    revoked_at BIGINT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_auth_sessions_token_hash (token_hash),
    KEY ix_auth_sessions_user (user_id),
    CONSTRAINT fk_auth_sessions_user
        FOREIGN KEY (user_id) REFERENCES users (id)
        ON DELETE CASCADE
) ENGINE=InnoDB;
