package lf.botonera.midi

import android.annotation.SuppressLint
import android.content.Context
import android.media.midi.MidiDevice
import android.media.midi.MidiDeviceInfo
import android.media.midi.MidiManager
import android.net.Uri
import android.os.Build
import android.os.Handler
import app.tauri.plugin.Channel

@Suppress("DEPRECATION")
@SuppressLint("MissingPermission")
internal class MidiPorts(context: Context, private val main: Handler, private val bluetooth: MidiBluetooth) {
    private data class UsbPort(val id: String, val name: String, val info: MidiDeviceInfo)

    private val manager = context.getSystemService(MidiManager::class.java)
    val supported get() = manager != null
    private var channel: Channel? = null
    private var selected = emptySet<String>()
    private var scanned = emptyMap<String, BleMidiDevice>()
    private val opened = mutableMapOf<String, MidiSession>()
    private val pending = mutableMapOf<String, Any>()
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
            opened.filterValues { it.device.info.id == device.id }.keys.forEach(::closeSession)
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
        if (ids == selected) return
        selected = ids
        pending.keys.retainAll(ids)
        main.removeCallbacks(retry)
        retry.run()
    }

    fun discovered(devices: List<BleMidiDevice>) {
        scanned = devices.associateBy { "ble:" + it.device.address }
        reconcile()
    }

    private fun usbPorts(): List<UsbPort> {
        val devices = if (Build.VERSION.SDK_INT >= 33) {
            manager?.getDevicesForTransport(MidiManager.TRANSPORT_MIDI_BYTE_STREAM).orEmpty()
        } else {
            manager?.devices.orEmpty().toList()
        }
        val usb = devices.filter { it.type == MidiDeviceInfo.TYPE_USB && it.outputPortCount > 0 }.map { info ->
            val props = info.properties
            val name = props.getString(MidiDeviceInfo.PROPERTY_NAME)
                ?: props.getString(MidiDeviceInfo.PROPERTY_PRODUCT) ?: "USB MIDI"
            val serial = props.getString(MidiDeviceInfo.PROPERTY_SERIAL_NUMBER).orEmpty()
            UsbPort("usb:" + Uri.encode(name) + ":" + Uri.encode(serial), name, info)
        }
        return usb.groupBy { it.id }.values.flatMap { group ->
            if (group.size == 1) group else group.map { it.copy(id = it.id + ":" + it.info.id) }
        }
    }

    private fun reconcile() {
        if (channel == null) return
        val usb = usbPorts().associateBy { it.id }
        opened.filter { (id, session) ->
            id !in selected || (id.startsWith("usb:") && usb[id]?.info?.id != session.device.info.id)
        }.keys.forEach(::closeSession)
        for (id in selected) {
            if (id in opened || id in pending) continue
            val usbPort = usb[id]
            val remote = if (id.startsWith("ble:")) {
                scanned[id]?.device ?: bluetooth.remote(id.removePrefix("ble:"))
            } else null
            if (usbPort == null && remote == null) continue
            val name = usbPort?.name ?: scanned[id]?.name ?: runCatching { remote?.name }.getOrNull()
            open(id, name, usbPort?.info, remote)
        }
        publish(usb)
    }

    private fun open(id: String, name: String?, info: MidiDeviceInfo?, remote: android.bluetooth.BluetoothDevice?) {
        val service = manager ?: return
        val token = Any()
        pending[id] = token
        val timeout = Runnable { if (pending[id] === token) pending.remove(id) }
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
                        val label = name ?: device.info.properties.getString(MidiDeviceInfo.PROPERTY_NAME) ?: id
                        val session = MidiSession(device, id, label, events)
                        if (session.connect()) opened[id] = session else session.close()
                    }
                    publish(usbPorts().associateBy { it.id })
                }
            }
        }
        try {
            if (info != null) service.openDevice(info, listener, main)
            else service.openBluetoothDevice(requireNotNull(remote), listener, main)
            main.postDelayed(timeout, 12_000)
        } catch (_: RuntimeException) {
            pending.remove(id)
        }
    }

    private fun publish(usb: Map<String, UsbPort>) {
        val devices = sortedMapOf<String, String>()
        usb.forEach { (id, port) -> devices[id] = port.name }
        scanned.forEach { (id, device) -> devices[id] = device.name }
        opened.forEach { (id, session) -> devices[id] = session.name }
        val snapshot = mapOf(
            "kind" to "devices",
            "ports" to devices.map { (id, name) -> mapOf("id" to id, "name" to name) },
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
