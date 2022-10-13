#!/bin/bash

#
# Build the tests first to make sure root doesn't have to create a bunch of
# files in target/
#
if ! cargo test --locked --no-run; then
	exit 1
fi

pfexec dladm create-simnet sim0
pfexec dladm create-simnet sim1
pfexec dladm modify-simnet -p sim0 sim1

pfexec cargo test --locked -- --nocapture
e=$?

pfexec dladm delete-simnet sim0
pfexec dladm delete-simnet sim1

exit $e
