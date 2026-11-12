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
"$adb" install "$1"
"$adb" shell input keyevent 82
"$adb" logcat -c
pkg=${LF_ANDROID_PACKAGE:-io.github.yosoyluisfernando.lfbotonera}
activity=$("$adb" shell cmd package resolve-activity --brief "$pkg" | tail -1 | tr -d '\r')
for attempt in 1 2; do
    "$adb" shell am force-stop "$pkg"
    "$adb" shell am start -W -n "$activity"
    sleep 10
    "$adb" shell pidof "$pkg"
    "$adb" shell dumpsys activity activities > android-smoke/activities.txt
    grep -E "(topResumedActivity|mResumedActivity).* $pkg/" android-smoke/activities.txt
done
timeout 15 "$adb" exec-out screencap -p > android-smoke/window.png
timeout 15 "$adb" logcat -d -v threadtime > "$log"
if grep -E 'ndk-context-.+panicked|android context was not initialized|UnsatisfiedLinkError.*libtauri_app_lib|Android MIDI attach:' "$log"; then
    exit 1
fi
# Genera un WAV breve propio para probar sin publicar archivos del usuario.
python3 - <<'PYTHON'
import math
import struct
import wave

with wave.open('android-smoke/LF-Smoke.wav', 'wb') as wav:
    wav.setparams((1, 2, 44100, 0, 'NONE', 'not compressed'))
    wav.writeframes(b''.join(struct.pack('<h', round(2048 * math.sin(2 * math.pi * 440 * n / 44100)))
                            for n in range(44100 * 2)))
PYTHON
# El MP3 de prueba es sintético y no contiene audios del usuario.
ffmpeg -hide_banner -loglevel error -i android-smoke/LF-Smoke.wav -c:a libmp3lame -q:a 4 android-smoke/LF-Roundtrip.mp3
"$adb" shell mkdir -p /sdcard/Download
"$adb" push android-smoke/LF-Roundtrip.mp3 /sdcard/Download/LF-Roundtrip.mp3
"$adb" shell am broadcast -a android.intent.action.MEDIA_SCANNER_SCAN_FILE -d file:///sdcard/Download/LF-Roundtrip.mp3
"$adb" push android-smoke/LF-Smoke.wav /sdcard/Download/LF-Smoke.wav
"$adb" shell am broadcast -a android.intent.action.MEDIA_SCANNER_SCAN_FILE -d file:///sdcard/Download/LF-Smoke.wav
timeout --kill-after=10 480 node scripts/android-import-smoke.mjs "$adb"
