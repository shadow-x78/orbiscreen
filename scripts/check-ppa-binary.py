#!/usr/bin/env python3
"""Check Launchpad PPA binary publication and build state.

Exit codes:
  0  the binary package is published
  1  the binary package is not on Launchpad yet
  2  Launchpad could not be queried
  3  the build is queued or in progress
  4  the build failed
"""
import json
import sys
import urllib.request

BINARIES = (
    "https://api.launchpad.net/1.0/~shadow-x78/+archive/ubuntu/ppa"
    "?ws.op=getPublishedBinaries&binary_name=orbiscreen&exact_match=true"
    "&status=Published&ws.size=300"
)
BUILDS = (
    "https://api.launchpad.net/1.0/~shadow-x78/+archive/ubuntu/ppa"
    "?ws.op=getBuildRecords&source_name=orbiscreen"
)
PENDING = {"Needs building", "Currently building", "Dependency wait"}

def fetch(url: str) -> str:
    request = urllib.request.Request(url, headers={"Cache-Control": "no-cache"})
    with urllib.request.urlopen(request, timeout=20) as response:
        return response.read().decode("utf-8", "replace")

def published(version: str, series: str) -> bool:
    wanted = f"{version}~ubuntu{series}1"
    data = json.loads(fetch(BINARIES))
    for entry in data.get("entries", []):
        if str(entry.get("binary_package_version", "")) != wanted:
            continue
        series_link = str(entry.get("distro_arch_series_link", ""))
        if f"/{series}/" in series_link:
            return True
    return False

def build_status(version: str, series: str) -> int:
    wanted = f"{version}~ubuntu{series}1"
    data = json.loads(fetch(BUILDS))
    states = [
        str(entry.get("buildstate", ""))
        for entry in data.get("entries", [])
        if str(entry.get("source_package_version", "")) == wanted
    ]
    if "Failed to build" in states:
        return 4
    if states and all(state in PENDING or state == "Successfully built" for state in states):
        if any(state in PENDING for state in states):
            return 3
        return 1
    return 1

def main() -> int:
    if len(sys.argv) != 3:
        return 2
    version, series = sys.argv[1], sys.argv[2]
    try:
        if published(version, series):
            return 0
        return build_status(version, series)
    except Exception:
        return 2

if __name__ == "__main__":
    sys.exit(main())