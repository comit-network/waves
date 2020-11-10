#!/bin/bash
set -e

if [[ "$1" == "elements-cli" || "$1" == "elementsd" ]]; then
	ELEMENTS_DATA="/data"

	# ensure correct ownership and linking of data directory
	# we do not update group ownership here, in case users want to mount
	# a host directory and still retain access to it
	chown -R elements "$ELEMENTS_DATA"
	ln -sfn "$ELEMENTS_DATA" /home/elements/.elements
	chown -h elements:elements /home/elements/.elements

	exec gosu elements "$@"
else
	exec "$@"
fi