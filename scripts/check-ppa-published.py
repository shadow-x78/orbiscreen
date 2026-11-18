#!/usr/bin/env python3
"""Check Launchpad PPA publication state for a version.

Exit codes:
  0  the source is published
  1  the source is not on Launchpad yet
  2  Launchpad could not be queried
  3  the source is queued or building (Pending)
  4  the source will not be published (Superseded, Deleted, Obsolete)
"""
import json
import sys
import urllib.request

URL = (
    "https://api.launchpad.net/1.0/~shadow-x78/+archive/ubuntu/ppa"
    "?ws.op=getPublishedSources&ws.size=300&exact_match=true"
)

UNPUBLISHABLE = {"Superseded", "Deleted", "Obsolete"}


def upstream_version(debian_version: str) -> str:
    """Reduce a Debian version to the upstream part.

    "0.44.1~ubuntunoble1" and "0.44.1-1~ubuntu24.04.1" both become "0.44.1", so a
    release is not mistaken for a published "0.44.10".
    """
    return debian_version.split("~", 1)[0].split("-", 1)[0]


def main() -> int:
    if len(sys.argv) != 2:
        return 2
    version = sys.argv[1]
    try:
        with urllib.request.urlopen(URL, timeout=20) as response:
            data = json.load(response)
    except Exception:
        return 2
    statuses = [
        entry.get("status", "")
        for entry in data.get("entries", [])
        if entry.get("source_package_name") == "orbiscreen"
        and upstream_version(str(entry.get("source_package_version", ""))) == version
    ]
    if not statuses:
        return 1
    if "Published" in statuses:
        return 0
    if all(status in UNPUBLISHABLE for status in statuses):
        return 4
    return 3


if __name__ == "__main__":
    sys.exit(main())
