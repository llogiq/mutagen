#!/bin/bash
#
# Copyright 2015 The Rust Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution and at
# http://rust-lang.org/COPYRIGHT.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

# A script to update the references for particular tests. The idea is
# that you do a run, which will generate files in the build directory
# containing the (normalized) actual output of the compiler. This
# script will then copy that output and replace the "expected output"
# files. You can then commit the changes.
#
# If you find yourself manually editing a foo.stderr file, you're
# doing it wrong.

if [[ "$1" == "--help" || "$1" == "-h" || "$1" == "" || "$2" == "" ]]; then
    echo "usage: $0 <build-directory> <relative-path-to-rs-files>"
    echo ""
    echo "For example:"
    echo "   $0 ../../../build/x86_64-apple-darwin/test/mutagen *.rs */*.rs"
fi

MYDIR=$(dirname $0)
BUILD_DIR="$1"
echo $BUILD_DIR
shift

while [[ "$1" != "" ]]; do
    MUT_NAME="${1/%.rs/.mut}"

    shift
    if [ -f $BUILD_DIR/$MUT_NAME ] && \
           ! (diff $BUILD_DIR/$MUT_NAME $MYDIR/$MUT_NAME >& /dev/null); then
        echo updating $MYDIR/$MUT_NAME
        cp $BUILD_DIR/$MUT_NAME $MYDIR/$MUT_NAME
    fi
done


