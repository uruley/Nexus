#!/usr/bin/env bash
set -euo pipefail

uvicorn server:app --host "${HOST:-127.0.0.1}" --port "${PORT:-5055}" --reload

