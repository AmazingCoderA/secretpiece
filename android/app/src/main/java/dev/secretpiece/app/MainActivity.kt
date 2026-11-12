package dev.secretpiece.app

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.content.pm.ShortcutInfo
import android.content.pm.ShortcutManager
import android.graphics.drawable.Icon
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.OpenableColumns
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
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
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
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
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.sin

private const val PREFS = "secretpiece"
private const val PREF_DISGUISE = "disguise"
private const val PREF_SKY = "sky"
private const val PREF_LANG = "lang"

private val Bg = Color(0xFF05080F)
private val Deep = Color(0xFF0A1020)
private val SurfaceDark = Color(0xFF0D1424)
private val SurfaceSoft = Color(0xFF121B2E)
private val Cyan = Color(0xFF4DE8FF)
private val Violet = Color(0xFF9B6BFF)
private val Muted = Color(0xFF8EA3C8)
private val Faint = Color(0xFF6F84A8)
private val Danger = Color(0xFFFF7A90)
private val Gold = Color(0xFFFFD48A)

private enum class Lang { Ru, En }

/** Ambient effect drawn behind the whole app. */
private enum class Sky { None, Snow, Rain, Sun }

private data class Copy(
    val tagline: String,
    val pick: String,
    val chosen: String,
    val password: String,
    val open: String,
    val create: String,
    val createHint: String,
    val createFailed: String,
    val unlockTitle: String,
    val unlockHint: String,
    val show: String,
    val hide: String,
    val ok: String,
    val done: String,
    val settingsTitle: String,
    val lookTitle: String,
    val lookHint: String,
    val language: String,
    val shortcut: String,
    val shortcutHint: String,
    val skyNone: String,
    val skySnow: String,
    val skyRain: String,
    val skySun: String,
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
        create = "Создать новый сейф",
        createHint = "Пустой контейнер .spv на флешке или в памяти телефона.",
        createFailed = "Не удалось создать хранилище.",
        unlockTitle = "Открыть сейф",
        unlockHint = "Пароль не покидает устройство: ключ выводится через Argon2id.",
        show = "Показать",
        hide = "Скрыть",
        ok = "Понятно",
        done = "Готово",
        settingsTitle = "Настройки",
        lookTitle = "Атмосфера",
        lookHint = "Живой фон приложения. На работу сейфа не влияет.",
        language = "Язык",
        shortcut = "Создать ярлык",
        shortcutHint = "Закрепить SecretPiece на главном экране.",
        skyNone = "Нет",
        skySnow = "Снег",
        skyRain = "Дождь",
        skySun = "Солнце",
        scenarioTitle = "Сценарий: флешка + телефон",
        scenario = listOf(
            "1. Подключите USB-флешку через OTG/USB-C.",
            "2. Нажмите «Выбрать файл .spv» — откроется системный проводник.",
            "3. Введите пароль и откройте хранилище.",
            "4. Читайте и копируйте файлы прямо с флешки.",
        ),
        platforms = "Android 8+ · ARM64/ARMv7 · без root · доступ через SAF",
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
        create = "Create new vault",
        createHint = "An empty .spv container on the drive or in phone storage.",
        createFailed = "Cannot create the vault.",
        unlockTitle = "Unlock your vault",
        unlockHint = "The password never leaves the device: the key is derived with Argon2id.",
        show = "Show",
        hide = "Hide",
        ok = "Got it",
        done = "Done",
        settingsTitle = "Settings",
        lookTitle = "Ambience",
        lookHint = "A living background. It does not affect the vault.",
        language = "Language",
        shortcut = "Create shortcut",
        shortcutHint = "Pin SecretPiece to the home screen.",
        skyNone = "None",
        skySnow = "Snow",
        skyRain = "Rain",
        skySun = "Sun",
        scenarioTitle = "Scenario: USB drive + phone",
        scenario = listOf(
            "1. Plug a USB drive in via OTG/USB-C.",
            "2. Tap \"Choose .spv file\" — the system picker opens.",
            "3. Enter the password and open the vault.",
            "4. Read and copy files directly from the drive.",
        ),
        platforms = "Android 8+ · ARM64/ARMv7 · no root · SAF access",
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

private fun prefs(context: Context) =
    context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

private fun disguiseEnabled(context: Context): Boolean =
    prefs(context).getBoolean(PREF_DISGUISE, false)

private fun loadLang(context: Context): Lang =
    if (prefs(context).getString(PREF_LANG, "ru") == "en") Lang.En else Lang.Ru

private fun loadSky(context: Context): Sky =
    when (prefs(context).getString(PREF_SKY, "none")) {
        "snow" -> Sky.Snow
        "rain" -> Sky.Rain
        "sun" -> Sky.Sun
        else -> Sky.None
    }

private fun saveSky(context: Context, sky: Sky) {
    val value = when (sky) {
        Sky.None -> "none"
        Sky.Snow -> "snow"
        Sky.Rain -> "rain"
        Sky.Sun -> "sun"
    }
    prefs(context).edit().putString(PREF_SKY, value).apply()
}

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
    prefs(context).edit().putBoolean(PREF_DISGUISE, enabled).apply()
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

/** Asks the launcher to pin a home-screen shortcut. */
private fun pinShortcut(context: Context) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
    val manager = context.getSystemService(ShortcutManager::class.java) ?: return
    if (!manager.isRequestPinShortcutSupported) return
    val intent = Intent(context, MainActivity::class.java)
        .setAction(Intent.ACTION_MAIN)
        .addCategory(Intent.CATEGORY_LAUNCHER)
    val shortcut = ShortcutInfo.Builder(context, "secretpiece-vault")
        .setShortLabel("SecretPiece")
        .setLongLabel("SecretPiece encrypted vault")
        .setIcon(Icon.createWithResource(context, R.mipmap.ic_launcher))
        .setIntent(intent)
        .build()
    runCatching { manager.requestPinShortcut(shortcut, null) }
}

