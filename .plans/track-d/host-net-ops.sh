#!/usr/bin/env bash
# Host network ops for match sessions — READY-TO-RUN PROPOSAL, NOT EXECUTED.
#
# RECON ONLY: no iptables/ip/dnsmasq change was applied while writing this.
# Review every variable, then run explicitly. Default MODE=plan prints the
# exact commands without touching the host; MODE=apply executes them.
#
# Recon baseline 2026-09-20 (read-only):
# - Bridge waydroid0 DOWN (container stopped); stock MAC 00:16:3e:00:00:01,
#   net 192.168.240.1/24, DHCP 192.168.240.2-254 (waydroid-net.sh defaults).
# - Lease file /var/lib/misc/dnsmasq.waydroid0.leases:
#   192.168.240.112 / 00:16:3e:f9:d3:03 / hostname Samsung-S26-Ultra.
# - Egress: default via 192.168.1.1 dev wlp61s0; VPN tables wg1/table-200,
#   wg2/table-300 present (flap = default route jumping tables/devices).
# - waydroid.cfg [properties] carries qemu.hw.mainkeys=1 (see qemu guard).
#
# Usage:
#   MODE=plan ./host-net-ops.sh <profile> [android_id]   # print only (default)
#   MODE=apply sudo ./host-net-ops.sh <profile> [android_id]
#   MODE=rollback sudo ./host-net-ops.sh <profile>       # restore stock
#
# <profile> seeds the stable bridge MAC (LA unicast, mirrors wd-spoof mac.rs:
# FNV-1a 64 over "profile\\0android_id\\0bridge", SplitMix64 expand,
# byte0 = (b & 0xfe) | 0x02). Pass the MAC explicitly via BRIDGE_MAC to skip
# derivation. Lease hostname defaults to <profile> camelized via HOSTNAME.
set -euo pipefail

MODE="${MODE:-plan}"
PROFILE="${1:-}"
AID="${2:-}"
BRIDGE=waydroid0
STOCK_MAC="00:16:3e:00:00:01"
STOCK_ADDR="192.168.240.1/24"
LEASE_FILE="/var/lib/misc/dnsmasq.${BRIDGE}.leases"
HOSTNAME="${HOSTNAME:-${PROFILE}}"
MARK="${MARK:-0x77}"

if [ -z "$PROFILE" ] && [ "$MODE" != "rollback" ]; then
    echo "usage: MODE=plan|apply|rollback $0 <profile> [android_id]" >&2
    exit 2
fi

# Stable LA-unicast bridge MAC from profile seed (python3 stdlib only).
derive_mac() {
    python3 - "$PROFILE" "$AID" <<'EOF'
import sys
def fnv(data: bytes) -> int:
    h = 0xCBF29CE484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001B3) & 0xFFFFFFFFFFFFFFFF
    return h
def splitmix(s: int) -> int:
    s = (s + 0x9E3779B97F4A7C15) & 0xFFFFFFFFFFFFFFFF
    z = s
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & 0xFFFFFFFFFFFFFFFF
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & 0xFFFFFFFFFFFFFFFF
    return z ^ (z >> 31)
seed = fnv(sys.argv[1].encode() + b"\x00" + sys.argv[2].encode() + b"\x00bridge")
s, out = seed, []
for _ in range(6):
    s = splitmix(s)
    out.extend([(s >> 56) & 0xFF, (s >> 48) & 0xFF])
    s = splitmix(s)
out = out[:6]
out[0] = (out[0] & 0xFE) | 0x02
print(":".join(f"{b:02x}" for b in out))
EOF
}

BRIDGE_MAC="${BRIDGE_MAC:-$(derive_mac)}"

run() {
    if [ "$MODE" = "plan" ]; then
        printf 'plan: %s\n' "$*"
    else
        echo "apply: $*"
        "$@"
    fi
}

snapshot() {
    echo "== snapshot =="
    run ip addr show dev "$BRIDGE"
    run ip route show table main
    run ip rule show
    run iptables -S
    run cat "$LEASE_FILE"
}

rotate_bridge_mac() {
    echo "== bridge MAC rotation: $BRIDGE -> $BRIDGE_MAC =="
    run ip link set dev "$BRIDGE" down
    run ip link set dev "$BRIDGE" address "$BRIDGE_MAC"
    run ip addr flush dev "$BRIDGE"
    run ip addr add "$STOCK_ADDR" broadcast + dev "$BRIDGE"
    run ip link set dev "$BRIDGE" up
}

align_lease_hostname() {
    echo "== lease hostname alignment: $HOSTNAME =="
    # dnsmasq honors client hostname; pin it so the lease file stops
    # drifting (was Samsung-S26-Ultra). Requires container restart to renew.
    echo "plan: restart waydroid session so DHCP renews with hostname $HOSTNAME"
    if [ "$MODE" = "apply" ]; then
        echo "apply: verify with: cat $LEASE_FILE (after session restart)"
    fi
}

pin_egress() {
    echo "== egress pin (mark $MARK -> main table, VPN-flap-proof) =="
    run ip rule add fwmark "$MARK" table main priority 100
    run iptables -t mangle -A OUTPUT -m cgroup --path "waydroid" -j MARK --set-mark "$MARK" 2>/dev/null \
        || run iptables -t mangle -A OUTPUT -s 192.168.240.0/24 -j MARK --set-mark "$MARK"
    run ip route flush cache
}

vpn_flap_watch() {
    echo "== VPN-flap detection (read-only, safe in plan mode) =="
    ip route show table all | grep -E '^default' || true
    echo "alert when: default dev/table churns between wg1/wg2/wlp61s0 mid-match"
}

rollback() {
    echo "== ROLLBACK to stock =="
    MODE=apply run ip link set dev "$BRIDGE" down 2>/dev/null || true
    if [ "$MODE" = "plan" ]; then
        printf 'plan: %s\n' "ip link set dev $BRIDGE address $STOCK_MAC"
        printf 'plan: %s\n' "ip rule del fwmark $MARK table main priority 100"
        printf 'plan: %s\n' "iptables -t mangle -D OUTPUT -s 192.168.240.0/24 -j MARK --set-mark $MARK"
        printf 'plan: %s\n' "ip link set dev $BRIDGE up"
    else
        run ip link set dev "$BRIDGE" address "$STOCK_MAC"
        run ip rule del fwmark "$MARK" table main priority 100
        run iptables -t mangle -D OUTPUT -s 192.168.240.0/24 -j MARK --set-mark "$MARK"
        run ip addr flush dev "$BRIDGE"
        run ip addr add "$STOCK_ADDR" broadcast + dev "$BRIDGE"
        run ip link set dev "$BRIDGE" up
    fi
    echo "then: waydroid session restart + re-check lease file + qemu guard"
}

case "$MODE" in
    plan|apply) snapshot; rotate_bridge_mac; align_lease_hostname; pin_egress; vpn_flap_watch ;;
    rollback) rollback ;;
    *) echo "MODE must be plan|apply|rollback" >&2; exit 2 ;;
esac
