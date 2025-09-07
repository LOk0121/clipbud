#!/bin/bash

cargo bundle --release && \
codesign -fs evilsocket --options runtime target/release/bundle/osx/Clipboard\ Buddy.app