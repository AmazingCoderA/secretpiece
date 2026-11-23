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
import android.graphics.BitmapFactory
import android.os.Bundle
import android.view.ViewGroup
import android.widget.ImageView
import android.widget.VideoView
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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.rememberCoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlin.math.roundToInt
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.CompositionLocalProvider
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
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.viewinterop.AndroidView
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.sin

private const val PREFS = "secretpiece"
private const val PREF_DISGUISE = "disguise"
private const val PREF_SKY = "sky"
private const val PREF_LANG = "lang"
private const val PREF_THEME = "theme"
private const val PREF_PRESETS = "server_presets"
private const val PREF_MAX_PREVIEW_MB = "max_preview_mb"
private const val PREF_MEDIA_PREVIEW = "media_preview"
private const val PREF_MEDIA_STORAGE = "media_storage"
private const val PREF_DISGUISE_KIND = "disguise_kind"
private const val PREF_UI_SCALE = "ui_scale"

data class ServerPreset(
    val name: String,
    val host: String,
    val port: String,
    val user: String,
    val password: String,
    val path: String,
    val memoryOnly: Boolean = false,
    val vaultPassword: String = "",
    val kind: String = "ftp",
)

private fun loadPresets(context: Context): List<ServerPreset> = runCatching {
    val array = org.json.JSONArray(
        org.json.JSONTokener(prefs(context).getString(PREF_PRESETS, "[]")),
    )
    (0 until array.length()).map { index ->
        val item = array.getJSONObject(index)
        ServerPreset(
            name = item.optString("name"),
            host = item.optString("host"),
            port = item.optString("port"),
            user = item.optString("user"),
            password = item.optString("password"),
            path = item.optString("path"),
            memoryOnly = item.optBoolean("memoryOnly", false),
            vaultPassword = item.optString("vaultPassword", ""),
            kind = item.optString("kind", "ftp"),
        )
    }
}.getOrDefault(emptyList())

private fun savePresets(context: Context, presets: List<ServerPreset>) {
    val array = org.json.JSONArray()
    presets.forEach { preset ->
        array.put(
            org.json.JSONObject().apply {
                put("name", preset.name)
                put("host", preset.host)
                put("port", preset.port)
                put("user", preset.user)
                put("password", preset.password)
                put("path", preset.path)
                put("memoryOnly", preset.memoryOnly)
                put("vaultPassword", preset.vaultPassword)
                put("kind", preset.kind)
            },
        )
    }
    prefs(context).edit().putString(PREF_PRESETS, array.toString()).apply()
}

private fun wipeFile(file: java.io.File) {
    runCatching {
        if (file.exists()) {
            java.io.RandomAccessFile(file, "rws").use { it.setLength(0) }
            file.delete()
        }
    }
}

private var Bg = Color(0xFF05080F)
private var Deep = Color(0xFF0A1020)
private var SurfaceDark = Color(0xFF0D1424)
private var SurfaceSoft = Color(0xFF121B2E)
private var Cyan = Color(0xFF4DE8FF)
private var Violet = Color(0xFF9B6BFF)
private var Muted = Color(0xFF8EA3C8)
private var Faint = Color(0xFF6F84A8)
private var Danger = Color(0xFFFF7A90)
private var Gold = Color(0xFFFFD48A)
private var Ink = Color(red = 232, green = 240, blue = 255)

private enum class Theme {
    Night, Light, Paper, Mint, Sunset, Forest, Neon, Sepia, Ocean, Rose,
    Aurora, Cyber, Coal, Cream, Lavender, Honey, Winter, Desert, Storm,
    Emerald, Ruby, Amber, Slate, Orchid, Coffee, Meadow, Coral, Indigo,
    Sand, Frost, Ember, Lilac, Olive, Sky, Plum, Copper, Charcoal, Pearl,
    Cobalt, Velvet,
}

private data class Palette(
    val bg: Color, val deep: Color, val surface: Color, val soft: Color,
    val cyan: Color, val violet: Color, val muted: Color, val faint: Color,
    val danger: Color, val gold: Color, val ink: Color, val light: Boolean,
)

private fun simpleTheme(
    bg: Long, surface: Long, accent: Long, violet: Long, ink: Long, light: Boolean,
) = Palette(
    Color(bg), Color(bg), Color(surface),
    Color(surface).copy(alpha = 0.85f),
    Color(accent), Color(violet),
    Color(ink).copy(alpha = 0.72f), Color(ink).copy(alpha = 0.5f),
    Color(0xFFFF7A90), Color(0xFF4ADE80), Color(ink), light,
)

