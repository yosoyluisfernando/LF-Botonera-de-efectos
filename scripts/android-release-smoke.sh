#!/usr/bin/env bash
set -euo pipefail
sdk=${ANDROID_HOME:?}
tools="$sdk/build-tools/36.0.0"
adb="$sdk/platform-tools/adb"
export ANDROID_USER_HOME="$RUNNER_TEMP/lf-android-user"
export ANDROID_AVD_HOME="$ANDROID_USER_HOME/avd"
mapfile -t apks < <(find src-tauri/gen/android/app/build/outputs/apk -type f -iname '*release*.apk')
test "${#apks[@]}" -eq 1
mkdir -p android-release-artifact
name=LF-Botonera-1.6.0-android-arm64.apk
# La firma efímera valida el APK en CI. La entrega se firma localmente con
# la clave de producción del autor; esa clave nunca se envía al ejecutor.
export LF_CI_SIGN_PASSWORD
LF_CI_SIGN_PASSWORD=$(openssl rand -hex 32)
key="$RUNNER_TEMP/lf-ci-release.p12"
keytool -genkeypair -storetype PKCS12 -keystore "$key" -storepass:env LF_CI_SIGN_PASSWORD \
  -alias lf-ci -keyalg RSA -keysize 3072 -validity 3 -dname 'CN=LF CI temporary'
"$tools/apksigner" sign --ks "$key" --ks-key-alias lf-ci --ks-pass env:LF_CI_SIGN_PASSWORD \
  --v4-signing-enabled false --out "android-release-artifact/$name" "${apks[0]}"
unset LF_CI_SIGN_PASSWORD
"$tools/apksigner" verify --verbose "android-release-artifact/$name"
"$tools/aapt" dump badging "android-release-artifact/$name" > android-release-artifact/package.txt
grep -F "name='io.github.yosoyluisfernando.lfbotonera'" android-release-artifact/package.txt
grep -F "versionName='1.6.0'" android-release-artifact/package.txt
grep -F "versionCode='1006001'" android-release-artifact/package.txt
grep -F "application-label:'LF Botonera de Efectos'" android-release-artifact/package.txt
grep -F "native-code: 'arm64-v8a'" android-release-artifact/package.txt
! grep -q 'application-debuggable' android-release-artifact/package.txt
"$sdk/emulator/emulator" -avd lf-smoke -no-window -no-audio -no-boot-anim -no-snapshot \
  -gpu swiftshader -memory 3072 > android-release-artifact/emulator.txt 2>&1 &
emulator_pid=$!
cleanup() {
  timeout 15 "$adb" logcat -d -v threadtime > android-release-artifact/logcat.txt 2>/dev/null || true
  kill "$emulator_pid" 2>/dev/null || true
}
trap cleanup EXIT
timeout 120 "$adb" wait-for-device
for ((attempt=0; attempt<90; attempt++)); do
  [ "$("$adb" shell getprop sys.boot_completed | tr -d '\r')" = 1 ] && break
  sleep 2
done
test "$attempt" -lt 90
# Solo se retira la aplicación del emulador de CI, nunca de un teléfono real.
test "$("$adb" shell getprop ro.kernel.qemu | tr -d '\r')" = 1
pkg=io.github.yosoyluisfernando.lfbotonera
"$adb" uninstall "$pkg"
"$adb" install "android-release-artifact/$name"
activity=$("$adb" shell cmd package resolve-activity --brief "$pkg" | tail -1 | tr -d '\r')
"$adb" logcat -c
for attempt in 1 2; do
  "$adb" shell am force-stop "$pkg"
  "$adb" shell am start -W -n "$activity"
  sleep 10
  "$adb" shell pidof "$pkg"
  "$adb" shell dumpsys activity activities | grep -E "(topResumedActivity|mResumedActivity).* $pkg/"
done
python3 scripts/android-release-ui.py "$adb" android-release-artifact
"$adb" logcat -d > android-release-artifact/logcat.txt
! grep -E 'FATAL EXCEPTION|Fatal signal|Android MIDI attach:|Android app attach:' android-release-artifact/logcat.txt
(cd android-release-artifact && sha256sum "$name" > "$name.sha256")
