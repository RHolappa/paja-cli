#!/bin/sh
# SPDX-License-Identifier: Apache-2.0
# Run on the Docker host. Existing configuration and volumes are preserved.
set -eu
cd "$(dirname "$0")/.."
python3 - "$@" <<'PY'
import os
from pathlib import Path
import re
import secrets
import sys
path = Path('deploy/.env')
if not path.exists():
    if len(sys.argv) != 2 or not re.fullmatch(r'(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,63}', sys.argv[1]):
        sys.exit('Usage: sh scripts/deploy-vps.sh paja.example.com')
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(fd, 'w') as out:
        out.write(f'PAJA_DOMAIN={sys.argv[1]}\nPAJA_PASSWORD={secrets.token_urlsafe(32)}\nPAJA_SPEECH_MODEL=small\n')
    print('Created private deploy/.env. The access key is stored there; it is not printed in logs.')
elif len(sys.argv) > 1:
    domain = next((line.split('=', 1)[1] for line in path.read_text().splitlines() if line.startswith('PAJA_DOMAIN=')), '')
    if domain != sys.argv[1]:
        sys.exit('The requested domain differs from deploy/.env. Review that file before deploying.')
PY
docker compose --env-file deploy/.env -f deploy/compose.yaml build
docker compose --env-file deploy/.env -f deploy/compose.yaml run --rm --no-deps app python web/transcribe.py --download
docker compose --env-file deploy/.env -f deploy/compose.yaml up -d --wait
printf 'Paja is running. Retrieve your private access key from deploy/.env. Back up the paja-voice_paja_data volume.\n'
