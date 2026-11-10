package lf.botonera.midi

import android.media.midi.MidiDevice
import android.media.midi.MidiOutputPort
import android.media.midi.MidiReceiver
import app.tauri.plugin.Channel

internal class MidiSession(
    val device: MidiDevice,
    private val id: String,
    val name: String,
    private val events: Channel
) {
    private val ports = mutableListOf<MidiOutputPort>()
    private val lock = Any()
    private var active = true

    fun connect(): Boolean {
        repeat(device.info.outputPortCount) { stream ->
            val port = runCatching { device.openOutputPort(stream) }.getOrNull() ?: return@repeat
            ports.add(port)
            port.connect(object : MidiReceiver() {
                override fun onSend(bytes: ByteArray, offset: Int, count: Int, timestamp: Long) {
                    val data = (offset until offset + count).map { bytes[it].toInt() and 255 }
                    synchronized(lock) {
                        if (active) events.sendObject(mapOf(
                            "kind" to "bytes", "id" to id, "name" to name, "stream" to stream, "data" to data
                        ))
                    }
                }
            })
        }
        return ports.isNotEmpty()
    }

    fun close() {
        synchronized(lock) {
            active = false
            events.sendObject(mapOf("kind" to "closed", "id" to id))
        }
        ports.forEach { runCatching { it.close() } }
        runCatching { device.close() }
    }
}
