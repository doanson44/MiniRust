-- Rename the administration user-management menu and align its UI route.
UPDATE menus
SET name = 'User Management',
    path = '/admin/users'
WHERE path = '/admin';
