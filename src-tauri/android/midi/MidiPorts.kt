package lf.botonera.midi

import android.annotation.SuppressLint
import android.content.Context
import android.media.midi.MidiDeviceInfo
import android.media.midi.MidiManager
import android.os.Build
import android.os.Handler
import app.tauri.plugin.Channel

@Suppress("DEPRECATION")
@SuppressLint("MissingPermission")
internal class MidiPorts(context: Context, private val main: Handler, private val bluetooth: MidiBluetooth) {

    private val manager = context.getSystemService(MidiManager::class.java)
    val supported get() = manager != null
    private var channel: Channel? = null
    private var selected = emptySet<String>()
    private var scanned = emptyMap<String, BleMidiDevice>()
    private val opened = mutableMapOf<String, MidiSession>()
    private val pending = mutableMapOf<String, Any>()
    private val failed = mutableSetOf<String>()
    private var lastSnapshot: Map<String, Any>? = null
    private val retry = object : Runnable {
        override fun run() {
            reconcile()
            if (selected.isNotEmpty()) main.postDelayed(this, 5_000)
        }
    }
    private val callback = object : MidiManager.DeviceCallback() {
        override fun onDeviceAdded(device: MidiDeviceInfo) = reconcile()
        override fun onDeviceRemoved(device: MidiDeviceInfo) {
            opened.filterValues { it.device.info.id == device.id }.keys.toList().forEach {
                closeSession(it)
                if (it.startsWith("ble:") && it in selected) failed.add(it)
            }
            reconcile()
        }
    }

    fun attach(events: Channel) {
        val first = channel == null
        channel = events
        lastSnapshot = null
        if (first && manager != null) {
            if (Build.VERSION.SDK_INT >= 33) {
                manager.registerDeviceCallback(MidiManager.TRANSPORT_MIDI_BYTE_STREAM, { main.post(it) }, callback)
            } else {
                manager.registerDeviceCallback(callback, main)
            }
        }
        reconcile()
    }

    fun select(ids: Set<String>) {
        selected = ids
        failed.clear()
        pending.keys.retainAll(ids)
        main.removeCallbacks(retry)
        retry.run()
    }

    fun discovered(devices: List<BleMidiDevice>) {
        val previous = scanned
        scanned = devices.associateBy { "ble:" + it.device.address }.mapValues { (id, found) ->
            if (found.name == found.device.address) previous[id] ?: found else found
        }
        reconcile()
    }

    private fun reconcile() {
        if (channel == null) return
        val usb = usbPorts(manager).associateBy { it.id }
        opened.filter { (id, session) ->
            id !in selected || (id.startsWith("usb:") && usb[id]?.info?.id != session.device.info.id)
        }.keys.forEach(::closeSession)
        for (id in selected) {
            if (id in opened || id in pending || id in failed) continue
            val usbPort = usb[id]
            val remote = if (id.startsWith("ble:")) {
                scanned[id]?.device ?: bluetooth.remote(id.removePrefix("ble:"))
            } else null
            if (usbPort == null && remote == null) {
                if (id.startsWith("ble:")) failed.add(id)
                continue
            }
            val name = usbPort?.name ?: scanned[id]?.name ?: runCatching { remote?.name }.getOrNull()
            open(id, name, usbPort?.info, remote)
        }
        publish(usb)
    }

    private fun open(id: String, name: String?, info: MidiDeviceInfo?, remote: android.bluetooth.BluetoothDevice?) {
        val service = manager ?: return
        val token = Any()
        pending[id] = token
        val timeout = Runnable {
            if (pending[id] === token) {
                pending.remove(id)
                failed.add(id)
                publish(usbPorts(manager).associateBy { it.id })
            }
        }
        val listener = MidiManager.OnDeviceOpenedListener { device ->
            main.removeCallbacks(timeout)
            if (pending[id] !== token) {
                device?.let { runCatching { it.close() } }
            } else {
                pending.remove(id)
                if (device != null) {
                    val events = channel
                    if (events == null) {
                        runCatching { device.close() }
                    } else {
                        val label = device.info.properties.getString(MidiDeviceInfo.PROPERTY_NAME)
                            ?.takeIf { it.isNotBlank() && (name == null || name == remote?.address) }
                            ?: name ?: id
                        val session = MidiSession(device, id, label, events)
                        if (runCatching { session.connect() }.getOrDefault(false)) opened[id] = session
                        else { session.close(); failed.add(id) }
                    }
                } else failed.add(id)
                publish(usbPorts(manager).associateBy { it.id })
            }
        }
        try {
            if (info != null) service.openDevice(info, listener, main)
            else service.openBluetoothDevice(requireNotNull(remote), listener, main)
            main.postDelayed(timeout, 12_000)
        } catch (_: RuntimeException) {
            pending.remove(id)
            failed.add(id)
        }
    }

    private fun publish(usb: Map<String, UsbPort>) {
        val devices = sortedMapOf<String, String>()
        selected.filter { it.startsWith("ble:") }.forEach { devices[it] = it.removePrefix("ble:") }
        usb.forEach { (id, port) -> devices[id] = port.name }
        scanned.forEach { (id, device) -> devices[id] = device.name }
        opened.forEach { (id, session) -> devices[id] = session.name }
        val snapshot = mapOf(
            "kind" to "devices",
            "ports" to devices.map { (id, name) -> mapOf(
                "id" to id, "name" to name, "connection_state" to when (id) {
                    in opened -> "connected"
                    in pending -> "connecting"
                    in failed -> "failed"
                    else -> "disconnected"
                }
            ) },
            "connected" to opened.keys.sorted()
        )
        if (snapshot != lastSnapshot) {
            lastSnapshot = snapshot
            channel?.sendObject(snapshot)
        }
    }

    private fun closeSession(id: String) {
        opened.remove(id)?.close()
    }

    fun close() {
        if (channel != null) manager?.unregisterDeviceCallback(callback)
        channel = null
        main.removeCallbacks(retry)
        pending.clear()
        opened.keys.toList().forEach(::closeSession)
    }
}
