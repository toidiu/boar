#!/bin/bash
set -e

source scripts/virt_common.sh

# Source: Setup adapted from https://wiki.cfdata.org/pages/viewpage.action?pageId=1187491234
#
# Development
# - ip COMMAND CHEAT SHEET
#   - https://access.redhat.com/sites/default/files/attachments/rh_ip_command_cheatsheet_1214_jcs_print.pdf
# - Support docker?

# Current topology
#
#           fq
# --------- -->                                      -->  ---------
# | ns-s1 |                                               | ns-c1 |
# --------- <--         htb          netem           <--  ---------
#               --------- --> --------- --> ---------
#               | ns-m1 |     | ns-m2 |     | ns-m3 |
#               --------- <-- --------- <-- ---------
#                           netem         htb        -->  ---------
#                                                         | ns-c2 |
#                                                    <--  ---------
#
#
# TODO: add a second server interface
#           fq
# --------- -->                                      -->  ---------
# | ns-s1 |                                               | ns-c1 |
# --------- <--         htb          netem           <--  ---------
#               --------- --> --------- --> ---------
#               | ns-m1 |     | ns-m2 |     | ns-m3 |
#           fq  --------- <-- --------- <-- ---------
# --------- -->             netem         htb        -->  ---------
# | ns-s2 |                                               | ns-c2 |
# --------- <--                                      <--  ---------

echo "[-] Delete prev configuration"
############
# Delete NS
############
for NS in $NS_S1 $NS_M1 $NS_M2 $NS_M3 $NS_C1 $NS_C2 ; do
  ip netns del $NS 2>/dev/null || true
done
############
# Delete Veth
############
ip link del $VETH_S1_M1 2>/dev/null || true
ip link del $VETH_M1_M2 2>/dev/null || true
ip link del $VETH_M2_M3 2>/dev/null || true
ip link del $VETH_M3_C1 2>/dev/null || true
ip link del $VETH_M3_C2 2>/dev/null || true

echo "[-] Configure NS and Veth"
############
# Create NS
############
ip netns add $NS_S1
ip netns add $NS_M1
ip netns add $NS_M2
ip netns add $NS_M3
ip netns add $NS_C1
ip netns add $NS_C2
############
# Link NS to each other
############
ip link add name $VETH_S1_M1 type veth peer name $VETH_M1_S1
ip link add name $VETH_M1_M2 type veth peer name $VETH_M2_M1
ip link add name $VETH_M2_M3 type veth peer name $VETH_M3_M2
ip link add name $VETH_M3_C1 type veth peer name $VETH_C1_M3
ip link add name $VETH_M3_C2 type veth peer name $VETH_C2_M3
############
# Associate Veth to the NS
############
# s1
ip link set dev $VETH_S1_M1 netns $NS_S1
# m1
ip link set dev $VETH_M1_S1 netns $NS_M1
ip link set dev $VETH_M1_M2 netns $NS_M1
# m2
ip link set dev $VETH_M2_M1 netns $NS_M2
ip link set dev $VETH_M2_M3 netns $NS_M2
# m3
ip link set dev $VETH_M3_M2 netns $NS_M3
ip link set dev $VETH_M3_C1 netns $NS_M3
ip link set dev $VETH_M3_C2 netns $NS_M3
# c
ip link set dev $VETH_C1_M3 netns $NS_C1
ip link set dev $VETH_C2_M3 netns $NS_C2
############
# Assign IPs to Veth
############
# s1
ip -n $NS_S1 addr add $IP_S1_EG/24 dev $VETH_S1_M1
# m1
ip -n $NS_M1 addr add $IP_M1_IN/24 dev $VETH_M1_S1
ip -n $NS_M1 addr add $IP_M1_EG/24 dev $VETH_M1_M2
# m2
ip -n $NS_M2 addr add $IP_M2_IN/24 dev $VETH_M2_M1
ip -n $NS_M2 addr add $IP_M2_EG/24 dev $VETH_M2_M3
# m3
ip -n $NS_M3 addr add $IP_M3_IN/24 dev $VETH_M3_M2
ip -n $NS_M3 addr add $IP_M3_EG1/24 dev $VETH_M3_C1
ip -n $NS_M3 addr add $IP_M3_EG2/24 dev $VETH_M3_C2
# c
ip -n $NS_C1 addr add $IP_C1_IN/24 dev $VETH_C1_M3
ip -n $NS_C2 addr add $IP_C2_IN/24 dev $VETH_C2_M3

