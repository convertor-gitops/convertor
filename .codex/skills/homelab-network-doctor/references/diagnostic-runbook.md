# Diagnostic Runbook

## Table of Contents

- Topology facts
- Flow model
- Symptom routing
- Read-only probes
- Repair boundaries
- Report format

## Topology Facts

Use the live repository as the source of truth when it differs from this reference.

```text
Internet -> bridge modem -> GS7 main router 10.0.0.1 -> GT-AC5300 10.0.1.1
                                                        |
                                                        10.0.1.0/24 homelab LAN
```

Important hosts:

| IP | Host | Role |
| --- | --- | --- |
| 10.0.1.1 | gt-ac5300 | Sub-router; must route 100.64.0.0/10 via 10.0.1.7 |
| 10.0.1.5 | truenas | NFS data layer for mihomo, tailscale, haproxy, npm |
| 10.0.1.6 | infra | NPM, HAProxy, traefik-internal, API entry points |
| 10.0.1.7 | tailscale | Tailscale subnet router advertising 10.0.1.0/24 |
| 10.0.1.8 | mihomo | Transparent proxy gateway and DNS |
| 10.0.1.101 | node-home-control-1 | Talos control plane |
| 10.0.1.151-.153 | node-home-worker-51-.53 | Home Talos workers |
| 167.179.111.233 / 100.64.100.31 | node-vultr-worker-101 | Vultr Talos worker |

Talos nodes do not run SSH. Ignore stale SSH aliases such as `node-home-*` or `node-vultr-1`.

## Flow Model

Keep these paths separate.

| Flow | Path | Common failure |
| --- | --- | --- |
| F1 home device or Pod to internet | client -> 10.0.1.8 mihomo -> internet | mihomo service, NFS mount, DNS/sniffer/rules |
| F2 Vultr to internet | Vultr public network direct | Vultr network or DNS |
| F3 home to Vultr | home -> mihomo DIRECT 100.64/10 -> gt-ac5300 route -> tailscale -> Vultr | missing route, subnet router logged out, peer offline |
| F4 Vultr to home | Vultr -> tailscale -> 10.0.1.7 -> LAN | asymmetric return path, missing NOTRACK on mihomo |
| F5 home Pod DNS | Pod -> kube-dns -> mihomo | fake-ip confusion; use curl, not dig alone |
| F6 Vultr Pod DNS | Pod -> coredns-vultr -> public DNS | coredns-vultr scheduling/Corefile |
| E2E | API, Talos, Ingress, workloads | infra HAProxy, Talos API, K8s readiness |
| NPM | public or internal HTTP entry | NPM/NFS/container/traefik backend |

## Symptom Routing

| Symptom | Start with |
| --- | --- |
| Vultr node NotReady or `talosctl -n 100.64.100.31` fails | F3/F4 plus Talos API probes |
| home Pod cannot reach Vultr Pod | F3/F4, gt-ac5300 route, tailscale peer, mihomo NOTRACK |
| Vultr Pod cannot resolve or reach internet | F2/F6, coredns-vultr, Vultr node resolvers/routes |
| home Pod DNS returns 198.18.x.x | F5; verify with curl before treating as failure |
| kubectl/talosctl is slow or unavailable | infra HAProxy stats, control-plane Talos health, route to endpoint |
| NPM or Ingress 502 | NPM/infra, HAProxy stats, traefik backend health |

## Read-Only Probes

Use only the probes relevant to the locked flow. Add `sandbox_permissions=require_escalated` when local sandboxing blocks SSH, kubectl, or talosctl.

### K8s

```bash
kubectl get nodes -o wide
kubectl describe node node-vultr-worker-101
kubectl get pods -A -o wide --field-selector spec.nodeName=node-vultr-worker-101
kubectl -n kube-system get pods -o wide -l k8s-app=coredns-vultr
```

### Talos

```bash
TC=talos/homelab/base/talosconfig
talosctl --talosconfig "$TC" -n 100.64.100.31 version
talosctl --talosconfig "$TC" -n 100.64.100.31 services
talosctl --talosconfig "$TC" -n 100.64.100.31 get nodeaddresses
talosctl --talosconfig "$TC" -n 100.64.100.31 get routes
talosctl --talosconfig "$TC" -n 167.179.111.233 version
```

