#!/usr/bin/env bash

DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
FILE="${DIR}/../src/benchmark"

cp "${DIR}/../tests/garbage_data" ./garbage_data
if [ "$?" != "0" ]; then
    echo "Could not copy tests/garbage_data" >&2
    exit 1
fi

rustc -C opt-level=3 "${FILE}.rs" && ./benchmark && rm ./benchmark
if [ "$?" != "0" ]; then
    echo "Error when using \"${FILE}.ts\""
fi

rm ./garbage_data
