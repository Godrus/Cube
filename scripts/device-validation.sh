#!/usr/bin/env bash
# Real-device validation of the world renderers (faces, greedy) on an Android ARM64 phone.
#
#   scripts/device-validation.sh [--runs N] [--modes "faces greedy"] [--out DIR] [--no-soak] [--allow-emulator]
#
# Needs: adb in PATH (or ADB=/path/to/adb), one USB-debugging device, the APKs in godot-client/build/
# (test-<mode>-arm64.apk, soak-<mode>-arm64.apk; build them with the Godot export presets "Test arm64 <mode>" and
# "Soak arm64 <mode>", see docs/spike/rust-world-generation/build-and-targets.md).
#
# It refuses to run (exit 2, "Blocked: no physical ARM64 device") unless exactly one PHYSICAL ARM64 device is
# attached. --allow-emulator only checks the script itself against an emulator: results are then labelled
# "EMULATOR - NOT A DEVICE RESULT" and must not be used as device results.
#
# Per mode it: (1) launches the test APK N times (default 3), waits for the scripted smoke test (Fill, Edit, Undo,
# Redo, Jump, Save, Load, renderer switch, faces-vs-greedy picture check), and records start time, the app's own
# timings and counters, peak memory (VmHWM) / PSS, battery temperature and thermal status, and crash / ANR signs;
# (2) sends REAL touch events (adb input tap) to the Edit, Undo, Redo, Save, Load, Jump and Renderer buttons and
# checks the app's log; (3) runs the soak APK (the app itself walks, edits, undoes and fills for 10 minutes) sampling
# memory and temperature every 60 s and waiting for the app's `SOAK DONE`. Everything is written to DIR (default device-results-<date>) with a summary.md.
set -u
export MSYS_NO_PATHCONV=1
ADB=${ADB:-adb}
RUNS=3; SOAK=1; MODES="faces greedy"; OUT=""; ALLOW_EMU=0
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BUILD="$ROOT/godot-client/build"
PKG=org.cube.spike.world
ACT=com.godot.game.GodotAppLauncher
while [ $# -gt 0 ]; do
  case "$1" in
    --runs) RUNS=$2; shift 2;;
    --no-soak) SOAK=0; shift;;
    --modes) MODES=$2; shift 2;;
    --out) OUT=$2; shift 2;;
    --allow-emulator) ALLOW_EMU=1; shift;;
    *) echo "unknown argument $1"; exit 64;;
  esac
done
OUT=${OUT:-"$ROOT/device-results-$(date +%Y%m%d-%H%M%S)"}
SUM="$OUT/summary.md"

# ---- 1. is this a physical ARM64 device? ---------------------------------------------------------------
devs=$("$ADB" devices | tr -d '\r' | awk 'NR>1 && $2=="device" {print $1}')
n=$(echo "$devs" | grep -c . || true)
if [ "$n" -ne 1 ]; then
  echo "Blocked: no physical ARM64 device (adb sees $n devices; attach exactly one)."; exit 2
fi
SER=$devs
a() { "$ADB" -s "$SER" "$@"; }
abi=$(a shell getprop ro.product.cpu.abi | tr -d '\r')
qemu=$(a shell getprop ro.kernel.qemu | tr -d '\r')
model=$(a shell getprop ro.product.model | tr -d '\r')
fp=$(a shell getprop ro.build.fingerprint | tr -d '\r')
emu=0; case "$SER" in emulator-*) emu=1;; esac; [ "$qemu" = 1 ] && emu=1
LABEL="PHYSICAL DEVICE"
if [ $emu = 1 ]; then
  if [ $ALLOW_EMU = 0 ]; then echo "Blocked: no physical ARM64 device ($SER is an emulator)."; exit 2; fi
  LABEL="EMULATOR - NOT A DEVICE RESULT"
fi
if [ "$abi" != arm64-v8a ] && [ $emu = 0 ]; then echo "Blocked: no physical ARM64 device ($SER reports ABI $abi)."; exit 2; fi
mkdir -p "$OUT"
{
  echo "# Device validation summary ($LABEL)"
  echo
  echo "- device: $model, serial $SER, ABI $abi"
  echo "- fingerprint: $fp"
  echo "- android: $(a shell getprop ro.build.version.release | tr -d '\r') (SDK $(a shell getprop ro.build.version.sdk | tr -d '\r'))"
  echo "- date: $(date -u +%FT%TZ), runs per mode: $RUNS, soak: $([ $SOAK = 1 ] && echo '10 min' || echo skipped)"
  echo
} > "$SUM"