############
# Offload
############
echo "[-] Configure offload: '$OFFLOAD'"
# s1
ip netns exec $NS_S1 ethtool -K $VETH_S1_M1 $OFFLOAD
# m1
ip netns exec $NS_M1 ethtool -K $VETH_M1_S1 $OFFLOAD
ip netns exec $NS_M1 ethtool -K $VETH_M1_M2 $OFFLOAD
# m2
ip netns exec $NS_M2 ethtool -K $VETH_M2_M1 $OFFLOAD
ip netns exec $NS_M2 ethtool -K $VETH_M2_M3 $OFFLOAD
# m3
ip netns exec $NS_M3 ethtool -K $VETH_M3_M2 $OFFLOAD
ip netns exec $NS_M3 ethtool -K $VETH_M3_C1 $OFFLOAD
ip netns exec $NS_M3 ethtool -K $VETH_M3_C2 $OFFLOAD
# c
ip netns exec $NS_C1 ethtool -K $VETH_C1_M3 $OFFLOAD
ip netns exec $NS_C2 ethtool -K $VETH_C2_M3 $OFFLOAD

############
# Link up
############
echo "[-] Bring Veth online"
# s1
ip -n $NS_S1 link set dev $VETH_S1_M1 up
ip -n $NS_S1 link set dev lo up
# m1
ip -n $NS_M1 link set dev $VETH_M1_S1 up
ip -n $NS_M1 link set dev $VETH_M1_M2 up
ip -n $NS_M1 link set dev lo up
# m2
ip -n $NS_M2 link set dev $VETH_M2_M1 up
ip -n $NS_M2 link set dev $VETH_M2_M3 up
ip -n $NS_M2 link set dev lo up
# m3
ip -n $NS_M3 link set dev $VETH_M3_M2 up
ip -n $NS_M3 link set dev $VETH_M3_C1 up
ip -n $NS_M3 link set dev $VETH_M3_C2 up
ip -n $NS_M3 link set dev lo up
# c
ip -n $NS_C1 link set dev $VETH_C1_M3 up
ip -n $NS_C1 link set dev lo up
ip -n $NS_C2 link set dev $VETH_C2_M3 up
ip -n $NS_C2 link set dev lo up

############
# Forward path
############
echo "[-] Add Forward routes"
# Route all ips to IP_M1_IN
ip -n $NS_S1 route add default via $IP_M1_IN
# Route all ips to IP_M2_IN
ip -n $NS_M1 route add default via $IP_M2_IN
# Route all ips to IP_M3_IN
ip -n $NS_M2 route add default via $IP_M3_IN
######
# Create routing tables
######
echo "[--] Routing table for multiple clients"
# Add routes to routing tables
ip -n $NS_C1 route add 0/0 via $IP_C1_IN dev $VETH_C1_M3 table 100  # Default route for table 100
ip -n $NS_C2 route add 0/0 via $IP_C2_IN dev $VETH_C2_M3 table 200  # Default route for table 200
# Add routing rules
ip rule add from $IP_C1_IN lookup 100 # Route traffic from IP_C1 to table 100
ip rule add from $IP_C2_IN lookup 200 # Route traffic from IP_C2 to table 200

