from pathlib import Path
import shutil
import sys
import xml.etree.ElementTree as ET


def prepare(project):
    main = project / 'app/src/main'
    source = Path(__file__).resolve().parents[1] / 'src-tauri/android/midi'
    target = main / 'java/lf/botonera/midi'
    shutil.copytree(source, target, dirs_exist_ok=True)

    android = 'http://schemas.android.com/apk/res/android'
    ET.register_namespace('android', android)
    manifest = main / 'AndroidManifest.xml'
    tree = ET.parse(manifest)
    root = tree.getroot()
    name_attr = f'{{{android}}}name'
    permissions = {
        'BLUETOOTH': {'maxSdkVersion': '30'},
        'BLUETOOTH_ADMIN': {'maxSdkVersion': '30'},
        'ACCESS_FINE_LOCATION': {'maxSdkVersion': '30'},
        'BLUETOOTH_CONNECT': {},
        'BLUETOOTH_SCAN': {'usesPermissionFlags': 'neverForLocation'},
    }
    for name, attributes in permissions.items():
        name = 'android.permission.' + name
        item = next((entry for entry in root.findall('uses-permission')
                     if entry.get(name_attr) == name), None)
        if item is None:
            item = ET.Element('uses-permission', {name_attr: name})
            root.insert(0, item)
        for key, value in attributes.items():
            item.set(f'{{{android}}}{key}', value)
    for name in ('android.hardware.bluetooth', 'android.hardware.bluetooth_le',
                 'android.software.midi', 'android.hardware.usb.host'):
        if not any(entry.get(name_attr) == name for entry in root.findall('uses-feature')):
            root.insert(0, ET.Element('uses-feature', {
                name_attr: name, f'{{{android}}}required': 'false',
            }))
    ET.indent(tree)
    tree.write(manifest, encoding='utf-8', xml_declaration=True)

    proguard = project / 'app/proguard-rules.pro'
    keep = '-keep class lf.botonera.midi.** { *; }'
    rules = proguard.read_text(encoding='utf-8') if proguard.exists() else ''
    if keep not in rules:
        proguard.write_text(rules + '\n' + keep + '\n', encoding='utf-8')


if __name__ == '__main__':
    prepare(Path(sys.argv[1]))
