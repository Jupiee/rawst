#!/bin/bash

URL="https://github.com/Jupiee/rawst/releases/download/0.4.0/rawst-x86_64-unknown-linux-gnu.tar.gz"

if [[ "$OSTYPE" == "linux-gnu"* ]]; then

    DESTINATION="/usr/local/bin"
else

    echo "Unsupported OS"
    exit 1
fi

curl -L "$URL" -o "rawst-x86_64-unknown-linux-gnu.tar.gz"

chmod +x "rawst-x86_64-unknown-linux-gnu.tar.gz"

tar -xzvf "rawst-x86_64-unknown-linux-gnu.tar.gz" -C "/usr/local/bin"

echo "Installation completed successfully."