-- Replace the single required_role column with explicit per-tier access flags.
--
-- Menu visibility is matched exactly against the caller's tier:
--   - a normal account sees menus with allow_user = 1
--   - a premium account sees menus with allow_premium = 1
--   - a system admin always sees every menu
--
-- Premium remains a user entitlement and admin remains a system role; the flags
-- only describe which menus each tier may open.

ALTER TABLE menus DROP INDEX ix_menus_role_active_order;
ALTER TABLE menus
    ADD COLUMN allow_user TINYINT(1) NOT NULL DEFAULT 0 AFTER icon,
    ADD COLUMN allow_premium TINYINT(1) NOT NULL DEFAULT 0 AFTER allow_user;

UPDATE menus SET allow_user = 1, allow_premium = 1 WHERE required_role = 'user';

ALTER TABLE menus DROP COLUMN required_role;
ALTER TABLE menus ADD KEY ix_menus_active_order (is_active, sort_order);
