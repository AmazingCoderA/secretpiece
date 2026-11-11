package dev.secretpiece.app

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Bundle
import android.provider.OpenableColumns
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

private const val PREFS = "secretpiece"
private const val PREF_DISGUISE = "disguise"

private val Bg = Color(0xFF05080F)
private val SurfaceDark = Color(0xFF0D1424)
private val Cyan = Color(0xFF4DE8FF)
private val Violet = Color(0xFF9B6BFF)
private val Muted = Color(0xFF8EA3C8)
private val Danger = Color(0xFFFF7A90)

private enum class Lang { Ru, En }

private data class Copy(
    val tagline: String,
    val pick: String,
    val chosen: String,
    val password: String,
    val open: String,
    val ok: String,
    val scenarioTitle: String,
    val scenario: List<String>,
    val platforms: String,
    val advancedTitle: String,
    val maskTitle: String,
    val maskSummary: String,
    val maskNote: String,
    val maskOnTitle: String,
    val maskOnBody: String,
    val maskOffTitle: String,
    val maskOffBody: String,
    val browserTitle: String,
    val readingMode: String,
    val up: String,
    val emptyFolder: String,
    val lock: String,
    val close: String,
    val openFailed: String,
    val readFailed: String,
    val binaryFile: String,
    val verify: String,
    val integrityOk: String,
    val integrityFail: String,
)

private fun copyFor(lang: Lang): Copy = if (lang == Lang.Ru) {
    Copy(
        tagline = "Зашифрованный сейф в обычном файле",
        pick = "Выбрать файл .spv",
        chosen = "Выбран файл",
        password = "Пароль",
        open = "Открыть хранилище",
        ok = "Понятно",
        scenarioTitle = "Сценарий: флешка + телефон",
        scenario = listOf(
            "1. Подключите USB-флешку через OTG/USB-C.",
            "2. Нажмите «Выбрать файл .spv» — откроется системный проводник.",
            "3. Введите пароль и откройте хранилище.",
            "4. Читайте и копируйте файлы прямо с флешки.",
        ),
        platforms = "Android 8+ · ARM64 · без root · доступ через SAF",
        advancedTitle = "Дополнительно",
        maskTitle = "Маскировка приложения",
        maskSummary = "Иконка и название «Калькулятор» в лаунчере.",
        maskNote = "Опционально. Не меняет файл контейнера и его пароль.",
        maskOnTitle = "Маскировка включена",
        maskOnBody = "Приложение теперь называется «Калькулятор». Откройте его из лаунчера. " +
            "Выключите переключатель, чтобы вернуть название SecretPiece.",
        maskOffTitle = "Маскировка выключена",
        maskOffBody = "Иконка и название SecretPiece снова видны в лаунчере.",
        browserTitle = "Хранилище",
        readingMode = "только чтение · прямо с носителя",
        up = "Вверх",
        emptyFolder = "Папка пуста",
        lock = "Закрыть",
        close = "Закрыть",
        openFailed = "Не удалось открыть: неверный пароль или повреждённый файл.",
        readFailed = "Не удалось прочитать файл.",
        binaryFile = "Двоичный файл — предпросмотр недоступен.",
        verify = "Проверить",
        integrityOk = "Целостность в порядке.",
        integrityFail = "Проверка не пройдена: данные повреждены.",
    )
} else {
    Copy(
        tagline = "An encrypted vault inside an ordinary file",
        pick = "Choose .spv file",
        chosen = "Selected file",
        password = "Password",
        open = "Open vault",
        ok = "Got it",
        scenarioTitle = "Scenario: USB drive + phone",
        scenario = listOf(
            "1. Plug a USB drive in via OTG/USB-C.",
            "2. Tap \"Choose .spv file\" — the system picker opens.",
            "3. Enter the password and open the vault.",
            "4. Read and copy files directly from the drive.",
        ),
        platforms = "Android 8+ · ARM64 · no root · SAF access",
        advancedTitle = "Advanced",
        maskTitle = "Disguise app",
        maskSummary = "Show «Calculator» as the launcher icon and name.",
        maskNote = "Optional. Does not change the container file or its password.",
        maskOnTitle = "Disguise enabled",
        maskOnBody = "The app is now listed as Calculator. Open it from the launcher. " +
            "Turn the switch off to restore the SecretPiece name.",
        maskOffTitle = "Disguise disabled",
        maskOffBody = "The SecretPiece icon and name are back on the launcher.",
        browserTitle = "Vault",
        readingMode = "read-only · straight from the drive",
        up = "Up",
        emptyFolder = "This folder is empty",
        lock = "Lock",
        close = "Close",
        openFailed = "Cannot open: wrong password or a damaged file.",
        readFailed = "Cannot read the file.",
        binaryFile = "Binary file — preview is not available.",
        verify = "Verify",
        integrityOk = "Integrity check passed.",
        integrityFail = "Integrity check failed: data is damaged.",
    )
}

