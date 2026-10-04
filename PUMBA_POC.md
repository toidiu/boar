# Pumba POC

Sketch only — not wired up. The idea: drive [alexei-led/pumba](https://github.com/alexei-led/pumba)
against the boar containers to script chaos patterns beyond what `scripts/docker_tc.sh`
sets up as a static baseline.

Pumba talks to `/var/run/docker.sock` and shells out to `tc netem` inside the
target container. Same mechanism as `docker_tc.sh`, just orchestrated from
outside with a CLI for time-bounded scenarios.

## One-shot from host

No compose changes needed — just point Pumba at a running `boar-server`:

```sh
# 5% packet loss on boar-server's eth0 for 60s, then auto-reverts.
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
    gaiaadm/pumba:latest \
    pumba netem --duration 60s loss --percent 5 boar-server

# 200ms one-way latency for 2m.
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
    gaiaadm/pumba:latest \
    pumba netem --duration 2m delay --time 200 boar-server

# 1mbit/s cap (rate, not htb — just for comparison).
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
    gaiaadm/pumba:latest \
    pumba netem --duration 2m rate --rate 1mbit boar-server
```

Container kill / pause without netem:

```sh
# SIGKILL boar-server (boar-client should reconnect on the next iteration).
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
    gaiaadm/pumba:latest \
    pumba kill --signal SIGKILL boar-server

# Pause 30s mid-run to probe quiche-client's idle-timeout (--idle-timeout=5).
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
    gaiaadm/pumba:latest \
    pumba pause --duration 30s boar-server
```

## As a compose sidecar

If you want chaos as part of `make docker-up`:

```yaml
# Append to docker-compose.yml
  pumba:
    image: gaiaadm/pumba:latest
    container_name: boar-pumba
    depends_on: [boar-server]
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    # Example: 3% Gilbert-Elliott burst loss against boar-server for the run.
    command:
      - pumba
      - netem
      - --duration
      - 10m
      - loss
      - gemodel
      - --pg
      - "3"
      - --pb
      - "90"
      - boar-server
```

## Scenarios worth a try

| Goal | Pumba command |
| --- | --- |
| Steady 5% random loss on the download path | `pumba netem --duration 5m loss --percent 5 boar-server` |
| Bursty Gilbert-Elliott loss | `pumba netem --duration 5m loss gemodel --pg 1 --pb 90 boar-server` |
| Latency spike during transfer | `pumba netem --duration 30s delay --time 500 --jitter 100 boar-server` |
| Mid-run server pause (idle-timeout test) | `pumba pause --duration 10s boar-server` |
| HTTP/3 connection-migration probe | `pumba kill --signal SIGKILL boar-server` then restart |
| Concurrent rate cap stacked over our htb | `pumba netem --duration 2m rate --rate 5mbit boar-server` |

## Caveats

- **qdisc conflict.** `scripts/docker_tc.sh` already attaches qdiscs to
  `boar-client`'s `eth0` and `ifb0`. Pumba attaches its own `netem` root qdisc
  to whichever container it targets. If you point Pumba at `boar-client` the
  two will clobber each other. **Target `boar-server` instead** so the
  responsibilities stay disjoint (boar = client-side shaping, Pumba =
  server-side chaos events).
- **Pumba auto-reverts.** When `--duration` elapses, Pumba removes its qdisc.
  If Pumba dies mid-run the qdisc may linger — `docker exec boar-server tc
  qdisc del dev eth0 root` to clean up.
- **No `--pull` policy in the sketch.** Pin the image tag in real use.
- **Same UDP caveat as the proxy tools** — these work because they're
  L3/qdisc; TCP-only chaos proxies (Toxiproxy, Trixter) wouldn't touch QUIC.

## Open questions

- Do we want chaos described declaratively in `docker-compose.yml`, or as a
  separate `make docker-chaos SCENARIO=loss` invocation?
- Should `--cc-algorithm` comparison runs (Cubic vs BBR) loop through a fixed
  set of Pumba scenarios so reports are cross-comparable?
- Pumba's `re2:^boar-` regex lets you target multiple containers at once —
  useful if we ever add multi-client topologies.
