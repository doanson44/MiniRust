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