class MainActivity : ComponentActivity() {
    private val incomingUri = mutableStateOf<Uri?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        syncDisguise(this)
        incomingUri.value = intent?.data
        setContent { SecretPieceApp(incomingUri.value) }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        incomingUri.value = intent.data
    }
}

private fun disguiseEnabled(context: Context): Boolean =
    context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        .getBoolean(PREF_DISGUISE, false)

private fun launcherAlias(context: Context, suffix: String): ComponentName =
    ComponentName(context, "${context.packageName}.$suffix")

private fun applyDisguise(context: Context, enabled: Boolean) {
    val pm = context.packageManager
    val normal = launcherAlias(context, "LauncherSecretPiece")
    val calculator = launcherAlias(context, "LauncherCalculator")
    val enabledState = PackageManager.COMPONENT_ENABLED_STATE_ENABLED
    val disabledState = PackageManager.COMPONENT_ENABLED_STATE_DISABLED
    pm.setComponentEnabledSetting(
        normal,
        if (enabled) disabledState else enabledState,
        PackageManager.DONT_KILL_APP,
    )
    pm.setComponentEnabledSetting(
        calculator,
        if (enabled) enabledState else disabledState,
        PackageManager.DONT_KILL_APP,
    )
    context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
        .edit()
        .putBoolean(PREF_DISGUISE, enabled)
        .apply()
}

/** Re-apply the saved choice after a reinstall or a package manager reset. */
private fun syncDisguise(context: Context) {
    val wanted = disguiseEnabled(context)
    val expected = if (wanted) {
        PackageManager.COMPONENT_ENABLED_STATE_ENABLED
    } else {
        PackageManager.COMPONENT_ENABLED_STATE_DISABLED
    }
    val current = context.packageManager
        .getComponentEnabledSetting(launcherAlias(context, "LauncherCalculator"))
    if (current != expected) applyDisguise(context, wanted)
}

@Composable
fun SecretPieceApp(initialUri: Uri? = null) {
    var lang by remember { mutableStateOf(Lang.Ru) }
    val t = copyFor(lang)

    MaterialTheme(
        colorScheme = darkColorScheme(
            primary = Cyan,
            secondary = Violet,
            background = Bg,
            surface = SurfaceDark,
            onPrimary = Color(0xFF031018),
            onBackground = Color(0xFFE8F0FF),
            onSurface = Color(0xFFE8F0FF),
        ),
    ) {
        Surface(modifier = Modifier.fillMaxSize(), color = Bg) {
            VaultScreen(t, lang, initialUri) { lang = it }
        }
    }
}

