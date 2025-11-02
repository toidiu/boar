#!/bin/bash
set -e

source scripts/virt_common.sh

# Source: Setup adapted from https://wiki.cfdata.org/pages/viewpage.action?pageId=1187491234

set_htb() {
  NSNAME=$1
  DEVNAME=$2
  RATE=$3
  QUEUE=$4

  # tc -n $NSNAME qdisc del dev $DEVNAME root

  tc -n $NSNAME qdisc add dev $DEVNAME root handle 1: htb default 99
  if [ $? -ne 0 ]; then
    exit $?
  fi

  tc -n $NSNAME class add dev $DEVNAME parent 1: classid 1:99 htb quantum 1514 rate $RATE ceil $RATE
  if [ $? -ne 0 ]; then
    exit $?
  fi

  tc -n $NSNAME qdisc add dev $DEVNAME parent 1:99 handle 99: $QUEUE
  if [ $? -ne 0 ]; then
    exit $?
  fi
}

set_htb $NS_M1 $VETH_M1_M2 "20mbit" "pfifo limit 10800"
# set_htb m-ns3 veth3t2 "20mbit" "pfifo limit 108" # do we want to set upload low
set_htb $NS_M3 $VETH_M3_M2 "20mbit" "pfifo limit 10800"

# examples
#QUEUE="fq_codel noecn"
#QUEUE="fq_codel noecn target 2ms interval 40ms"
#QUEUE="fq limit 1000000 flow_limit 26666 nopacing"
#QUEUE="pfifo limit 8333"
