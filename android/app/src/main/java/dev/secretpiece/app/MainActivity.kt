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

private enum class Lang { Ru, En }

private data class Copy(
    val tagline: String,
    val pick: String,
    val chosen: String,
    val password: String,
    val open: String,
    val pendingTitle: String,
    val pendingBody: String,
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
)

private fun copyFor(lang: Lang): Copy = if (lang == Lang.Ru) {
    Copy(
        tagline = "Зашифрованный сейф в обычном файле",
        pick = "Выбрать файл .spv",
        chosen = "Выбран файл",
        password = "Пароль",
        open = "Открыть хранилище",
        pendingTitle = "Криптоядро ещё не подключено",
        pendingBody = "UI и доступ к флешке уже работают. Argon2id и XChaCha20-Poly1305 " +
            "подключаются в v1.4 через общий Rust-крейт sp-ffi — без переписывания формата.",
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
    )
} else {
    Copy(
        tagline = "An encrypted vault inside an ordinary file",
        pick = "Choose .spv file",
        chosen = "Selected file",
        password = "Password",
        open = "Open vault",
        pendingTitle = "Crypto core is not wired yet",
        pendingBody = "The UI and USB access already work. Argon2id and XChaCha20-Poly1305 " +
            "land in v1.4 through the shared Rust crate sp-ffi — without changing the format.",
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
    var showPending by remember { mutableStateOf(false) }
    var masked by remember { mutableStateOf(disguiseEnabled(context)) }
    var maskDialog by remember { mutableStateOf<String?>(null) }

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
            onClick = { showPending = true },
            enabled = fileUri != null && password.isNotEmpty(),
            modifier = Modifier.fillMaxWidth().height(52.dp),
            shape = RoundedCornerShape(16.dp),
            colors = ButtonDefaults.buttonColors(containerColor = Violet),
        ) {
            Text(t.open, color = Color(0xFF0B0618), fontWeight = FontWeight.Bold)
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

    if (showPending) {
        AlertDialog(
            onDismissRequest = { showPending = false },
            confirmButton = {
                TextButton(onClick = { showPending = false }) { Text(t.ok) }
            },
            title = { Text(t.pendingTitle) },
            text = { Text(t.pendingBody) },
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
