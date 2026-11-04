#!/bin/bash
export MSYS_NO_PATHCONV=1
SDK="$LOCALAPPDATA/Android/Sdk"; ADB="$SDK/platform-tools/adb.exe"; PKG=org.cube.spike.world
cleanup() {
  "$ADB" emu kill >/dev/null 2>&1; sleep 8
  taskkill //F //IM qemu-system-x86_64.exe >/dev/null 2>&1
  taskkill //F //IM emulator.exe >/dev/null 2>&1
  taskkill //F //IM crashpad_handler.exe >/dev/null 2>&1
  "$ADB" kill-server >/dev/null 2>&1
  echo "cleanup done"
}
trap cleanup EXIT
"$SDK/emulator/emulator.exe" -avd Medium_Phone -no-snapshot -no-audio > /c/Users/Godrus/AppData/Local/Temp/claude/9a9f99d73db6a0d5/fe08ec09-37a6-45d2-8d52-168209f62bea/scratchpad/emu.log 2>&1 &
"$ADB" wait-for-device
for i in $(seq 1 60); do b=$("$ADB" shell getprop sys.boot_completed 2>/dev/null | tr -d '\r'); [ "$b" = 1 ] && break; sleep 5; done
echo booted=$b
S=/c/Users/Godrus/AppData/Local/Temp/claude/9a9f99d73db6a0d5/fe08ec09-37a6-45d2-8d52-168209f62bea/scratchpad/iso_run.sh
for m in cubes faces greedy; do
  $S arm64_noexec_$m  /d/Projects/GitHub/Games/Cube/godot-client/build/cube-world-noexec-$m-arm64.apk  org.cube.spike.world "WORLDTEST DONE" 3
done
for m in cubes faces greedy; do
  $S x86_64_noexec_$m /d/Projects/GitHub/Games/Cube/godot-client/build/cube-world-noexec-$m-x86_64.apk org.cube.spike.world "WORLDTEST DONE" 2
done
OUT=/c/Users/Godrus/AppData/Local/Temp/claude/9a9f99d73db6a0d5/fe08ec09-37a6-45d2-8d52-168209f62bea/scratchpad/iso2/memory.txt; : > $OUT
for abi in x86_64 arm64; do
  for m in cubes faces greedy; do
    "$ADB" uninstall $PKG >/dev/null 2>&1
    "$ADB" install -r "$(cygpath -m /d/Projects/GitHub/Games/Cube/godot-client/build/cube-world-noexec-$m-$abi.apk)" >/dev/null 2>&1
    "$ADB" logcat -c; "$ADB" shell am force-stop $PKG; sleep 2
    "$ADB" shell am start -W -n $PKG/com.godot.game.GodotAppLauncher >/dev/null
    v=TIMEOUT
    for i in $(seq 1 200); do
      L=$("$ADB" logcat -d 2>/dev/null)
      if echo "$L" | grep -q "WORLDTEST DONE"; then v=OK; break; fi
      if echo "$L" | grep -q "Fatal signal"; then v=SIGSEGV; break; fi
      sleep 2
    done
    PID=$("$ADB" shell pidof $PKG | tr -d '\r')
    hwm=$("$ADB" shell "grep VmHWM /proc/$PID/status" | tr -d '\r' | awk '{print $2}')
    pss=$("$ADB" shell dumpsys meminfo $PKG | tr -d '\r' | grep "TOTAL PSS" | awk '{print $3}')
    echo "abi=$abi mode=$m verdict=$v VmHWM_kB=$hwm PSS_kB=$pss" | tee -a $OUT
  done
done
echo SET-D-DONE
