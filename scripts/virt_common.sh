#!/bin/bash
set -e

############################
############
# NS
############
# server
NS_S1=ns_s1
# middle
NS_M1=ns_m1
NS_M2=ns_m2
NS_M3=ns_m3
# client
NS_C1=ns_c1
NS_C2=ns_c2
############
# Veth
############
# s1
VETH_S1_M1=veth_s1_m1
# m1
VETH_M1_S1=veth_m1_s1
VETH_M1_M2=veth_m1_m2
# m2
VETH_M2_M1=veth_m2_m1
VETH_M2_M3=veth_m2_m3
# m3
VETH_M3_M2=veth_m3_m2
VETH_M3_C1=veth_m3_c1
VETH_M3_C2=veth_m3_c2
# c
VETH_C1_M3=veth_c1_m3
VETH_C2_M3=veth_c2_m3
############
# ADDR
############
# addr server
IP_S1_EG=10.55.10.1
SUBNET_S1=10.55.10.0/24
# mid 1
IP_M1_IN=10.55.10.2
IP_M1_EG=10.55.20.1
SUBNET_M1=10.55.20.0/24
# mid 2
IP_M2_IN=10.55.20.2
IP_M2_EG=10.55.30.1
SUBNET_M2=10.55.30.0/24
# mid 3
IP_M3_IN=10.55.30.2
IP_M3_EG1=10.55.41.1
IP_M3_EG2=10.55.42.1
# addr client
IP_C1_IN=10.55.41.2
IP_C2_IN=10.55.42.2
############
# PORT
############
PORT_S1=5555
############
# Offload
############
OFFLOAD="rx on tx on tso off gso off gro off"
############################
