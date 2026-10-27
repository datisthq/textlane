#!/usr/bin/env bash
# Stages the shipped skills as wheel data, installed to <prefix>/share/textlane/skills.
set -euo pipefail
cd "$(dirname "$0")/.."
rm -rf textlane.data
mkdir -p textlane.data/data/share/textlane
cp -R ../.skills textlane.data/data/share/textlane/skills
