#!/system/bin/sh
# Copyright 2023-2025, shadow3aaa
#
# This file is part of fas-rs.
#
# fas-rs is free software: you can redistribute it and/or modify it under
# the terms of the GNU General Public License as published by the Free
# Software Foundation, either version 3 of the License, or (at your option)
# any later version.
#
# fas-rs is distributed in the hope that it will be useful, but WITHOUT ANY
# WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
# FOR A PARTICULAR PURPOSE. See the GNU General Public License for more
# details.
#
# You should have received a copy of the GNU General Public License along
# with fas-rs. If not, see <https://www.gnu.org/licenses/>.

DIR=/sdcard/Android/fas-rs

{
	until [ -d $DIR ] && [ -d /data ]; do
		sleep 1
	done

	rm -rf $DIR
	rm -f /data/powercfg.json
	rm -f /data/powercfg.sh
} & # do not block boot
