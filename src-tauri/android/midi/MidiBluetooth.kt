package lf.botonera.midi

import android.Manifest.permission
import android.annotation.SuppressLint
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothManager
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanFilter
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.os.Handler
import android.os.ParcelUuid

internal data class BleMidiDevice(val device: BluetoothDevice, val name: String)

@SuppressLint("MissingPermission")
internal class MidiBluetooth(private val context: Context, private val main: Handler) {
    private val adapter get() = context.getSystemService(BluetoothManager::class.java)?.adapter
    private val service = ParcelUuid.fromString("03B80E5A-EDE8-4B33-A751-6CE34EC4C700")
    private var stop: ((String?) -> Unit)? = null
    private val names = mutableMapOf<String, String>()

    fun remote(id: String): BluetoothDevice? {
        if (Build.VERSION.SDK_INT >= 31 &&
            context.checkSelfPermission(permission.BLUETOOTH_CONNECT) != PackageManager.PERMISSION_GRANTED
        ) return null
        return try {
            adapter?.takeIf { it.isEnabled }?.getRemoteDevice(id)
        } catch (_: IllegalArgumentException) {
            null
        } catch (_: SecurityException) {
            null
        }
    }

    fun scan(done: (List<BleMidiDevice>, String?) -> Unit) {
        if (stop != null) {
            done(emptyList(), "bluetooth_busy")
            return
        }
        try {
            val scanner = adapter?.bluetoothLeScanner
            if (scanner == null) {
                done(emptyList(), "bluetooth_disabled")
                return
            }
            val found = linkedMapOf<String, BleMidiDevice>()
            lateinit var callback: ScanCallback
            var active = true
            val timeout = Runnable { stop?.invoke(null) }
            stop = { error ->
                active = false
                stop = null
                main.removeCallbacks(timeout)
                try {
                    scanner.stopScan(callback)
                } catch (_: SecurityException) {
                    // Android can revoke Bluetooth access while a scan is active.
                }
                done(found.values.toList(), error)
            }
            callback = object : ScanCallback() {
                override fun onScanResult(callbackType: Int, result: ScanResult) {
                    main.post {
                        if (!active) return@post
                        val device = result.device
                        val address = device.address
                        val name = result.scanRecord?.deviceName?.takeIf { it.isNotBlank() }
                            ?: runCatching { device.name }.getOrNull()?.takeIf { it.isNotBlank() }
                            ?: names[address] ?: address
                        names[address] = name
                        found[device.address] = BleMidiDevice(device, name)
                    }
                }

                override fun onScanFailed(errorCode: Int) {
                    main.post { if (active) stop?.invoke("bluetooth_scan_failed") }
                }
            }
            val filter = ScanFilter.Builder().setServiceUuid(service).build()
            val settings = ScanSettings.Builder().setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY).build()
            scanner.startScan(listOf(filter), settings, callback)
            main.postDelayed(timeout, 6_000)
        } catch (_: SecurityException) {
            stop = null
            done(emptyList(), "bluetooth_permission_denied")
        } catch (_: IllegalStateException) {
            stop = null
            done(emptyList(), "bluetooth_disabled")
        }
    }

    fun close() {
        stop?.invoke("bluetooth_scan_failed")
    }
}
