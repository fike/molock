#!/bin/bash
# find_code_smells.sh
# Runs cargo clippy with pedantic and style lints to identify areas for improvement.

FILE=$1

if [ -n "$FILE" ]; then
    echo "Running pedantic clippy on $FILE..."
    cargo clippy --file "$FILE" -- -W clippy::pedantic -W clippy::style -W clippy::nursery
else
    echo "Running pedantic clippy on the whole project..."
    cargo clippy -- -W clippy::pedantic -W clippy::style -W clippy::nursery
fi
