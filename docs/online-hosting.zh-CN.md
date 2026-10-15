# Ubuntu 房间部署

使用 Ubuntu 22.04 x86_64，源码放在 `/srv/BongoCat`，Docker 运行房间服务，宿主机 Caddy 对外代理，systemd 负责开机启动和异常重启。无需域名、证书或数据库；连接是匿名明文 WebSocket。

先安装 [Docker Engine 与 Compose 插件](https://docs.docker.com/engine/install/ubuntu/)及 [Caddy](https://caddyserver.com/docs/install#debian-ubuntu-raspbian)。Compose 需为 2.24.4 或更高版本，以支持端口完整替换；`docker` 可执行文件应为 `/usr/bin/docker`。本方案使用系统级 Docker 服务，不适用于 rootless Docker。

## 源码与配置

将包含 `/bongocat` 修改及 `deploy/` 目录的当前源码放到 `/srv/BongoCat`。尚未提交或推送的修改不会出现在远端克隆中。目录中应有 `Dockerfile`、`compose.yaml`、`Cargo.toml`、`Cargo.lock`、`rust-toolchain.toml`、`crates/` 和 `deploy/`。

所有 Compose 命令在源码目录执行：

```sh
cd /srv/BongoCat
```

预期：当前目录为 `/srv/BongoCat`。

```sh
docker compose version
```

预期：Compose 2.24.4 或更高版本。

```sh
sudo docker compose -f compose.yaml -f deploy/compose.caddy.yaml config
```

预期：项目名称为 `bongocat-room`，只有 `127.0.0.1:18080` 映射到容器 TCP 8080，`restart` 为 `no`。覆盖文件复用基础资源限制与日志配置，并完整替换公开端口列表。不要省略其中一个 `-f` 参数。

如需换宿主机端口，在源码目录的 `.env` 中设置 `BONGOCAT_ROOM_PORT=其他端口`，并同步修改 Caddy 的转发目标。容器仍监听 `0.0.0.0:8080`，`/health` 检查不变。

## 构建与 systemd

```sh
sudo docker compose -f compose.yaml -f deploy/compose.caddy.yaml build room
```

预期：房间镜像构建成功。首次会下载依赖并编译 Rust，耗时较长；宿主机无需安装 Rust。systemd 启动时只运行已有镜像，不执行构建或下载。

```sh
sudo install -m 0644 deploy/bongocat-room.service /etc/systemd/system/bongocat-room.service
```

预期：安装服务文件。源码目录不是 `/srv/BongoCat` 时，先修改服务文件的 `WorkingDirectory`。

```sh
sudo systemd-analyze verify /etc/systemd/system/bongocat-room.service
```

预期：没有服务文件错误。

```sh
sudo systemctl daemon-reload
```

预期：systemd 重新读取服务文件。

```sh
sudo systemctl enable --now docker.service
```

预期：Docker 正在运行并设置为开机启动。

```sh
sudo systemctl enable --now bongocat-room.service
```

预期：房间服务启动并设置为开机启动；退出 SSH 不影响运行。

```sh
systemctl status bongocat-room.service --no-pager
```

预期：`active (running)`。systemd 持续监督前台 Compose 进程，容器退出时五秒后重启；Docker 自身重启策略关闭。通过 systemctl 重启 Docker 时，房间服务跟随重启；手动停止房间服务不会触发自动拉起。状态为 running 不代替下面的健康检查。

```sh
curl --fail --retry 10 --retry-delay 1 --retry-connrefused http://127.0.0.1:18080/health
```

预期：返回 `ok`。

## Caddy 与连接

参考 `deploy/Caddyfile.example`，在 `/etc/caddy/Caddyfile` 中添加以下配置，将示例 IP 换为服务器公网 IP；若同一 IP 的站点块已存在，只将 `reverse_proxy` 行合入该块：

```caddyfile
http://203.0.113.10 {
    reverse_proxy /bongocat 127.0.0.1:18080
}
```

路径原样转发，服务端直接接受 `/bongocat`，无需改写。旧 `/ws` 地址在路径可达时仍可使用，但该代理只提供 `/bongocat`。

```sh
sudo caddy validate --config /etc/caddy/Caddyfile
```

预期：配置验证通过。

```sh
sudo systemctl enable --now caddy.service
```

预期：Caddy 正在运行并设置为开机启动。

```sh
sudo systemctl reload caddy.service
```

预期：Caddy 加载代理配置。

通过云平台安全组和服务器防火墙开放 TCP 80；无需对公网开放 18080 或 8080。在包含本次修改的 Windows 客户端“设置 → 联机”输入 `ws://服务器公网IP/bongocat`，点击“加入”。编辑地址不会改变现有连接；已连接或正在重连时，先“离开”或“取消”，再“加入”。

## 日志、更新与停止

```sh
sudo journalctl -u bongocat-room.service -n 100 --no-pager
```

预期：显示 systemd 启动、重启及前台 Compose 输出。

```sh
sudo docker compose -f compose.yaml -f deploy/compose.caddy.yaml ps
```

预期：显示 healthy 和仅绑定本机的端口。

```sh
sudo docker compose -f compose.yaml -f deploy/compose.caddy.yaml logs --tail=100 room
```

预期：显示容器日志，不含按键、坐标或逐条动作；Docker 日志最多三个 10 MB 文件。前台日志也进入 journald，其保留量由服务器的 journald 配置管理。

更新 `/srv/BongoCat` 中的源码后，先重新执行上面的 `build room` 命令，成功后再重启：

```sh
sudo systemctl restart bongocat-room.service
```

预期：应用新镜像，客户端断开后重新加入。服务文件更新后，还需重新执行 install 和 daemon-reload。

```sh
sudo systemctl stop bongocat-room.service
```

预期：优雅停止容器，不自动重启；源码、镜像和容器均保留。之后用 `systemctl start` 启动。

```sh
sudo systemctl disable --now bongocat-room.service
```

预期：停止服务并取消其开机启动及 Docker 启动关联。此部署不要用独立的 `docker compose up -d` 启动第二套管理流程。

房间最多八人，包括自己；满员后手动重试。网络失败自动重连，临时 ID 可能改变。服务端重启会清空成员，不持久化房间或动作；systemd 持久运行不代表保存房间数据。双真实客户端和服务器重启验收仍需实机验证。
