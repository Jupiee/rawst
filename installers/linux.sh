#!/bin/bash

URL="https://github.com/Jupiee/rawst/releases/download/0.3/rawst"

if [[ "$OSTYPE" == "linux-gnu"* ]]; then

    DESTINATION="/usr/local/bin"
else

    echo "Unsupported OS"
    exit 1
fi

curl -L "$URL" -o "rawst"

chmod +x "rawst"

mv "rawst" "$DESTINATION"