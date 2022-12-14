#!/bin/bash

#
# jq(1) may be in a different place on OmniOS or Helios systems:
#
export PATH=$PATH:/opt/ooce/bin

if ! command -v jq >/dev/null; then
	printf 'ERROR: need jq(1) installed\n' >&2
	exit 1
fi

#
# Build the tests first to make sure root doesn't have to create a bunch of
# files in target/.  There are some hoops we then need to jump through to
# record the names of all the test binaries.
#
if ! j=$(cargo build --locked --workspace --tests \
    --message-format json-render-diagnostics); then
	printf 'ERROR: failed to build tests\n' >&2
	exit 1
fi

#
# Parse the JSON into an array of test binary names.
#
if ! t=$(jq -r -s 'map(
    	select(.reason == "compiler-artifact") |
    	select(.target.test) |
    	select(.executable) |
    	.executable
    ) | flatten | .[]' <<< "$j"); then
	printf 'ERROR: failed to parse JSON for test binaries\n' >&2
	exit 1
fi
if ! readarray -t tests <<< "$t"; then
	exit 1
fi

if (( ${#tests[@]} == 0 )); then
	printf 'ERROR: no test binaries?\n' >&2
	exit 1
fi
printf 'found %d test binaries\n' "${#tests[@]}" >&2

#
# Set up simnets for this test, and be sure to try to tear them down no matter
# why we end up exiting:
#
trap 'pfexec dladm delete-simnet sim0; pfexec dladm delete-simnet sim1' EXIT
if ! pfexec dladm create-simnet sim0 ||
    ! pfexec dladm create-simnet sim1 ||
    ! pfexec dladm modify-simnet -p sim0 sim1; then
	printf 'ERROR: simnet setup failure\n' >&2
	exit 1
fi

e=0
for (( i = 0; i < ${#tests[@]}; i++ )); do
	tbin="${tests[$i]}"

	printf 'EXEC: %s\n' "$tbin" >&2
	if ! pfexec "$tbin" --nocapture; then
		printf 'FAILURE (%s)\n' "$tbin" >&2
		e=1
	fi
done

exit $e
