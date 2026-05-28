# Running boar in Docker

Two-container setup over a Docker bridge instead of the legacy `ns_s1..ns_c2`
netns chain. Useful when you don't want to touch the host's network
namespaces directly, or want to run on a machine without all the host-side
networking tools wired up.

## Prereqs

```
sudo modprobe ifb sch_netem
git submodule update --init
```

`ifb` and `sch_netem` need to be loaded on the **host** kernel; the entrypoint
can't load them from inside the container. The submodule populates
`deps/quiche`, which `build.rs` compiles at image build time.

## Quickstart

```
make docker-up         # build, server up, one client run, tear down
make docker-clean      # tear down and wipe ./reports
```

Reports land in `./reports/<uuid>/` owned by your host user (chowned on exit
by the entrypoint).

## Iterative use

```
make docker-server                          # leave server detached
make docker-run ARGS="-d 50mb -c 5"         # repeat client runs
make docker-run ARGS="--delay-ms 100"       # tweak shaping per run
```

## Topology

```
┌─ boar-server ─┐    docker bridge     ┌─ boar-client ─┐
│ async_http3   │◄──── veth ─────────► │ quiche-client │
│ :9999         │  (tc/netem on        │  + boar       │
└───────────────┘   client eth0)       └───────────────┘
```

Shaping lives on the client's eth0 ingress via IFB so it lands on the
download path — semantically equivalent to the legacy netem-on-`ns_m2` hop.

## Configuration

| Env / flag | Default | What it does |
| --- | --- | --- |
| `BOAR_SERVER_CCA` (env) | `bbr2_gcongestion` | Server-side CCA. Set on the `boar-server` container at compose time. |
| `HOST_UID` / `HOST_GID` (env) | `1000` / `1000` | Ownership applied to `./reports` on exit. Override if your UID isn't 1000. |
| `BOAR_STARTUP_WAIT` (env) | `2` | Seconds the client waits before launching, in lieu of a real server-readiness probe. |
| `--delay-ms` / `--rate-mbit` / `--loss-model` | `50` / `20` / `random 0%` | tc parameters applied per run by `boar`. |
| `-d` / `-c` | `1mb` / `2` | Download size and iteration count. |

## Caveats

- **`--cc-algorithm` is label-only.** Boar doesn't start the server in docker
  mode, so its actual CCA comes from `BOAR_SERVER_CCA`. To compare CCAs,
  restart the server with a new env: `BOAR_SERVER_CCA=cubic make docker-up`.
- **`StartupExit` is `NaN`.** That metric was built from the server's
  in-process stderr in host mode; it's not yet wired through a shared log
  volume in docker.
- **Server readiness is a 2s sleep**, not a real probe. Bump
  `BOAR_STARTUP_WAIT` if you see early-connect failures.
