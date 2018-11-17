#!/usr/bin/env bash

REOSC_ARCHIVE_NAME="reosc-$TRAVIS_OS_NAME-$TRAVIS_TAG"
zip -j "$REOSC_ARCHIVE_NAME.zip" target/release/reosc
