package dev.secretpiece.app

import org.json.JSONArray
import org.json.JSONObject

/**
 * JNI bridge over the shared Rust core (`sp-ffi`).
 *
 * The vault is opened straight from the SAF file descriptor: the container is
 * never copied into app storage, so reading from an OTG USB drive stays direct.
 * All operations here are read-only.
 */
object NativeVault {
    init {
        System.loadLibrary("sp_ffi")
    }

    external fun openFd(fd: Int, password: String): Long
    external fun createFd(fd: Int, password: String): Long
    external fun list(handle: Long, prefix: String): String
    external fun read(handle: Long, path: String): ByteArray?
    external fun info(handle: Long): String?
    external fun verify(handle: Long): Int
    external fun close(handle: Long)
    external fun version(): String
}

data class VaultEntry(
    val name: String,
    val path: String,
    val isDir: Boolean,
    val size: Long,
)

data class VaultStats(
    val bytes: Long,
    val payload: Long,
    val files: Long,
    val dirs: Long,
    val generation: Long,
)

fun parseEntries(json: String?): List<VaultEntry> = runCatching {
    val array = JSONArray(json ?: "[]")
    buildList {
        for (index in 0 until array.length()) {
            val item = array.getJSONObject(index)
            add(
                VaultEntry(
                    name = item.optString("name"),
                    path = item.optString("path"),
                    isDir = item.optString("kind") == "dir",
                    size = item.optLong("size"),
                ),
            )
        }
    }.sortedWith(compareByDescending<VaultEntry> { it.isDir }.thenBy { it.name.lowercase() })
}.getOrDefault(emptyList())

fun parseStats(json: String?): VaultStats? = runCatching {
    val objectValue = JSONObject(json ?: return null)
    VaultStats(
        bytes = objectValue.optLong("bytes"),
        payload = objectValue.optLong("payload"),
        files = objectValue.optLong("files"),
        dirs = objectValue.optLong("dirs"),
        generation = objectValue.optLong("generation"),
    )
}.getOrNull()

fun humanBytes(bytes: Long): String {
    if (bytes < 1024) return "$bytes B"
    val units = listOf("KiB", "MiB", "GiB", "TiB")
    var value = bytes.toDouble()
    var unit = -1
    while (value >= 1024.0 && unit < units.lastIndex) {
        value /= 1024.0
        unit++
    }
    return String.format("%.1f %s", value, units[unit])
}
