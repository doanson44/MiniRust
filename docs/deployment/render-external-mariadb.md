# Deploy MiniRust on Render with external MariaDB

This deployment keeps MariaDB outside Render. Render runs three services:

- `minirust-api`: private Axum API, listening on `0.0.0.0:3000`.
- `minirust-web`: private Leptos SSR app, listening on `0.0.0.0:3001`.
- `minirust-edge`: public Caddy service, listening on Render's default HTTP port `10000`.

Caddy routes `/api/*` to the API and all other paths to the Web service. Render terminates inbound HTTPS at its edge and forwards HTTP to Caddy.

## Before creating the Blueprint

1. Find the region where the external MariaDB instance runs.
2. Set the same `region` value on all three services in `render.yaml`. Render defaults to Oregon when `region` is omitted; do not leave that default if the database is elsewhere.
3. Create the Blueprint from the repository root.
4. Choose a plan that supports private services. Private services do not have a Free plan.

## MariaDB connection settings

The Blueprint asks for the following database settings separately on both `minirust-api` and `minirust-web`:

- `MINIRUST_DB_HOST`: provider's TLS-enabled hostname
- `MINIRUST_DB_PORT`: provider's TLS-enabled TCP port
- `MINIRUST_DB_NAME`: database/schema name
- `MINIRUST_DB_USER`: dedicated application account
- `MINIRUST_DB_PASSWORD`: application account password
- `MINIRUST_DB_SSL_MODE=required`: require encrypted transport

MiniRust builds the SQLx URL from these values and percent-encodes username, password, and database components.

SQLx's `required` mode forces an encrypted connection and fails if TLS cannot be negotiated. It does not authenticate the server certificate. Once the provider's CA certificate is available, prefer `verify_identity` and configure the provider CA bundle for SQLx. Do not switch to that mode until the CA bundle and hostname requirements have been confirmed.

The optional `MINIRUST_DATABASE_URL` override still takes precedence over the component variables. If you use it, include `?ssl-mode=REQUIRED` in the URL and still provide the component variables because current production configuration validates them at startup.

Do not put credentials in this file or commit them to Git.

## Required secrets and settings

The API and Web bind settings are configured in the Blueprint:

- API: `MINIRUST_API_HOST=0.0.0.0`, `MINIRUST_API_PORT=3000`
- Web: `MINIRUST_WEB_HOST=0.0.0.0`, `MINIRUST_WEB_PORT=3001`

The Blueprint creates a shared environment group for:

- `MINIRUST_ENV=production`
- `MINIRUST_AUTH_SECRET` (generated once and shared by API and Web)
- `MINIRUST_DB_MAX_CONNECTIONS=5`

During initial Blueprint creation, supply the following secrets when prompted. Render prompts for `sync: false` values only during initial creation:

### API

- `MINIRUST_DB_HOST`
- `MINIRUST_DB_PORT`
- `MINIRUST_DB_NAME`
- `MINIRUST_DB_USER`
- `MINIRUST_DB_PASSWORD`
- `MINIRUST_ADMIN_EMAIL`
- `MINIRUST_ADMIN_OTP` (exactly six digits)
- `MINIRUST_WEB_URL` (the public HTTPS URL, e.g. `https://app.example.com`)

### Web

- `MINIRUST_DB_HOST`, `MINIRUST_DB_PORT`, `MINIRUST_DB_NAME`, `MINIRUST_DB_USER`, `MINIRUST_DB_PASSWORD` (same values as API)
- `MINIRUST_ADMIN_EMAIL` (same value as API)
- `MINIRUST_ADMIN_OTP` (same value as API)
- `MINIRUST_WEB_URL` (same value as API)

The API and Web must use the same database settings, TLS mode, authentication secret, admin email, admin OTP, and public web URL. The Blueprint shares the generated authentication secret and connection limit through an environment group; set the other duplicated values identically.

SMTP is optional. To enable email delivery, add `MINIRUST_SMTP_HOST`, `MINIRUST_SMTP_PORT`, `MINIRUST_SMTP_USERNAME`, `MINIRUST_SMTP_PASSWORD`, and `MINIRUST_SMTP_FROM_EMAIL` to the API service. If SMTP is not configured, email-based registration/login flows cannot deliver verification messages.

## Allowlist Render outbound IPs at the database provider

Both API and Web open direct MariaDB connections. For each service:

1. Open the service in Render Dashboard.
2. Select **Connect → Outbound**.
3. Copy the displayed outbound CIDR ranges for the selected Render region.
4. Add those ranges to the database provider's network allowlist for the MariaDB port.
5. Repeat after changing Render region or networking configuration.

Render's default outbound ranges are shared by services in the same region and are not exclusive static IPs. If the database provider requires individual static IP addresses rather than CIDR ranges, consider Render Dedicated Outbound IPs. Dedicated IPs require a Pro workspace or higher and incur additional charges.

Do not open MariaDB to `0.0.0.0/0`. Use a dedicated application database account with only the privileges MiniRust needs. Keep TLS enabled at the provider.

## Deployment order

1. Confirm external MariaDB is reachable from Render and its firewall permits the API/Web outbound ranges.
2. Create the Render Blueprint.
3. Wait for API and Web to deploy successfully. The API runs database migrations at startup; Web connects to the same database.
4. Add the public custom domain to `minirust-edge`.
5. Test:
   - `https://YOUR_DOMAIN/health`
   - `https://YOUR_DOMAIN/api/v1/hello`
   - login/registration and SMTP delivery
   - avatar upload and persistence

## Operational notes

- API and Web Dockerfiles already bind to `0.0.0.0` using their configured application ports. They are private services, so they do not need to bind to Render's public `PORT`.
- Caddy is the only public Render service. Render terminates external HTTPS; Caddy listens on HTTP port `10000` inside the Render service.
- The API's `/health` endpoint checks MariaDB. The Caddy health check uses Web's `/health` endpoint, which checks the Web process only.
- API uploads and logs are written to the container filesystem unless a persistent disk is configured. Add a persistent disk to the API service if uploaded files must survive deploys/restarts; mount it at `/var/lib/minirust` and set `MINIRUST_UPLOAD_DIR=/var/lib/minirust/uploads` and `MINIRUST_LOG_DIR=/var/lib/minirust/logs`.
- Render's default service region in a Blueprint is Oregon. Set the region deliberately before the first deploy because region is not freely changeable after service creation.
