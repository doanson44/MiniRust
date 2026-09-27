ALTER TABLE users
    ADD COLUMN full_name VARCHAR(200) NULL AFTER email,
    ADD COLUMN avatar_url VARCHAR(2048) NULL AFTER full_name,
    ADD COLUMN locked_at BIGINT NULL AFTER avatar_url;
