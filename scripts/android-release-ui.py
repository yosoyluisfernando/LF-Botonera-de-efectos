"""Check the production APK through Android accessibility, without WebView debug."""
import json
import re
import subprocess
import sys
import time
import xml.etree.ElementTree as ET
from pathlib import Path

adb, output = sys.argv[1], Path(sys.argv[2])
output.mkdir(parents=True, exist_ok=True)


def shell(*args):
    return subprocess.check_output([adb, 'shell', *args], timeout=30).decode('utf-8').strip()


def snapshot(name='current'):
    shell('uiautomator', 'dump', '/sdcard/lf-release-ui.xml')
    data = subprocess.check_output([adb, 'exec-out', 'cat', '/sdcard/lf-release-ui.xml'], timeout=20)
    (output / (name + '.xml')).write_bytes(data)
    return list(ET.fromstring(data).iter('node'))


def text(node):
    return ' '.join((node.get('text') or node.get('content-desc') or '').split())


def bounds(node):
    return tuple(map(int, re.findall(r'-?\d+', node.get('bounds', ''))))


def click(node):
    name = 'action-' + str(time.time_ns())
    snapshot(name)
    screenshot(name)
    left, top, right, bottom = bounds(node)
    assert right > left and bottom > top, 'Action has no visible bounds'
    print('Tap:', text(node), (left, top, right, bottom), flush=True)
    shell('input', 'tap', str((left + right) // 2), str((top + bottom) // 2))
    time.sleep(1)


def wait_for(predicate, message, attempts=20):
    for _ in range(attempts):
        try:
            nodes = snapshot()
            result = predicate(nodes)
            if result is not None and result is not False:
                return result
        except (subprocess.SubprocessError, ET.ParseError):
            pass
        time.sleep(1)
    screenshot('production-failure')
    raise AssertionError(message)


def label(nodes, value):
    matches = [n for n in nodes if text(n) == value and n.get('enabled') == 'true'
               and len(bounds(n)) == 4 and bounds(n)[2] > bounds(n)[0]
               and bounds(n)[3] > bounds(n)[1]]
    return next((n for n in matches if n.get('clickable') == 'true'),
                matches[0] if matches else None)


def screenshot(name):
    (output / (name + '.png')).write_bytes(
        subprocess.check_output([adb, 'exec-out', 'screencap', '-p'], timeout=20))


# Complete the real first-run UI; do not seed or change the app's database.
def enter_notes(nodes):
    if any('Qué hay de nuevo' in text(n) for n in nodes):
        return True
    for value in ['Comenzar', 'Ahora no', 'Cancelar']:
        action = label(nodes, value)
        if action is not None:
            click(action)
            break
    return False


wait_for(enter_notes, 'Production APK did not open release notes')
expected = ('La distribución disponible se realiza mediante GitHub Releases. La app '
            'identifica Android y muestra el canal responsable de las actualizaciones.')
found = False
for index in range(14):
    nodes = snapshot('production-notes-' + str(index))
    if any(not len(n) and expected in text(n) for n in nodes):
        found = True
        screenshot('production-notes')
        break
    size = re.findall(r'(\d+)x(\d+)', shell('wm', 'size'))[-1]
    width, height = map(int, size)
    shell('input', 'swipe', str(width // 2), str(height * 3 // 4),
          str(width // 2), str(height * 2 // 5), '350')
    time.sleep(1)
assert found, 'Production release notes split a single list item into separate paragraphs'
click(wait_for(lambda ns: label(ns, 'Entendido'), 'Missing notes acceptance'))


def main_toolbar(nodes):
    # A normalization prompt can follow the notes on the first launch.
    for value in ['Ahora no', 'Cancelar', 'Más tarde']:
        action = label(nodes, value)
        if action is not None:
            click(action)
            return None
    return next((n for n in nodes if text(n).replace('\ufe0f', '') == '⚙'), None)


click(wait_for(main_toolbar, 'Missing production Settings button'))
nodes = wait_for(lambda ns: ns if label(ns, 'Acerca de') is not None
                 and label(ns, 'Control remoto') is not None else None,
                 'Missing production settings sections')
about = label(nodes, 'Acerca de')
remote = max((n for n in nodes if text(n) == 'Control remoto'), key=lambda n: bounds(n)[1])
assert bounds(about)[1] > bounds(remote)[1], 'About is not after Remote control in production'
snapshot('production-settings-order')
screenshot('production-settings-order')
click(about)
nodes = wait_for(lambda ns: ns if any('Android' in text(n) for n in ns) else None,
                 'Production About does not identify Android')
assert any('GitHub Releases' in text(n) for n in nodes), 'Wrong production distribution channel'
snapshot('production-about')
screenshot('production-about')
shell('input', 'keyevent', '4')
wait_for(lambda ns: label(ns, 'Acerca de'), 'Back did not return to sections')
shell('input', 'keyevent', '4')
wait_for(main_toolbar, 'Back did not return to main screen')
shell('input', 'keyevent', '4')
wait_for(lambda ns: any('Quieres salir' in text(n) for n in ns), 'Missing exit confirmation')
shell('input', 'keyevent', '4')
wait_for(main_toolbar, 'Back did not cancel exit confirmation')
(output / 'production-ui-proof.json').write_text(json.dumps({
    'release_notes_single_paragraph': True,
    'about_after_remote_control': True,
    'platform_android': True,
    'distribution_github': True,
    'back_navigation_and_exit_confirmation': True,
}, indent=2) + '\n')
print('Production APK UI: continuous notes, About last, Android/GitHub and Back verified.')
