#!/usr/bin/env python3
import json
import sys
import urllib.request

URL = (
    "https://api.launchpad.net/1.0/~shadow-x78/+archive/ubuntu/ppa"
    "?ws.op=getPublishedSources&ws.size=300&exact_match=true"
)


def main() -> int:
    if len(sys.argv) != 2:
        return 2
    version = sys.argv[1]
    try:
        with urllib.request.urlopen(URL, timeout=20) as response:
            data = json.load(response)
    except Exception:
        return 2
    for entry in data.get("entries", []):
        if (
            entry.get("source_package_name") == "orbiscreen"
            and str(entry.get("source_package_version", "")).startswith(version)
        ):
            return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())