private val palettes = mapOf(
    Theme.Night to Palette(
        Color(0xFF05080F), Color(0xFF0A1020), Color(0xFF0D1424), Color(0xFF121B2E),
        Color(0xFF4DE8FF), Color(0xFF9B6BFF), Color(0xFF8EA3C8), Color(0xFF6F84A8),
        Color(0xFFFF7A90), Color(0xFFFFD48A), Color(0xFFE8F0FF), false),
    Theme.Light to Palette(
        Color(0xFFF4F6FB), Color(0xFFE8ECF5), Color.White, Color(0xFFEDF1F8),
        Color(0xFF1663C7), Color(0xFF6B46C1), Color(0xFF465268), Color(0xFF6B7689),
        Color(0xFFC0263A), Color(0xFF9A6B00), Color(0xFF121826), true),
    Theme.Paper to Palette(
        Color(0xFFF7F1E7), Color(0xFFEFE6D7), Color(0xFFFFFBF4), Color(0xFFF1E8D9),
        Color(0xFF8A5A2B), Color(0xFF6D4C41), Color(0xFF5C5144), Color(0xFF7B6E5D),
        Color(0xFFA33A2A), Color(0xFF9A6B00), Color(0xFF2A2318), true),
    Theme.Mint to Palette(
        Color(0xFFF1FAF6), Color(0xFFE0F3EA), Color.White, Color(0xFFE6F4EC),
        Color(0xFF0E7A5F), Color(0xFF2E7D56), Color(0xFF43604F), Color(0xFF5F7A6A),
        Color(0xFFB3261E), Color(0xFF8A6A00), Color(0xFF0F2A20), true),
    Theme.Sunset to Palette(
        Color(0xFF1A0F1E), Color(0xFF261226), Color(0xFF2A1530), Color(0xFF351A3A),
        Color(0xFFFFB86B), Color(0xFFFF6FA5), Color(0xFFD3B2C9), Color(0xFFA98CA0),
        Color(0xFFFF7A90), Color(0xFFFFD48A), Color(0xFFFCEBFA), false),
    Theme.Forest to Palette(
        Color(0xFF08120C), Color(0xFF0C1D13), Color(0xFF10281A), Color(0xFF163222),
        Color(0xFF7BE0A0), Color(0xFFB8D46C), Color(0xFF9ABAA6), Color(0xFF6F8F7B),
        Color(0xFFFF8A80), Color(0xFFE8C77A), Color(0xFFE4F5E9), false),
    Theme.Neon to Palette(
        Color(0xFF08040F), Color(0xFF130726), Color(0xFF1A0B33), Color(0xFF251044),
        Color(0xFF00FFD5), Color(0xFFFF3EBA), Color(0xFFC79BE8), Color(0xFF9168B8),
        Color(0xFFFF4D6D), Color(0xFFFFE66D), Color(0xFFEAE0FF), false),
    Theme.Sepia to Palette(
        Color(0xFF1D1710), Color(0xFF2A2017), Color(0xFF33281D), Color(0xFF403225),
        Color(0xFFE8B878), Color(0xFFD89A6A), Color(0xFFC2AA8E), Color(0xFF937F68),
        Color(0xFFFF8A6A), Color(0xFFF2D08A), Color(0xFFF6EBDC), false),
    Theme.Ocean to Palette(
        Color(0xFF04121C), Color(0xFF072033), Color(0xFF0A2B42), Color(0xFF0E3854),
        Color(0xFF5DD6FF), Color(0xFF7C9CFF), Color(0xFFA3C6D8), Color(0xFF7294A8),
        Color(0xFFFF7E7E), Color(0xFFFBD98A), Color(0xFFE2F4FF), false),
    Theme.Rose to Palette(
        Color(0xFF1E0E15), Color(0xFF2C1420), Color(0xFF381A29), Color(0xFF472234),
        Color(0xFFFF9EC4), Color(0xFFFFC178), Color(0xFFE0B6C6), Color(0xFFAF8698),
        Color(0xFFFF7588), Color(0xFFFFD98A), Color(0xFFFFEBF2), false),
    Theme.Aurora to simpleTheme(0x050B1E, 0x0C2140, 0x5FF6C8, 0x8A7BFF, 0xE4F7FF, false),
    Theme.Cyber to simpleTheme(0x06060A, 0x11131C, 0x00E5FF, 0xFF2E88, 0xE6F6FF, false),
    Theme.Coal to simpleTheme(0x101010, 0x1C1C1C, 0xC0C0C0, 0x8A8AFF, 0xF2F2F2, false),
    Theme.Cream to simpleTheme(0xFFFDF8EF, 0xFFFFFFFF, 0xA1622B, 0x7C5C3A, 0x2A2118, true),
    Theme.Lavender to simpleTheme(0xF3EFFF, 0xFFFFFF, 0x6D4AFF, 0xA855F7, 0x241C3A, true),
    Theme.Honey to simpleTheme(0xFFF9E6, 0xFFFDF5, 0x916B00, 0x7A5B12, 0x2C230C, true),
    Theme.Winter to simpleTheme(0xEAF4FF, 0xFFFFFF, 0x0E6BA8, 0x3D6DB5, 0x10253A, true),
    Theme.Desert to simpleTheme(0x2A1E14, 0x3B2A1C, 0xE8B878, 0xCC8E5B, 0xF7E9D6, false),
    Theme.Storm to simpleTheme(0x0B1018, 0x151D2A, 0x9FB3C8, 0x6C8EBF, 0xE2EAF2, false),
    Theme.Emerald to simpleTheme(0x04140E, 0x0A241A, 0x34D399, 0x6EE7B7, 0xDCFCE7, false),
    Theme.Ruby to simpleTheme(0x1A0409, 0x2C0812, 0xFF6B81, 0xFFB36B, 0xFFE4E9, false),
    Theme.Amber to simpleTheme(0x1A1204, 0x2A1F08, 0xFBBF24, 0xF59E0B, 0xFEF3C7, false),
    Theme.Slate to simpleTheme(0x0F172A, 0x1E293B, 0x94A3B8, 0x818CF8, 0xF1F5F9, false),
    Theme.Orchid to simpleTheme(0x1A0A24, 0x2C123C, 0xE9A8FF, 0xA78BFA, 0xF7E8FF, false),
    Theme.Coffee to simpleTheme(0x1F130C, 0x322017, 0xD6A27A, 0xA87B5D, 0xF5E6D8, false),
    Theme.Meadow to simpleTheme(0xEDF7ED, 0xFFFFFF, 0x2E7D32, 0x558B2F, 0x142A15, true),
    Theme.Coral to simpleTheme(0x1B0C0C, 0x2E1414, 0xFF8A7A, 0xFFB4A0, 0xFFEBE7, false),
    Theme.Indigo to simpleTheme(0x080C1E, 0x121936, 0x8C9EFF, 0xC7D2FE, 0xE8ECFF, false),
    Theme.Sand to simpleTheme(0xFAF4E6, 0xFFFFFF, 0x8D6E3C, 0x6D5A32, 0x2E2617, true),
    Theme.Frost to simpleTheme(0xE6F6FA, 0xFFFFFF, 0x0277BD, 0x00838F, 0x0C2733, true),
    Theme.Ember to simpleTheme(0x1A0803, 0x2C1006, 0xFF7043, 0xFFAB40, 0xFFEDE0, false),
    Theme.Lilac to simpleTheme(0xF5EEFF, 0xFFFFFF, 0x7E57C2, 0x9575CD, 0x271B3A, true),
    Theme.Olive to simpleTheme(0x141A08, 0x222E10, 0xAED581, 0x9CCC65, 0xE8F2DC, false),
    Theme.Sky to simpleTheme(0xE8F4FD, 0xFFFFFF, 0x0288D1, 0x5C6BC0, 0x0D2A3E, true),
    Theme.Plum to simpleTheme(0x1C0716, 0x2F0D26, 0xF06292, 0xBA68C8, 0xFCE4EC, false),
    Theme.Copper to simpleTheme(0x1B1009, 0x2E1C10, 0xE68A5B, 0xC77C3C, 0xF8E8DC, false),
    Theme.Charcoal to simpleTheme(0x0A0A0A, 0x171717, 0xB0BEC5, 0x90A4AE, 0xECF0F1, false),
    Theme.Pearl to simpleTheme(0xF7F9FC, 0xFFFFFF, 0x455A64, 0x5C6BC0, 0x1C2833, true),
    Theme.Cobalt to simpleTheme(0x040A24, 0x0B163B, 0x448AFF, 0x7C4DFF, 0xE3E8FF, false),
    Theme.Velvet to simpleTheme(0x170A17, 0x281228, 0xFF77C2, 0xB388FF, 0xFFEAF7, false),
)

private fun themeKey(theme: Theme) = theme.name.lowercase()

private fun loadTheme(context: Context): Theme =
    Theme.values().firstOrNull { themeKey(it) == prefs(context).getString(PREF_THEME, "night") }
        ?: Theme.Night

