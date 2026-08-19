#!/usr/bin/env bash
# Download the Rise of Nations: Extended Edition Windows files on macOS.
#
# Extended Edition has no Mac build, but Phase 1 needs the *files*, not a
# running game. SteamCMD will happily fetch a Windows depot on macOS when told
# to stop caring about the host platform.
#
# You must own the game on the Steam account you log in with. Nothing is
# redistributed by this repo; this pulls your own copy into ./game, which is
# gitignored.
#
# Usage:  scripts/fetch-depot.sh [install-dir]

set -euo pipefail

APP_ID=287450 # Rise of Nations: Extended Edition
INSTALL_DIR="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/game}"

if ! command -v steamcmd >/dev/null 2>&1; then
	cat >&2 <<-'MSG'
		steamcmd not found. Install it with:

		    brew install --cask steamcmd

		Alternatively, DepotDownloader works too and handles 2FA more gracefully:

		    brew tap steamre/tools && brew install depotdownloader
	MSG
	exit 1
fi

# Anonymous login cannot fetch an owned game, so a real account is required.
if [ -z "${STEAM_USER:-}" ]; then
	printf 'Steam username: '
	read -r STEAM_USER
fi

if [ -z "$STEAM_USER" ]; then
	echo "No Steam username given; nothing to log in as." >&2
	exit 1
fi

mkdir -p "$INSTALL_DIR"

echo
echo "Downloading app $APP_ID (Windows depot) as '$STEAM_USER' into:"
echo "  $INSTALL_DIR"
echo
echo "You will be prompted for your password and Steam Guard code. Both are"
echo "handled by steamcmd directly and are not stored by this script."
echo

# force_install_dir must precede login, and login must precede app_update.
steamcmd \
	+@sSteamCmdForcePlatformType windows \
	+force_install_dir "$INSTALL_DIR" \
	+login "$STEAM_USER" \
	+app_update "$APP_ID" validate \
	+quit

echo
echo "Done. Data of interest:"
echo "  $INSTALL_DIR/Data/       rules.xml, unitrules.xml, buildingrules.xml"
echo "  $INSTALL_DIR/art/        BIG archives (models, textures)"
echo
echo "Next: see docs/FORMATS.md."
