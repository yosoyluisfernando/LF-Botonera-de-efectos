package lf.botonera.midi

import android.content.Context
import android.hardware.input.InputManager
import android.os.Build
import android.os.Handler
import android.view.InputDevice
import app.tauri.plugin.Channel

internal class MidiKeyboards(context: Context, private val main: Handler) : InputManager.InputDeviceListener {
    private val manager = context.getSystemService(InputManager::class.java)
    private var channel: Channel? = null
    private var previous: List<Map<String, String>>? = null

    fun devices(): List<Map<String, String>> = InputDevice.getDeviceIds().mapNotNull { id ->
        val device = InputDevice.getDevice(id) ?: return@mapNotNull null
        val external = if (Build.VERSION.SDK_INT >= 29) device.isExternal
            else device.keyboardType == InputDevice.KEYBOARD_TYPE_ALPHABETIC
        if (!external || device.isVirtual || !device.supportsSource(InputDevice.SOURCE_KEYBOARD)) {
            return@mapNotNull null
        }
        mapOf("id" to "keyboard:$id", "name" to device.name)
    }.sortedBy { it["name"] }

    fun attach(events: Channel) {
        if (channel == null) manager?.registerInputDeviceListener(this, main)
        channel = events
        previous = null
        publish()
    }

    private fun publish() {
        val current = devices()
        if (current != previous) {
            previous = current
            channel?.sendObject(mapOf("kind" to "keyboards", "devices" to current))
        }
    }

    override fun onInputDeviceAdded(deviceId: Int) = publish()
    override fun onInputDeviceChanged(deviceId: Int) = publish()
    override fun onInputDeviceRemoved(deviceId: Int) = publish()

    fun close() {
        if (channel != null) manager?.unregisterInputDeviceListener(this)
        channel = null
    }
}
