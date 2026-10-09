#!/bin/sh
set -eu
repo_root=$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)
if rg -n 'loadFileURL|loadHTMLString|loadData_|evaluateJavaScript|WKUserContentController|addScriptMessageHandler|WKUIDelegate|UIWebView' "$repo_root/platform/ios/ios-web/src"; then
    echo "ios-web source uses an API outside the bounded HTTPS view surface" >&2
    exit 1
fi