Interpretation:

- `no route to host` to `100.64.100.31:50000`: usually local tailnet/subnet route problem.
- `i/o timeout` to `100.64.100.31:50000` after `tailscale ping` works: suspect Vultr Talos node-local API/kubelet problem.
- `authentication handshake failed: EOF` to public `167.179.111.233`: do not assume graceful `talosctl reboot` will work.

### tailscale subnet router

```bash
ssh tailscale 'set +e
echo --- status; sudo tailscale status
echo --- ping vultr; sudo tailscale ping --c 3 100.64.100.31
echo --- prefs; sudo tailscale debug prefs | grep -iE "WantRunning|LoggedOut|AdvertiseRoutes|RouteAll"
echo --- routes; ip -4 route
echo --- units; systemctl is-active tailscaled tailscale-autoconnect
echo --- auth shape; if [ -f /srv/tailscale/tailscale/auth.env ]; then grep -E "^(TS_AUTHKEY|TS_ROUTES)=" /srv/tailscale/tailscale/auth.env | sed -E "s/^(TS_AUTHKEY)=.*/\1=<redacted>/"; else echo missing; fi'
```

If `tailscale-autoconnect` fails, inspect without printing secrets:

```bash
ssh tailscale 'systemctl status tailscaled tailscale-autoconnect --no-pager --lines=80'
ssh tailscale 'journalctl -u tailscaled -u tailscale-autoconnect -n 100 --no-pager'
```

### gt-ac5300 route

```bash
ssh gt-ac5300 'echo --- 100.64 route; ip route show 100.64.0.0/10
echo --- arp; ip neigh | grep -E "10.0.1.7|10.0.1.8"'
```

Expected route:

```text
100.64.0.0/10 via 10.0.1.7 dev br0 metric 1
```

### mihomo and NOTRACK

```bash
ssh mihomo 'set +e
echo --- route; ip -4 route
echo --- sysctl; sysctl net.ipv4.ip_forward net.ipv4.conf.all.rp_filter
echo --- ts_raw; sudo nft list table inet ts_raw
echo --- mihomo; systemctl is-active mihomo nftables-ts-notrack
echo --- ports; sudo ss -lntup | grep -E ":53 |:7890|:7891|:9090|:7892"'
```

For F4, check whether NOTRACK counters increase across repeated probes.

### infra and HAProxy

```bash
ssh infra 'set +e
echo --- docker; sudo docker ps --format "table {{.Names}}\t{{.Status}}"
echo --- mounts; mountpoint /srv/npm; mountpoint /srv/haproxy
echo --- main haproxy; curl -s "http://127.0.0.1:8404/stats;csv" | awk -F, "NR==1 || (\$2!=\"FRONTEND\" && \$2!=\"BACKEND\" && \$1!=\"\") {print \$1,\$2,\$18,\$37}" | column -t
echo --- traefik-internal; curl -s "http://127.0.0.1:8405/stats;csv" | awk -F, "NR==1 || (\$2!=\"FRONTEND\" && \$2!=\"BACKEND\" && \$1!=\"\") {print \$1,\$2,\$18,\$37}" | column -t'
```

HAProxy may have expected DOWN slots when the config reserves more control-plane nodes than currently exist. Compare with actual control-plane node count.

## Repair Boundaries

Safe after user asks to fix:

- Restart `tailscale-autoconnect` if auth.env exists and the service is merely inactive.
- Run interactive `tailscale up` on `tailscale` when the user can approve the login URL.
- Apply a temporary route only if the user explicitly approves router mutation.

Coordinate or ask first:

- Vultr power reboot or API token use.
- Writing a fresh `TS_AUTHKEY`.
- Any production-impacting firewall, route, restart, or Kubernetes mutation.

Never treat auth keys as permanent. Tailscale auth keys expire; for persistent subnet routers, prefer disabling machine key expiry or using a tagged device.

## Report Format

Use concise Chinese output:

```text
症状摘要：source -> target, symptom, start/change if known
锁定流向：F3/F4/E2E...
已验证事实：PASS/FAIL/SKIP with command signal
已排除分支：...
根因：...
修复：...
复验：...
残余风险/后续：...
```
