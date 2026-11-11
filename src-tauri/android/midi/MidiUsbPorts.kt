package lf.botonera.midi

import android.media.midi.MidiDeviceInfo
import android.media.midi.MidiManager
import android.net.Uri
import android.os.Build

internal data class UsbPort(val id: String, val name: String, val info: MidiDeviceInfo)

@Suppress("DEPRECATION")
internal fun usbPorts(manager: MidiManager?): List<UsbPort> {
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