battery_temp() { a shell dumpsys battery | tr -d '\r' | awk '/temperature/ {print $2/10 " C"}'; }
thermal_status() { a shell dumpsys thermalservice 2>/dev/null | tr -d '\r' | grep -m1 -E "Thermal Status|mStatus" | sed 's/^ *//' ; }
mem_of() { # prints "VmHWM_kB VmRSS_kB PSS_kB"
  local pid; pid=$(a shell pidof $PKG | tr -d '\r')
  [ -z "$pid" ] && { echo "- - -"; return; }
  local hwm rss pss
  hwm=$(a shell "grep VmHWM /proc/$pid/status" | tr -d '\r' | awk '{print $2}')
  rss=$(a shell "grep VmRSS /proc/$pid/status" | tr -d '\r' | awk '{print $2}')
  pss=$(a shell dumpsys meminfo $PKG | tr -d '\r' | awk '/TOTAL PSS/ {print $3}')
  echo "${hwm:--} ${rss:--} ${pss:--}"
}
install() { a uninstall $PKG >/dev/null 2>&1; a install -r "$(cygpath -m "$1" 2>/dev/null || echo "$1")" 2>&1 | tail -1; }
wait_marker() { # marker timeout_s -> OK / SIGNAL / ANR / TIMEOUT
  local marker=$1 t=$2 i=0 L
  while [ $i -lt $t ]; do
    L=$(a logcat -d 2>/dev/null)
    echo "$L" | grep -q "$marker" && { echo OK; return; }
    echo "$L" | grep -q "Fatal signal" && { echo SIGNAL; return; }
    echo "$L" | grep -q "ANR in $PKG" && { echo ANR; return; }
    sleep 2; i=$((i+2))
  done
  echo TIMEOUT
}
stats() { grep -E "WORLDTEST (GEOM|MEM mode|FAIL|SUMMARY|IMAGE faces|VIEWS|FILL mode=$1 policy=partial|UNDO mode=$1 policy=partial|EDIT mode=$1 policy=partial scenario=(inside_section|chunk_border|corner))|TIME (first_5x5|load|edit)" "$2" | sed 's/^.*WORLDTEST /  /; s/^.*godot *: *//'; }

for mode in $MODES; do
  APK="$BUILD/test-$mode-arm64.apk"
  [ $emu = 1 ] && [ "$abi" != arm64-v8a ] && APK="$BUILD/test-$mode-x86_64.apk"
  [ -f "$APK" ] || { echo "missing $APK" | tee -a "$SUM"; continue; }
  echo "## Mode $mode: scripted smoke, $RUNS launches" >> "$SUM"
  install "$APK" >/dev/null
  ok=0; bad=0
  for r in $(seq 1 "$RUNS"); do
    a logcat -c; a shell am force-stop $PKG; sleep 2
    t_before=$(battery_temp); th_before=$(thermal_status)
    start=$(a shell am start -W -n $PKG/$ACT | tr -d '\r' | awk '/TotalTime/ {print $2}')
    verdict=$(wait_marker "WORLDTEST DONE" 900)
    read -r hwm rss pss <<<"$(mem_of)"
    t_after=$(battery_temp); th_after=$(thermal_status)
    a logcat -d > "$OUT/$mode.run$r.logcat.txt" 2>/dev/null
    grep -E "godot|WORLDTEST| F DEBUG |Fatal signal|ANR in|AndroidRuntime" "$OUT/$mode.run$r.logcat.txt" | cut -c1-300 > "$OUT/$mode.run$r.app.txt"
    fails=$(grep -c "WORLDTEST FAIL" "$OUT/$mode.run$r.app.txt")
    summary=$(grep -o "SUMMARY failures=[0-9]*" "$OUT/$mode.run$r.app.txt" | tail -1)
    segv=no; grep -q "Fatal signal" "$OUT/$mode.run$r.app.txt" && segv=YES
    anr=no; grep -q "ANR in" "$OUT/$mode.run$r.app.txt" && anr=YES
    errs=$(grep -c -E " E (godot|Godot|AndroidRuntime)" "$OUT/$mode.run$r.logcat.txt")
    [ "$verdict" = OK ] && [ "$fails" = 0 ] && ok=$((ok+1)) || bad=$((bad+1))
    {
      echo "### $mode run $r: $verdict, $summary, WORLDTEST FAIL lines $fails, SIGSEGV/abort: $segv, ANR: $anr, logcat error lines $errs"
      echo "- start (am start -W TotalTime): ${start:-?} ms; peak RSS VmHWM ${hwm} kB, VmRSS ${rss} kB, PSS ${pss} kB"
      echo "- battery temperature: $t_before -> $t_after; thermal: ${th_before:-n/a} -> ${th_after:-n/a}"
      echo '```'; stats "$mode" "$OUT/$mode.run$r.app.txt"; echo '```'
    } >> "$SUM"
    if [ $r = 1 ]; then
      # ---- real touch on the still-running app (first launch of each mode) ----
      echo "### $mode: real touch events (adb input tap) on the live app" >> "$SUM"
      a logcat -c
      # Screen pixels = viewport pixels + (screen - viewport) / 2: the viewport excludes the system bars.
      read -r scr_w scr_h < <(a shell wm size | tr -d '' | awk -F'[: x]+' '/Physical size/ {w=$3; h=$4; if (w<h) {t=w; w=h; h=t}; print w, h}')
      read -r vp_w vp_h < <(grep "WORLDTEST VIEWPORT" "$OUT/$mode.run1.app.txt" | head -1 | sed 's/^.*VIEWPORT //' | awk '{print $1, $2}')
      off_x=$(( (${scr_w:-0} - ${vp_w:-0}) / 2 )); off_y=$(( (${scr_h:-0} - ${vp_h:-0}) / 2 ))
      [ "$off_x" -lt 0 ] && off_x=0; [ "$off_y" -lt 0 ] && off_y=0
      echo "- screen ${scr_w:-?}x${scr_h:-?}, viewport ${vp_w:-?}x${vp_h:-?}, tap offset ($off_x, $off_y)" >> "$SUM"
      declare -A X Y
      while read -r name cx cy; do X[$name]=$cx; Y[$name]=$cy; done < <(grep "WORLDTEST BTN" "$OUT/$mode.run1.app.txt" | sed 's/^.*WORLDTEST BTN //')
      touched=0
      for b in Edit Undo Redo Save Load Jump_64 Renderer Edit; do
        [ -n "${X[$b]:-}" ] || continue
        a shell input tap "$(( ${X[$b]} + off_x ))" "$(( ${Y[$b]} + off_y ))"; sleep 2; touched=$((touched+1))
      done
      a logcat -d | grep -E "edit changed|undo restored|redo applied|save ok|load ok|jump to" | sed 's/^.*godot *: *//' > "$OUT/$mode.touch.txt"
      {
        echo "- taps sent: $touched; app log lines produced by the taps (empty = the taps did not reach the buttons, check $mode.after-touch.png):"
        echo '```'; cat "$OUT/$mode.touch.txt"; echo '```'
      } >> "$SUM"
      a exec-out screencap -p > "$OUT/$mode.after-touch.png" 2>/dev/null
    fi
  done
  echo "- **$mode smoke: $ok of $RUNS launches completed with no failures, $bad did not**" >> "$SUM"
  echo >> "$SUM"
  a shell am force-stop $PKG
