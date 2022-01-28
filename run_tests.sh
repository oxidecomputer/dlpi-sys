#!/bin/bash

pfexec dladm create-simnet sim0
pfexec dladm create-simnet sim1
pfexec dladm modify-simnet -p sim0 sim1

pfexec cargo test -- --nocapture
e=$?

pfexec dladm delete-simnet sim0
pfexec dladm delete-simnet sim1

exit $e
