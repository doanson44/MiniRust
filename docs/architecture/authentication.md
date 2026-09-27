# Authentication Architecture

MiniRust uses passwordless email verification codes for the initial authentication flow.

## Account model

A registered account is the baseline user state. Premium and Admin are separate access dimensions:

- normal: registered user without the Premium entitlement and without the Admin system role
- premium: active premium entitlement, expected to be granted from the future billing/payment bounded context
- admin: admin system role, granted only by an administrative operation

A user can therefore be both Premium and Admin. The client cannot select either access level during registration.

## Authentication flow

### Registration

1. POST /api/v1/auth/register/request-code
2. The backend normalizes the email and creates a short-lived, single-use challenge.
3. The code is sent through the EmailSender application port.
4. POST /api/v1/auth/register/verify-code
5. Successful verification creates the user and a server-side session in one MariaDB transaction.

### Login

1. POST /api/v1/auth/login/request-code
2. POST /api/v1/auth/login/verify-code
3. Successful verification creates a server-side session.

Registration and login request endpoints return the same public success shape for existing/non-existing accounts to reduce email-account enumeration.

## OTP security

- six numeric digits
- ten-minute lifetime
- five failed attempts per challenge
- one active challenge per email/purpose
- code is never persisted in plaintext
- code verification uses HMAC-SHA256 with MINIRUST_AUTH_SECRET
- OTPs are generated from the operating system cryptographic random source
- OTP values must never be logged

HMAC is used as a keyed integrity mechanism; the secret must be at least 32 bytes. See RFC 2104 and the Rust HMAC API.

## Session security

The session is server-side in MariaDB. The browser receives only an opaque random token:

- raw token is held in an HttpOnly cookie
- only the SHA-256 token hash is stored in MariaDB
- cookie uses SameSite=Lax
- production cookies use Secure
- session lifetime is thirty days
- logout revokes the server-side session

The opaque session token is intentionally not the UUIDv7 entity identity. UUIDv7 remains the domain identity strategy; session tokens require independent cryptographic randomness.

## Persistence

The first migration creates:

- users
- user_roles
- user_entitlements
- auth_challenges
- auth_sessions

MariaDB remains the only persistence dependency. SQLx's MySQL driver supports MariaDB, and SQLx migrations can be embedded and executed during application startup.

## Current boundary

The application layer owns the authentication workflow and exposes AuthRepository and EmailSender ports. The database crate implements AuthRepository; an external email provider will implement EmailSender.

The current API wires UnavailableEmailSender intentionally. Authentication persistence and HTTP contracts are therefore in place, but actual email delivery is not enabled until a concrete provider adapter is added. No OTP is printed to logs as a development shortcut.


## Bootstrap admin

On API startup, after migrations complete, the backend idempotently seeds the bootstrap account:

- email: admin@minirust.local
- system role: admin
- login OTP: 123456
- OTP challenge is recreated on each API startup so the bootstrap OTP is available again after restart

This is a development/bootstrap credential and is intentionally unsafe for production. A production deployment must replace the fixed OTP bootstrap with the normal email-delivery flow and remove or disable the bootstrap account.


## Admin user management

Administrative user management is exposed under /api/v1/admin/users and requires an authenticated session whose current user has the admin system role. Authorization is checked server-side for every request.

Endpoints:

- GET /api/v1/admin/users — list users
- POST /api/v1/admin/users — create a normal user from an email
- GET /api/v1/admin/users/{email} — read a user by email
- PATCH /api/v1/admin/users/{email} — change the user's email
- DELETE /api/v1/admin/users/{email} — delete the user
- PUT /api/v1/admin/users/{email}/role — assign or remove the admin system role

The role endpoint accepts:

- admin — add the admin system role
- none — remove the admin system role

Premium is deliberately not represented as a role. It remains a user entitlement and will eventually be managed by the billing/payment bounded context.

The bootstrap admin account is protected from email changes, deletion, and removal of its admin role.

Premium entitlement administration:

- PUT /api/v1/admin/users/{email}/entitlements/premium — assign or revoke Premium
- `active: true` assigns Premium; `active: false` revokes it
- `expires_at` is an optional Unix timestamp; when present it must be in the future
- Premium remains an entitlement in `user_entitlements`, not a system role


## Self-service profile and account lifecycle

An authenticated user may manage their own profile and account lifecycle:

- PATCH /api/v1/users/me — update full name and avatar URL
- POST /api/v1/users/me/lock — lock the current account and revoke all active sessions
- DELETE /api/v1/users/me — permanently delete the current account and revoke its persisted identity through database cascade

Profile fields are intentionally metadata references at this stage: avatar is represented by an HTTPS URL because the repository does not yet define an object/file storage port. Binary image upload is therefore not part of this API contract.

A locked account cannot authenticate. Administrative unlock is available through POST /api/v1/admin/users/{email}/unlock. The bootstrap admin account remains protected from self-lock and self-delete.