@Composable
private fun VaultScreen(
    t: Copy,
    lang: Lang,
    initialUri: Uri?,
    onLang: (Lang) -> Unit,
) {
    val context = LocalContext.current
    var fileUri by remember { mutableStateOf(initialUri) }
    var fileName by remember {
        mutableStateOf(initialUri?.let { displayName(context, it) } ?: "")
    }
    var password by remember { mutableStateOf("") }
    var masked by remember { mutableStateOf(disguiseEnabled(context)) }
    var maskDialog by remember { mutableStateOf<String?>(null) }
    var handle by remember { mutableStateOf(0L) }
    var openError by remember { mutableStateOf<String?>(null) }

    DisposableEffect(handle) {
        onDispose {
            if (handle != 0L) NativeVault.close(handle)
        }
    }

    if (handle != 0L) {
        VaultBrowser(handle, t) {
            handle = 0L
        }
        return
    }

    val picker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri != null) {
            runCatching {
                context.contentResolver.takePersistableUriPermission(
                    uri,
                    Intent.FLAG_GRANT_READ_URI_PERMISSION,
                )
            }
            fileUri = uri
            fileName = displayName(context, uri)
        }
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Box(
                    modifier = Modifier
                        .size(44.dp)
                        .clip(RoundedCornerShape(13.dp))
                        .background(Brush.linearGradient(listOf(Cyan, Violet))),
                    contentAlignment = Alignment.Center,
                ) {
                    Box(
                        modifier = Modifier
                            .size(18.dp)
                            .clip(CircleShape)
                            .background(Bg),
                    )
                }
                Spacer(Modifier.width(12.dp))
                Column {
                    Text(
                        "SecretPiece",
                        fontSize = 26.sp,
                        fontWeight = FontWeight.ExtraBold,
                        color = Cyan,
                    )
                    Text("Your secret, in pieces.", fontSize = 12.sp, color = Muted)
                }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                FilterChip(
                    selected = lang == Lang.Ru,
                    onClick = { onLang(Lang.Ru) },
                    label = { Text("RU") },
                )
                FilterChip(
                    selected = lang == Lang.En,
                    onClick = { onLang(Lang.En) },
                    label = { Text("EN") },
                )
            }
        }

        Text(t.tagline, fontSize = 20.sp, fontWeight = FontWeight.Bold, color = Color(0xFFE8F0FF))
        Text(t.platforms, fontSize = 12.sp, color = Muted, fontFamily = FontFamily.Monospace)

        Card(
            colors = CardDefaults.cardColors(containerColor = SurfaceDark),
            shape = RoundedCornerShape(20.dp),
            modifier = Modifier.fillMaxWidth(),
        ) {
            Column(Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                Text(
                    t.scenarioTitle,
                    fontSize = 16.sp,
                    fontWeight = FontWeight.Bold,
                    color = Color(0xFFE8F0FF),
                )
                t.scenario.forEach { line ->
                    Text(line, fontSize = 13.sp, color = Muted)
                }
            }
        }

        Button(
            onClick = { picker.launch(arrayOf("*/*")) },
            modifier = Modifier.fillMaxWidth().height(52.dp),
            shape = RoundedCornerShape(16.dp),
            colors = ButtonDefaults.buttonColors(containerColor = Cyan),
        ) {
            Text(t.pick, color = Color(0xFF031018), fontWeight = FontWeight.Bold)
        }

        if (fileUri != null) {
            Text("${t.chosen}: $fileName", fontSize = 13.sp, color = Cyan)
        }

        OutlinedTextField(
            value = password,
            onValueChange = { password = it },
            label = { Text(t.password) },
            singleLine = true,
            visualTransformation = PasswordVisualTransformation(),
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(16.dp),
        )

        Button(
            onClick = {
                val uri = fileUri
                if (uri != null && password.isNotEmpty()) {
                    openError = null
                    runCatching {
                        val descriptor = context.contentResolver
                            .openFileDescriptor(uri, "r")
                            ?: throw IllegalStateException("no descriptor")
                        val opened = NativeVault.openFd(descriptor.detachFd(), password)
                        if (opened == 0L) throw IllegalStateException("unlock failed")
                        handle = opened
                        password = ""
                    }.onFailure {
                        openError = t.openFailed
                    }
                }
            },
            enabled = fileUri != null && password.isNotEmpty(),
            modifier = Modifier.fillMaxWidth().height(52.dp),
            shape = RoundedCornerShape(16.dp),
            colors = ButtonDefaults.buttonColors(containerColor = Violet),
        ) {
            Text(t.open, color = Color(0xFF0B0618), fontWeight = FontWeight.Bold)
        }

        openError?.let { message ->
            Text(message, fontSize = 13.sp, color = Color(0xFFFF7A90))
        }

        Text(
            "SPV1 · XChaCha20-Poly1305 · Argon2id · BLAKE3",
            fontSize = 11.sp,
            color = Muted,
            fontFamily = FontFamily.Monospace,
        )

        Text(
            t.advancedTitle,
            fontSize = 15.sp,
            fontWeight = FontWeight.Bold,
            color = Color(0xFFE8F0FF),
            modifier = Modifier.padding(top = 6.dp),
        )

        Card(
            colors = CardDefaults.cardColors(containerColor = SurfaceDark),
            shape = RoundedCornerShape(18.dp),
            modifier = Modifier.fillMaxWidth(),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth().padding(16.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text(
                        t.maskTitle,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color(0xFFE8F0FF),
                    )
                    Text(t.maskSummary, fontSize = 12.sp, color = Muted)
                    Text(t.maskNote, fontSize = 11.sp, color = Color(0xFF6F84A8))
                }
                Switch(
                    checked = masked,
                    onCheckedChange = { checked ->
                        applyDisguise(context, checked)
                        masked = checked
                        maskDialog = if (checked) "on" else "off"
                    },
                )
            }
        }
    }

    maskDialog?.let { which ->
        AlertDialog(
            onDismissRequest = { maskDialog = null },
            confirmButton = {
                TextButton(onClick = { maskDialog = null }) { Text(t.ok) }
            },
            title = { Text(if (which == "on") t.maskOnTitle else t.maskOffTitle) },
            text = { Text(if (which == "on") t.maskOnBody else t.maskOffBody) },
        )
    }

}

