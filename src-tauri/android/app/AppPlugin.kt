package lf.botonera.app

import android.app.Activity
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback
import androidx.appcompat.app.AppCompatActivity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin

@InvokeArg
internal class BackArgs {
    lateinit var channel: Channel
}

@TauriPlugin
class AppPlugin(private val activity: Activity) : Plugin(activity) {
    private val main = Handler(Looper.getMainLooper())
    private var channel: Channel? = null
    private val callback = object : OnBackPressedCallback(true) {
        override fun handleOnBackPressed() {
            channel?.sendObject(mapOf("kind" to "back"))
        }
    }

    override fun load(webView: WebView) {
        val host = activity as AppCompatActivity
        host.onBackPressedDispatcher.addCallback(host, callback)
        main.postDelayed({
            webView.evaluateJavascript("JSON.stringify({ready:document.readyState,marks:performance.getEntriesByType('mark').map(e=>e.name),stage:document.getElementById('startup-stage')?.textContent,bridge:!!window.__TAURI__})") {
                android.util.Log.i("LFStartup", it)
            }
        }, 20000)
    }

    @Command
    fun attachBack(invoke: Invoke) {
        val args = invoke.parseArgs(BackArgs::class.java)
        main.post {
            channel = args.channel
            callback.isEnabled = true
            invoke.resolve()
        }
    }

    @Command
    fun getInstallerSource(invoke: Invoke) {
        try {
            val source = if (Build.VERSION.SDK_INT >= 30) {
                activity.packageManager.getInstallSourceInfo(activity.packageName).installingPackageName
            } else {
                @Suppress("DEPRECATION")
                activity.packageManager.getInstallerPackageName(activity.packageName)
            }
            invoke.resolveObject(source.orEmpty())
        } catch (_: Exception) {
            invoke.reject(null, "installer_source_unavailable")
        }
    }

    @Command
    fun exitApp(invoke: Invoke) {
        main.post {
            invoke.resolve()
            activity.finishAndRemoveTask()
        }
    }

    override fun onDestroy(activity: AppCompatActivity) {
        callback.remove()
        channel = null
    }
}
