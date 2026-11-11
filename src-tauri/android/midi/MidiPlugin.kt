package lf.botonera.midi

import android.Manifest.permission
import android.app.Activity
import android.content.Intent
import android.provider.Settings
import android.content.pm.PackageManager
import android.os.Build
import android.os.Handler
import android.os.Looper
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.PermissionCallback
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@InvokeArg
internal class AttachArgs {
    lateinit var channel: Channel
}

@InvokeArg
internal class SelectArgs {
    var ids: List<String> = emptyList()
}

@TauriPlugin(permissions = [
    Permission(alias = "nearby", strings = [permission.BLUETOOTH_SCAN, permission.BLUETOOTH_CONNECT]),
    Permission(alias = "location", strings = [permission.ACCESS_FINE_LOCATION])
])
class MidiPlugin(private val activity: Activity) : Plugin(activity) {
    private val main = Handler(Looper.getMainLooper())
    private val bluetooth = MidiBluetooth(activity, main)
    private val ports = MidiPorts(activity, main, bluetooth)
    private val keyboards = MidiKeyboards(activity, main)
    private var requestingPermission = false

    @Command
    fun attach(invoke: Invoke) {
        val args = invoke.parseArgs(AttachArgs::class.java)
        main.post {
            ports.attach(args.channel)
            keyboards.attach(args.channel)
            invoke.resolve()
        }
    }

    @Command
    fun getKeyboards(invoke: Invoke) {
        main.post { invoke.resolveObject(keyboards.devices()) }
    }

    @Command
    fun openKeyboardSettings(invoke: Invoke) {
        main.post {
            try {
                activity.startActivity(Intent(Settings.ACTION_BLUETOOTH_SETTINGS))
                invoke.resolve()
            } catch (_: RuntimeException) {
                invoke.reject(null, "keyboard_settings_unavailable")
            }
        }
    }

    @Command
    fun select(invoke: Invoke) {
        val ids = invoke.parseArgs(SelectArgs::class.java).ids.toSet()
        main.post {
            ports.select(ids)
            invoke.resolve()
        }
    }

    @Command
    fun scanBluetooth(invoke: Invoke) {
        main.post {
            if (!ports.supported || !activity.packageManager.hasSystemFeature(PackageManager.FEATURE_BLUETOOTH_LE)) {
                invoke.reject(null, "bluetooth_unavailable")
            } else if (requestingPermission) {
                invoke.reject(null, "bluetooth_busy")
            } else if (canScan()) {
                scan(invoke)
            } else {
                requestingPermission = true
                val alias = if (Build.VERSION.SDK_INT >= 31) "nearby" else "location"
                requestPermissionForAlias(alias, invoke, "bluetoothPermissionResult")
            }
        }
    }

    @PermissionCallback
    fun bluetoothPermissionResult(invoke: Invoke) {
        main.post {
            requestingPermission = false
            if (canScan()) scan(invoke) else invoke.reject(null, "bluetooth_permission_denied")
        }
    }

    private fun canScan(): Boolean {
        fun granted(name: String) = activity.checkSelfPermission(name) == PackageManager.PERMISSION_GRANTED
        return if (Build.VERSION.SDK_INT >= 31) {
            granted(permission.BLUETOOTH_SCAN) && granted(permission.BLUETOOTH_CONNECT)
        } else {
            granted(permission.ACCESS_FINE_LOCATION)
        }
    }

    private fun scan(invoke: Invoke) {
        bluetooth.scan { result, error ->
            if (error != null) {
                invoke.reject(null, error)
            } else {
                ports.discovered(result)
                invoke.resolveObject(result.size)
            }
        }
    }

    override fun onDestroy() {
        main.post {
            bluetooth.close()
            ports.close()
            keyboards.close()
        }
    }
}
