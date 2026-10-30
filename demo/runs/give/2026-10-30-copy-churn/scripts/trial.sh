#!/bin/bash
# trial of the setup and light-use scripts, on throwaway pods
APP=$1; S=~/svrngs-churn/scripts; B=~/svrngs-churn/trial-$APP; mkdir -p $B
POD=svrngs-churn-trial-$APP; VOL=svrngs-churn-trial-$APP-data
. $S/apps.sh
podman volume create $VOL >/dev/null; start_$APP >/dev/null
for i in $(seq 900); do ready_$APP 2>/dev/null && break; sleep 2; done
echo "$APP ready after ~$((i*2))s"
sleep 60; setup_$APP; use_$APP
podman pod rm -f $POD >/dev/null; podman volume rm $VOL >/dev/null; echo "$APP trial removed"
