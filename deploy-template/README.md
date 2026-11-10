# Deploying a Rocket app: build in CI, serve on the server

This is the deploy pattern for the RocketBoost apps (the Propel docs, the Ignite docs and
the homepage) and for any Rocket app you build with them. The compiling happens in GitHub
Actions. The server only receives finished files and switches to them.

## Why

- **The server stays free.** A Rust release build is the heaviest step, and it used to run
  on the same 2 GB server that serves the site. CI does it on its own machine.
- **The release is the same one that was built.** The binary and the assets are built once,
  uploaded, and switched in. Nothing is compiled again on the server.
- **Rollback is a symlink.** The server keeps the last few releases, so going back is one
  switch, with no rebuild.
- **Each key can do one thing.** The upload key can only write release files. The activation
  key can only run the activation script for one app. Neither can open a shell.

The same idea as the Bitbucket pipeline for Sage themes: build, copy the output, switch.

## What is in this folder

| File | Where it goes | What it does |
|---|---|---|
| `deploy.yml` | `.github/workflows/deploy.yml` in your repository | Builds the binary and the assets, uploads them, activates them. |
| `server/activate.sh` | `/home/deploy/bin/activate.sh` on the server | Runs migrations, switches the `current` symlink, restarts the service, checks health, and rolls back if the check fails. |
| `server/app.service` | `/etc/systemd/system/<app>.service` | Runs the app from the `current` symlink. |
| `server/deploy.env.example` | `/home/deploy/apps/<app>/shared/deploy.env` | The service name, the health URL, how many releases to keep. |
| `server/sudoers.example` | `/etc/sudoers.d/deploy-<app>` | Lets the deploy user restart this app's service, and nothing else. |
| `server/authorized_keys.example` | `/home/deploy/.ssh/authorized_keys` | The two keys, each with a forced command. |

## Folder layout on the server

```
/home/deploy/apps/<app>/
  releases/<commit-id>/    the uploaded files: the binary and public/
  current -> releases/...  the release that is live (a symlink)
  shared/.env              secrets and settings; never in git, never in a release
  shared/deploy.env        the activation settings
```

Each release is a folder named after the commit it was built from, so you always know what
is running.

## One-time setup for an app

1. **Server folders.** As the deploy user:
   ```sh
   mkdir -p ~/apps/<app>/releases ~/apps/<app>/shared ~/bin
   ```
2. **Activation script.** Copy `server/activate.sh` to `~/bin/activate.sh` and make it
   executable: `chmod 755 ~/bin/activate.sh`.
3. **Settings.** Copy `server/deploy.env.example` to `~/apps/<app>/shared/deploy.env` and
   fill in the service name, the health URL and the number of releases to keep. Put the
   app's own settings (database URL, secret key) in `~/apps/<app>/shared/.env`, with
   permissions `600`.
4. **Service.** Copy `server/app.service` to `/etc/systemd/system/<app>.service`, fill in the
   placeholders, then run `sudo systemctl daemon-reload && sudo systemctl enable <app>.service`.
   Do not start it yet: the first release is not there.
5. **Restart rule.** Copy `server/sudoers.example` to `/etc/sudoers.d/deploy-<app>`, using
   `sudo visudo -f`, and fill in the service name.
6. **Upload tool.** Check that `rrsync` exists:
   `ls /usr/share/doc/rsync/scripts/`. On some systems it is compressed (`rrsync.gz`). Use
   `gunzip` to unpack it to the path in the authorized-keys line.
7. **Keys.** On your own computer, make two keys. Keep the private files out of git.
   ```sh
   ssh-keygen -t ed25519 -N '' -C '<app>-ci-upload' -f ./<app>-upload
   ssh-keygen -t ed25519 -N '' -C '<app>-ci-activate' -f ./<app>-activate
   ```
   Add the two `.pub` lines to `~/.ssh/authorized_keys` on the server, using the forced
   commands in `server/authorized_keys.example`. Then delete the local private files once
   they are stored in GitHub.
8. **Host key.** Get the server's ed25519 host key with `ssh-keyscan -t ed25519 <host>`.
   Check its fingerprint against the one your hosting provider shows before you trust it.
9. **GitHub.** In the repository, create the environment `production`, and add:
   - Repository variable `DEPLOY_HOST`: the server's address.
   - Environment secrets `DEPLOY_UPLOAD_KEY` and `DEPLOY_ACTIVATE_KEY`: the private keys.
   - Environment secret `DEPLOY_KNOWN_HOSTS`: the host key line from step 8.
10. **Workflow.** Copy `deploy.yml` to `.github/workflows/deploy.yml`, and change the values
    marked TODO (`APP_NAME`, `BIN_NAME`).
11. **First deploy.** Push to `main`, or run the workflow by hand. When it finishes, the
    release is live and the health check has passed.

## What a deploy does

1. **Build (CI, Ubuntu 24.04):** installs Rust and Node, runs `npm ci && npm run build` if
   the repository has a `package.json`, then `cargo build --release --locked`.
2. **Stage:** copies the binary and `public/` into a folder.
3. **Upload:** `rsync` sends the folder to `releases/<commit-id>/` with the upload key.
4. **Activate:** the activation key runs `activate.sh` with the commit id. The script:
   - runs `migrate` if `MIGRATE_BIN` is set, before the switch;
   - points `current` at the new release, atomically;
   - restarts the service;
   - checks the health URL for up to 20 seconds;
   - if the check fails, points `current` back at the previous release and restarts again;
   - deletes releases older than `KEEP`.

## Rolling back

Re-run the deploy for an older commit, or, on the server:
```sh
ln -sfn ~/apps/<app>/releases/<older-commit> ~/apps/<app>/current.new
mv -Tf ~/apps/<app>/current.new ~/apps/<app>/current
sudo systemctl restart <app>.service
```
The release folder must still exist, which `KEEP` controls.

## Ignite apps

- Set `MIGRATE_BIN=ignite` in `deploy.env`. The release then runs `ignite migrate` before the
  switch. Migrations are a deploy step, as in Laravel, and the app does not migrate at boot.
- The app reads `DATABASE_URL` and `IGNITE_SECRET_KEY` from `shared/.env`. A release build
  refuses to start without the secret key.
- Stage the `ignite` binary with the app: set `EXTRA_BINS: "ignite"` in the workflow, so the
  release contains the binary that `MIGRATE_BIN` runs.

## Adapting it

- **Several binaries:** build and stage each one, and give each app its own folder, service
  and keys.
- **No assets:** delete the Node steps. The `hashFiles` guard skips them anyway.
- **Other runners:** keep `ubuntu-24.04` or newer, and never older than the server's release.
  A binary built on a newer system may not run on an older one.

## Troubleshooting

- **`Permission denied (publickey)` during upload or activation:** the key is not in the
  deploy user's `authorized_keys`, or the forced command is wrong. Check the line in
  `authorized_keys` matches the template.
- **`activate: release ... was not uploaded`:** the upload step failed, or the commit id is
  not the one the upload used.
- **`activate: ... is not healthy`:** the app did not answer its health URL. The script rolled
  back. Read the service log with `journalctl -u <app>.service -n 100`.
- **The service does not start:** check `shared/.env` and the unit's `WorkingDirectory`.
