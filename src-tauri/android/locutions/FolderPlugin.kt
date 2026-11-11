package lf.botonera.locutions

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.provider.DocumentsContract
import android.provider.DocumentsContract.Document
import androidx.activity.result.ActivityResult
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import org.json.JSONArray

@TauriPlugin
class FolderPlugin(private val activity: Activity) : Plugin(activity) {
    @Command
    fun pick(invoke: Invoke) {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE)
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        try {
            startActivityForResult(invoke, intent, "folderPicked")
        } catch (error: Exception) {
            invoke.reject("Cannot open the folder picker", error)
        }
    }

    @ActivityCallback
    fun folderPicked(invoke: Invoke, result: ActivityResult) {
        if (result.resultCode == Activity.RESULT_CANCELED) {
            invoke.resolve(JSObject().put("folder", org.json.JSONObject.NULL))
            return
        }

        val tree = result.data?.data
        if (result.resultCode != Activity.RESULT_OK || tree == null) {
            invoke.reject("No folder returned by Android")
            return
        }

        Thread {
            try {
                invoke.resolve(JSObject().put("folder", listFolder(tree)))
            } catch (error: Exception) {
                invoke.reject("Cannot list the selected folder", error)
            }
        }.start()
    }

    private fun listFolder(tree: Uri): JSObject {
        val resolver = activity.contentResolver
        val id = DocumentsContract.getTreeDocumentId(tree)
        val directory = DocumentsContract.buildDocumentUriUsingTree(tree, id)
        val name = resolver.query(directory, arrayOf(Document.COLUMN_DISPLAY_NAME), null, null, null)
            ?.use { if (it.moveToFirst()) it.getString(0) else null } ?: "audio"
        val children = DocumentsContract.buildChildDocumentsUriUsingTree(tree, id)
        val columns = arrayOf(Document.COLUMN_DOCUMENT_ID, Document.COLUMN_DISPLAY_NAME, Document.COLUMN_MIME_TYPE)
        val files = JSONArray()
        requireNotNull(resolver.query(children, columns, null, null, null)).use { cursor ->
            while (cursor.moveToNext()) {
                if (cursor.getString(2) == Document.MIME_TYPE_DIR) continue
                val uri = DocumentsContract.buildDocumentUriUsingTree(tree, cursor.getString(0))
                files.put(JSObject().put("name", cursor.getString(1)).put("uri", uri.toString()))
            }
        }
        return JSObject().put("name", name).put("files", files)
    }
}
