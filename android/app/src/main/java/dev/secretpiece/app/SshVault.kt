package dev.secretpiece.app

import com.jcraft.jsch.ChannelSftp
import com.jcraft.jsch.JSch
import com.jcraft.jsch.JSchException
import java.io.ByteArrayOutputStream

/** SSH/SFTP server access for Android using the pure-Java JSch client. */
object SshVault {
    private fun session(preset: ServerPreset): com.jcraft.jsch.Session {
        val jsch = JSch()
        val user = preset.user.ifBlank { "root" }
        val session = jsch.getSession(user, preset.host, preset.port.toIntOrNull() ?: 22)
        session.setPassword(preset.password)
        val config = java.util.Properties()
        config["StrictHostKeyChecking"] = "no"
        config["UserKnownHostsFile"] = "/dev/null"
        session.setConfig(config)
        return session
    }

    private inline fun <T> withSftp(preset: ServerPreset, action: (ChannelSftp) -> T): T {
        val s = session(preset)
        s.connect(6000)
        val channel = s.openChannel("sftp") as ChannelSftp
        channel.connect(6000)
        return try {
            action(channel)
        } finally {
            channel.disconnect()
            s.disconnect()
        }
    }

    fun test(preset: ServerPreset): String = runCatching {
        withSftp(preset) { channel ->
            channel.stat(preset.path.trim())
            "ok"
        }
    }.getOrElse { it.message ?: "SSH failed" }

    fun download(preset: ServerPreset, local: String): String = runCatching {
        withSftp(preset) { channel ->
            val bytes = ByteArrayOutputStream()
            channel.get(preset.path.trim(), bytes)
            val data = bytes.toByteArray()
            if (data.isEmpty()) throw IllegalStateException("remote vault is empty")
            java.io.File(local).writeBytes(data)
            "ok"
        }
    }.getOrElse { it.message ?: "SSH download failed" }

    fun upload(preset: ServerPreset, local: String): String = runCatching {
        val data = java.io.File(local).readBytes()
        withSftp(preset) { channel ->
            channel.put(java.io.ByteArrayInputStream(data), preset.path.trim())
            "ok"
        }
    }.getOrElse { it.message ?: "SSH upload failed" }

    fun bytes(preset: ServerPreset): ByteArray? = runCatching {
        withSftp(preset) { channel ->
            val out = ByteArrayOutputStream()
            channel.get(preset.path.trim(), out)
            out.toByteArray()
        }
    }.getOrNull()

    fun putBytes(preset: ServerPreset, data: ByteArray): String = runCatching {
        withSftp(preset) { channel ->
            channel.put(java.io.ByteArrayInputStream(data), preset.path.trim())
            "ok"
        }
    }.getOrElse { it.message ?: "SSH upload failed" }
}
