#!/bin/zsh
# build.sh — assemble ~/bin/RonDriver.app, the fixed-path owner of this
# machine's macOS TCC grants. See main.c for why it exists at all.
#
# **Run this once.** Every rebuild changes the executable's cdhash, and macOS
# keys an Accessibility grant to the signature as well as the path, so a
# rebuild costs a re-grant — exactly the tax this bundle is here to abolish.
# `--force` is the deliberate way to pay it.
#
# After the first build, grant the bundle its three permissions once, in
# System Settings → Privacy & Security:
#
#   Accessibility     — + , then ⇧⌘G , then ~/bin/RonDriver.app
#   Screen Recording  — the same, or approve the prompt on the first capture
#   Automation        — approve "RonDriver wants to control System Events"
#
# Nothing here is version-numbered, so those three survive every Claude Code
# update, which is the whole point.
set -e

APP=${RONDRIVER_APP:-$HOME/bin/RonDriver.app}
SRC=${0:A:h}/main.c

if [ -e "$APP" ] && [ "$1" != "--force" ]; then
  echo "$APP already exists — leaving it alone."
  echo "A rebuild changes the cdhash and costs a re-grant; pass --force if that is what you want."
  exit 0
fi

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>RonDriver</string>
  <key>CFBundleDisplayName</key><string>RonDriver</string>
  <key>CFBundleExecutable</key><string>RonDriver</string>
  <key>CFBundleIdentifier</key><string>com.ramonfabrega.rondriver</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>CFBundleShortVersionString</key><string>1.0</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>LSBackgroundOnly</key><true/>
  <key>NSAppleEventsUsageDescription</key>
  <string>RonDriver focuses the Rise of Nations window and reads its title while capturing a traced run.</string>
</dict>
PLIST
echo '</plist>' >> "$APP/Contents/Info.plist"

clang -O2 -Wall -Wextra -o "$APP/Contents/MacOS/RonDriver" "$SRC"
codesign --force --sign - --identifier com.ramonfabrega.rondriver "$APP"
codesign -dv "$APP" 2>&1 | sed -n '1,4p'

echo
echo "built $APP"
echo "cdhash: $(codesign -dvvv "$APP" 2>&1 | grep -i '^CDHash' || true)"
echo
echo "Now grant it, once, in System Settings → Privacy & Security → Accessibility:"
echo "  +  →  ⇧⌘G  →  $APP"
echo "Then run a capture through tools/gamelog/viadriver.sh and approve the"
echo "Screen Recording and Automation prompts as they appear."
