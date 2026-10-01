-- Registration verification links use the same challenge lifecycle as login OTPs,
-- but store a cryptographic token hash instead of a numeric verification code hash.
ALTER TABLE auth_challenges
    CHANGE COLUMN code_hash secret_hash BINARY(32) NOT NULL;
