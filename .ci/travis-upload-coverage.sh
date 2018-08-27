#!/usr/bin/env bash
set -e

KCOV_BUILD="kcov-build"
KCOV_OUT="target/cov"
KCOV="./$KCOV_BUILD/usr/local/bin/kcov"

kcov_install() {
    wget "https://github.com/SimonKagstrom/kcov/archive/master.tar.gz"
    tar xzf "master.tar.gz"
    mkdir "kcov-master/build"
    cd "kcov-master/build"
    cmake ..
    make && make install DESTDIR="../../$KCOV_BUILD"
    cd ../../
    rm -rf kcov-master
}

kcov_run() {
    rm -rf "$KCOV_OUT"

    local files=`cargo test --no-run --message-format=json | jq -r "select(.profile.test == true) | .filenames[]"`

    for file in $files; do
        local targetdir="$KCOV_OUT/$(basename $file)"
        mkdir -p $targetdir
        "$KCOV" --exclude-pattern=/.cargo,/usr/lib --verify "$targetdir" "$file"
    done
}

kcov_upload() {
    bash <(curl -s https://codecov.io/bash)
}

kcov_install
kcov_run
kcov_upload
