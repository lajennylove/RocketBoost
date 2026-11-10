#!/bin/bash
# Activates one uploaded release of one app. Runs on the server only, through the
# forced command of the activation key in authorized_keys, which fixes the app name:
#
#   command="/home/deploy/bin/activate.sh <app>",restrict  ssh-ed25519 AAAA... ci-activate
#
# The client sends the release id as the command (SSH_ORIGINAL_COMMAND). The release is
# switched in atomically with a symlink. If the app is not healthy afterwards, the
# previous release is switched back in. See deploy-template/README.md.
set -euo pipefail

APP="${1:?the app name is fixed by the forced command}"
RELEASE="${SSH_ORIGINAL_COMMAND:-}"
if [[ ! "$RELEASE" =~ ^[0-9a-f]{40}$ ]]; then
  echo "activate: the release id must be a full 40-character commit id" >&2
  exit 2
fi

APP_DIR="$HOME/apps/$APP"
# shellcheck source=/dev/null
. "$APP_DIR/shared/deploy.env"   # SERVICE, HEALTH_URL, KEEP, and optionally MIGRATE_BIN
REL_DIR="$APP_DIR/releases/$RELEASE"
if [ ! -d "$REL_DIR" ]; then
  echo "activate: release $RELEASE was not uploaded" >&2
  exit 2
fi

PREVIOUS="$(readlink "$APP_DIR/current" || true)"

# Migrations run before the switch, as a deploy step (Laravel's `migrate`).
if [ -n "${MIGRATE_BIN:-}" ]; then
  set -a
  # shellcheck source=/dev/null
  . "$APP_DIR/shared/.env"
  set +a
  "$REL_DIR/$MIGRATE_BIN" migrate
fi

switch() {
  ln -sfn "$1" "$APP_DIR/current.new"
  mv -Tf "$APP_DIR/current.new" "$APP_DIR/current"
  sudo /bin/systemctl restart "$SERVICE"
}

healthy() {
  for _ in $(seq 1 20); do
    if curl -fsS -o /dev/null --max-time 3 "$HEALTH_URL"; then
      return 0
    fi
    sleep 1
  done
  return 1
}

switch "$REL_DIR"
if ! healthy; then
  echo "activate: $RELEASE is not healthy at $HEALTH_URL" >&2
  if [ -n "$PREVIOUS" ] && [ -d "$PREVIOUS" ]; then
    switch "$PREVIOUS"
    healthy || true
    echo "activate: rolled back to $(basename "$PREVIOUS")" >&2
  fi
  exit 1
fi

# Keep the newest releases for rollback and delete the rest.
ls -1dt "$APP_DIR"/releases/*/ | tail -n +"$((KEEP + 1))" | xargs -r rm -rf
echo "live $RELEASE"
