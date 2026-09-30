-- The sidebar is rendered from the menu registry, so an administration page needs a
-- registry row to appear there. Both access flags stay clear because the entry is
-- admin-only: system admins always have full access.
INSERT INTO menus
    (id, parent_id, name, path, icon, allow_user, allow_premium, sort_order, is_active, created_at, updated_at)
VALUES
    (UNHEX(REPLACE('0199a1b2-0007-7000-8000-000000000001', '-', '')), NULL, 'Menu permissions', '/admin/menus', 'shield-check', 0, 0, 95, 1, 0, 0);
