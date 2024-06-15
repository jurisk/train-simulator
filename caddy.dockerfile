FROM caddy:latest

EXPOSE 80/tcp
EXPOSE 443/tcp

COPY Caddyfile /etc/caddy/Caddyfile

ENTRYPOINT ["caddy", "run", "--config", "/etc/caddy/Caddyfile"]