@Composable
fun SecretPieceApp(initialUri: Uri? = null) {
    val context = LocalContext.current
    var lang by remember { mutableStateOf(loadLang(context)) }
    var sky by remember { mutableStateOf(loadSky(context)) }
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
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Brush.verticalGradient(listOf(Bg, Deep, Bg))),
        ) {
            Ambient(sky)
            Surface(modifier = Modifier.fillMaxSize(), color = Color.Transparent) {
                VaultScreen(
                    t = t,
                    lang = lang,
                    initialUri = initialUri,
                    onLang = {
                        lang = it
                        prefs(context).edit()
                            .putString(PREF_LANG, if (it == Lang.En) "en" else "ru")
                            .apply()
                    },
                    onSky = {
                        sky = it
                        saveSky(context, it)
                    },
                )
            }
        }
    }
}

@Composable
private fun Ambient(sky: Sky) {
    if (sky == Sky.None) return
    val transition = rememberInfiniteTransition(label = "ambient")
    val drift by transition.animateFloat(
        initialValue = 0f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(
            animation = tween(
                durationMillis = if (sky == Sky.Rain) 1100 else 9000,
                easing = LinearEasing,
            ),
            repeatMode = RepeatMode.Restart,
        ),
        label = "drift",
    )
    Canvas(modifier = Modifier.fillMaxSize()) {
        when (sky) {
            Sky.Snow -> {
                for (index in 0 until 90) {
                    val seed = (index * 0.6180339f) % 1f
                    val sway = sin((drift * 6.283f) + index.toFloat()) * size.width * 0.02f
                    val x = ((seed + drift * 0.06f) % 1f) * size.width + sway
                    val y = ((seed * 1.7f + drift) % 1f) * size.height
                    val radius = 1.1f + (index % 4) * 0.75f
                    drawCircle(
                        color = Color(0xCCBBD8FF),
                        radius = radius,
                        center = Offset(x, y),
                    )
                }
            }
            Sky.Rain -> {
                for (index in 0 until 80) {
                    val seed = (index * 0.7548776f) % 1f
                    val x = ((seed + drift * 0.22f) % 1f) * size.width
                    val y = ((seed * 1.3f + drift * 1.15f) % 1f) * size.height
                    drawLine(
                        color = Color(0x554DE8FF),
                        start = Offset(x, y),
                        end = Offset(x - 14f, y + 46f),
                        strokeWidth = 1.6f,
                    )
                }
            }
            Sky.Sun -> {
                val cx = size.width * 0.82f
                val cy = size.height * 0.10f
                val pulse = 0.5f + 0.5f * sin(drift * 6.283f)
                drawCircle(
                    brush = Brush.radialGradient(
                        colors = listOf(Color(0x33FFC46B), Color(0x00FFC46B)),
                        center = Offset(cx, cy),
                        radius = 240f + pulse * 30f,
                    ),
                    radius = 240f + pulse * 30f,
                    center = Offset(cx, cy),
                )
                drawCircle(
                    color = Color(0x22FFD48A),
                    radius = 64f + pulse * 8f,
                    center = Offset(cx, cy),
                )
                drawCircle(
                    color = Gold,
                    radius = 22f + pulse * 2.5f,
                    center = Offset(cx, cy),
                )
            }
            Sky.None -> Unit
        }
    }
}

