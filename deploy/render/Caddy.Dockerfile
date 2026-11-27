FROM caddy:2-alpine

COPY deploy/render/Caddyfile /etc/caddy/Caddyfile

EXPOSE 10000
