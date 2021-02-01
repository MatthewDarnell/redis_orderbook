#!/usr/bin/env sh


x=0
echo "Attempting to Connect to Redis at $REDIS_IP"
while [ $x -eq 0 ]
do
  pong=$(redis-cli -u $REDIS_IP "PING")
  if [ "$pong" = "PONG" ]; then
     echo "Redis Started. Starting Application"
    x=$(( $x + 1 ))
  else
    echo "Redis Not Yet Started"
    sleep 5;
  fi
done

echo "Executing $BIN_NAME"
$BIN_NAME