private fun applyTheme(theme: Theme) {
    val p = palettes.getValue(theme)
    Bg = p.bg; Deep = p.deep; SurfaceDark = p.surface; SurfaceSoft = p.soft
    Cyan = p.cyan; Violet = p.violet; Muted = p.muted; Faint = p.faint
    Danger = p.danger; Gold = p.gold; Ink = p.ink
}

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
    val confirmPassword: String,
    val createMismatch: String,
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
    val copyText: String,
    val copied: String,
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
        confirmPassword = "Повторите пароль",
        createMismatch = "Пароли не совпадают или короче 8 символов.",
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
        copyText = "Копировать",
        copied = "Скопировано",
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
        confirmPassword = "Repeat password",
        createMismatch = "Passwords do not match or are shorter than 8 characters.",
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
        copyText = "Copy",
        copied = "Copied",
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
        incomingUri.value = sharedUri(intent)
        setContent { SecretPieceApp(incomingUri.value) }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        incomingUri.value = sharedUri(intent)
    }

    /**
     * `VIEW` delivers the file in `data`; `SEND` (share sheet) puts it into
     * `EXTRA_STREAM`. Both land on the unlock screen with the path prefilled.
     */
    @Suppress("DEPRECATION")
    private fun sharedUri(intent: Intent?): Uri? {
        if (intent == null) return null
        intent.data?.let { return it }
        return intent.getParcelableExtra(Intent.EXTRA_STREAM)
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

private fun disguiseKind(context: Context): Int =
    prefs(context).getInt(PREF_DISGUISE_KIND, 0)

private fun applyDisguise(context: Context, kind: Int) {
    val pm = context.packageManager
    val enabledState = PackageManager.COMPONENT_ENABLED_STATE_ENABLED
    val disabledState = PackageManager.COMPONENT_ENABLED_STATE_DISABLED
    val aliases = listOf(
        "LauncherSecretPiece",
        "LauncherCalculator",
        "LauncherGallery",
        "LauncherNotes",
        "LauncherWeather",
    )
    aliases.forEachIndexed { index, suffix ->
        pm.setComponentEnabledSetting(
            launcherAlias(context, suffix),
            if (index == kind) enabledState else disabledState,
            PackageManager.DONT_KILL_APP,
        )
    }
    prefs(context).edit()
        .putInt(PREF_DISGUISE_KIND, kind)
        .putBoolean(PREF_DISGUISE, kind > 0)
        .apply()
}

/** Re-apply the saved choice after a reinstall or a package manager reset. */
private fun syncDisguise(context: Context) {
    applyDisguise(context, disguiseKind(context))
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
    var theme by remember { mutableStateOf(loadTheme(context)) }
    var uiScale by remember {
        mutableStateOf(prefs(context).getFloat(PREF_UI_SCALE, 1f))
    }
    val baseDensity = LocalDensity.current
    val t = copyFor(lang)
    applyTheme(theme)

    val scheme = if (palettes.getValue(theme).light) {
        lightColorScheme(
            primary = Cyan,
            secondary = Violet,
            background = Bg,
            surface = SurfaceDark,
            onPrimary = Color.White,
            onBackground = Ink,
            onSurface = Ink,
            error = Danger,
        )
    } else {
        darkColorScheme(
            primary = Cyan,
            secondary = Violet,
            background = Bg,
            surface = SurfaceDark,
            onPrimary = Color(0xFF031018),
            onBackground = Ink,
            onSurface = Ink,
            error = Danger,
        )
    }
    MaterialTheme(colorScheme = scheme) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Brush.verticalGradient(listOf(Bg, Deep, Bg))),
        ) {
            Ambient(sky)
            Surface(modifier = Modifier.fillMaxSize(), color = Color.Transparent) {
                CompositionLocalProvider(
                    LocalDensity provides Density(
                        baseDensity.density * uiScale,
                        baseDensity.fontScale,
                    ),
                ) {
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
                        theme = theme,
                        onTheme = {
                            theme = it
                            prefs(context).edit().putString(PREF_THEME, themeKey(it)).apply()
                        },
                        uiScale = uiScale,
                        onUiScale = {
                            uiScale = it
                            prefs(context).edit().putFloat(PREF_UI_SCALE, it).apply()
                        },
                    )
                }
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
    theme: Theme,
    onTheme: (Theme) -> Unit,
    uiScale: Float,
    onUiScale: (Float) -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var fileUri by remember { mutableStateOf(initialUri) }
    var fileName by remember {
        mutableStateOf(initialUri?.let { displayName(context, it) } ?: "")
    }
    var password by remember { mutableStateOf("") }
    var visible by remember { mutableStateOf(false) }
    var masked by remember { mutableStateOf(disguiseEnabled(context)) }
    var maskKind by remember { mutableStateOf(disguiseKind(context)) }
    var maskDialog by remember { mutableStateOf<String?>(null) }
    var settings by remember { mutableStateOf(false) }
    var sky by remember { mutableStateOf(loadSky(context)) }
    var handle by remember { mutableStateOf(0L) }
    var exportName by remember { mutableStateOf("secretpiece-export.txt") }
    var exportData by remember { mutableStateOf("") }
    val settingsExporter = rememberLauncherForActivityResult(
        ActivityResultContracts.CreateDocument("*/*"),
    ) { uri ->
        if (uri != null && exportData.isNotEmpty()) {
            runCatching {
                context.contentResolver.openOutputStream(uri)?.use {
                    it.write(exportData.toByteArray(Charsets.UTF_8))
                }
            }
            exportData = ""
        }
    }
    var openError by remember { mutableStateOf<String?>(null) }
    var presets by remember { mutableStateOf(loadPresets(context)) }
    var maxPreviewMb by remember {
        mutableStateOf(prefs(context).getInt(PREF_MAX_PREVIEW_MB, 16))
    }
    var mediaPreview by remember {
        mutableStateOf(prefs(context).getBoolean(PREF_MEDIA_PREVIEW, false))
    }
    var mediaStorage by remember {
        mutableStateOf(prefs(context).getString(PREF_MEDIA_STORAGE, "cache") ?: "cache")
    }
    var selectedPreset by remember { mutableStateOf<ServerPreset?>(null) }
    var remoteCache by remember { mutableStateOf<java.io.File?>(null) }
    var memoryVault by remember { mutableStateOf<Int?>(null) }
    var sessionPassword by remember { mutableStateOf("") }
    var createOpen by remember { mutableStateOf(false) }
    var newPassword by remember { mutableStateOf("") }
    var newConfirm by remember { mutableStateOf("") }
    var newVisible by remember { mutableStateOf(false) }
    var newError by remember { mutableStateOf<String?>(null) }

    fun serverTest(preset: ServerPreset) = if (preset.kind == "ssh") {
        SshVault.test(preset)
    } else {
        NativeVault.ftpTest(
            preset.host, preset.port, preset.user, preset.password, preset.path,
        )
    }

    fun serverDownload(preset: ServerPreset, local: String) = if (preset.kind == "ssh") {
        SshVault.download(preset, local)
    } else {
        NativeVault.ftpDownload(
            preset.host, preset.port, preset.user, preset.password,
            preset.path, local,
        )
    }

    fun serverUpload(preset: ServerPreset, local: String) = if (preset.kind == "ssh") {
        SshVault.upload(preset, local)
    } else {
        NativeVault.ftpUpload(
            preset.host, preset.port, preset.user, preset.password,
            preset.path, local,
        )
    }

    fun serverBytes(preset: ServerPreset) = if (preset.kind == "ssh") {
        SshVault.bytes(preset)
    } else {
        NativeVault.ftpBytes(
            preset.host, preset.port, preset.user, preset.password, preset.path,
        )
    }

    fun serverPutBytes(preset: ServerPreset, bytes: ByteArray) = if (preset.kind == "ssh") {
        SshVault.putBytes(preset, bytes)
    } else {
        NativeVault.ftpPutBytes(
            preset.host, preset.port, preset.user, preset.password,
            preset.path, bytes,
        )
    }

    DisposableEffect(Unit) {
        onDispose {
            if (handle != 0L) NativeVault.close(handle)
        }
    }

    if (handle != 0L) {
        VaultBrowser(handle, t, maxPreviewMb, mediaPreview, mediaStorage) {
            NativeVault.close(handle)
            handle = 0L
            val cache = remoteCache
            val preset = selectedPreset
            if (cache != null && preset != null) {
                scope.launch(Dispatchers.IO) {
                    val status = runCatching {
                        serverUpload(preset, cache.path)
                    }.getOrDefault("Upload failed")
                    if (status == "ok") {
                        wipeFile(cache)
                        remoteCache = null
                    } else {
                        openError = "Server upload failed (local copy kept): $status"
                    }
                }
            }
            val memory = memoryVault
            if (memory != null && preset != null) {
                scope.launch(Dispatchers.IO) {
                    val status = runCatching {
                        val bytes = NativeVault.memRead(memory) ?: ByteArray(0)
                        serverPutBytes(preset, bytes)
                    }.getOrDefault("Upload failed")
                    NativeVault.memClose(memory)
                    memoryVault = null
                    if (status != "ok") openError = "Server upload failed: $status"
                }
            }
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
        ActivityResultContracts.CreateDocument("application/octet-stream"),
    ) { uri ->
        if (uri != null) {
            openError = null
            runCatching {
                val descriptor = context.contentResolver
                    .openFileDescriptor(uri, "rw")
                    ?: throw IllegalStateException("no descriptor")
                val created = NativeVault.createFd(descriptor.detachFd(), newPassword)
                if (created == 0L) throw IllegalStateException("create failed")
                handle = created
            }.onFailure {
                openError = t.createFailed
            }
        }
        // Never keep the typed password after the picker closes.
        newPassword = ""
        newConfirm = ""
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
            sessionPassword = password
            password = ""
        }.onFailure {
            openError = t.openFailed
        }
    }

    fun openRemote() {
        val preset = selectedPreset
        if (preset == null) return
        val vaultPass = preset.vaultPassword.ifBlank { password }
        if (vaultPass.isEmpty()) {
            openError = "Enter the vault password"
            return
        }
        openError = null
        if (preset.memoryOnly) {
            val bytes = runCatching {
                serverBytes(preset)
            }.getOrNull()
            val data = bytes ?: ByteArray(0)
            if (data.isEmpty()) {
                openError = "Server download failed (memory mode)"
                return
            }
            val opened = runCatching {
                val fd = NativeVault.memCreate(data)
                if (fd < 0) return@runCatching 0L
                val dup = NativeVault.memDup(fd)
                val handle = NativeVault.openFd(fd, vaultPass)
                if (handle == 0L) {
                    if (dup >= 0) NativeVault.memClose(dup)
                } else {
                    memoryVault = dup
                }
                handle
            }.getOrDefault(0L)
            if (opened == 0L) {
                openError = t.openFailed
                return
            }
            sessionPassword = vaultPass
            handle = opened
            password = ""
            return
        }
        val cache = java.io.File(context.cacheDir, "remote-${System.currentTimeMillis()}.spv")
        val status = runCatching {
            serverDownload(preset, cache.path)
        }.getOrDefault("Download failed")
        if (status != "ok") {
            openError = status
            wipeFile(cache)
            return
        }
        val opened = runCatching {
            val fd = android.os.ParcelFileDescriptor.open(
                cache,
                android.os.ParcelFileDescriptor.MODE_READ_WRITE,
            )
            NativeVault.openFd(fd.detachFd(), vaultPass)
        }.getOrDefault(0L)
        if (opened == 0L) {
            openError = t.openFailed
            wipeFile(cache)
            return
        }
        sessionPassword = vaultPass
        remoteCache = cache
        handle = opened
        password = ""
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
            Box(
                modifier = Modifier
                    .size(46.dp)
                    .clip(RoundedCornerShape(15.dp))
                    .background(SurfaceSoft)
                    .clickable { settings = true },
                contentAlignment = Alignment.Center,
            ) {
                Text("⚙", fontSize = 20.sp, color = Cyan)
            }
        }

        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(
                t.tagline,
                fontSize = 24.sp,
                fontWeight = FontWeight.ExtraBold,
                color = Ink,
            )
            Text(t.platforms, fontSize = 11.sp, color = Muted, fontFamily = FontFamily.Monospace)
            Spacer(Modifier.height(6.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                listOf("SPV1", "Argon2id", "XChaCha20").forEach { label ->
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(8.dp))
                            .background(SurfaceSoft)
                            .padding(horizontal = 10.dp, vertical = 5.dp),
                    ) {
                        Text(
                            label,
                            fontSize = 10.sp,
                            color = Faint,
                            fontFamily = FontFamily.Monospace,
                        )
                    }
                }
            }
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
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(3.dp)
                        .clip(RoundedCornerShape(2.dp))
                        .background(Brush.horizontalGradient(listOf(Cyan, Violet))),
                )
                Text(
                    t.unlockTitle,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold,
                    color = Ink,
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
                                color = Ink,
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
                    leadingIcon = {
                        Text(if (visible) "🔓" else "🔒", fontSize = 15.sp)
                    },
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
                        IconButton(onClick = { visible = !visible }) {
                            Text(if (visible) "🙈" else "👁", fontSize = 16.sp)
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

                if (presets.isNotEmpty()) {
                    Text("Серверные пресеты", fontSize = 12.sp, color = Cyan)
                    presets.forEach { preset ->
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            FilterChip(
                                selected = selectedPreset?.name == preset.name,
                                onClick = { selectedPreset = preset },
                                label = { Text(preset.name, fontSize = 11.sp) },
                                modifier = Modifier.weight(1f),
                            )
                            TextButton(onClick = {
                                val status = runCatching {
                                    serverTest(preset)
                                }.getOrDefault("Test failed")
                                openError = if (status == "ok") null else status
                            }) { Text("Тест", fontSize = 11.sp, color = Violet) }
                            TextButton(onClick = {
                                val next = presets.filter { it.name != preset.name }
                                presets = next
                                savePresets(context, next)
                                if (selectedPreset?.name == preset.name) selectedPreset = null
                            }) { Text("✕", fontSize = 11.sp, color = Danger) }
                        }
                    }
                    Button(
                        onClick = { openRemote() },
                        enabled = selectedPreset != null &&
                            (password.isNotEmpty() || selectedPreset?.vaultPassword?.isNotEmpty() == true),
                        modifier = Modifier.fillMaxWidth(),
                    ) { Text("Открыть с сервера", fontSize = 12.sp) }
                }

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    HorizontalDivider(Modifier.weight(1f), color = Color(0xFF1C2A44))
                    Text("·", color = Faint)
                    HorizontalDivider(Modifier.weight(1f), color = Color(0xFF1C2A44))
                }

                OutlinedButton(
                    onClick = {
                        newPassword = ""
                        newConfirm = ""
                        newError = null
                        createOpen = true
                    },
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
                    color = Ink,
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
            color = Ink,
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
                            color = Ink,
                        )
                        Text(t.maskSummary, fontSize = 12.sp, color = Muted)
                        Text(t.maskNote, fontSize = 11.sp, color = Faint)
                    }
                    Switch(
                        checked = masked,
                        onCheckedChange = { checked ->
                            val kind = if (checked) (maskKind.takeIf { it > 0 } ?: 1) else 0
                            maskKind = kind
                            applyDisguise(context, kind)
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
                            color = Ink,
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
            theme = theme,
            onTheme = {
                onTheme(it)
            },
            uiScale = uiScale,
            onUiScale = onUiScale,
            unlockPassword = sessionPassword,
            maxPreviewMb = maxPreviewMb,
            onMaxPreviewMb = { value ->
                maxPreviewMb = value.coerceIn(1, 2048)
                prefs(context).edit().putInt(PREF_MAX_PREVIEW_MB, maxPreviewMb).apply()
            },
            mediaPreview = mediaPreview,
            onMediaPreview = {
                mediaPreview = it
                prefs(context).edit().putBoolean(PREF_MEDIA_PREVIEW, it).apply()
            },
            mediaStorage = mediaStorage,
            onMediaStorage = {
                mediaStorage = it
                prefs(context).edit().putString(PREF_MEDIA_STORAGE, it).apply()
            },
            onDisguise = { checked ->
                val kind = if (checked) (maskKind.takeIf { it > 0 } ?: 1) else 0
                maskKind = kind
                applyDisguise(context, kind)
                masked = checked
            },
            maskKind = maskKind,
            onMaskKind = { kind ->
                maskKind = kind
                applyDisguise(context, kind)
                masked = kind > 0
            },
            onShortcut = { pinShortcut(context) },
            onExportSettings = {
                exportName = "secretpiece-settings.json"
                exportData = org.json.JSONObject().apply {
                    put("app", "SecretPiece")
                    put("version", NativeVault.version())
                    put("language", lang.name)
                    put("theme", themeKey(theme))
                    put("sky", sky.name)
                    put("disguise", masked)
                }.toString(2)
                settingsExporter.launch(exportName)
            },
            onExportKeys = {
                exportName = "secretpiece-shares.txt"
                exportData = runCatching {
                    val array = org.json.JSONArray(
                        org.json.JSONTokener(
                            NativeVault.shares(handle, sessionPassword, 3, 2),
                        ),
                    )
                    (0 until array.length()).joinToString("\n") { array.getString(it) }
                }.getOrDefault("")
                settingsExporter.launch(exportName)
            },
            onExportPresets = {
                exportName = "secretpiece-presets.json"
                exportData = org.json.JSONArray().apply {
                    presets.forEach { preset ->
                        put(
                            org.json.JSONObject().apply {
                                put("name", preset.name)
                                put("host", preset.host)
                                put("port", preset.port)
                                put("user", preset.user)
                                put("password", preset.password)
                                put("path", preset.path)
                                put("memoryOnly", preset.memoryOnly)
                put("vaultPassword", preset.vaultPassword)
                put("kind", preset.kind)
                            },
                        )
                    }
                }.toString(2)
                settingsExporter.launch(exportName)
            },
            onSavePreset = { preset ->
                val next = listOf(preset) + presets.filter { it.name != preset.name }
                presets = next
                savePresets(context, next)
                selectedPreset = preset
            },
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

    if (createOpen) {
        AlertDialog(
            onDismissRequest = { createOpen = false },
            confirmButton = {
                TextButton(
                    onClick = {
                        if (newPassword.length < 8 || newPassword != newConfirm) {
                            newError = t.createMismatch
                        } else {
                            createOpen = false
                            creator.launch("vault.spv")
                        }
                    },
                ) {
                    Text(t.create, color = Violet, fontWeight = FontWeight.Bold)
                }
            },
            dismissButton = {
                TextButton(onClick = { createOpen = false }) {
                    Text(t.close, color = Muted)
                }
            },
            title = { Text(t.create) },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text(t.createHint, fontSize = 12.sp, color = Muted)
                    OutlinedTextField(
                        value = newPassword,
                        onValueChange = { newPassword = it },
                        label = { Text(t.password) },
                        singleLine = true,
                        visualTransformation = if (newVisible) {
                            VisualTransformation.None
                        } else {
                            PasswordVisualTransformation()
                        },
                        trailingIcon = {
                            IconButton(onClick = { newVisible = !newVisible }) {
                                Text(if (newVisible) "🙈" else "👁", fontSize = 16.sp)
                            }
                        },
                        shape = RoundedCornerShape(14.dp),
                        modifier = Modifier.fillMaxWidth(),
                    )
                    OutlinedTextField(
                        value = newConfirm,
                        onValueChange = { newConfirm = it },
                        label = { Text(t.confirmPassword) },
                        singleLine = true,
                        visualTransformation = PasswordVisualTransformation(),
                        shape = RoundedCornerShape(14.dp),
                        modifier = Modifier.fillMaxWidth(),
                    )
                    newError?.let { message ->
                        Text(message, fontSize = 12.sp, color = Danger)
                    }
                }
            },
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
    theme: Theme,
    onTheme: (Theme) -> Unit,
    uiScale: Float,
    onUiScale: (Float) -> Unit,
    unlockPassword: String,
    maxPreviewMb: Int,
    onMaxPreviewMb: (Int) -> Unit,
    mediaPreview: Boolean,
    onMediaPreview: (Boolean) -> Unit,
    mediaStorage: String,
    onMediaStorage: (String) -> Unit,
    onDisguise: (Boolean) -> Unit,
    maskKind: Int,
    onMaskKind: (Int) -> Unit,
    onShortcut: () -> Unit,
    onExportSettings: () -> Unit,
    onExportKeys: () -> Unit,
    onExportPresets: () -> Unit,
    onSavePreset: (ServerPreset) -> Unit,
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
                modifier = Modifier
                    .heightIn(max = 420.dp)
                    .verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                Text(t.lookTitle, fontSize = 14.sp, fontWeight = FontWeight.Bold)
                Text(t.lookHint, fontSize = 11.sp, color = Muted)
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    Theme.values().toList().chunked(5).forEach { row ->
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.spacedBy(4.dp),
                        ) {
                            row.forEach { option ->
                                FilterChip(
                                    selected = theme == option,
                                    onClick = { onTheme(option) },
                                    label = {
                                        Text(
                                            option.name,
                                            fontSize = 9.sp,
                                            maxLines = 1,
                                        )
                                    },
                                    modifier = Modifier.weight(1f),
                                )
                            }
                        }
                    }
                }
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    FilterChip(
                        selected = sky == Sky.None,
                        onClick = { onSky(Sky.None) },
                        label = { Text(t.skyNone) },
                        modifier = Modifier.weight(1f),
                    )
                    FilterChip(
                        selected = sky == Sky.Snow,
                        onClick = { onSky(Sky.Snow) },
                        label = { Text(t.skySnow) },
                        modifier = Modifier.weight(1f),
                    )
                }
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    FilterChip(
                        selected = sky == Sky.Rain,
                        onClick = { onSky(Sky.Rain) },
                        label = { Text(t.skyRain) },
                        modifier = Modifier.weight(1f),
                    )
                    FilterChip(
                        selected = sky == Sky.Sun,
                        onClick = { onSky(Sky.Sun) },
                        label = { Text(t.skySun) },
                        modifier = Modifier.weight(1f),
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
                Text("Масштаб интерфейса", fontSize = 12.sp, fontWeight = FontWeight.Bold)
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    listOf(0.75f, 0.9f, 1f, 1.15f).forEach { value ->
                        FilterChip(
                            selected = uiScale == value,
                            onClick = { onUiScale(value) },
                            label = { Text("${(value * 100).toInt()}%", fontSize = 10.sp) },
                        )
                    }
                }

                HorizontalDivider(color = Color(0xFF1C2A44))
                OutlinedTextField(
                    value = maxPreviewMb.toString(),
                    onValueChange = { text ->
                        onMaxPreviewMb(text.toIntOrNull() ?: 16)
                    },
                    label = { Text("Макс. размер просмотра, МБ", fontSize = 10.sp) },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )

                HorizontalDivider(color = Color(0xFF1C2A44))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Column(Modifier.weight(1f)) {
                        Text("Экспериментально: фото/видео из ОЗУ", fontSize = 12.sp)
                        Text("Временно распаковать для просмотра", fontSize = 10.sp, color = Muted)
                    }
                    Switch(checked = mediaPreview, onCheckedChange = onMediaPreview)
                }
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    FilterChip(
                        selected = mediaStorage == "ram",
                        onClick = { onMediaStorage("ram") },
                        label = { Text("ОЗУ", fontSize = 10.sp) },
                    )
                    FilterChip(
                        selected = mediaStorage == "cache",
                        onClick = { onMediaStorage("cache") },
                        label = { Text("Накопитель", fontSize = 10.sp) },
                    )
                }

                HorizontalDivider(color = Color(0xFF1C2A44))
                var presetName by remember { mutableStateOf("") }
                var presetHost by remember { mutableStateOf("") }
                var presetPort by remember { mutableStateOf("21") }
                var presetUser by remember { mutableStateOf("") }
                var presetPassword by remember { mutableStateOf("") }
                var presetPath by remember { mutableStateOf("vault.spv") }
                var presetVaultPass by remember { mutableStateOf("") }
                var presetKind by remember { mutableStateOf("ftp") }
                var presetMemory by remember { mutableStateOf(false) }
                Text("Сервер (FTP)", fontSize = 14.sp, fontWeight = FontWeight.Bold)
                listOf(
                    "Имя" to presetName,
                    "Хост" to presetHost,
                    "Порт" to presetPort,
                    "Логин" to presetUser,
                    "Пароль" to presetPassword,
                    "Путь" to presetPath,
                    "Пароль сейфа" to presetVaultPass,
                ).forEach { (label, value) ->
                    OutlinedTextField(
                        value = value,
                        onValueChange = { text ->
                            when (label) {
                                "Имя" -> presetName = text
                                "Хост" -> presetHost = text
                                "Порт" -> presetPort = text
                                "Логин" -> presetUser = text
                                "Пароль" -> presetPassword = text
                                "Пароль сейфа" -> presetVaultPass = text
                                else -> presetPath = text
                            }
                        },
                        label = { Text(label, fontSize = 10.sp) },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                    )
                }
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                    FilterChip(
                        selected = presetKind == "ftp",
                        onClick = { presetKind = "ftp" },
                        label = { Text("FTP") },
                    )
                    FilterChip(
                        selected = presetKind == "ssh",
                        onClick = { presetKind = "ssh" },
                        label = { Text("SSH / SFTP") },
                    )
                }
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text("Только в памяти", fontSize = 11.sp, color = Muted)
                    Switch(checked = presetMemory, onCheckedChange = { presetMemory = it })
                }
                Button(onClick = {
                    if (presetHost.isNotBlank() && presetPath.isNotBlank()) {
                        onSavePreset(
                            ServerPreset(
                                name = presetName.ifBlank { presetHost },
                                host = presetHost.trim(),
                                port = presetPort.trim(),
                                user = presetUser.trim(),
                                password = presetPassword,
                                path = presetPath.trim(),
                                memoryOnly = presetMemory,
                                vaultPassword = presetVaultPass,
                                kind = presetKind,
                            ),
                        )
                    }
                }) { Text("Сохранить пресет", fontSize = 11.sp) }

                HorizontalDivider(color = Color(0xFF1C2A44))
                Text("Экспорт", fontSize = 14.sp, fontWeight = FontWeight.Bold)
                var exportPassword by remember { mutableStateOf("") }
                val unlocked = exportPassword.isNotEmpty() && exportPassword == unlockPassword
                OutlinedTextField(
                    value = exportPassword,
                    onValueChange = { exportPassword = it },
                    label = { Text("Пароль сейфа (для ключей и пресетов)", fontSize = 10.sp) },
                    singleLine = true,
                    visualTransformation = androidx.compose.ui.text.input.PasswordVisualTransformation(),
                    modifier = Modifier.fillMaxWidth(),
                )
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    OutlinedButton(onClick = onExportSettings) {
                        Text("Настройки", fontSize = 11.sp)
                    }
                    OutlinedButton(onClick = onExportKeys, enabled = unlocked) {
                        Text("Ключи (Shamir)", fontSize = 11.sp)
                    }
                    OutlinedButton(onClick = onExportPresets, enabled = unlocked) {
                        Text("Пресеты", fontSize = 11.sp)
                    }
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
                Text("Маскировка под приложение", fontSize = 11.sp, color = Muted)
                Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    listOf("Калькулятор", "Галерея", "Заметки", "Погода")
                        .forEachIndexed { index, label ->
                            FilterChip(
                                selected = maskKind == index + 1,
                                onClick = { onMaskKind(index + 1) },
                                label = { Text(label, fontSize = 9.sp, maxLines = 1) },
                            )
                        }
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