@Composable
private fun VaultScreen(
    t: Copy,
    lang: Lang,
    initialUri: Uri?,
    onLang: (Lang) -> Unit,
    onSky: (Sky) -> Unit,
) {
    val context = LocalContext.current
    var fileUri by remember { mutableStateOf(initialUri) }
    var fileName by remember {
        mutableStateOf(initialUri?.let { displayName(context, it) } ?: "")
    }
    var password by remember { mutableStateOf("") }
    var visible by remember { mutableStateOf(false) }
    var masked by remember { mutableStateOf(disguiseEnabled(context)) }
    var maskDialog by remember { mutableStateOf<String?>(null) }
    var settings by remember { mutableStateOf(false) }
    var sky by remember { mutableStateOf(loadSky(context)) }
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
            openError = null
        }
    }

    val creator = rememberLauncherForActivityResult(
        ActivityResultContracts.CreateDocument("application/x-secretpiece"),
    ) { uri ->
        if (uri != null) {
            openError = null
            runCatching {
                val descriptor = context.contentResolver
                    .openFileDescriptor(uri, "rw")
                    ?: throw IllegalStateException("no descriptor")
                val created = NativeVault.createFd(descriptor.detachFd(), password)
                if (created == 0L) throw IllegalStateException("create failed")
                handle = created
                password = ""
            }.onFailure {
                openError = t.createFailed
            }
        }
    }

    fun unlock() {
        val uri = fileUri
        if (uri == null || password.isEmpty()) return
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

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .imePadding()
            .padding(horizontal = 20.dp, vertical = 18.dp),
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
                        .size(46.dp)
                        .clip(RoundedCornerShape(14.dp))
                        .background(Brush.linearGradient(listOf(Cyan, Violet))),
                    contentAlignment = Alignment.Center,
                ) {
                    Box(
                        modifier = Modifier
                            .size(19.dp)
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
                TextButton(onClick = { settings = true }) {
                    Text("⚙", fontSize = 20.sp, color = Cyan)
                }
            }
        }

        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(
                t.tagline,
                fontSize = 24.sp,
                fontWeight = FontWeight.ExtraBold,
                color = Color(0xFFE8F0FF),
            )
            Text(t.platforms, fontSize = 11.sp, color = Muted, fontFamily = FontFamily.Monospace)
        }

        Card(
            colors = CardDefaults.cardColors(containerColor = SurfaceDark),
            shape = RoundedCornerShape(24.dp),
            modifier = Modifier.fillMaxWidth(),
        ) {
            Column(
                modifier = Modifier.padding(20.dp),
                verticalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                Text(
                    t.unlockTitle,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold,
                    color = Color(0xFFE8F0FF),
                )
                Text(t.unlockHint, fontSize = 12.sp, color = Muted)

                OutlinedButton(
                    onClick = { picker.launch(arrayOf("*/*")) },
                    modifier = Modifier.fillMaxWidth().height(54.dp),
                    shape = RoundedCornerShape(16.dp),
                ) {
                    Text(
                        "＋  ${t.pick}",
                        color = Cyan,
                        fontWeight = FontWeight.Bold,
                        fontSize = 15.sp,
                    )
                }

                OutlinedButton(
                    onClick = { creator.launch("vault.spv") },
                    enabled = password.isNotEmpty(),
                    modifier = Modifier.fillMaxWidth().height(52.dp),
                    shape = RoundedCornerShape(16.dp),
                ) {
                    Text(
                        "✚  ${t.create}",
                        color = Violet,
                        fontWeight = FontWeight.Bold,
                        fontSize = 14.sp,
                    )
                }
                Text(t.createHint, fontSize = 11.sp, color = Faint)

                if (fileUri != null) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(14.dp))
                            .background(SurfaceSoft)
                            .padding(horizontal = 14.dp, vertical = 10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Box(
                            modifier = Modifier
                                .size(8.dp)
                                .clip(CircleShape)
                                .background(Cyan),
                        )
                        Column(Modifier.weight(1f)) {
                            Text(t.chosen, fontSize = 10.sp, color = Faint)
                            Text(
                                fileName,
                                fontSize = 13.sp,
                                color = Color(0xFFE8F0FF),
                                fontWeight = FontWeight.Medium,
                            )
                        }
                    }
                }

                OutlinedTextField(
                    value = password,
                    onValueChange = { password = it },
                    label = { Text(t.password) },
                    singleLine = true,
                    visualTransformation = if (visible) {
                        VisualTransformation.None
                    } else {
                        PasswordVisualTransformation()
                    },
                    keyboardOptions = KeyboardOptions(
                        keyboardType = KeyboardType.Password,
                        imeAction = ImeAction.Done,
                    ),
                    keyboardActions = KeyboardActions(onDone = { unlock() }),
                    trailingIcon = {
                        TextButton(onClick = { visible = !visible }) {
                            Text(
                                if (visible) t.hide else t.show,
                                fontSize = 11.sp,
                                color = Cyan,
                            )
                        }
                    },
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(16.dp),
                )

                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(54.dp)
                        .alpha(if (fileUri != null && password.isNotEmpty()) 1f else 0.45f)
                        .clip(RoundedCornerShape(16.dp))
                        .background(Brush.horizontalGradient(listOf(Cyan, Violet)))
                        .clickable(enabled = fileUri != null && password.isNotEmpty()) { unlock() },
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        t.open,
                        color = Color(0xFF06101A),
                        fontWeight = FontWeight.ExtraBold,
                        fontSize = 16.sp,
                    )
                }

                openError?.let { message ->
                    Text(message, fontSize = 13.sp, color = Danger)
                }
            }
        }

        Card(
            colors = CardDefaults.cardColors(containerColor = SurfaceDark),
            shape = RoundedCornerShape(20.dp),
            modifier = Modifier.fillMaxWidth(),
        ) {
            Column(Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    t.scenarioTitle,
                    fontSize = 15.sp,
                    fontWeight = FontWeight.Bold,
                    color = Color(0xFFE8F0FF),
                )
                t.scenario.forEach { line ->
                    Text(line, fontSize = 13.sp, color = Muted)
                }
            }
        }

        Text(
            t.advancedTitle,
            fontSize = 15.sp,
            fontWeight = FontWeight.Bold,
            color = Color(0xFFE8F0FF),
            modifier = Modifier.padding(top = 2.dp),
        )

        Card(
            colors = CardDefaults.cardColors(containerColor = SurfaceDark),
            shape = RoundedCornerShape(20.dp),
            modifier = Modifier.fillMaxWidth(),
        ) {
            Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
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
                        Text(t.maskNote, fontSize = 11.sp, color = Faint)
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
                HorizontalDivider(color = Color(0xFF1C2A44))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                        Text(
                            t.shortcut,
                            fontSize = 14.sp,
                            fontWeight = FontWeight.Bold,
                            color = Color(0xFFE8F0FF),
                        )
                        Text(t.shortcutHint, fontSize = 11.sp, color = Faint)
                    }
                    OutlinedButton(
                        onClick = { pinShortcut(context) },
                        shape = RoundedCornerShape(12.dp),
                    ) {
                        Text("＋", color = Cyan, fontSize = 18.sp)
                    }
                }
            }
        }

        Text(
            "SPV1 · XChaCha20-Poly1305 · Argon2id · BLAKE3",
            fontSize = 10.sp,
            color = Faint,
            fontFamily = FontFamily.Monospace,
            modifier = Modifier.padding(bottom = 8.dp),
        )
    }

    if (settings) {
        SettingsDialog(
            t = t,
            lang = lang,
            sky = sky,
            masked = masked,
            onLang = onLang,
            onSky = {
                sky = it
                onSky(it)
            },
            onDisguise = { checked ->
                applyDisguise(context, checked)
                masked = checked
            },
            onShortcut = { pinShortcut(context) },
            onDismiss = { settings = false },
        )
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
private fun SettingsDialog(
    t: Copy,
    lang: Lang,
    sky: Sky,
    masked: Boolean,
    onLang: (Lang) -> Unit,
    onSky: (Sky) -> Unit,
    onDisguise: (Boolean) -> Unit,
    onShortcut: () -> Unit,
    onDismiss: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        confirmButton = {
            TextButton(onClick = onDismiss) { Text(t.done, color = Cyan) }
        },
        title = { Text(t.settingsTitle) },
        text = {
            Column(
                modifier = Modifier.verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                Text(t.lookTitle, fontSize = 14.sp, fontWeight = FontWeight.Bold)
                Text(t.lookHint, fontSize = 11.sp, color = Muted)
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    FilterChip(
                        selected = sky == Sky.None,
                        onClick = { onSky(Sky.None) },
                        label = { Text(t.skyNone) },
                    )
                    FilterChip(
                        selected = sky == Sky.Snow,
                        onClick = { onSky(Sky.Snow) },
                        label = { Text(t.skySnow) },
                    )
                    FilterChip(
                        selected = sky == Sky.Rain,
                        onClick = { onSky(Sky.Rain) },
                        label = { Text(t.skyRain) },
                    )
                    FilterChip(
                        selected = sky == Sky.Sun,
                        onClick = { onSky(Sky.Sun) },
                        label = { Text(t.skySun) },
                    )
                }

                HorizontalDivider(color = Color(0xFF1C2A44))
                Text(t.language, fontSize = 14.sp, fontWeight = FontWeight.Bold)
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    FilterChip(
                        selected = lang == Lang.Ru,
                        onClick = { onLang(Lang.Ru) },
                        label = { Text("Русский") },
                    )
                    FilterChip(
                        selected = lang == Lang.En,
                        onClick = { onLang(Lang.En) },
                        label = { Text("English") },
                    )
                }

                HorizontalDivider(color = Color(0xFF1C2A44))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Column(Modifier.weight(1f)) {
                        Text(t.maskTitle, fontSize = 14.sp, fontWeight = FontWeight.Bold)
                        Text(t.maskNote, fontSize = 11.sp, color = Muted)
                    }
                    Switch(checked = masked, onCheckedChange = onDisguise)
                }

                OutlinedButton(
                    onClick = onShortcut,
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(14.dp),
                ) {
                    Text(t.shortcut, color = Cyan)
                }
            }
        },
    )
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
                    cursor.getString(index)?.takeIf { it.isNotBlank() }?.let { return it }
                }
            }
    }
    return uri.lastPathSegment ?: "vault.spv"
}
