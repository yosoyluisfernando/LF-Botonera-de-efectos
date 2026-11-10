#!/usr/bin/env bash
set -euo pipefail

sdk=${ANDROID_HOME:?}
adb="$sdk/platform-tools/adb"
export ANDROID_USER_HOME="$RUNNER_TEMP/lf-android-user"
export ANDROID_AVD_HOME="$ANDROID_USER_HOME/avd"
mkdir -p "$ANDROID_AVD_HOME" android-smoke
log=android-smoke/logcat.txt
emulator_log=android-smoke/emulator.txt

test -c /dev/kvm
sudo chmod a+rw /dev/kvm
timeout --kill-after=10 240 sdkmanager --sdk_root="$sdk" 'platform-tools' 'emulator' 'system-images;android-35;google_apis;x86_64'
printf 'no\n' | avdmanager create avd -n lf-smoke -k 'system-images;android-35;google_apis;x86_64' -p "$ANDROID_AVD_HOME/lf-smoke.avd"
test -f "$ANDROID_AVD_HOME/lf-smoke.ini"
"$adb" start-server
"$sdk/emulator/emulator" -avd lf-smoke -no-window -no-audio -no-boot-anim -no-snapshot -gpu swiftshader -memory 3072 > "$emulator_log" 2>&1 &
emulator_pid=$!
cleanup() {
    timeout 15 "$adb" logcat -d -v threadtime > "$log" 2>/dev/null || true
    if [ "${1:-0}" != 0 ]; then
        grep -E 'FATAL EXCEPTION|Fatal signal|panicked|UnsatisfiedLinkError|AndroidRuntime' "$log" | tail -65 || true
        timeout 15 "$adb" exec-out screencap -p > android-smoke/failure.png 2>/dev/null || true
        timeout 15 "$adb" exec-out cat /sdcard/lf-picker.xml > android-smoke/picker.xml 2>/dev/null || true
        tail -20 "$emulator_log"
    fi
    kill "$emulator_pid" 2>/dev/null || true
}
trap 'cleanup "$?"' EXIT

timeout 120 "$adb" wait-for-device
for ((attempt=0; attempt<90; attempt++)); do
    [ "$("$adb" shell getprop sys.boot_completed | tr -d '\r')" = 1 ] && break
    sleep 2
done
test "$attempt" -lt 90
"$adb" shell getprop ro.product.cpu.abilist | grep -q arm64-v8a
python3 - <<'PYTHON'
import math, struct, wave
with wave.open('android-smoke/LF-Latency.wav','wb') as wav:
    wav.setparams((1,2,44100,0,'NONE','not compressed'))
    wav.writeframes(b''.join(struct.pack('<h',round(8192*math.sin(2*math.pi*440*n/44100))) for n in range(88200)))
PYTHON
pkg=io.github.yosoyluisfernando.lfbotonera
"$adb" shell input keyevent 82
for version in anterior ajustes; do
    (cd "$version" && sha256sum -c *.sha256)
    "$adb" uninstall "$pkg" || true
    "$adb" install "$version"/*.apk
    activity=$("$adb" shell cmd package resolve-activity --brief "$pkg" | tail -1 | tr -d '\r')
    "$adb" shell am start -W -n "$activity"
    sleep 5
    "$adb" shell -T "run-as $pkg sh -c 'mkdir -p files; cat > files/LF-Latency.wav'" < android-smoke/LF-Latency.wav
    timeout --kill-after=10 150 node scripts/android-latency-check.mjs "$adb" "$version"
done