############
# Reverse path
############
######
# C1: Tell C1 that S1, M1, M2 ip should be forwarded to M3.
######
# If you need to get to S1 send via ip M3
ip -n $NS_C1 route add $SUBNET_S1 via $IP_M3_EG1
# If you need to get to M1 send via ip M3
ip -n $NS_C1 route add $SUBNET_M1 via $IP_M3_EG1
# If you need to get to M2 send via ip M3
ip -n $NS_C1 route add $SUBNET_M2 via $IP_M3_EG1
######
# C2: Tell C2 that S1, M1, M2 ip should be forwarded to M3.
######
# If you need to get to S1 send via ip M3
ip -n $NS_C2 route add $SUBNET_S1 via $IP_M3_EG2
# If you need to get to M1 send via ip M3
ip -n $NS_C2 route add $SUBNET_M1 via $IP_M3_EG2
# If you need to get to M2 send via ip M3
ip -n $NS_C2 route add $SUBNET_M2 via $IP_M3_EG2
######
# M3: Tell M3 that S1 and M1 ip should be forwarded to M2.
######
# If you need to get to S1 send via ip M2
ip -n $NS_M3 route add $SUBNET_S1 via $IP_M2_EG
# If you need to get to M1 send via ip M2
ip -n $NS_M3 route add $SUBNET_M1 via $IP_M2_EG
######
# M2: Tell M2 that S1 ip should be forwarded to M1.
######
# If you need to get to S1 send via ip M1
ip -n $NS_M2 route add $SUBNET_S1 via $IP_M1_EG

# threaded NAPI
echo "[-] Threaded NAPI"
# Debugging -------
# echo "----- $VETH_S1_M1"
# ip netns exec NS_S1 bash -c "echo $USER"
# ip netns exec NS_S1 bash -c "ls -al /sys/devices/virtual/net/$VETH_S1_M1 | grep threaded"
# ip netns exec NS_S1 bash -c "cat /sys/devices/virtual/net/$VETH_S1_M1/threaded"
# ip netns exec NS_S1 bash -c "echo 1 > /sys/devices/virtual/net/$VETH_S1_M1/threaded"
# ip netns exec NS_S1 bash -c "sysctl net.core.knapid_enabled=1"
# Debugging -------

# ip netns exec $NS_S1 bash -c "echo 1 > /sys/devices/virtual/net/$VETH_S1_M1/threaded"
# ip netns exec $NS_M1 bash -c "echo 1 > /sys/devices/virtual/net/$VETH_M1_S1/threaded"
# ip netns exec $NS_M1 bash -c "echo 1 > /sys/devices/virtual/net/$VETH_M1_M2/threaded"
# ip netns exec $NS_M2 bash -c "echo 1 > /sys/devices/virtual/net/$VETH_M2_M1/threaded"
# ip netns exec $NS_M2 bash -c "echo 1 > /sys/devices/virtual/net/$VETH_M2_C1/threaded"
# ip netns exec $NS_C1 bash -c "echo 1 > /sys/devices/virtual/net/$VETH_C1_M2/threaded"

echo "[-] Configure tc"
tc -n $NS_S1 qdisc add dev $VETH_S1_M1 root fq
# tc -n $NS_M1 qdisc add dev $VETH_M1_S1 root fq
# tc -n $NS_M1 qdisc add dev $VETH_M1_M2 root fq
# tc -n $NS_M2 qdisc add dev $VETH_M2_M1 root fq
# tc -n $NS_M2 qdisc add dev $VETH_M2_C1 root fq
# tc -n $NS_C1 qdisc add dev $VETH_C1_M2 root fq
./scripts/virt_tc_netem.sh
./scripts/virt_tc_htb.sh

echo "[-] Configure ipv4"
for NS in $NS_S1 $NS_M1 $NS_M2 $NS_M3 $NS_C1 $NS_C2 ; do
  sudo ip netns exec $NS sysctl net.ipv4.tcp_notsent_lowat=131072
  sudo ip netns exec $NS sysctl net.ipv4.ip_forward=1
done

echo "[-] Complete"
cat << EOF

                         $IP_C1_IN
   -----                   -----
 | $NS_S1 |   <=======>  | $NS_C1 |
   -----           ||      -----
 $IP_S1_EG         ||
                   ||      -----
                   ===>  | $NS_C2 |
                           -----
                         $IP_C2_IN

[-] Test connection
sudo ip netns exec $NS_S1 tcpdump -i any -n
sudo ip netns exec $NS_C1 ping $IP_S1_EG
sudo ip netns exec $NS_C2 ping $IP_S1_EG

[-] Test connection reverse
sudo ip netns exec $NS_C1 tcpdump -i any -n
sudo ip netns exec $NS_C2 tcpdump -i any -n
sudo ip netns exec $NS_S1 ping $IP_C1_IN
sudo ip netns exec $NS_S1 ping $IP_C2_IN

EOF
