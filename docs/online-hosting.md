# Hosting a room on Ubuntu

Keep the source at `/srv/BongoCat` on Ubuntu 22.04 x86_64. Docker runs the room, Caddy runs on the host, and systemd handles boot startup and restarts. No domain, certificate, or database is required; the public room uses anonymous, unencrypted WebSocket connections.

Install [Docker Engine and the Compose plugin](https://docs.docker.com/engine/install/ubuntu/) and [Caddy](https://caddyserver.com/docs/install#debian-ubuntu-raspbian). Compose 2.24.4 or newer is required to replace the complete port list. Docker must be installed at `/usr/bin/docker`. This guide uses the system Docker service rather than rootless Docker.

## Source and configuration

Place the current source, including the `/bongocat` changes and `deploy/`, at `/srv/BongoCat`. Uncommitted or unpushed changes are not included in a remote clone. The directory must contain `Dockerfile`, `compose.yaml`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `crates/`, and `deploy/`.

Run all Compose commands from the source directory:

```sh
cd /srv/BongoCat
```

Expected: the working directory is `/srv/BongoCat`.

```sh
docker compose version
```

Expected: Compose 2.24.4 or newer.

```sh
sudo docker compose -f compose.yaml -f deploy/compose.caddy.yaml config
```

Expected: project `bongocat-room`, one published TCP port mapping from `127.0.0.1:18080` to container port 8080, and `restart: no`. The override inherits resource and logging limits while replacing the entire public port list. Always supply both `-f` arguments.

To change the host port, set `BONGOCAT_ROOM_PORT=YOUR_PORT` in the source directory's `.env` and update the Caddy upstream accordingly. The container still listens on `0.0.0.0:8080`; its `/health` check does not change.

## Build and systemd

```sh
sudo docker compose -f compose.yaml -f deploy/compose.caddy.yaml build room
```

Expected: a successful room image build. The first build downloads dependencies and compiles Rust, so it can take some time. The host does not need Rust. Service startup uses the existing image without building or downloading.

```sh
sudo install -m 0644 deploy/bongocat-room.service /etc/systemd/system/bongocat-room.service
```

Expected: the service file is installed. Change its `WorkingDirectory` first if the source is not at `/srv/BongoCat`.

```sh
sudo systemd-analyze verify /etc/systemd/system/bongocat-room.service
```

Expected: no service-file errors.

```sh
sudo systemctl daemon-reload
```

Expected: systemd reloads the service definition.

```sh
sudo systemctl enable --now docker.service
```

Expected: Docker is running and enabled at boot.

```sh
sudo systemctl enable --now bongocat-room.service
```

Expected: the room starts and is enabled at boot. Closing SSH does not stop it.

```sh
systemctl status bongocat-room.service --no-pager
```

Expected: `active (running)`. systemd supervises foreground Compose and restarts it after five seconds when the container exits. Docker's restart policy is disabled. A systemctl restart of Docker also restarts the room; explicitly stopping the room does not trigger automatic restart. Running status does not replace the health check below.

```sh
curl --fail --retry 10 --retry-delay 1 --retry-connrefused http://127.0.0.1:18080/health
```

Expected: `ok`.

## Caddy and clients

Use `deploy/Caddyfile.example` as a reference and add the following to `/etc/caddy/Caddyfile`, replacing the example IP with your public IP. If that IP already has a site block, merge only the `reverse_proxy` line into it:

```caddyfile
http://203.0.113.10 {
    reverse_proxy /bongocat 127.0.0.1:18080
}
```

Keep the path unchanged: the server accepts `/bongocat` directly. Existing `/ws` addresses still work when that path is reachable, but this proxy exposes only `/bongocat`.

```sh
sudo caddy validate --config /etc/caddy/Caddyfile
```

Expected: configuration validation succeeds.

```sh
sudo systemctl enable --now caddy.service
```

Expected: Caddy is running and enabled at boot.

```sh
sudo systemctl reload caddy.service
```

Expected: Caddy loads the proxy configuration.

Allow TCP 80 in the cloud security group and host firewall. Do not expose 18080 or 8080 to the public network. In a Windows client containing these changes, open **Settings → Online**, enter `ws://YOUR_PUBLIC_IP/bongocat`, and click **Join**. Editing the address does not change an active connection; click **Leave** or **Cancel** before joining another address.

## Logs, updates, and stopping

```sh
sudo journalctl -u bongocat-room.service -n 100 --no-pager
```

Expected: service startup, restart, and foreground Compose output.

```sh
sudo docker compose -f compose.yaml -f deploy/compose.caddy.yaml ps
```

Expected: healthy status and a port bound only to loopback.

```sh
sudo docker compose -f compose.yaml -f deploy/compose.caddy.yaml logs --tail=100 room
```

Expected: container logs without keys, coordinates, or per-action messages. Docker retains at most three 10 MB log files. Foreground logs also go to journald, whose retention is controlled by the host's journald configuration.

After updating the source in `/srv/BongoCat`, run the `build room` command above again. After a successful build, restart:

```sh
sudo systemctl restart bongocat-room.service
```

Expected: the new image is used and clients disconnect and rejoin. After changing the service definition, also repeat install and daemon-reload.

```sh
sudo systemctl stop bongocat-room.service
```

Expected: the container stops gracefully without automatic restart. Source, images, and containers are retained. Use `systemctl start` to start again.

```sh
sudo systemctl disable --now bongocat-room.service
```

Expected: the room stops and its boot and Docker-start associations are disabled. Do not run a separate `docker compose up -d` management path for this deployment.

The room supports eight people including yourself. A full room requires manual retry. Network failures reconnect automatically and may change temporary IDs. Server restarts clear membership; room state and actions are not persisted. A persistent systemd service does not persist room data. Two real clients and host-reboot acceptance still require machine validation.