private fun collectDirs(
    handle: Long,
    parent: String,
    depth: Int = 0,
): List<Triple<String, Int, String>> {
    if (depth > 4) return emptyList()
    val out = mutableListOf<Triple<String, Int, String>>()
    val children = runCatching { parseEntries(NativeVault.list(handle, parent)) }
        .getOrDefault(emptyList())
    for (entry in children) {
        if (!entry.isDir) continue
        out += Triple(entry.path, depth, entry.name)
        out += collectDirs(handle, entry.path, depth + 1)
    }
    return out
}

private data class MediaPreview(
    val name: String,
    val mime: String,
    val bytes: ByteArray,
    val path: String = "",
)

private fun mediaMime(name: String): String? = when (
    name.substringAfterLast('.').lowercase()
) {
    "jpg", "jpeg", "jpe", "png", "webp", "gif", "bmp", "heic", "heif", "avif" -> "image/*"
    "mp4", "m4v", "mov", "avi", "mkv", "webm", "flv", "wmv", "3gp", "ts", "mpg", "mpeg" -> "video/*"
    "mp3", "wav", "ogg", "opus", "m4a", "aac", "flac", "wma", "aiff", "mid", "midi" -> "audio/*"
    else -> null
}

@Composable
private fun VaultBrowser(
    handle: Long,
    t: Copy,
    maxPreviewMb: Int,
    mediaPreview: Boolean,
    mediaStorage: String,
    onLock: () -> Unit,
) {
    val context = LocalContext.current
    var prefix by remember { mutableStateOf("") }
    var tick by remember { mutableStateOf(0) }
    var error by remember { mutableStateOf<String?>(null) }
    var notice by remember { mutableStateOf<String?>(null) }
    var query by remember { mutableStateOf("") }
    var previewTitle by remember { mutableStateOf("") }
    var previewOffset by remember { mutableStateOf(Offset.Zero) }
    var media by remember { mutableStateOf<MediaPreview?>(null) }
    var mediaFile by remember { mutableStateOf<java.io.File?>(null) }
    var mediaFd by remember { mutableStateOf(-1) }
    DisposableEffect(Unit) {
        onDispose {
            mediaFile?.let { wipeFile(it) }
            if (mediaFd > 0) NativeVault.memClose(mediaFd)
        }
    }
    var preview by remember { mutableStateOf<String?>(null) }
    var newFolder by remember { mutableStateOf(false) }
    var createFileMode by remember { mutableStateOf(false) }
    var newName by remember { mutableStateOf("") }
    var renameFrom by remember { mutableStateOf<String?>(null) }
    var renameTo by remember { mutableStateOf("") }
    var shredMode by remember { mutableStateOf(false) }
    var shredName by remember { mutableStateOf("") }
    var exportPath by remember { mutableStateOf<String?>(null) }

    val exporter = rememberLauncherForActivityResult(ActivityResultContracts.CreateDocument("*/*")) { uri ->
        val path = exportPath
        if (uri != null && path != null) {
            val data = runCatching { NativeVault.read(handle, path) }.getOrNull()
            if (data == null) {
                error = t.readFailed
            } else {
                val written = runCatching {
                    context.contentResolver.openOutputStream(uri)?.use { it.write(data) }
                }.isSuccess
                notice = if (written) t.copied else t.readFailed
                data.fill(0)
            }
        }
        exportPath = null
    }

    val importer = rememberLauncherForActivityResult(
        ActivityResultContracts.OpenMultipleDocuments(),
    ) { uris ->
        if (uris.isNotEmpty()) {
            var done = 0
            var failed = 0
            for (uri in uris) {
                val name = displayName(context, uri)
                val bytes = runCatching {
                    context.contentResolver.openInputStream(uri)?.use { it.readBytes() }
                }.getOrNull()
                if (bytes == null) {
                    failed++
                    continue
                }
                val target = listOf(prefix.trim('/'), name)
                    .filter { it.isNotEmpty() }
                    .joinToString("/")
                if (NativeVault.put(handle, target, bytes) == 1) {
                    done++
                } else {
                    failed++
                }
                bytes.fill(0)
            }
            tick++
            error = if (failed > 0) "Imported $done, failed $failed" else null
            notice = "✓ $done"
        }
    }

    val entries = remember(prefix, tick, query) {
        runCatching { parseEntries(NativeVault.list(handle, prefix)) }
            .getOrDefault(emptyList())
            .filter { query.isBlank() || it.name.contains(query, ignoreCase = true) }
    }
    val stats = remember(handle, tick) {
        runCatching { parseStats(NativeVault.info(handle)) }.getOrNull()
    }
    val tree = remember(tick) { collectDirs(handle, "") }

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

        OutlinedTextField(
            value = query,
            onValueChange = { query = it },
            label = { Text("Поиск", fontSize = 10.sp) },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )

        stats?.let { data ->
            Text(
                "${humanBytes(data.bytes)} · ${data.files} ${t.browserTitle.lowercase()} · " +
                    "SPV1 · gen ${data.generation}",
                fontSize = 11.sp,
                color = Muted,
                fontFamily = FontFamily.Monospace,
            )
        }

        Row(
            modifier = Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            if (prefix.isNotEmpty()) {
                TextButton(onClick = {
                    prefix = prefix.substringBeforeLast('/', "")
                    error = null
                }) {
                    Text("← ${t.up}", color = Cyan)
                }
            }
            Text(
                "root" + if (prefix.isEmpty()) "" else " / $prefix",
                fontSize = 11.sp,
                color = Faint,
                fontFamily = FontFamily.Monospace,
                modifier = Modifier.weight(1f),
            )
            TextButton(onClick = {
                newName = ""
                createFileMode = false
                newFolder = true
            }) { Text("📁 +", color = Cyan, fontSize = 13.sp) }
            TextButton(onClick = {
                newName = ""
                createFileMode = true
                newFolder = true
            }) { Text("📄 +", color = Cyan, fontSize = 13.sp) }
            TextButton(onClick = { importer.launch(arrayOf("*/*")) }) {
                Text("📥", color = Cyan, fontSize = 13.sp)
            }
        }

        if (tree.isNotEmpty()) {
            Text("📁 /", fontSize = 12.sp, color = Cyan, fontFamily = FontFamily.Monospace)
            tree.forEach { node ->
                Text(
                    "   ".repeat(node.second) + "↳ " + node.third,
                    fontSize = 12.sp,
                    color = if (prefix == node.first) Cyan else Muted,
                    fontFamily = FontFamily.Monospace,
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable {
                            prefix = node.first
                            error = null
                        }
                        .padding(vertical = 2.dp),
                )
            }
            Text("—", fontSize = 10.sp, color = Faint)
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
                            val data = runCatching { NativeVault.read(handle, entry.path) }
                                .getOrNull()
                            if (data == null) {
                                error = t.readFailed
                            } else {
                                val mime = mediaMime(entry.name)
                                if (mediaPreview && mime != null) {
                                    val path = if (
                                        mime.startsWith("video/") || mime.startsWith("audio/")
                                    ) {
                                        if (mediaStorage == "ram") {
                                            val fd = NativeVault.memCreate(data)
                                            if (fd > 0) {
                                                mediaFd = fd
                                                "/proc/self/fd/$fd"
                                            } else {
                                                ""
                                            }
                                        } else {
                                            val file = java.io.File(
                                                context.cacheDir,
                                                "media-${System.currentTimeMillis()}-${entry.name}",
                                            )
                                            runCatching {
                                                file.writeBytes(data)
                                                mediaFile = file
                                                file.path
                                            }.getOrDefault("")
                                        }
                                    } else {
                                        ""
                                    }
                                    media = MediaPreview(entry.name, mime, data, path)
                                    preview = null
                                } else {
                                    previewTitle = entry.name
                                    preview = when {
                                        data.size > maxPreviewMb.toLong() * 1024L * 1024L ||
                                            data.take(8192).any { it == 0.toByte() } ->
                                            t.binaryFile
                                        else -> String(data, Charsets.UTF_8)
                                            .takeIf { !it.contains('\uFFFD') }
                                            ?: t.binaryFile
                                    }
                                }
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
                            color = Ink,
                        )
                        Text(
                            if (entry.isDir) t.browserTitle else humanBytes(entry.size),
                            fontSize = 11.sp,
                            color = Muted,
                        )
                    }
                    if (!entry.isDir) {
                        TextButton(onClick = {
                            exportPath = entry.path
                            exporter.launch(entry.name)
                        }) { Text("⬇", color = Violet, fontSize = 13.sp) }
                    }
                    TextButton(onClick = {
                        renameFrom = entry.path
                        renameTo = entry.name
                        shredMode = false
                    }) { Text("✎", color = Cyan, fontSize = 13.sp) }
                    if (!entry.isDir) {
                        TextButton(onClick = {
                            shredName = entry.name
                            shredMode = true
                        }) { Text("✂", color = Danger, fontSize = 13.sp) }
                    }
                    TextButton(onClick = {
                        error = null
                        notice = null
                        if (NativeVault.remove(handle, entry.path) == 1) {
                            notice = "✓ ${entry.name}"
                            tick++
                        } else {
                            error = t.readFailed
                        }
                    }) { Text("🗑", color = Danger, fontSize = 13.sp) }
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
        Dialog(onDismissRequest = { preview = null }) {
            Surface(
                shape = RoundedCornerShape(16.dp),
                color = SurfaceDark,
                modifier = Modifier.offset {
                    IntOffset(
                        previewOffset.x.roundToInt(),
                        previewOffset.y.roundToInt(),
                    )
                },
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .pointerInput(Unit) {
                                detectDragGestures { change, amount ->
                                    change.consume()
                                    previewOffset += amount
                                }
                            },
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text("⠿", color = Faint)
                        Text(
                            previewTitle,
                            modifier = Modifier.weight(1f),
                            fontWeight = FontWeight.Bold,
                            color = Cyan,
                        )
                        TextButton(onClick = { preview = null }) { Text(t.close) }
                    }
                    Column(
                        modifier = Modifier
                            .height(360.dp)
                            .verticalScroll(rememberScrollState()),
                    ) {
                        val rows = text.lines().filter { it.isNotBlank() }.take(200)
                        val delim = when {
                            rows.firstOrNull()?.contains('\t') == true -> '\t'
                            rows.firstOrNull()?.contains(';') == true -> ';'
                            else -> ','
                        }
                        val cells = rows.map { it.split(delim) }
                        if (cells.size > 1 && cells.first().size > 1) {
                            Column(modifier = Modifier.horizontalScroll(rememberScrollState())) {
                                cells.forEachIndexed { index, row ->
                                    Row {
                                        row.forEach { cell ->
                                            Text(
                                                cell.trim(),
                                                modifier = Modifier.width(140.dp),
                                                fontSize = 11.sp,
                                                fontFamily = FontFamily.Monospace,
                                                color = if (index == 0) Cyan else Ink,
                                                fontWeight = if (index == 0) FontWeight.Bold
                                                else FontWeight.Normal,
                                            )
                                        }
                                    }
                                }
                            }
                        } else {
                            Text(text, fontFamily = FontFamily.Monospace, fontSize = 12.sp)
                        }
                    }
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.End,
                    ) {
                        TextButton(onClick = {
                            runCatching {
                                val clipboard = context.getSystemService(Context.CLIPBOARD_SERVICE)
                                    as? android.content.ClipboardManager
                                clipboard?.setPrimaryClip(
                                    android.content.ClipData.newPlainText(previewTitle, text),
                                )
                                notice = t.copied
                            }
                            preview = null
                        }) { Text(t.copyText, color = Cyan) }
                    }
                }
            }
        }
    }
    media?.let { item ->
        Dialog(onDismissRequest = {
            media = null
            mediaFile?.let { wipeFile(it) }
            mediaFile = null
            if (mediaFd > 0) {
                NativeVault.memClose(mediaFd)
                mediaFd = -1
            }
        }) {
            Surface(shape = RoundedCornerShape(16.dp), color = SurfaceDark) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(item.name, color = Cyan, modifier = Modifier.weight(1f))
                        TextButton(onClick = {
                            media = null
                            mediaFile?.let { wipeFile(it) }
                            mediaFile = null
            if (mediaFd > 0) {
                NativeVault.memClose(mediaFd)
                mediaFd = -1
            }
                        }) { Text(t.close) }
                    }
                    if (
                        (item.mime.startsWith("video/") || item.mime.startsWith("audio/")) &&
                        item.path.isNotEmpty()
                    ) {
                        AndroidView(
                            factory = { ctx ->
                                VideoView(ctx).apply {
                                    layoutParams = ViewGroup.LayoutParams(
                                        ViewGroup.LayoutParams.MATCH_PARENT, 600,
                                    )
                                    setVideoPath(item.path)
                                    setOnPreparedListener { it.isLooping = true; start() }
                                }
                            },
                            modifier = Modifier.height(360.dp).fillMaxWidth(),
                        )
                    } else {
                        AndroidView(
                            factory = { ctx ->
                                ImageView(ctx).apply {
                                    adjustViewBounds = true
                                    scaleType = ImageView.ScaleType.FIT_CENTER
                                    setImageBitmap(
                                        BitmapFactory.decodeByteArray(
                                            item.bytes, 0, item.bytes.size,
                                        ),
                                    )
                                }
                            },
                            modifier = Modifier.heightIn(max = 500.dp).fillMaxWidth(),
                        )
                    }
                }
            }
        }
    }

    if (newFolder) {
        AlertDialog(
            onDismissRequest = { newFolder = false },
            confirmButton = {
                TextButton(onClick = {
                    val clean = newName.trim().trim('/')
                    if (clean.isNotEmpty()) {
                        val target = listOf(prefix.trim('/'), clean)
                            .filter { it.isNotEmpty() }
                            .joinToString("/")
                        error = if (createFileMode) {
                            if (NativeVault.put(handle, target, ByteArray(0)) == 1) {
                                notice = "✓ $clean"
                                tick++
                                null
                            } else {
                                t.readFailed
                            }
                        } else if (NativeVault.mkdir(handle, target) == 1) {
                            notice = "✓ $clean"
                            tick++
                            null
                        } else {
                            t.readFailed
                        }
                    }
                    newFolder = false
                }) { Text(t.done, color = Cyan) }
            },
            dismissButton = {
                TextButton(onClick = { newFolder = false }) { Text(t.close) }
            },
            title = { Text("📁 +") },
            text = {
                OutlinedTextField(
                    value = newName,
                    onValueChange = { newName = it },
                    singleLine = true,
                )
            },
        )
    }

    renameFrom?.let { from ->
        AlertDialog(
            onDismissRequest = { renameFrom = null },
            confirmButton = {
                TextButton(onClick = {
                    val clean = renameTo.trim().trim('/')
                    if (clean.isNotEmpty()) {
                        val parent = from.substringBeforeLast('/', "")
                        val target = listOf(parent, clean)
                            .filter { it.isNotEmpty() }
                            .joinToString("/")
                        error = if (shredMode) {
                            val oldName = from.substringAfterLast('/')
                            if (clean != oldName) {
                                val data = runCatching { NativeVault.read(handle, from) }.getOrNull()
                                if (data != null) {
                                    NativeVault.put(handle, target, data)
                                    data.fill(0)
                                }
                            }
                            if (NativeVault.shred(handle, from) == 1) {
                                notice = "✂ $clean"
                                tick++
                                null
                            } else {
                                t.readFailed
                            }
                        } else if (NativeVault.move(handle, from, target) == 1) {
                            notice = "✓ $clean"
                            tick++
                            null
                        } else {
                            t.readFailed
                        }
                    }
                    renameFrom = null
                }) { Text(t.done, color = Cyan) }
            },
            dismissButton = {
                TextButton(onClick = { renameFrom = null }) { Text(t.close) }
            },
            title = { Text(if (shredMode) "✂ Уничтожить" else "✎") },
            text = {
                OutlinedTextField(
                    value = if (shredMode) shredName else renameTo,
                    onValueChange = {
                        if (shredMode) shredName = it else renameTo = it
                    },
                    singleLine = true,
                    label = {
                        Text(
                            if (shredMode) "Новое имя перед уничтожением" else "Имя",
                        )
                    },
                )
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