@Composable
private fun VaultBrowser(handle: Long, t: Copy, onLock: () -> Unit) {
    var prefix by remember { mutableStateOf("") }
    var tick by remember { mutableStateOf(0) }
    var error by remember { mutableStateOf<String?>(null) }
    var notice by remember { mutableStateOf<String?>(null) }
    var previewTitle by remember { mutableStateOf("") }
    var preview by remember { mutableStateOf<String?>(null) }

    val entries = remember(prefix, tick) { parseEntries(NativeVault.list(handle, prefix)) }
    val stats = remember(handle, tick) { parseStats(NativeVault.info(handle)) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(20.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Column {
                Text(
                    t.browserTitle,
                    fontSize = 24.sp,
                    fontWeight = FontWeight.ExtraBold,
                    color = Cyan,
                )
                Text(
                    t.readingMode,
                    fontSize = 11.sp,
                    color = Muted,
                    fontFamily = FontFamily.Monospace,
                )
            }
            Button(
                onClick = onLock,
                shape = RoundedCornerShape(14.dp),
                colors = ButtonDefaults.buttonColors(containerColor = SurfaceDark),
            ) {
                Text(t.lock, color = Cyan, fontWeight = FontWeight.Bold)
            }
        }

        stats?.let { data ->
            Text(
                "${humanBytes(data.bytes)} · ${data.files} ${t.browserTitle.lowercase()} · " +
                    "SPV1 · gen ${data.generation}",
                fontSize = 11.sp,
                color = Muted,
                fontFamily = FontFamily.Monospace,
            )
        }

        if (prefix.isNotEmpty()) {
            TextButton(onClick = {
                prefix = prefix.substringBeforeLast('/', "")
                error = null
            }) {
                Text("← ${t.up}", color = Cyan)
            }
        }

        if (entries.isEmpty()) {
            Text(t.emptyFolder, fontSize = 13.sp, color = Muted)
        }

        entries.forEach { entry ->
            Card(
                colors = CardDefaults.cardColors(containerColor = SurfaceDark),
                shape = RoundedCornerShape(14.dp),
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable {
                        error = null
                        if (entry.isDir) {
                            prefix = entry.path
                        } else {
                            val data = NativeVault.read(handle, entry.path)
                            previewTitle = entry.name
                            preview = when {
                                data == null -> {
                                    error = t.readFailed
                                    null
                                }
                                data.size > 2_000_000 || data.any { it == 0.toByte() } ->
                                    t.binaryFile
                                else -> String(data, Charsets.UTF_8)
                                    .takeIf { !it.contains('\uFFFD') }
                                    ?: t.binaryFile
                            }
                        }
                    },
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(14.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                        Text(
                            entry.name,
                            fontSize = 15.sp,
                            fontWeight = FontWeight.Medium,
                            color = Color(0xFFE8F0FF),
                        )
                        Text(
                            if (entry.isDir) t.browserTitle else humanBytes(entry.size),
                            fontSize = 11.sp,
                            color = Muted,
                        )
                    }
                    if (entry.isDir) Text("›", fontSize = 20.sp, color = Muted)
                }
            }
        }

        error?.let { Text(it, fontSize = 13.sp, color = Danger) }
        notice?.let { Text(it, fontSize = 13.sp, color = Cyan) }

        TextButton(onClick = {
            notice = if (NativeVault.verify(handle) == 1) t.integrityOk else t.integrityFail
        }) {
            Text(t.verify, color = Violet)
        }
    }

    preview?.let { text ->
        AlertDialog(
            onDismissRequest = { preview = null },
            confirmButton = {
                TextButton(onClick = { preview = null }) { Text(t.close) }
            },
            title = { Text(previewTitle) },
            text = {
                Column(
                    modifier = Modifier
                        .height(360.dp)
                        .verticalScroll(rememberScrollState()),
                ) {
                    Text(text, fontFamily = FontFamily.Monospace, fontSize = 12.sp)
                }
            },
        )
    }
}

private fun displayName(context: Context, uri: Uri): String {
    runCatching {
        context.contentResolver
            .query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)
            ?.use { cursor ->
                val index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                if (index >= 0 && cursor.moveToFirst()) {
                    return cursor.getString(index)
                }
            }
    }
    return uri.lastPathSegment ?: "vault.spv"
}