done

# ---- soak ---------------------------------------------------------------------------------------------
for mode in $([ $SOAK = 1 ] && echo $MODES); do
  APK="$BUILD/soak-$mode-arm64.apk"
  [ $emu = 1 ] && [ "$abi" != arm64-v8a ] && APK="$BUILD/soak-$mode-x86_64.apk"
  [ -f "$APK" ] || { echo "missing $APK" | tee -a "$SUM"; continue; }
  echo "## Soak $mode: 10 minutes (in-app loop)" >> "$SUM"
  install "$APK" >/dev/null
  a logcat -c; a shell am force-stop $PKG; sleep 2
  a shell am start -W -n $PKG/$ACT >/dev/null
  : > "$OUT/soak-$mode.samples.csv"
  echo "t_s,vmrss_kb,pss_kb,battery_temp,thermal" >> "$OUT/soak-$mode.samples.csv"
  t0=$(date +%s); end=$((t0 + 600 + 600)); alive=yes   # 10 min soak + start-up, smoke test and slack
  while [ "$(date +%s)" -lt "$end" ]; do
    pidof_now=$(a shell pidof $PKG | tr -d '\r'); [ -z "$pidof_now" ] && { alive=NO; break; }
    read -r hwm rss pss <<<"$(mem_of)"
    echo "$(( $(date +%s) - t0 )),$rss,$pss,$(battery_temp),$(thermal_status | tr ',' ';')" >> "$OUT/soak-$mode.samples.csv"
    a logcat -d 2>/dev/null | grep -q "WORLDTEST SOAK DONE" && break
    sleep 60
  done
  a logcat -d > "$OUT/soak-$mode.logcat.txt" 2>/dev/null
  grep -E "WORLDTEST SOAK|WORLDTEST FAIL|Fatal signal|ANR in" "$OUT/soak-$mode.logcat.txt" | sed 's/^.*WORLDTEST /  /' | cut -c1-300 > "$OUT/soak-$mode.app.txt"
  read -r hwm rss pss <<<"$(mem_of)"
  fin=$(grep -c "SOAK DONE" "$OUT/soak-$mode.app.txt")
  {
    echo "- process alive at the end: $alive; SOAK DONE reported: $fin; Fatal signal/ANR lines: $(grep -c -E 'Fatal signal|ANR in' "$OUT/soak-$mode.app.txt"); peak RSS VmHWM ${hwm} kB"
    echo '```'; cat "$OUT/soak-$mode.app.txt"; echo '```'
    echo "(memory/temperature samples: soak-$mode.samples.csv)"
    echo
  } >> "$SUM"
  a shell am force-stop $PKG
done
echo "Done. $LABEL. Summary: $SUM"
