#!/bin/bash
# Isolation harness. usage: iso_run.sh <label> <apk> <package> <ready-marker-regex> <N> [prop=value ...]
# For every run: force-stop, clear logcat, set properties, start the app, wait until the marker appears in
# logcat (success) or a fatal signal / ANR / timeout is seen; save the filtered log, the crash backtrace and
# a one-line verdict to $ISO/<label>.runs.txt. Nothing is retried: every launch is one counted run.
export MSYS_NO_PATHCONV=1
ADB="$LOCALAPPDATA/Android/Sdk/platform-tools/adb.exe"
ISO=/c/Users/Godrus/AppData/Local/Temp/claude/D--Projects-GitHub-Games-Cube/fe08ec09-37a6-45d2-8d52-168209f62bea/scratchpad/iso
mkdir -p "$ISO"
LABEL=$1; APK=$2; PKG=$3; MARK=$4; N=$5; shift 5
"$ADB" uninstall "$PKG" >/dev/null 2>&1
"$ADB" install -r "$(cygpath -m "$APK")" 2>&1 | tail -1
ok=0; bad=0
for n in $(seq 1 "$N"); do
  for kv in "$@"; do "$ADB" shell setprop "${kv%%=*}" "${kv#*=}"; done
  "$ADB" logcat -c
  "$ADB" shell am force-stop "$PKG"; sleep 2
  t0=$(date +%s)
  start=$("$ADB" shell am start -W -n "$PKG/com.godot.game.GodotAppLauncher" | tr -d '\r' | grep -E "Status|TotalTime" | tr '\n' ' ')
  verdict=TIMEOUT
  for i in $(seq 1 300); do
    L=$("$ADB" logcat -d 2>/dev/null)
    if echo "$L" | grep -q -E "$MARK"; then verdict=OK; break; fi
    if echo "$L" | grep -q -E "Fatal signal"; then verdict=SIGSEGV; sleep 3; break; fi
    if echo "$L" | grep -q -E "ANR in org.cube|Application Not Responding"; then verdict=ANR; break; fi
    sleep 2
  done
  L=$("$ADB" logcat -d 2>/dev/null)
  segv=no; anr=no
  echo "$L" | grep -q "Fatal signal" && segv=yes
  echo "$L" | grep -q -E "ANR in org.cube" && anr=yes
  alive=$("$ADB" shell pidof "$PKG" | tr -d '\r'); [ -n "$alive" ] && alive=yes || alive=no
  secs=$(( $(date +%s) - t0 ))
  echo "$L" | grep -E "godot|WORLDTEST|MINIMAL| F DEBUG |Fatal signal|ANR in|libndk_translation|libc +: crash" | cut -c1-260 > "$ISO/$LABEL.run$n.log"
  echo "$L" | grep -E " F DEBUG " | grep -E "pid:|signal|#[0-9]+ pc|Cause|abi" | cut -c1-260 > "$ISO/$LABEL.run$n.tombstone.txt"
  line="label=$LABEL run=$n verdict=$verdict sigsegv=$segv anr=$anr process_alive_at_end=$alive secs=$secs am_start=[$start]"
  echo "$line" | tee -a "$ISO/$LABEL.runs.txt"
  if [ "$verdict" = OK ]; then ok=$((ok+1)); else bad=$((bad+1)); fi
done
echo "SUMMARY label=$LABEL runs=$N ok=$ok failed=$bad" | tee -a "$ISO/$LABEL.runs.txt"
"$ADB" shell am force-stop "$PKG"
