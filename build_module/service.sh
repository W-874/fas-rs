#!/system/bin/sh
MODDIR=${0%/*}

chmod a+x $MODDIR/fas-rs

until [ -d "/sdcard/Android" ]; do
	sleep 1
done

killall fas-rs
nohup $MODDIR/fas-rs >/dev/null 2>&1 &
