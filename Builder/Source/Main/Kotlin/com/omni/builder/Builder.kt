package com.omni.builder

import android.animation.Animator
import android.animation.AnimatorListenerAdapter
import android.animation.LayoutTransition
import android.animation.ValueAnimator
import android.app.Activity
import android.app.AlertDialog
import android.app.Application
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.job.JobInfo
import android.app.job.JobParameters
import android.app.job.JobScheduler
import android.app.job.JobService
import android.content.ClipData
import android.content.ClipboardManager
import android.content.ComponentName
import android.content.ContentProvider
import android.content.ContentUris
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.content.res.ColorStateList
import android.content.res.Configuration
import android.content.res.Resources
import android.database.Cursor
import android.database.MatrixCursor
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.ColorFilter
import android.graphics.LinearGradient
import android.graphics.Matrix
import android.graphics.Paint
import android.graphics.Path
import android.graphics.PathMeasure
import android.graphics.PixelFormat
import android.graphics.PorterDuff
import android.graphics.PorterDuffXfermode
import android.graphics.RadialGradient
import android.graphics.RectF
import android.graphics.Shader
import android.graphics.Typeface
import android.graphics.drawable.ColorDrawable
import android.graphics.drawable.Drawable
import android.graphics.drawable.GradientDrawable
import android.graphics.drawable.LayerDrawable
import android.graphics.drawable.RippleDrawable
import android.graphics.drawable.StateListDrawable
import android.net.nsd.NsdManager
import android.net.nsd.NsdServiceInfo
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import android.os.ParcelFileDescriptor
import android.os.SystemClock
import android.provider.OpenableColumns
import android.provider.Settings
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import android.text.Editable
import android.text.InputType
import android.text.Spannable
import android.text.SpannableStringBuilder
import android.text.TextWatcher
import android.text.style.ForegroundColorSpan
import android.text.style.StyleSpan
import android.util.Base64
import android.util.Log
import android.util.TypedValue
import android.view.Gravity
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.View
import android.view.ViewGroup
import android.view.ViewGroup.LayoutParams.MATCH_PARENT
import android.view.ViewGroup.LayoutParams.WRAP_CONTENT
import android.view.WindowInsets
import android.view.WindowInsetsController
import android.view.accessibility.AccessibilityNodeInfo
import android.view.animation.AccelerateInterpolator
import android.view.animation.DecelerateInterpolator
import android.view.animation.OvershootInterpolator
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import android.widget.Button
import android.widget.EditText
import android.widget.FrameLayout
import android.widget.HorizontalScrollView
import android.widget.ImageView
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import android.window.OnBackInvokedCallback
import android.window.OnBackInvokedDispatcher
import java.io.BufferedInputStream
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.FileOutputStream
import java.io.IOException
import java.io.InputStream
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder
import java.security.KeyFactory
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.Signature
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import java.nio.file.Files
import java.nio.file.attribute.PosixFileAttributes
import java.nio.file.attribute.PosixFilePermission
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Date
import java.util.Locale
import kotlin.math.PI
import kotlin.math.abs
import kotlin.math.cos
import kotlin.math.hypot
import kotlin.math.max
import kotlin.math.min
import kotlin.math.sin
import org.json.JSONArray
import org.json.JSONObject

enum class LogLevel {
    TRACE,

    INFO,

    WARN,

    ERROR,
}

object OmniLog {

    const val DIRECTORY_NAME: String = "Omni_Builder"

    const val SESSION_FILE: String = "Session_Log.txt"

    const val CRASH_FILE: String = "Crash_Log.txt"

    const val MAX_BYTES: Int = 256 * 1024

    private const val TAG = "OmniBuilder"

    private val lock = Any()
    private val session = StringBuilder(8 * 1024)

    @Volatile
    private var context: Context? = null

    @Volatile
    private var installedAt: String = "not started"

    private fun timestamp(): String =
        SimpleDateFormat("yyyy-MM-dd HH:mm:ss.SSS", Locale.US).format(Date())

    fun install(application: Application) {
        context = application.applicationContext
        installedAt = timestamp()

        val previous = Thread.getDefaultUncaughtExceptionHandler()
        Thread.setDefaultUncaughtExceptionHandler { thread, error ->
            try {
                recordCrash(thread, error)
            } catch (secondary: Throwable) {
                Log.e(TAG, "The crash could not be written to the log.", secondary)
            }
            if (previous != null) {
                previous.uncaughtException(thread, error)
            } else {
                android.os.Process.killProcess(android.os.Process.myPid())
            }
        }

        event(LogLevel.INFO, "session", "Session started.")
        event(LogLevel.INFO, "session", describeEnvironment(application))
    }

    fun event(level: LogLevel, tag: String, message: String) {
        val line = "${timestamp()}  ${level.name.padEnd(5)}  ${tag.padEnd(12)}  $message"
        synchronized(lock) { session.append(line).append('\n') }
        when (level) {
            LogLevel.ERROR -> Log.e(TAG, "$tag: $message")
            LogLevel.WARN -> Log.w(TAG, "$tag: $message")
            else -> Log.i(TAG, "$tag: $message")
        }
    }

    fun flushSession(): String {
        val started = System.nanoTime()
        val where = write(SESSION_FILE, sessionDocument(), append = false)
        val elapsedMilliseconds = (System.nanoTime() - started) / 1_000_000
        event(LogLevel.TRACE, "log", "Session written in $elapsedMilliseconds ms to $where")
        return where
    }

    fun recordCrash(thread: Thread, error: Throwable) {
        event(LogLevel.ERROR, "crash", "${error.javaClass.name}: ${error.message}")

        val record = buildString {
            append('\n').append("=".repeat(72)).append('\n')
            append("CRASH  ").append(timestamp()).append('\n')
            append("=".repeat(72)).append('\n')
            append("Thread: ").append(thread.name).append('\n')
            context?.let { append(describeEnvironment(it)).append('\n') }
            append('\n')
            append(Log.getStackTraceString(error))
            append('\n')
            append("Events leading up to the failure:").append('\n')
            append(synchronized(lock) { session.toString() })
        }

        writeBlocking(CRASH_FILE, record, append = true)
        writeBlocking(SESSION_FILE, sessionDocument(), append = false)
    }

    private fun sessionDocument(): String = buildString {
        append("Omni Builder session log\n")
        append("Started: ").append(installedAt).append('\n')
        append("Written: ").append(timestamp()).append('\n')
        append("-".repeat(72)).append('\n')
        append(synchronized(lock) { session.toString() })
    }

    fun transcript(): String = synchronized(lock) { session.toString() }

    fun clearTranscript() {
        synchronized(lock) { session.setLength(0) }
        event(LogLevel.INFO, "log", "The log was cleared from the screen.")
    }

    private fun describeEnvironment(context: Context): String {
        val info = context.applicationInfo
        return buildString {
            append("device=").append(Build.MANUFACTURER).append(' ').append(Build.MODEL)
            append("; android=").append(Build.VERSION.RELEASE)
            append(" (API ").append(Build.VERSION.SDK_INT).append(')')
            append("; abis=").append(Build.SUPPORTED_ABIS.joinToString("/"))
            append("; package=").append(context.packageName)
            append("; minSdk=").append(info.minSdkVersion)
            append("; targetSdk=").append(info.targetSdkVersion)
        }
    }

    private fun privateFile(name: String): File? = context?.let { File(it.filesDir, name) }

    private fun write(name: String, text: String, append: Boolean): String =
        writePrivate(name, text, append)

    private fun writeBlocking(name: String, text: String, append: Boolean): String =
        writePrivate(name, text, append)

    private fun Exception.describe(): String =
        "${javaClass.simpleName}: ${message ?: "no detail"}"

    private fun writePrivate(name: String, text: String, append: Boolean): String {
        val target = privateFile(name) ?: return "(not started)"
        return try {
            FileOutputStream(target, append).use { it.write(text.toByteArray(Charsets.UTF_8)) }
            trim(target)
            target.absolutePath
        } catch (failure: Exception) {
            Log.e(TAG, "The log could not be written.", failure)
            "(unwritable: ${failure.describe()})"
        }
    }

    private fun trim(file: File) {
        if (file.length() <= MAX_BYTES) {
            return
        }
        val keep = file.readBytes().let { it.copyOfRange(it.size - MAX_BYTES / 2, it.size) }
        FileOutputStream(file, false).use { stream ->
            stream.write("[earlier entries dropped at ${timestamp()}]\n".toByteArray(Charsets.UTF_8))
            stream.write(keep)
        }
    }

}

class BuilderApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        Glyphs.open(this)
        OmniLog.install(this)
        bindTheSharedKeyToThisDevice()
        Sentry.arm(this)
    }

    private fun bindTheSharedKeyToThisDevice() {
        val identifier = runCatching {
            Settings.Secure.getString(contentResolver, Settings.Secure.ANDROID_ID)
        }.getOrNull().orEmpty()

        val answer = runCatching { Builder.nativeBindDevice(identifier) }.getOrNull()
        val bound = answer?.contains("\"bound\":true") == true
        OmniLog.event(
            LogLevel.INFO,
            "keys",
            if (bound) {
                "The shared key is sealed to this device as well as to its password."
            } else {
                "Android would not identify this device, so the shared key is sealed " +
                    "to its password alone."
            },
        )
    }
}

object Sentry {

    const val JOB_ID: Int = 0x0117

    const val INTERVAL_MILLIS: Long = 15 * 60 * 1000L

    private const val FILE = "omni_sentry"
    private const val REFUSED = "refused"
    private const val LAST_STANDING = "standing"
    private const val LAST_CHECKED = "checked"

    private fun store(context: Context) =
        context.getSharedPreferences(FILE, Context.MODE_PRIVATE)

    fun arm(context: Context) {
        val scheduler = context.getSystemService(JobScheduler::class.java) ?: return
        val job = JobInfo.Builder(JOB_ID, ComponentName(context, SentryService::class.java))
            .setPeriodic(INTERVAL_MILLIS)
            .setRequiresDeviceIdle(false)
            .setRequiresCharging(false)
            .build()
        val outcome = runCatching { scheduler.schedule(job) }.getOrDefault(JobScheduler.RESULT_FAILURE)
        OmniLog.event(
            LogLevel.INFO,
            "sentry",
            if (outcome == JobScheduler.RESULT_SUCCESS) {
                "Watch armed at ${INTERVAL_MILLIS / 60000} minute intervals."
            } else {
                "Android refused to schedule the watch; the check at every start still runs."
            },
        )
    }

    fun check(context: Context): String {
        if (Builder.load() !is Builder.LoadState.Loaded) {
            return "UNKNOWN"
        }
        val expected = context.getString(R.string.omni_expected_certificate).ifEmpty { null }
        val standing = runCatching {
            SelfCheck.parse(
                Builder.nativeVerifySelf(context.applicationInfo.sourceDir, expected)
            ).standing
        }.getOrDefault("UNKNOWN")

        store(context).edit()
            .putString(LAST_STANDING, standing)
            .putLong(LAST_CHECKED, System.currentTimeMillis())
            .apply()

        if (standing == "TAMPERED") {
            store(context).edit().putBoolean(REFUSED, true).apply()
            OmniLog.event(
                LogLevel.ERROR,
                "sentry",
                "The signature on this package is not the one it was built with.",
            )
        }
        return standing
    }

    fun refused(context: Context): Boolean = store(context).getBoolean(REFUSED, false)

    fun lastChecked(context: Context): Long = store(context).getLong(LAST_CHECKED, 0L)
}

class PackageProvider : ContentProvider() {

    companion object {
        const val PACKAGE_TYPE: String = "application/vnd.android.package-archive"
        const val BUNDLE_TYPE: String = "application/octet-stream"
        const val FOLDER: String = "Built"

        const val OUTPUTS: String = "Outputs"

        val SERVED: List<String> = listOf(".apk", ".aab", ".idsig")

        fun authority(context: Context): String = "${context.packageName}.packages"

        fun folder(context: Context): File =
            File(context.getExternalFilesDir(null) ?: context.filesDir, FOLDER)

        fun own(context: Context): File = context.getExternalFilesDir(null) ?: context.filesDir

        fun uriFor(context: Context, file: File): Uri {
            val root = runCatching { own(context).canonicalPath }.getOrDefault("")
            val here = runCatching { file.canonicalPath }.getOrDefault(file.absolutePath)
            val under = if (root.isNotEmpty() && here.startsWith("$root${File.separator}")) {
                here.substring(root.length + 1)
            } else {
                file.name
            }
            val built = Uri.Builder().scheme("content").authority(authority(context))
            under.split(File.separatorChar).filter { it.isNotEmpty() }
                .forEach { built.appendPath(it) }
            return built.build()
        }
    }

    override fun onCreate(): Boolean = true

    private fun resolve(uri: Uri): File? {
        val here = context ?: return null
        val parts = uri.pathSegments ?: return null
        if (parts.isEmpty() || parts.size > 8) {
            return null
        }
        for (part in parts) {
            if (part.isEmpty() || part == "." || part == ".." || part.contains('\\')) {
                return null
            }
        }
        val name = parts.last()
        if (SERVED.none { name.endsWith(it, ignoreCase = true) }) {
            return null
        }
        val root = own(here)
        val file = parts.fold(root) { at, part -> File(at, part) }
        val inside = runCatching {
            val under = root.canonicalPath + File.separator
            file.canonicalPath.startsWith(under)
        }.getOrDefault(false)
        return if (inside && file.isFile) file else null
    }

    override fun getType(uri: Uri): String =
        if (uri.lastPathSegment?.endsWith(".apk", ignoreCase = true) == true) {
            PACKAGE_TYPE
        } else {
            BUNDLE_TYPE
        }

    override fun openFile(uri: Uri, mode: String): ParcelFileDescriptor? {
        if (mode != "r") {
            return null
        }
        val file = resolve(uri) ?: return null
        return ParcelFileDescriptor.open(file, ParcelFileDescriptor.MODE_READ_ONLY)
    }

    override fun query(
        uri: Uri,
        projection: Array<out String>?,
        selection: String?,
        selectionArgs: Array<out String>?,
        sortOrder: String?,
    ): Cursor? {
        val file = resolve(uri) ?: return null
        val columns = projection ?: arrayOf(
            OpenableColumns.DISPLAY_NAME,
            OpenableColumns.SIZE,
        )
        val cursor = MatrixCursor(columns, 1)
        val row = arrayOfNulls<Any>(columns.size)
        columns.forEachIndexed { index, column ->
            row[index] = when (column) {
                OpenableColumns.DISPLAY_NAME -> file.name
                OpenableColumns.SIZE -> file.length()
                else -> null
            }
        }
        cursor.addRow(row)
        return cursor
    }

    override fun insert(uri: Uri, values: ContentValues?): Uri? = null

    override fun update(
        uri: Uri,
        values: ContentValues?,
        selection: String?,
        selectionArgs: Array<out String>?,
    ): Int = 0

    override fun delete(uri: Uri, selection: String?, selectionArgs: Array<out String>?): Int = 0
}

class SentryService : JobService() {

    override fun onStartJob(params: JobParameters?): Boolean {
        if (params == null) {
            runCatching { Sentry.check(applicationContext) }
            return false
        }
        Thread {
            runCatching { Sentry.check(applicationContext) }
            jobFinished(params, false)
        }.start()
        return true
    }

    override fun onStopJob(params: JobParameters?): Boolean = true
}

object Builder {

    sealed interface LoadState {
        data object Loaded : LoadState

        data class Failed(val reason: String) : LoadState
    }

    @Volatile
    private var loadState: LoadState? = null

    @Synchronized
    fun load(): LoadState {
        loadState?.let { return it }
        val state = try {
            System.loadLibrary("omni_builder")
            OmniLog.event(LogLevel.INFO, "native", "libomni_builder.so loaded.")
            LoadState.Loaded
        } catch (error: UnsatisfiedLinkError) {
            val reason = "The native library could not be loaded: " +
                "${error.message ?: "no detail"}. This usually means the Omni Core " +
                "was not linked into this build, or the bridge and the Core " +
                "disagree on the ABI version."
            OmniLog.event(LogLevel.ERROR, "native", reason)
            LoadState.Failed(reason)
        }
        loadState = state
        return state
    }

    external fun nativeCreateProject(root: String, spec: String): String

    external fun nativeBuildAll(
        root: String,
        packagePath: String,
        bundlePath: String,
        keyPath: String,
        keyPassword: CharArray?,
    ): String

    external fun nativeBuildProgress(): String

    external fun nativeBuildExpect(timings: String?)

    external fun nativeBuildStop()

    external fun nativeSpeak(tag: String): String

    external fun nativeTerminalDocs(tag: String): String

    external fun nativeShellOpen(folder: String, home: String, rows: Int, columns: Int): Int

    external fun nativeShellRead(handle: Int, into: ByteArray, wait: Int): Int

    external fun nativeShellWrite(handle: Int, from: ByteArray): Int

    external fun nativeShellResize(handle: Int, rows: Int, columns: Int)

    external fun nativeShellClose(handle: Int)

    external fun nativeShellInterrupt(handle: Int)

    external fun nativeVerifySelf(packagePath: String, expectedCertificate: String?): String

    external fun nativeCreateKey(directory: String, spec: String, keyPassword: CharArray): String

    external fun nativeDefaultKey(directory: String): String

    external fun nativeBindDevice(secret: String): String

    external fun nativeListKeys(directory: String): String

    external fun nativeDeleteKey(path: String): String

    external fun nativeCheckKey(path: String, keyPassword: CharArray): String

    external fun nativeListProjects(directory: String): String

    external fun nativeProjectTree(root: String): String


    external fun nativeLayOut(name: String, text: String): String

    external fun nativeDependencies(root: String): String

    external fun nativeDependencyRemove(root: String, name: String): String

    external fun nativeManifestFacts(root: String): String

    external fun nativeManifestSet(root: String, field: String, value: String): String

    external fun nativeManifestPermission(root: String, name: String, wanted: Boolean): String

    external fun nativeSymbols(root: String, needle: String): String

    external fun nativeWhereWritten(root: String, qualified: String): String

    external fun nativeInspectPackage(path: String): String

    external fun nativeCheckProject(root: String): String

    external fun nativeSearchProject(
        root: String,
        needle: String,
        sensitive: Boolean,
        wholeWord: Boolean,
    ): String

    external fun nativeReadFile(root: String, relative: String): String

    external fun nativeWriteFile(root: String, relative: String, contents: String): String

    external fun nativeGitCheckout(packPath: String, head: String, into: String): String

    external fun nativeMavenPlan(root: String): String

    external fun nativeProjectRepairs(root: String): String

    external fun nativeApplyRepairs(root: String, repairs: String): String

    external fun nativeDigestFile(path: String): String

    external fun nativePackArchive(
        root: String,
        names: String,
        into: String,
        format: String,
        password: String,
    ): String

    external fun nativeOpenArchive(
        archivePath: String,
        into: String,
        password: String,
    ): String

    external fun nativeNewFolder(root: String, relative: String): String

    external fun nativeRemovePath(root: String, relative: String, trashRoot: String): String

    external fun nativeRenamePath(root: String, from: String, to: String): String

    external fun nativeCopyPath(root: String, from: String, to: String): String

    external fun nativeBuildUnsigned(
        root: String,
        packagePath: String,
        certificate: String,
    ): String

    external fun nativeSealPackage(
        packagePath: String,
        sealedPath: String,
        certificate: String,
        signatureV2: String,
        signatureV3: String,
        signatureV31: String,
        minSdk: Int,
        maxSdk: Int,
    ): String

    external fun nativeListBuilt(directory: String): String

    external fun nativeTrashSend(trashRoot: String, path: String): String

    external fun nativeTrashList(trashRoot: String): String

    external fun nativeTrashRestore(trashRoot: String, id: String): String

    external fun nativeTrashPurge(trashRoot: String, id: String): String

    external fun nativeTrashEmpty(trashRoot: String): String

    external fun nativeSetIcon(root: String, source: String): String

    fun observedEnvironment(context: Context): String {
        val info = context.applicationInfo
        return buildString {
            append("minSdk=").append(info.minSdkVersion)
            append(';')
            append("targetSdk=").append(info.targetSdkVersion)
        }
    }
}

data class ToolchainRow(
    val displayName: String,
    val pinned: String,
    val observed: String?,
    val state: String,
    val checksumPinned: Boolean,
)

data class PluginRow(
    val displayName: String,
    val status: String,
    val roadmapPhase: String,
)

data class ProjectSpec(
    val packageName: String,
    val label: String,
    val versionName: String,
    val versionCode: Int,
    val abis: List<String>,
    val minSdk: Int,
    val targetSdk: Int,
    val languages: List<String>,
    val locales: List<String>,
    val page: String = "",
    val pageParts: List<String> = emptyList(),
) {
    fun encode(): String = buildString {
        append("package=").append(packageName).append(';')
        append("label=").append(label).append(';')
        append("versionName=").append(versionName).append(';')
        append("versionCode=").append(versionCode).append(';')
        append("abis=").append(abis.joinToString(",")).append(';')
        append("minSdk=").append(minSdk).append(';')
        append("targetSdk=").append(targetSdk).append(';')
        append("languages=").append(languages.joinToString(",")).append(';')
        append("locales=").append(locales.joinToString(","))
        if (page.isNotEmpty()) {
            append(';').append("webParts=").append(pageParts.joinToString(","))
            append(';').append("web=").append(page)
        }
    }
}

data class KeySpec(
    val alias: String,
    val commonName: String,
    val organisation: String,
    val country: String,
    val validityYears: Int,
    val bits: Int,
) {
    fun encode(): String = buildString {
        append("alias=").append(alias).append(';')
        append("commonName=").append(commonName).append(';')
        append("organisation=").append(organisation).append(';')
        append("country=").append(country).append(';')
        append("validityYears=").append(validityYears).append(';')
        append("bits=").append(bits)
    }
}

data class Built(
    val name: String,
    val path: String,
    val folder: String,
    val bytes: Long,
    val writtenAt: Long,
    val bundle: Boolean,
) {
    companion object {
        fun list(document: String): List<Built> {
            val array = JSONObject(document).optJSONArray("built") ?: return emptyList()
            return (0 until array.length()).map { index ->
                val item = array.getJSONObject(index)
                Built(
                    name = item.getString("name"),
                    path = item.getString("path"),
                    folder = item.optString("folder"),
                    bytes = item.optLong("bytes"),
                    writtenAt = item.optLong("writtenAt"),
                    bundle = item.optBoolean("bundle", false),
                )
            }
        }
    }
}

data class Trashed(
    val id: String,
    val name: String,
    val origin: String,
    val folder: Boolean,
    val bytes: Long,
    val secondsLeft: Long,
    val restorable: Boolean,
) {
    companion object {
        fun parse(item: JSONObject) = Trashed(
            id = item.getString("id"),
            name = item.getString("name"),
            origin = item.optString("origin"),
            folder = item.optBoolean("folder", false),
            bytes = item.optLong("bytes"),
            secondsLeft = item.optLong("secondsLeft"),
            restorable = item.optBoolean("restorable", false),
        )

        fun list(document: String): List<Trashed> {
            val array = JSONObject(document).optJSONArray("trashed") ?: return emptyList()
            return (0 until array.length()).map { parse(array.getJSONObject(it)) }
        }
    }
}

data class Refusal(
    val code: String?,
    val message: String?,
    val suggestion: String?,
    val context: List<String>,
) {
    companion object {
        fun parse(root: JSONObject): Refusal {
            val context = mutableListOf<String>()
            root.optJSONArray("context")?.let { array ->
                for (index in 0 until array.length()) context.add(array.getString(index))
            }
            return Refusal(
                code = root.optString("code").ifEmpty { null },
                message = root.optString("error").ifEmpty { null },
                suggestion = root.optString("suggestion").ifEmpty { null },
                context = context,
            )
        }
    }
}

data class CodeCheck(
    val clear: Boolean,
    val classes: Int,
    val resources: Int,
    val locales: Int,
    val packageName: String,
    val refusal: Refusal?,
    val file: String,
    val line: Int,
    val column: Int,
) {
    companion object {
        fun parse(document: String): CodeCheck {
            val root = JSONObject(document)
            val code = root.optJSONObject("code") ?: JSONObject()
            val clear = code.optBoolean("clear", false)
            val place = code.optJSONObject("location")
            return CodeCheck(
                clear = clear,
                classes = code.optInt("classes"),
                resources = code.optInt("resources"),
                locales = code.optInt("locales"),
                packageName = code.optString("package"),
                refusal = if (clear) null else Refusal.parse(code),
                file = place?.optString("file").orEmpty(),
                line = place?.optInt("line") ?: 0,
                column = place?.optInt("column") ?: 0,
            )
        }
    }
}

data class SigningKey(
    val alias: String,
    val path: String,
    val subject: String,
    val issued: String,
    val expires: String,
    val fingerprint: String,
    val bits: Int,
) {
    companion object {
        fun parse(item: JSONObject) = SigningKey(
            alias = item.getString("alias"),
            path = item.getString("path"),
            subject = item.optString("subject"),
            issued = item.optString("issued"),
            expires = item.optString("expires"),
            fingerprint = item.optString("fingerprint"),
            bits = item.optInt("bits"),
        )

        fun list(document: String): List<SigningKey> {
            val array = JSONObject(document).optJSONArray("keys") ?: return emptyList()
            return (0 until array.length()).map { parse(array.getJSONObject(it)) }
        }
    }
}

data class ProjectSummary(
    val name: String,
    val root: String,
    val packageName: String,
    val label: String,
    val versionName: String,
    val versionCode: Int,
    val minSdk: Int,
    val targetSdk: Int,
    val files: Int,
    val icon: String?,
) {
    companion object {
        fun parse(item: JSONObject): ProjectSummary {
            val image = item.optJSONObject("icon")
            return ProjectSummary(
                name = item.getString("name"),
                root = item.getString("root"),
                packageName = item.optString("package"),
                label = item.optString("label"),
                versionName = item.optString("versionName"),
                versionCode = item.optInt("versionCode"),
                minSdk = item.optInt("minSdk"),
                targetSdk = item.optInt("targetSdk"),
                files = item.optInt("files"),
                icon = image?.let { "${it.optInt("width")}x${it.optInt("height")} ${it.optString("colour")}" },
            )
        }

        fun list(document: String): List<ProjectSummary> {
            val array = JSONObject(document).optJSONArray("projects") ?: return emptyList()
            return (0 until array.length()).map { parse(array.getJSONObject(it)) }
        }
    }
}

data class FileEntry(
    val path: String,
    val folder: Boolean,
    val bytes: Long,
    val changed: Long = 0L,
) {
    companion object {
        fun list(document: String): List<FileEntry> {
            val array = JSONObject(document).optJSONArray("entries") ?: return emptyList()
            return (0 until array.length()).map { index ->
                val item = array.getJSONObject(index)
                FileEntry(
                    path = item.getString("path"),
                    folder = item.optBoolean("folder", false),
                    bytes = item.optLong("bytes"),
                    changed = item.optLong("changed"),
                )
            }
        }
    }
}

data class CreateOutcome(
    val created: Boolean,
    val root: String?,
    val folders: List<String>,
    val files: List<String>,
    val error: String?,
    val suggestion: String?,
) {
    companion object {
        fun parse(document: String): CreateOutcome {
            val root = JSONObject(document)
            val layout = root.optJSONObject("layout")
            fun strings(name: String): List<String> {
                val array = layout?.optJSONArray(name) ?: return emptyList()
                return (0 until array.length()).map { array.getString(it) }
            }
            return CreateOutcome(
                created = root.optBoolean("created", false),
                root = layout?.optString("root")?.ifEmpty { null },
                folders = strings("folders"),
                files = strings("files"),
                error = root.optString("error").ifEmpty { null },
                suggestion = root.optString("suggestion").ifEmpty { null },
            )
        }
    }
}

data class BuildOutcome(
    val built: Boolean,
    val path: String?,
    val bytes: Long,
    val entries: Long,
    val signed: Boolean,
    val carriesCode: Boolean,
    val findings: List<String>,
    val error: String?,
    val developerKey: Boolean,
    val signedBy: String?,
    val bundlePath: String?,
    val bundleBytes: Long,
    val locales: Long,

    val timings: String?,

    val signatureSchemes: List<String>,
    val signaturesVerified: Long,
    val signatureKeyMatches: Boolean,
) {
    companion object {
        fun parse(document: String): BuildOutcome {
            val root = JSONObject(document)
            val packaged = root.optJSONObject("package")
            val findings = mutableListOf<String>()
            root.optJSONArray("diagnostics")?.let { array ->
                for (index in 0 until array.length()) {
                    val item = array.getJSONObject(index)
                    val text = "${item.getString("code")}: ${item.getString("message")}"
                    if (!findings.contains(text)) findings.add(text)
                }
            }
            return BuildOutcome(
                built = root.optBoolean("built", false),
                path = root.optString("path").ifEmpty { null },
                bytes = packaged?.optLong("bytes") ?: 0L,
                entries = packaged?.optLong("entries") ?: 0L,
                signed = packaged?.optBoolean("signed", false) ?: false,
                carriesCode = packaged?.optBoolean("carriesCode", false) ?: false,
                findings = findings,
                error = root.optString("error").ifEmpty { null },
                developerKey = root.optBoolean("developerKey", false),
                signedBy = root.optJSONObject("signedBy")?.optString("subject")?.ifEmpty { null },
                bundlePath = root.optString("bundlePath").ifEmpty { null },
                bundleBytes = root.optJSONObject("bundle")?.optLong("bytes") ?: 0L,
                locales = root.optLong("locales"),
                timings = root.optString("timings").ifEmpty { null },
                signatureSchemes = packaged?.optJSONObject("verified")
                    ?.optJSONArray("schemes")
                    ?.let { array -> (0 until array.length()).map { array.getString(it) } }
                    .orEmpty(),
                signaturesVerified =
                    packaged?.optJSONObject("verified")?.optLong("signaturesVerified") ?: 0L,
                signatureKeyMatches = packaged?.optJSONObject("verified")
                    ?.optBoolean("keyMatchesCertificate", false) ?: false,
            )
        }
    }
}

data class SelfCheck(
    val standing: String,
    val reason: String,
    val certificate: String?,
) {
    companion object {
        fun parse(document: String): SelfCheck {
            val integrity = JSONObject(document).getJSONObject("integrity")
            return SelfCheck(
                standing = integrity.getString("standing"),
                reason = integrity.getString("reason"),
                certificate = integrity.optString("certificate").ifEmpty { null },
            )
        }
    }
}

data class Palette(
    val key: String,
    val label: String,
    val background: Int,
    val surface: Int,
    val raised: Int,
    val foreground: Int,
    val muted: Int,
    val accent: Int,
    val marker: Int,
    val ok: Int,
    val warning: Int,
    val error: Int,
    val divider: Int,
    val glowFirst: Int,
    val glowSecond: Int,
    val glowThird: Int,
) {
    companion object {
        val DARK = Palette(
            key = "dark",
            label = "Dark Theme",
            background = 0xFF030F18.toInt(),
            surface = 0xFF061521.toInt(),
            raised = 0xFF0C1A22.toInt(),
            foreground = 0xFFFFFFFF.toInt(),
            muted = 0xFF8FB7CD.toInt(),
            accent = 0xFF008BFE.toInt(),
            marker = 0xFF00CDFE.toInt(),
            ok = 0xFF1FC77A.toInt(),
            warning = 0xFFF2A63D.toInt(),
            error = 0xFFF84048.toInt(),
            divider = 0xFF142D3E.toInt(),
            glowFirst = 0xFF0A2A44.toInt(),
            glowSecond = 0xFF06203A.toInt(),
            glowThird = 0xFF04182C.toInt(),
        )

        val HACKER = Palette(
            key = "hacker",
            label = "Hacker",
            background = 0xFF000904.toInt(),
            surface = 0xFF03150B.toInt(),
            raised = 0xFF062011.toInt(),
            foreground = 0xFF9BFFB4.toInt(),
            muted = 0xFF3F8256.toInt(),
            accent = 0xFF00FF66.toInt(),
            marker = 0xFF4BFFA8.toInt(),
            ok = 0xFF5BFF9C.toInt(),
            warning = 0xFFD6FF3B.toInt(),
            error = 0xFFFF4D4D.toInt(),
            divider = 0xFF0B3019.toInt(),
            glowFirst = 0xFF0B7A38.toInt(),
            glowSecond = 0xFF045C2A.toInt(),
            glowThird = 0xFF073C1E.toInt(),
        )

        val ANIME = Palette(
            key = "anime",
            label = "Anime",
            background = 0xFF1A1430.toInt(),
            surface = 0xFF241C42.toInt(),
            raised = 0xFF2F2455.toInt(),
            foreground = 0xFFF7ECFF.toInt(),
            muted = 0xFFA893CC.toInt(),
            accent = 0xFFFF7EB6.toInt(),
            marker = 0xFF9BC5FF.toInt(),
            ok = 0xFF7BE8C8.toInt(),
            warning = 0xFFFFD37E.toInt(),
            error = 0xFFFF6584.toInt(),
            divider = 0xFF3A2D66.toInt(),
            glowFirst = 0xFF7A3E8C.toInt(),
            glowSecond = 0xFF3E5AA8.toInt(),
            glowThird = 0xFFA8506E.toInt(),
        )

        val ALL: List<Palette> = listOf(DARK, HACKER, ANIME)

        fun of(key: String): Palette = ALL.firstOrNull { it.key == key } ?: DARK
    }
}

object Type {
    val heading: Typeface = Typeface.create("sans-serif", Typeface.BOLD)
    val label: Typeface = Typeface.create("sans-serif-condensed", Typeface.BOLD)
    val body: Typeface = Typeface.create("sans-serif", Typeface.NORMAL)
    val strong: Typeface = Typeface.create("sans-serif-medium", Typeface.NORMAL)
    val data: Typeface = Typeface.MONOSPACE

    const val TRACKING = 0.14f
}

object Preferences {

    private const val FILE = "omni_settings"
    private const val LANGUAGE = "language"
    private const val THEME = "theme"
    private const val SIGNING_KEY = "signing_key"
    private const val TIMINGS = "build_timings"
    private const val NEW_PACKAGE = "new_package"
    private const val NEW_MIN_SDK = "new_min_sdk"
    private const val NEW_TARGET_SDK = "new_target_sdk"
    private const val SEARCHES = "searches"
    private const val SEARCHES_KEPT = 40
    private val LOCKED_THEMES = setOf("hacker", "anime")
    private const val FEEDBACK_DRAFT = "feedback_draft"
    private const val FEEDBACK_SENT = "feedback_sent"
    private const val ASSISTANT_KEY = "assistant_key"
    private const val ASSISTANT_VENDOR = "assistant_vendor"
    private const val VENDOR_REST = "vendor_rest"
    private const val VENDOR_FAULTS = "vendor_faults"
    private const val VENDOR_MODELS = "vendor_models"
    private const val ASSISTANT_MODEL = "assistant_model"
    private const val ASSISTANT_EFFORT = "assistant_effort"
    private const val ASSISTANT_AGENT = "assistant_agent"

    val LANGUAGES: List<Triple<String, String, String>> = listOf(
        Triple("tr", "Türkçe", "TR"),
        Triple("en", "English", "US"),
        Triple("de", "Deutsch", "DE"),
        Triple("fr", "Français", "FR"),
        Triple("es", "Español", "ES"),
        Triple("ru", "Русский", "RU"),
        Triple("ar", "العربية", "AR"),
        Triple("ja", "日本語", "JA"),
        Triple("zh", "中文", "ZH-CN"),
        Triple("pt", "Português", "PT"),
        Triple("it", "Italiano", "IT"),
        Triple("ko", "한국어", "KO"),
        Triple("hi", "हिन्दी", "HI"),
        Triple("id", "Bahasa Indonesia", "ID"),
        Triple("pt-BR", "Português (Brasil)", "BR"),
    )

    private fun store(context: Context) =
        context.getSharedPreferences(FILE, Context.MODE_PRIVATE)

    fun language(context: Context): String = store(context).getString(LANGUAGE, "").orEmpty()

    fun vendorKey(context: Context, vendor: String): String =
        store(context).getString(ASSISTANT_KEY + "_" + vendor, "").orEmpty()

    fun setVendorKey(context: Context, vendor: String, sealed: String) {
        store(context).edit().putString(ASSISTANT_KEY + "_" + vendor, sealed).apply()
    }

    fun forgetVendorKey(context: Context, vendor: String) {
        store(context).edit().remove(ASSISTANT_KEY + "_" + vendor).apply()
    }

    fun assistantVendor(context: Context): String =
        store(context).getString(ASSISTANT_VENDOR, "").orEmpty()
            .ifEmpty { Brain.VENDORS.first().id }

    fun setAssistantVendor(context: Context, id: String) {
        store(context).edit().putString(ASSISTANT_VENDOR, id).apply()
    }

    fun assistantModel(context: Context): String = modelFor(context, assistantVendor(context))

    fun modelFor(context: Context, vendor: String): String {
        val held = Brain.vendorOf(vendor)
        return store(context).getString(ASSISTANT_MODEL + "_" + held.id, "").orEmpty()
            .ifEmpty { held.models.first().id }
    }

    fun setAssistantModel(context: Context, id: String) {
        val vendor = assistantVendor(context)
        store(context).edit().putString(ASSISTANT_MODEL + "_" + vendor, id).apply()
    }

    fun restingUntil(context: Context, vendor: String): Long =
        store(context).getLong(VENDOR_REST + "_" + vendor, 0L)

    fun setRestingUntil(context: Context, vendor: String, at: Long) {
        store(context).edit().putLong(VENDOR_REST + "_" + vendor, at).apply()
    }

    fun faults(context: Context, vendor: String): Int =
        store(context).getInt(VENDOR_FAULTS + "_" + vendor, 0)

    fun setFaults(context: Context, vendor: String, many: Int) {
        store(context).edit().putInt(VENDOR_FAULTS + "_" + vendor, many).apply()
    }

    fun fetchedModels(context: Context, vendor: String): List<String> =
        store(context).getString(VENDOR_MODELS + "_" + vendor, "").orEmpty()
            .split('\n')
            .filter { it.isNotBlank() }

    fun setFetchedModels(context: Context, vendor: String, named: List<String>) {
        store(context).edit()
            .putString(VENDOR_MODELS + "_" + vendor, named.joinToString("\n"))
            .apply()
    }

    fun assistantEffort(context: Context): String =
        store(context).getString(ASSISTANT_EFFORT, "").orEmpty().ifEmpty { "high" }

    fun setAssistantEffort(context: Context, named: String) {
        store(context).edit().putString(ASSISTANT_EFFORT, named).apply()
    }

    fun assistantAgent(context: Context): Boolean =
        store(context).getBoolean(ASSISTANT_AGENT, true)

    fun setAssistantAgent(context: Context, on: Boolean) {
        store(context).edit().putBoolean(ASSISTANT_AGENT, on).apply()
    }

    fun setLanguage(context: Context, tag: String) {
        store(context).edit().putString(LANGUAGE, tag).apply()
    }

    fun newPackage(context: Context, fallback: String): String =
        store(context).getString(NEW_PACKAGE, "").orEmpty().ifEmpty { fallback }

    fun setNewPackage(context: Context, value: String) {
        store(context).edit().putString(NEW_PACKAGE, value).apply()
    }

    fun newMinSdk(context: Context, fallback: Int): Int =
        store(context).getInt(NEW_MIN_SDK, fallback)

    fun newTargetSdk(context: Context, fallback: Int): Int =
        store(context).getInt(NEW_TARGET_SDK, fallback)

    fun setNewPlatforms(context: Context, minimum: Int, target: Int) {
        store(context).edit().putInt(NEW_MIN_SDK, minimum).putInt(NEW_TARGET_SDK, target).apply()
    }

    fun palette(context: Context): Palette {
        val held = store(context).getString(THEME, Palette.DARK.key).orEmpty()
        if (held in LOCKED_THEMES) {
            setPalette(context, Palette.DARK.key)
            return Palette.DARK
        }
        return Palette.of(held)
    }

    fun setPalette(context: Context, key: String) {
        store(context).edit().putString(THEME, key).apply()
    }

    fun signingKey(context: Context): String = store(context).getString(SIGNING_KEY, "").orEmpty()

    fun setSigningKey(context: Context, path: String) {
        store(context).edit().putString(SIGNING_KEY, path).apply()
    }

    fun timings(context: Context): String = store(context).getString(TIMINGS, "").orEmpty()

    fun setTimings(context: Context, measured: String) {
        store(context).edit().putString(TIMINGS, measured).apply()
    }

    fun searches(context: Context): List<String> =
        store(context).getString(SEARCHES, "").orEmpty()
            .split('\n')
            .filter { it.isNotBlank() }

    fun remember(context: Context, word: String) {
        val trimmed = word.trim()
        if (trimmed.isEmpty()) return
        val kept = (listOf(trimmed) + searches(context).filter { it != trimmed })
            .take(SEARCHES_KEPT)
        store(context).edit().putString(SEARCHES, kept.joinToString("\n")).apply()
    }

    fun forgetSearches(context: Context) {
        store(context).edit().remove(SEARCHES).apply()
    }

    const val READING = "reading"
    private const val WRAPPING = "wrapping"
    private const val COMPLETING = "completing"

    fun wrapping(context: Context): Boolean = store(context).getBoolean(WRAPPING, true)

    fun setWrapping(context: Context, on: Boolean) {
        store(context).edit().putBoolean(WRAPPING, on).apply()
    }

    fun completing(context: Context): Boolean = store(context).getBoolean(COMPLETING, true)

    fun setCompleting(context: Context, on: Boolean) {
        store(context).edit().putBoolean(COMPLETING, on).apply()
    }

    fun reading(context: Context): Float = store(context).getFloat(READING, 13f)

    fun setReading(context: Context, size: Float) {
        store(context).edit().putFloat(READING, size).apply()
    }

    fun feedbackDraft(context: Context): String =
        store(context).getString(FEEDBACK_DRAFT, "").orEmpty()

    fun setFeedbackDraft(context: Context, held: String) {
        store(context).edit().putString(FEEDBACK_DRAFT, held).apply()
    }

    fun forgetFeedbackDraft(context: Context) {
        store(context).edit().remove(FEEDBACK_DRAFT).apply()
    }

    fun feedbackSentAt(context: Context): Long = store(context).getLong(FEEDBACK_SENT, 0L)

    fun setFeedbackSentAt(context: Context, at: Long) {
        store(context).edit().putLong(FEEDBACK_SENT, at).apply()
    }

}

internal class Screenful(var rows: Int, var columns: Int) {

    companion object {
        const val DEFAULT_INK = -1
        const val DEFAULT_GROUND = -2
        const val BOLD = 1
        const val FAINT = 2
        const val UNDERLINE = 4
        const val REVERSE = 8
        const val ITALIC = 16
        const val STRIKE = 32
        const val SCROLLBACK = 4000
    }

    class Cell {
        var letter: Char = ' '
        var ink: Int = DEFAULT_INK
        var ground: Int = DEFAULT_GROUND
        var style: Int = 0

        fun clear(ink: Int, ground: Int) {
            letter = ' '
            this.ink = ink
            this.ground = ground
            style = 0
        }
    }

    var lines: Array<Array<Cell>> = Array(rows) { Array(columns) { Cell() } }

    fun asText(): String {
        val out = StringBuilder()
        val room = CharArray(columns)
        fun take(line: Array<Cell>) {
            var wide = 0
            for (at in line.indices) {
                if (at >= room.size) {
                    break
                }
                room[at] = line[at].letter
                if (line[at].letter != ' ') {
                    wide = at + 1
                }
            }
            out.append(room, 0, wide.coerceAtMost(room.size))
            out.append('\n')
        }
        for (line in history) {
            take(line)
        }
        for (line in lines) {
            take(line)
        }
        return out.toString().trimEnd('\n')
    }
    val history = ArrayDeque<Array<Cell>>()
    var onAlternate = false
    var bracketed = false
    var applicationKeys = false
    private var kept: Array<Array<Cell>>? = null
    private var keptRow = 0
    private var keptColumn = 0
    var row = 0
    var column = 0
    var ink = DEFAULT_INK
    var ground = DEFAULT_GROUND
    var style = 0
    var wrapping = true
    var showCursor = true
    var scrollTop = 0
    var scrollBottom = rows - 1
    private var pending = false
    private var savedRow = 0
    private var savedColumn = 0

    fun resize(nextRows: Int, nextColumns: Int) {
        if (nextRows == rows && nextColumns == columns) {
            return
        }
        lines = Array(nextRows) { r ->
            Array(nextColumns) { c ->
                val was = lines.getOrNull(r)?.getOrNull(c)
                Cell().apply {
                    if (was != null) {
                        letter = was.letter
                        ink = was.ink
                        ground = was.ground
                        style = was.style
                    }
                }
            }
        }
        rows = nextRows
        columns = nextColumns
        row = row.coerceIn(0, rows - 1)
        column = column.coerceIn(0, columns - 1)
        scrollTop = 0
        scrollBottom = rows - 1
    }

    fun at(r: Int, c: Int): Cell = lines[r][c]

    fun put(letter: Char) {
        if (pending && wrapping) {
            column = 0
            down()
            pending = false
        }
        if (column >= columns) {
            column = columns - 1
        }
        val cell = lines[row][column]
        cell.letter = letter
        cell.ink = ink
        cell.ground = ground
        cell.style = style
        if (column == columns - 1) {
            pending = true
        } else {
            column += 1
        }
    }

    fun place(r: Int, c: Int) {
        row = r.coerceIn(0, rows - 1)
        column = c.coerceIn(0, columns - 1)
        pending = false
    }

    fun move(dr: Int, dc: Int) = place(row + dr, column + dc)

    fun save() {
        savedRow = row
        savedColumn = column
    }

    fun restore() = place(savedRow, savedColumn)

    fun down() {
        if (row == scrollBottom) {
            scrollUp()
        } else if (row < rows - 1) {
            row += 1
        }
        pending = false
    }

    fun up() {
        if (row == scrollTop) {
            scrollDown()
        } else if (row > 0) {
            row -= 1
        }
        pending = false
    }

    fun useAlternate(wanted: Boolean) {
        if (wanted == onAlternate) {
            return
        }
        if (wanted) {
            kept = lines
            keptRow = row
            keptColumn = column
            lines = Array(rows) { Array(columns) { Cell().apply { clear(ink, ground) } } }
            place(0, 0)
        } else {
            lines = kept ?: Array(rows) { Array(columns) { Cell() } }
            kept = null
            place(keptRow, keptColumn)
        }
        onAlternate = wanted
        scrollTop = 0
        scrollBottom = rows - 1
    }

    fun scrollUp() {
        if (!onAlternate && scrollTop == 0) {
            history.addLast(lines[scrollTop])
            while (history.size > SCROLLBACK) {
                history.removeFirst()
            }
        }
        for (r in scrollTop until scrollBottom) {
            lines[r] = lines[r + 1]
        }
        lines[scrollBottom] = Array(columns) { Cell().apply { clear(ink, ground) } }
    }

    fun scrollDown() {
        for (r in scrollBottom downTo scrollTop + 1) {
            lines[r] = lines[r - 1]
        }
        lines[scrollTop] = Array(columns) { Cell().apply { clear(ink, ground) } }
    }

    fun eraseInLine(how: Int) {
        val from = if (how == 0) column else 0
        val until = (if (how == 1) column else columns - 1).coerceAtMost(columns - 1)
        for (c in from..until) {
            lines[row][c].clear(ink, ground)
        }
        pending = false
    }

    fun eraseInDisplay(how: Int) {
        when (how) {
            0 -> {
                eraseInLine(0)
                for (r in row + 1 until rows) {
                    for (c in 0 until columns) lines[r][c].clear(ink, ground)
                }
            }
            1 -> {
                eraseInLine(1)
                for (r in 0 until row) {
                    for (c in 0 until columns) lines[r][c].clear(ink, ground)
                }
            }
            else -> {
                for (r in 0 until rows) {
                    for (c in 0 until columns) lines[r][c].clear(ink, ground)
                }
            }
        }
    }

    fun insertLines(count: Int) {
        repeat(count.coerceIn(1, rows)) {
            for (r in scrollBottom downTo row + 1) {
                lines[r] = lines[r - 1]
            }
            lines[row] = Array(columns) { Cell().apply { clear(ink, ground) } }
        }
    }

    fun deleteLines(count: Int) {
        repeat(count.coerceIn(1, rows)) {
            for (r in row until scrollBottom) {
                lines[r] = lines[r + 1]
            }
            lines[scrollBottom] = Array(columns) { Cell().apply { clear(ink, ground) } }
        }
    }

    private fun copyInto(target: Cell, source: Cell) {
        target.letter = source.letter
        target.ink = source.ink
        target.ground = source.ground
        target.style = source.style
    }

    fun deleteCharacters(count: Int) {
        val many = count.coerceIn(1, columns)
        for (c in column until columns) {
            val from = c + many
            if (from < columns) {
                copyInto(lines[row][c], lines[row][from])
            } else {
                lines[row][c].clear(ink, ground)
            }
        }
    }

    fun insertCharacters(count: Int) {
        val many = count.coerceIn(1, columns)
        for (c in columns - 1 downTo column) {
            val from = c - many
            if (from >= column) {
                copyInto(lines[row][c], lines[row][from])
            } else {
                lines[row][c].clear(ink, ground)
            }
        }
    }

    fun eraseCharacters(count: Int) {
        val many = count.coerceIn(1, columns)
        for (c in column until (column + many).coerceAtMost(columns)) {
            lines[row][c].clear(ink, ground)
        }
    }

    fun tab() {
        column = (((column / 8) + 1) * 8).coerceAtMost(columns - 1)
        pending = false
    }

    fun carriageReturn() {
        column = 0
        pending = false
    }

    fun backspace() {
        if (column > 0) {
            column -= 1
        }
        pending = false
    }
}

internal class Sequences(private val screen: Screenful, private val reply: (String) -> Unit) {

    private enum class Where { GROUND, ESCAPE, CSI, OSC, CHARSET }

    private var where = Where.GROUND
    private val digits = StringBuilder()
    private val text = StringBuilder()
    private var question = false
    private var carried = ByteArray(0)

    fun feed(bytes: ByteArray, length: Int) {
        val whole = if (carried.isEmpty()) bytes.copyOf(length) else carried + bytes.copyOf(length)
        var at = 0
        while (at < whole.size) {
            val lead = whole[at].toInt() and 0xFF
            val need = when {
                lead < 0x80 -> 1
                lead and 0xE0 == 0xC0 -> 2
                lead and 0xF0 == 0xE0 -> 3
                lead and 0xF8 == 0xF0 -> 4
                else -> 1
            }
            if (at + need > whole.size) {
                break
            }
            if (need == 1) {
                take(lead.toChar())
            } else {
                for (letter in String(whole, at, need, Charsets.UTF_8)) {
                    take(letter)
                }
            }
            at += need
        }
        carried = if (at < whole.size) whole.copyOfRange(at, whole.size) else ByteArray(0)
    }

    private fun take(letter: Char) {
        when (where) {
            Where.GROUND -> ground(letter)
            Where.ESCAPE -> escape(letter)
            Where.CSI -> csi(letter)
            Where.OSC -> osc(letter)
            Where.CHARSET -> where = Where.GROUND
        }
    }

    private fun ground(letter: Char) {
        when (letter) {
            '\u001B' -> {
                where = Where.ESCAPE
                digits.setLength(0)
                question = false
            }
            '\n' -> screen.down()
            '\r' -> screen.carriageReturn()
            '\b' -> screen.backspace()
            '\t' -> screen.tab()
            '\u0007' -> Unit
            '\u000B', '\u000C' -> screen.down()
            else -> if (letter.code >= 32) screen.put(letter)
        }
    }

    private fun escape(letter: Char) {
        when (letter) {
            '[' -> {
                where = Where.CSI
                digits.setLength(0)
                question = false
            }
            ']' -> {
                where = Where.OSC
                text.setLength(0)
            }
            '(', ')', '*', '+' -> where = Where.CHARSET
            'M' -> {
                screen.up()
                where = Where.GROUND
            }
            'D' -> {
                screen.down()
                where = Where.GROUND
            }
            'E' -> {
                screen.carriageReturn()
                screen.down()
                where = Where.GROUND
            }
            '7' -> {
                screen.save()
                where = Where.GROUND
            }
            '8' -> {
                screen.restore()
                where = Where.GROUND
            }
            'c' -> {
                screen.eraseInDisplay(2)
                screen.place(0, 0)
                where = Where.GROUND
            }
            else -> where = Where.GROUND
        }
    }

    private fun numbers(): List<Int> =
        digits.toString().split(';').map { it.trim().toIntOrNull() ?: 0 }

    private fun first(fallback: Int): Int {
        val held = numbers().firstOrNull() ?: 0
        return if (held == 0) fallback else held
    }

    private fun csi(letter: Char) {
        if (letter == '?' || letter == '>' || letter == '!') {
            question = question || letter == '?'
            return
        }
        if (letter.isDigit() || letter == ';' || letter == ':' || letter == ' ') {
            digits.append(letter)
            return
        }
        where = Where.GROUND
        when (letter) {
            'A' -> screen.move(-first(1), 0)
            'B' -> screen.move(first(1), 0)
            'C' -> screen.move(0, first(1))
            'D' -> screen.move(0, -first(1))
            'E' -> screen.place(screen.row + first(1), 0)
            'F' -> screen.place(screen.row - first(1), 0)
            'G' -> screen.place(screen.row, first(1) - 1)
            'd' -> screen.place(first(1) - 1, screen.column)
            'H', 'f' -> {
                val held = numbers()
                val r = held.getOrNull(0)?.takeIf { it > 0 } ?: 1
                val c = held.getOrNull(1)?.takeIf { it > 0 } ?: 1
                screen.place(r - 1, c - 1)
            }
            'J' -> screen.eraseInDisplay(numbers().firstOrNull() ?: 0)
            'K' -> screen.eraseInLine(numbers().firstOrNull() ?: 0)
            'L' -> screen.insertLines(first(1))
            'M' -> screen.deleteLines(first(1))
            'P' -> screen.deleteCharacters(first(1))
            '@' -> screen.insertCharacters(first(1))
            'X' -> screen.eraseCharacters(first(1))
            'S' -> repeat(first(1)) { screen.scrollUp() }
            'T' -> repeat(first(1)) { screen.scrollDown() }
            'r' -> {
                val held = numbers()
                val top = (held.getOrNull(0)?.takeIf { it > 0 } ?: 1) - 1
                val bottom = (held.getOrNull(1)?.takeIf { it > 0 } ?: screen.rows) - 1
                screen.scrollTop = top.coerceIn(0, screen.rows - 1)
                screen.scrollBottom = bottom.coerceIn(screen.scrollTop, screen.rows - 1)
                screen.place(0, 0)
            }
            's' -> screen.save()
            'u' -> screen.restore()
            'm' -> paint(numbers())
            'h', 'l' -> mode(letter == 'h')
            'n' -> if (numbers().firstOrNull() == 6) {
                reply("\u001B[${screen.row + 1};${screen.column + 1}R")
            }
            'c' -> reply("\u001B[?6c")
            else -> Unit
        }
    }

    private fun mode(on: Boolean) {
        if (!question) {
            return
        }
        for (one in numbers()) {
            when (one) {
                1 -> screen.applicationKeys = on
                7 -> screen.wrapping = on
                25 -> screen.showCursor = on
                2004 -> screen.bracketed = on
                1049 -> {
                    if (on) {
                        screen.save()
                    }
                    screen.useAlternate(on)
                    if (!on) {
                        screen.restore()
                    }
                }
                1047, 47 -> screen.useAlternate(on)
            }
        }
    }

    private fun paint(codes: List<Int>) {
        if (codes.isEmpty()) {
            return
        }
        var at = 0
        while (at < codes.size) {
            when (val code = codes[at]) {
                0 -> {
                    screen.ink = Screenful.DEFAULT_INK
                    screen.ground = Screenful.DEFAULT_GROUND
                    screen.style = 0
                }
                1 -> screen.style = screen.style or Screenful.BOLD
                2 -> screen.style = screen.style or Screenful.FAINT
                3 -> screen.style = screen.style or Screenful.ITALIC
                4 -> screen.style = screen.style or Screenful.UNDERLINE
                7 -> screen.style = screen.style or Screenful.REVERSE
                9 -> screen.style = screen.style or Screenful.STRIKE
                22 -> screen.style = screen.style and (Screenful.BOLD or Screenful.FAINT).inv()
                23 -> screen.style = screen.style and Screenful.ITALIC.inv()
                24 -> screen.style = screen.style and Screenful.UNDERLINE.inv()
                27 -> screen.style = screen.style and Screenful.REVERSE.inv()
                29 -> screen.style = screen.style and Screenful.STRIKE.inv()
                39 -> screen.ink = Screenful.DEFAULT_INK
                49 -> screen.ground = Screenful.DEFAULT_GROUND
                in 30..37 -> screen.ink = code - 30
                in 40..47 -> screen.ground = code - 40
                in 90..97 -> screen.ink = code - 90 + 8
                in 100..107 -> screen.ground = code - 100 + 8
                38, 48 -> {
                    val kind = codes.getOrNull(at + 1)
                    val chosen = when (kind) {
                        5 -> codes.getOrNull(at + 2) ?: 0
                        2 -> 256 + ((codes.getOrNull(at + 2) ?: 0) shl 16) +
                            ((codes.getOrNull(at + 3) ?: 0) shl 8) + (codes.getOrNull(at + 4) ?: 0)
                        else -> null
                    }
                    if (chosen != null) {
                        if (code == 38) screen.ink = chosen else screen.ground = chosen
                    }
                    at += if (kind == 2) 4 else 2
                }
                else -> Unit
            }
            at += 1
        }
    }

    private fun osc(letter: Char) {
        if (letter == '\u0007' || letter == '\u009C' || letter == '\u001B') {
            text.setLength(0)
            where = Where.GROUND
            return
        }
        if (text.length < 256) {
            text.append(letter)
        }
    }
}

class AuroraView(context: Context, private var palette: Palette) : View(context) {

    private companion object {
        const val PERIOD_MILLIS = 24_000f
        const val FRAME_MILLIS = 32L
        const val COLUMN_DP = 15f
        const val RAIN_ROWS = 26
        const val PETALS = 22
        const val SPARKS = 30
    }

    private val paint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val place = Matrix()
    private val petal = Path()
    private val started = SystemClock.uptimeMillis()
    private var running = false

    private var glows: Array<RadialGradient>? = null
    private var fade: LinearGradient? = null
    private var sky: LinearGradient? = null
    private var madeAt = 0
    private var madeFor = 0

    private val step = object : Runnable {
        override fun run() {
            if (!running) {
                return
            }
            invalidate()
            postDelayed(this, FRAME_MILLIS)
        }
    }

    fun repaint(next: Palette) {
        palette = next
        glows = null
        fade = null
        sky = null
        invalidate()
    }

    fun resumeDrawing() {
        if (running) {
            return
        }
        if (Motion.still()) {
            invalidate()
            return
        }
        running = true
        post(step)
    }

    fun pauseDrawing() {
        running = false
        removeCallbacks(step)
    }

    override fun onDetachedFromWindow() {
        pauseDrawing()
        super.onDetachedFromWindow()
    }

    override fun onDraw(canvas: Canvas) {
        val w = width.toFloat()
        val h = height.toFloat()
        if (w <= 0f || h <= 0f) {
            return
        }
        canvas.drawColor(Color.BLACK)
        val life = (SystemClock.uptimeMillis() - started) / 1000f
        when (palette.key) {
            Palette.HACKER.key -> rain(canvas, w, h, life)
            Palette.ANIME.key -> petals(canvas, w, h, life)
            else -> quiet(canvas, w, h)
        }
    }

    private fun quiet(canvas: Canvas, w: Float, h: Float) {
        ready(height)
        val edge = fade ?: return
        paint.style = Paint.Style.FILL
        paint.shader = edge
        paint.alpha = 255
        canvas.drawRect(0f, 0f, w, h, paint)
        paint.shader = null
    }

    private fun rain(canvas: Canvas, w: Float, h: Float, life: Float) {
        val density = resources.displayMetrics.density
        val column = COLUMN_DP * density
        val columns = (w / column).toInt().coerceAtLeast(1)
        val row = h / RAIN_ROWS
        paint.shader = null
        paint.style = Paint.Style.FILL
        paint.typeface = Type.data
        paint.textSize = column * 0.86f
        for (index in 0 until columns) {
            val seed = index * 2654435761L.toInt()
            val speed = 2.2f + ((seed ushr 3) and 7) * 0.55f
            val drop = ((life * speed + (seed and 63) * 0.37f) % (RAIN_ROWS + 8f))
            for (back in 0 until 12) {
                val at = drop - back
                if (at < 0f || at > RAIN_ROWS) {
                    continue
                }
                val bit = ((seed + at.toInt() * 31 + (life / 3f).toInt() * 7) and 1)
                val head = back == 0
                paint.color = if (head) palette.ok else palette.accent
                paint.alpha = if (head) 255 else (170 - back * 14).coerceAtLeast(18)
                canvas.drawText(
                    if (bit == 0) "0" else "1",
                    index * column + column * 0.1f,
                    at * row,
                    paint,
                )
            }
        }
        paint.alpha = 255
        paint.color = Ink.fade(palette.background, 0.55f)
        var y = 0f
        while (y < h) {
            canvas.drawRect(0f, y, w, y + 1f, paint)
            y += 3f * density
        }
    }

    private fun petals(canvas: Canvas, w: Float, h: Float, life: Float) {
        ready(height)
        sky?.let {
            paint.shader = it
            paint.alpha = 255
            canvas.drawRect(0f, 0f, w, h, paint)
            paint.shader = null
        }
        val held = glows
        if (held != null) {
            val turn = ((life * 1000f % PERIOD_MILLIS) / PERIOD_MILLIS * 2.0 * Math.PI).toFloat()
            val radius = maxOf(w, h) * 0.62f
            blob(canvas, held[0], w * (0.24f + 0.14f * sin(turn)), h * 0.16f, radius)
            blob(canvas, held[1], w * (0.80f + 0.10f * cos(turn * 1.2f)), h * 0.30f, radius * 0.8f)
            blob(canvas, held[2], w * 0.5f, h * (0.88f + 0.06f * sin(turn * 0.8f)), radius * 0.9f)
        }
        paint.style = Paint.Style.FILL
        for (index in 0 until PETALS) {
            val seed = index * 1103515245 + 12345
            val across = ((seed ushr 5) and 1023) / 1023f
            val speed = 0.10f + ((seed ushr 9) and 7) * 0.035f
            val drift = sin(life * (0.5f + (seed and 3) * 0.2f) + index.toFloat()) * w * 0.06f
            val down = ((life * speed + across) % 1.2f) - 0.1f
            val x = across * w + drift
            val y = down * h
            val size = 5f + ((seed ushr 13) and 3) * 2.4f
            val spin = (life * (24f + (seed and 15) * 3f) + index * 37f) % 360f
            paint.color = if ((seed and 1) == 0) palette.accent else palette.error
            paint.alpha = 120 + ((seed ushr 17) and 63)
            canvas.save()
            canvas.translate(x, y)
            canvas.rotate(spin)
            leaf(canvas, size * resources.displayMetrics.density * 0.5f)
            canvas.restore()
        }
        paint.alpha = 255
        for (index in 0 until SPARKS) {
            val seed = index * 2246822519L.toInt()
            val x = (((seed ushr 7) and 2047) / 2047f) * w
            val y = (((seed ushr 17) and 2047) / 2047f) * h
            val beat = (sin(life * (1.4f + (seed and 7) * 0.3f) + index.toFloat()) + 1f) * 0.5f
            paint.color = palette.foreground
            paint.alpha = (30 + beat * 150f).toInt().coerceIn(0, 255)
            canvas.drawCircle(x, y, 1.2f * resources.displayMetrics.density * (0.4f + beat), paint)
        }
        paint.alpha = 255
        fade?.let {
            paint.shader = it
            canvas.drawRect(0f, 0f, w, h, paint)
            paint.shader = null
        }
    }

    private fun leaf(canvas: Canvas, radius: Float) {
        petal.reset()
        petal.moveTo(0f, -radius)
        petal.cubicTo(radius * 0.9f, -radius * 0.5f, radius * 0.7f, radius * 0.7f, 0f, radius)
        petal.cubicTo(-radius * 0.7f, radius * 0.7f, -radius * 0.9f, -radius * 0.5f, 0f, -radius)
        petal.close()
        canvas.drawPath(petal, paint)
    }

    private fun ready(tall: Int) {
        if (glows != null && madeAt == palette.background && madeFor == tall) {
            return
        }
        madeAt = palette.background
        madeFor = tall
        glows = arrayOf(
            unit(palette.glowFirst),
            unit(palette.glowSecond),
            unit(palette.glowThird),
        )
        fade = LinearGradient(
            0f, 0f, 0f, tall.toFloat(),
            Ink.hot(palette.background, 0.017f),
            Ink.dim(palette.background, 0.12f),
            Shader.TileMode.CLAMP,
        )
        sky = LinearGradient(
            0f, 0f, 0f, tall.toFloat(),
            Ink.hot(palette.glowFirst, 0.25f),
            palette.background,
            Shader.TileMode.CLAMP,
        )
    }

    private fun unit(colour: Int) = RadialGradient(
        0f, 0f, 1f,
        Color.argb(150, Color.red(colour), Color.green(colour), Color.blue(colour)),
        Color.argb(0, Color.red(colour), Color.green(colour), Color.blue(colour)),
        Shader.TileMode.CLAMP,
    )

    private fun blob(canvas: Canvas, glow: RadialGradient, x: Float, y: Float, radius: Float) {
        place.setScale(radius, radius)
        place.postTranslate(x, y)
        glow.setLocalMatrix(place)
        paint.shader = glow
        canvas.drawCircle(x, y, radius, paint)
        paint.shader = null
    }
}

internal object Motion {

    const val TOUCH_UNITS = 12

    const val TAP = 70L
    const val REBOUND = 110L
    const val HOLD = 350L
    const val LEAVE = 140L
    const val ENTER = 260L
    const val SWEEP = 420L
    const val SETTLE = 620L
    const val REFUSAL = 1_400L

    private var scale = 1f

    fun readWhatTheDeviceAsksFor(context: Context) {
        scale = runCatching {
            Settings.Global.getFloat(
                context.contentResolver,
                Settings.Global.ANIMATOR_DURATION_SCALE,
                1f,
            )
        }.getOrDefault(1f).coerceIn(0f, 4f)
    }

    fun still(): Boolean = scale <= 0.01f

    fun of(millis: Long): Long = if (still()) 0L else (millis * scale).toLong().coerceAtLeast(1L)
}

internal class Chibi(private val who: Face, private val palette: Palette) : Drawable() {

    enum class Face {
        SPARK,
        CALM,
        WINK,
        RIBBON,
    }

    private companion object {
        const val GRID = 24f
    }

    private val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        strokeCap = Paint.Cap.ROUND
        strokeJoin = Paint.Join.ROUND
    }
    private val path = Path()

    private val skin = 0xFFFFE0CE.toInt()
    private val ink = 0xFF2A1F3A.toInt()

    private fun hair(): Int = when (who) {
        Face.SPARK -> palette.accent
        Face.CALM -> palette.glowSecond
        Face.WINK -> palette.warning
        Face.RIBBON -> palette.glowThird
    }

    override fun draw(canvas: Canvas) {
        val edge = min(bounds.width(), bounds.height()).toFloat()
        if (edge <= 0f) return
        val unit = edge / GRID
        val left = bounds.left + (bounds.width() - edge) / 2f
        val top = bounds.top + (bounds.height() - edge) / 2f
        canvas.save()
        canvas.translate(left, top)

        body(canvas, unit)
        head(canvas, unit)
        fringe(canvas, unit)
        eyes(canvas, unit)
        mouth(canvas, unit)
        blush(canvas, unit)
        crown(canvas, unit)

        canvas.restore()
    }

    private fun body(canvas: Canvas, unit: Float) {
        paint.style = Paint.Style.FILL
        paint.color = Ink.fade(hair(), 0.85f)
        path.reset()
        path.moveTo(7f * unit, 24f * unit)
        path.cubicTo(7.4f * unit, 18.6f * unit, 9.6f * unit, 16.8f * unit, 12f * unit, 16.8f * unit)
        path.cubicTo(14.4f * unit, 16.8f * unit, 16.6f * unit, 18.6f * unit, 17f * unit, 24f * unit)
        path.close()
        canvas.drawPath(path, paint)
    }

    private fun head(canvas: Canvas, unit: Float) {
        paint.style = Paint.Style.FILL
        paint.color = skin
        canvas.drawRoundRect(
            5.4f * unit, 4.2f * unit, 18.6f * unit, 17.6f * unit,
            6.4f * unit, 6.4f * unit, paint,
        )
        paint.color = hair()
        path.reset()
        path.moveTo(5.2f * unit, 11.4f * unit)
        path.cubicTo(4.6f * unit, 4.2f * unit, 9f * unit, 1.6f * unit, 12f * unit, 1.6f * unit)
        path.cubicTo(15f * unit, 1.6f * unit, 19.4f * unit, 4.2f * unit, 18.8f * unit, 11.4f * unit)
        path.cubicTo(18.2f * unit, 8.6f * unit, 17.4f * unit, 7.4f * unit, 16.4f * unit, 6.8f * unit)
        path.cubicTo(14.6f * unit, 8.8f * unit, 9.4f * unit, 8.8f * unit, 7.6f * unit, 6.8f * unit)
        path.cubicTo(6.6f * unit, 7.4f * unit, 5.8f * unit, 8.6f * unit, 5.2f * unit, 11.4f * unit)
        path.close()
        canvas.drawPath(path, paint)
    }

    private fun fringe(canvas: Canvas, unit: Float) {
        paint.style = Paint.Style.FILL
        paint.color = hair()
        canvas.drawRoundRect(
            4.4f * unit, 7.4f * unit, 6.6f * unit, 15.4f * unit,
            1.1f * unit, 1.1f * unit, paint,
        )
        canvas.drawRoundRect(
            17.4f * unit, 7.4f * unit, 19.6f * unit, 15.4f * unit,
            1.1f * unit, 1.1f * unit, paint,
        )
    }

    private fun eyes(canvas: Canvas, unit: Float) {
        paint.style = Paint.Style.FILL
        if (who == Face.CALM) {
            paint.style = Paint.Style.STROKE
            paint.strokeWidth = 0.9f * unit
            paint.color = ink
            arc(canvas, 9.4f * unit, 12.4f * unit, 1.5f * unit)
            arc(canvas, 14.6f * unit, 12.4f * unit, 1.5f * unit)
            paint.style = Paint.Style.FILL
            return
        }
        eye(canvas, unit, 9.4f * unit)
        if (who == Face.WINK) {
            paint.style = Paint.Style.STROKE
            paint.strokeWidth = 0.9f * unit
            paint.color = ink
            arc(canvas, 14.6f * unit, 12.4f * unit, 1.5f * unit)
            paint.style = Paint.Style.FILL
            return
        }
        eye(canvas, unit, 14.6f * unit)
    }

    private fun eye(canvas: Canvas, unit: Float, x: Float) {
        paint.color = ink
        canvas.drawRoundRect(
            x - 1.6f * unit, 10.6f * unit, x + 1.6f * unit, 14.2f * unit,
            1.6f * unit, 1.8f * unit, paint,
        )
        paint.color = palette.accent
        canvas.drawCircle(x, 13.1f * unit, 0.95f * unit, paint)
        paint.color = 0xFFFFFFFF.toInt()
        canvas.drawCircle(x - 0.55f * unit, 11.5f * unit, 0.62f * unit, paint)
        canvas.drawCircle(x + 0.7f * unit, 13.6f * unit, 0.3f * unit, paint)
    }

    private fun arc(canvas: Canvas, x: Float, y: Float, radius: Float) {
        path.reset()
        path.moveTo(x - radius, y + radius * 0.45f)
        path.quadTo(x, y - radius * 0.75f, x + radius, y + radius * 0.45f)
        canvas.drawPath(path, paint)
    }

    private fun mouth(canvas: Canvas, unit: Float) {
        paint.style = Paint.Style.STROKE
        paint.strokeWidth = 0.75f * unit
        paint.color = ink
        path.reset()
        if (who == Face.SPARK) {
            path.moveTo(10.9f * unit, 15.2f * unit)
            path.quadTo(12f * unit, 16.8f * unit, 13.1f * unit, 15.2f * unit)
        } else {
            path.moveTo(11.1f * unit, 15.7f * unit)
            path.quadTo(12f * unit, 15.1f * unit, 12.9f * unit, 15.7f * unit)
        }
        canvas.drawPath(path, paint)
        paint.style = Paint.Style.FILL
    }

    private fun blush(canvas: Canvas, unit: Float) {
        paint.style = Paint.Style.FILL
        paint.color = Ink.fade(palette.error, 0.42f)
        canvas.drawOval(6.6f * unit, 13.6f * unit, 9f * unit, 15.2f * unit, paint)
        canvas.drawOval(15f * unit, 13.6f * unit, 17.4f * unit, 15.2f * unit, paint)
    }

    private fun crown(canvas: Canvas, unit: Float) {
        paint.style = Paint.Style.FILL
        when (who) {
            Face.SPARK -> {
                paint.color = hair()
                path.reset()
                path.moveTo(12.2f * unit, 2.2f * unit)
                path.cubicTo(
                    13.4f * unit, 0.2f * unit,
                    15.6f * unit, 0.4f * unit,
                    15.2f * unit, 2.2f * unit,
                )
                path.cubicTo(
                    14.6f * unit, 1.2f * unit,
                    13.4f * unit, 1.4f * unit,
                    12.9f * unit, 2.9f * unit,
                )
                path.close()
                canvas.drawPath(path, paint)
            }
            Face.RIBBON -> {
                paint.color = palette.accent
                path.reset()
                path.moveTo(16.6f * unit, 5.4f * unit)
                path.lineTo(19.4f * unit, 3.4f * unit)
                path.lineTo(19.4f * unit, 7.4f * unit)
                path.close()
                canvas.drawPath(path, paint)
                path.reset()
                path.moveTo(16.6f * unit, 5.4f * unit)
                path.lineTo(13.8f * unit, 3.4f * unit)
                path.lineTo(13.8f * unit, 7.4f * unit)
                path.close()
                canvas.drawPath(path, paint)
                canvas.drawCircle(16.6f * unit, 5.4f * unit, 0.9f * unit, paint)
            }
            else -> Unit
        }
    }

    override fun setAlpha(alpha: Int) {
        paint.alpha = alpha
    }

    override fun setColorFilter(filter: ColorFilter?) {
        paint.colorFilter = filter
    }

    @Deprecated("Drawable declares it", ReplaceWith("PixelFormat.TRANSLUCENT"))
    override fun getOpacity(): Int = PixelFormat.TRANSLUCENT
}

internal class TerminalView(
    context: Context,
    private var palette: Palette,
    private val onKey: (ByteArray) -> Unit,
) : View(context) {

    val screen = Screenful(24, 80)
    private val reader = Sequences(screen) { said -> onKey(said.toByteArray(Charsets.UTF_8)) }
    private var lookingBack = 0
    private var dragFrom = 0f
    private var dragged = 0f
    private val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { typeface = Type.data }
    private var cellWidth = 0f
    private var cellHeight = 0f
    private var baseline = 0f
    private var lettering = 0f
    private var onResize: ((Int, Int) -> Unit)? = null

    init {
        isFocusableInTouchMode = true
        lettering = inPixels(13f)
        measureCell()
    }

    fun repaint(next: Palette) {
        palette = next
        invalidate()
    }

    fun sizeText(sp: Float) {
        lettering = inPixels(sp.coerceIn(8f, 24f))
        measureCell()
        fit(width, height)
        invalidate()
    }

    fun textSizeSp(): Float = lettering / inPixels(1f)

    private fun inPixels(sp: Float): Float = TypedValue.applyDimension(
        TypedValue.COMPLEX_UNIT_SP,
        sp,
        resources.displayMetrics,
    )

    fun whenResized(what: (Int, Int) -> Unit) {
        onResize = what
    }

    fun accept(bytes: ByteArray, length: Int) {
        reader.feed(bytes, length)
        lookingBack = 0
        postInvalidate()
    }

    private fun rowAt(r: Int): Array<Screenful.Cell>? {
        val back = lookingBack
        if (back == 0) {
            return screen.lines.getOrNull(r)
        }
        val from = screen.history.size - back
        val at = from + r
        return if (at < screen.history.size) {
            screen.history.getOrNull(at)
        } else {
            screen.lines.getOrNull(at - screen.history.size)
        }
    }

    @Suppress("ClickableViewAccessibility")
    override fun onTouchEvent(event: MotionEvent): Boolean {
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                dragFrom = event.y
                dragged = 0f
                return true
            }
            MotionEvent.ACTION_MOVE -> {
                if (screen.onAlternate || cellHeight <= 0f) {
                    return true
                }
                val moved = event.y - dragFrom
                dragged += kotlin.math.abs(moved)
                val steps = (moved / cellHeight).toInt()
                if (steps != 0) {
                    dragFrom = event.y
                    lookingBack = (lookingBack + steps).coerceIn(0, screen.history.size)
                    invalidate()
                }
                return true
            }
            MotionEvent.ACTION_UP -> {
                if (dragged < cellHeight) {
                    performClick()
                }
                return true
            }
        }
        return super.onTouchEvent(event)
    }

    private fun measureCell() {
        paint.textSize = lettering
        cellWidth = paint.measureText("M")
        val metrics = paint.fontMetrics
        cellHeight = metrics.descent - metrics.ascent + 1f
        baseline = -metrics.ascent
    }

    private fun fit(w: Int, h: Int) {
        if (w <= 0 || h <= 0 || cellWidth <= 0f || cellHeight <= 0f) {
            return
        }
        val columns = (w / cellWidth).toInt().coerceIn(20, 400)
        val rows = (h / cellHeight).toInt().coerceIn(4, 200)
        if (columns != screen.columns || rows != screen.rows) {
            screen.resize(rows, columns)
            onResize?.invoke(rows, columns)
        }
    }

    override fun onSizeChanged(w: Int, h: Int, was: Int, then: Int) {
        super.onSizeChanged(w, h, was, then)
        fit(w, h)
    }

    private fun colourOf(code: Int, ground: Boolean): Int {
        val found = when {
            code == Screenful.DEFAULT_INK -> palette.foreground
            code == Screenful.DEFAULT_GROUND -> Color.TRANSPARENT
            code >= 256 -> {
                val rgb = code - 256
                Color.rgb((rgb shr 16) and 0xFF, (rgb shr 8) and 0xFF, rgb and 0xFF)
            }
            code < 16 -> ANSI[code]
            code < 232 -> {
                val at = code - 16
                Color.rgb(STEPS[at / 36], STEPS[(at / 6) % 6], STEPS[at % 6])
            }
            else -> {
                val grey = (8 + (code - 232) * 10).coerceIn(0, 255)
                Color.rgb(grey, grey, grey)
            }
        }
        return if (ground && found == Color.TRANSPARENT) Color.BLACK else found
    }

    override fun onDraw(canvas: Canvas) {
        canvas.drawColor(Color.BLACK)
        paint.textSize = lettering
        val word = StringBuilder()
        for (r in 0 until screen.rows) {
            val line = rowAt(r) ?: continue
            var c = 0
            while (c < screen.columns) {
                val cell = line.getOrNull(c) ?: break
                var run = c
                while (run < screen.columns) {
                    val next = line.getOrNull(run) ?: break
                    if (next.ground != cell.ground || next.style != cell.style ||
                        next.ink != cell.ink
                    ) {
                        break
                    }
                    run += 1
                }
                val reversed = (cell.style and Screenful.REVERSE) != 0
                val ink = colourOf(if (reversed) cell.ground else cell.ink, reversed)
                val ground = colourOf(if (reversed) cell.ink else cell.ground, true)
                val left = c * cellWidth
                val top = r * cellHeight
                if (ground != Color.BLACK) {
                    paint.style = Paint.Style.FILL
                    paint.color = ground
                    canvas.drawRect(left, top, run * cellWidth, top + cellHeight, paint)
                }
                word.setLength(0)
                for (at in c until run) {
                    word.append(line.getOrNull(at)?.letter ?: ' ')
                }
                paint.color = if ((cell.style and Screenful.FAINT) != 0) {
                    Ink.fade(ink, 0.6f)
                } else {
                    ink
                }
                paint.isFakeBoldText = (cell.style and Screenful.BOLD) != 0
                paint.textSkewX = if ((cell.style and Screenful.ITALIC) != 0) -0.25f else 0f
                paint.style = Paint.Style.FILL
                canvas.drawText(word.toString(), left, top + baseline, paint)
                paint.textSkewX = 0f
                if ((cell.style and Screenful.UNDERLINE) != 0) {
                    paint.strokeWidth = 1f
                    canvas.drawLine(
                        left,
                        top + cellHeight - 2f,
                        run * cellWidth,
                        top + cellHeight - 2f,
                        paint,
                    )
                }
                if ((cell.style and Screenful.STRIKE) != 0) {
                    paint.strokeWidth = 1f
                    canvas.drawLine(
                        left,
                        top + cellHeight / 2f,
                        run * cellWidth,
                        top + cellHeight / 2f,
                        paint,
                    )
                }
                c = run
            }
        }
        if (screen.showCursor && lookingBack == 0) {
            paint.style = Paint.Style.FILL
            paint.color = Ink.fade(palette.accent, 0.55f)
            canvas.drawRect(
                screen.column * cellWidth,
                screen.row * cellHeight,
                (screen.column + 1) * cellWidth,
                (screen.row + 1) * cellHeight,
                paint,
            )
        }
    }

    override fun onCheckIsTextEditor(): Boolean = true

    override fun onCreateInputConnection(out: EditorInfo): InputConnection {
        out.inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
        out.imeOptions = EditorInfo.IME_ACTION_NONE or EditorInfo.IME_FLAG_NO_EXTRACT_UI or
            EditorInfo.IME_FLAG_NO_FULLSCREEN
        return object : BaseInputConnection(this, false) {
            override fun commitText(written: CharSequence?, position: Int): Boolean {
                written?.let { onKey(it.toString().toByteArray(Charsets.UTF_8)) }
                return true
            }

            override fun sendKeyEvent(event: KeyEvent?): Boolean {
                if (event != null && event.action == KeyEvent.ACTION_DOWN) {
                    return this@TerminalView.onKeyDown(event.keyCode, event)
                }
                return true
            }

            override fun deleteSurroundingText(before: Int, after: Int): Boolean {
                repeat(before) { onKey(byteArrayOf(0x7F)) }
                return true
            }
        }
    }

    override fun onKeyDown(code: Int, event: KeyEvent): Boolean {
        val said = when (code) {
            KeyEvent.KEYCODE_DEL -> byteArrayOf(0x7F)
            KeyEvent.KEYCODE_ENTER -> byteArrayOf(0x0D)
            KeyEvent.KEYCODE_TAB -> byteArrayOf(0x09)
            KeyEvent.KEYCODE_ESCAPE -> byteArrayOf(0x1B)
            KeyEvent.KEYCODE_DPAD_UP -> arrow('A')
            KeyEvent.KEYCODE_DPAD_DOWN -> arrow('B')
            KeyEvent.KEYCODE_DPAD_RIGHT -> arrow('C')
            KeyEvent.KEYCODE_DPAD_LEFT -> arrow('D')
            KeyEvent.KEYCODE_MOVE_HOME -> "\u001B[H".toByteArray()
            KeyEvent.KEYCODE_MOVE_END -> "\u001B[F".toByteArray()
            else -> {
                val letter = event.unicodeChar
                when {
                    event.isCtrlPressed && letter != 0 ->
                        byteArrayOf((letter.toChar().uppercaseChar().code and 0x1F).toByte())
                    letter != 0 -> letter.toChar().toString().toByteArray(Charsets.UTF_8)
                    else -> return super.onKeyDown(code, event)
                }
            }
        }
        onKey(said)
        return true
    }

    fun arrow(which: Char): ByteArray =
        if (screen.applicationKeys) {
            "\u001BO$which".toByteArray()
        } else {
            "\u001B[$which".toByteArray()
        }

    private companion object {
        val STEPS = intArrayOf(0, 95, 135, 175, 215, 255)

        val ANSI = intArrayOf(
            0xFF1E1E1E.toInt(), 0xFFCD3131.toInt(), 0xFF0DBC79.toInt(), 0xFFE5E510.toInt(),
            0xFF2472C8.toInt(), 0xFFBC3FBC.toInt(), 0xFF11A8CD.toInt(), 0xFFE5E5E5.toInt(),
            0xFF666666.toInt(), 0xFFF14C4C.toInt(), 0xFF23D18B.toInt(), 0xFFF5F543.toInt(),
            0xFF3B8EEA.toInt(), 0xFFD670D6.toInt(), 0xFF29B8DB.toInt(), 0xFFFFFFFF.toInt(),
        )
    }
}

internal object Glyphs {

    private val held = HashMap<Mark.Shape, Bitmap>()

    private var source: Resources? = null

    fun open(context: Context) {
        source = context.applicationContext.resources
    }

    fun of(shape: Mark.Shape): Bitmap? {
        if (shape.art == 0) {
            return null
        }
        held[shape]?.let { return it }
        val from = source ?: return null
        val painted = runCatching { BitmapFactory.decodeResource(from, shape.art) }
            .getOrNull() ?: return null
        val mask = runCatching { painted.extractAlpha() }.getOrNull()
        painted.recycle()
        if (mask == null) {
            return null
        }
        held[shape] = mask
        return mask
    }
}

internal class Mark(
    private val shape: Shape,
    private val colour: Int,
    private val heft: Float = 1f,
    private val glow: Boolean = false,
) : Drawable() {

    enum class Shape(val art: Int) {
        FOLDER(R.drawable.omni_icon_folder),
        FILE(R.drawable.omni_icon_file),
        MORE(R.drawable.omni_icon_more),
        CLOSE(R.drawable.omni_icon_close),
        GEAR(0),
        SAVE(R.drawable.omni_icon_save),
        FIND(R.drawable.omni_icon_find),
        CODE(R.drawable.omni_icon_code),
        IMAGE(R.drawable.omni_icon_image),
        MARKUP(0),
        NOTE(R.drawable.omni_icon_note),
        PACKAGE(R.drawable.omni_icon_package),
        ARCHIVE(R.drawable.omni_icon_archive),
        SOUND(R.drawable.omni_icon_sound),
        FILM(0),
        BINARY(0),
        FONT(R.drawable.omni_icon_font),
        TABLE(0),
        BRACES(R.drawable.omni_icon_braces),
        CUP(0),
        SORT(0),
        SORTED_UP(R.drawable.omni_icon_sorted_up),
        SORTED_DOWN(R.drawable.omni_icon_sorted_down),
        CHART(R.drawable.omni_icon_chart),
        TICK(0),
        MENU(R.drawable.omni_icon_menu),
        HOME(R.drawable.omni_icon_home),
        ROBOT(R.drawable.omni_icon_robot),
        LOG(R.drawable.omni_icon_log),
        BRUSH(R.drawable.omni_icon_brush),
        GLOBE(R.drawable.omni_icon_globe),
        SPEECH(R.drawable.omni_icon_speech),
        STAR(R.drawable.omni_icon_star),
        INFO(R.drawable.omni_icon_info),
        HELP(R.drawable.omni_icon_help),
        WEB(R.drawable.omni_icon_web),
        BRANCH(R.drawable.omni_icon_branch),
        PLAY(R.drawable.omni_icon_play),
        STOP(R.drawable.omni_icon_stop),
        SEND(R.drawable.omni_icon_send),
        LINK(R.drawable.omni_icon_link),
        SPARK(R.drawable.omni_icon_spark),
        ADD(R.drawable.omni_icon_add),
        CHEVRON_UP(R.drawable.omni_icon_chevron_up),
        CHEVRON_DOWN(R.drawable.omni_icon_chevron_down),
        CHEVRON_LEFT(R.drawable.omni_icon_chevron_left),
        CHEVRON_RIGHT(R.drawable.omni_icon_chevron_right),
        SHIELD(R.drawable.omni_icon_shield),
        MAIL(0),
        BOLT(0),
        SWAP(0),
        UNDO(0),
        REDO(0),
        HEART(R.drawable.omni_icon_heart),
        CLOCK(R.drawable.omni_icon_clock),
        TRASH(R.drawable.omni_icon_trash),
        CLIP(R.drawable.omni_icon_clip),
        DOWN(R.drawable.omni_icon_down),
        PHONE(R.drawable.omni_icon_phone),
        BOOK(R.drawable.omni_icon_book),
        WIDEN(R.drawable.omni_icon_widen),
        PEN(R.drawable.omni_icon_pen),
        DROID(R.drawable.omni_icon_droid),
        BROOM(R.drawable.omni_icon_broom),
        SHARE(R.drawable.omni_icon_share),
        COPY(R.drawable.omni_icon_copy),
        EXPORT(R.drawable.omni_icon_export),
        INFO_FILL(R.drawable.omni_icon_info_fill),
        PAGE_ADD(R.drawable.omni_icon_page_add),
        FOLDER_OPEN(R.drawable.omni_icon_folder_open),
        PEOPLE(R.drawable.omni_icon_people),
        BUG(0),
        BULB(0),
        STAR_LINE(R.drawable.omni_icon_star_line),
        BRAIN(R.drawable.omni_icon_brain),
        SCREEN(R.drawable.omni_icon_screen),
        NOTE_FILL(R.drawable.omni_icon_note_fill),
        MAIL_FILL(R.drawable.omni_icon_mail_fill),
        PERSON_FILL(R.drawable.omni_icon_person_fill),
        HEART_FILL(0),
        COPYRIGHT(R.drawable.omni_icon_copyright),
    }

    private companion object {
        const val GRID = 24f
        const val STROKE = 1.7f
        const val GLOW_STEPS = 3
        const val GLOW_SWELL = 2.1f
        const val ART_SHARE = 0.63f
        const val ART_HEFT = 0.04f
        val CLEARING = PorterDuffXfermode(PorterDuff.Mode.CLEAR)
    }

    private val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        strokeCap = Paint.Cap.ROUND
        strokeJoin = Paint.Join.ROUND
        isFilterBitmap = true
    }
    private val path = Path()
    private val spread = RectF()

    private var swell = 0f
    private var bleed = 255
    private var given: ColorFilter? = null

    override fun draw(canvas: Canvas) {
        val edge = min(bounds.width(), bounds.height()).toFloat()
        if (edge <= 0f) return
        val drawn = Glyphs.of(shape)
        if (drawn != null) {
            paintTheArt(canvas, drawn, edge)
            return
        }
        val widest = STROKE * heft + if (glow) GLOW_STEPS * GLOW_SWELL else 0f
        val room = GRID + widest
        val unit = edge / room
        if (glow) {
            var step = GLOW_STEPS
            while (step >= 1) {
                swell = step * GLOW_SWELL
                bleed = 44 - step * 10
                stamp(canvas, unit, widest)
                step -= 1
            }
            swell = 0f
            bleed = 255
        }
        stamp(canvas, unit, widest)
    }

    private fun paintTheArt(canvas: Canvas, drawn: Bitmap, edge: Float) {
        val side = edge * (ART_SHARE + ART_HEFT * heft)
        paint.style = Paint.Style.FILL
        paint.color = colour
        paint.colorFilter = given
        if (glow) {
            var step = GLOW_STEPS
            while (step >= 1) {
                paint.alpha = 44 - step * 10
                stampTheArt(canvas, drawn, side + side * step * GLOW_SWELL / GRID)
                step -= 1
            }
        }
        paint.alpha = bleed
        stampTheArt(canvas, drawn, side)
    }

    private fun stampTheArt(canvas: Canvas, drawn: Bitmap, side: Float) {
        val midX = bounds.exactCenterX()
        val midY = bounds.exactCenterY()
        spread.set(midX - side / 2f, midY - side / 2f, midX + side / 2f, midY + side / 2f)
        canvas.drawBitmap(drawn, null, spread, paint)
    }

    private fun stamp(canvas: Canvas, unit: Float, widest: Float) {
        val edge = unit * (GRID + widest)
        canvas.save()
        canvas.translate(
            bounds.left + (bounds.width() - edge) / 2f + unit * widest / 2f,
            bounds.top + (bounds.height() - edge) / 2f + unit * widest / 2f,
        )
        canvas.scale(unit, unit)
        paint.color = colour
        paint.alpha = bleed
        paint.strokeWidth = STROKE * heft + swell
        path.rewind()
        when (shape) {
            Shape.CUP -> {
                paint.style = Paint.Style.STROKE
                path.moveTo(6f, 11f)
                path.lineTo(6f, 15.4f)
                path.cubicTo(6f, 17.6f, 8f, 18.8f, 11f, 18.8f)
                path.cubicTo(14f, 18.8f, 16f, 17.6f, 16f, 15.4f)
                path.lineTo(16f, 11f)
                path.close()
                path.moveTo(16f, 12f)
                path.cubicTo(19.4f, 12f, 19.4f, 16f, 16.4f, 16.2f)
                path.moveTo(10f, 8.4f)
                path.cubicTo(8.6f, 6.6f, 11.4f, 5.8f, 10.4f, 3.4f)
                path.moveTo(13.2f, 8.4f)
                path.cubicTo(11.8f, 6.6f, 14.6f, 5.8f, 13.6f, 3.4f)
                canvas.drawPath(path, paint)
                paint.style = Paint.Style.STROKE
                canvas.drawLine(5f, 20.6f, 17f, 20.6f, paint)
            }
            Shape.GEAR -> {
                paint.style = Paint.Style.STROKE
                val teeth = 8
                val inner = 5.4f
                val outer = 8.6f
                val half = PI.toFloat() / teeth
                var tooth = 0
                while (tooth < teeth) {
                    val turn = tooth * 2f * half
                    val lip = half * 0.46f
                    path.moveTo(
                        12f + cos(turn - half + lip) * inner,
                        12f + sin(turn - half + lip) * inner,
                    )
                    path.lineTo(12f + cos(turn - lip) * outer, 12f + sin(turn - lip) * outer)
                    path.lineTo(12f + cos(turn + lip) * outer, 12f + sin(turn + lip) * outer)
                    path.lineTo(
                        12f + cos(turn + half - lip) * inner,
                        12f + sin(turn + half - lip) * inner,
                    )
                    tooth += 1
                }
                path.close()
                canvas.drawPath(path, paint)
                canvas.drawCircle(12f, 12f, 3.0f, paint)
            }
            Shape.MARKUP -> {
                paint.style = Paint.Style.STROKE
                path.moveTo(8.4f, 6.4f)
                path.lineTo(4.6f, 12f)
                path.lineTo(8.4f, 17.6f)
                path.moveTo(15.6f, 6.4f)
                path.lineTo(19.4f, 12f)
                path.lineTo(15.6f, 17.6f)
                path.moveTo(13.4f, 5.6f)
                path.lineTo(10.6f, 18.4f)
                canvas.drawPath(path, paint)
            }
            Shape.FILM -> {
                paint.style = Paint.Style.STROKE
                path.addRoundRect(3.5f, 6.5f, 20.5f, 17.5f, 1.4f, 1.4f, Path.Direction.CW)
                canvas.drawPath(path, paint)
                path.rewind()
                paint.style = Paint.Style.FILL
                var at = 5f
                while (at < 20f) {
                    path.addRect(at, 7.6f, at + 1.4f, 9f, Path.Direction.CW)
                    path.addRect(at, 15f, at + 1.4f, 16.4f, Path.Direction.CW)
                    at += 3f
                }
                canvas.drawPath(path, paint)
                path.rewind()
                path.moveTo(10.6f, 10.4f)
                path.lineTo(15f, 12.4f)
                path.lineTo(10.6f, 14.4f)
                path.close()
                canvas.drawPath(path, paint)
            }
            Shape.BINARY -> {
                paint.style = Paint.Style.STROKE
                path.addRoundRect(6.5f, 6.5f, 17.5f, 17.5f, 1.2f, 1.2f, Path.Direction.CW)
                var at = 8.6f
                while (at < 17f) {
                    path.moveTo(4.4f, at)
                    path.lineTo(6.5f, at)
                    path.moveTo(17.5f, at)
                    path.lineTo(19.6f, at)
                    path.moveTo(at, 4.4f)
                    path.lineTo(at, 6.5f)
                    path.moveTo(at, 17.5f)
                    path.lineTo(at, 19.6f)
                    at += 3f
                }
                canvas.drawPath(path, paint)
            }
            Shape.TABLE -> {
                paint.style = Paint.Style.STROKE
                path.addRoundRect(4.5f, 5.5f, 19.5f, 18.5f, 1.4f, 1.4f, Path.Direction.CW)
                path.moveTo(4.5f, 9.6f)
                path.lineTo(19.5f, 9.6f)
                path.moveTo(4.5f, 14.2f)
                path.lineTo(19.5f, 14.2f)
                path.moveTo(11f, 9.6f)
                path.lineTo(11f, 18.5f)
                canvas.drawPath(path, paint)
            }
            Shape.TICK -> {
                paint.style = Paint.Style.STROKE
                path.moveTo(6.5f, 12.5f)
                path.lineTo(10.5f, 16.5f)
                path.lineTo(17.5f, 8f)
                canvas.drawPath(path, paint)
            }
            Shape.SORT -> {
                paint.style = Paint.Style.STROKE
                path.moveTo(4.5f, 7f)
                path.lineTo(15f, 7f)
                path.moveTo(4.5f, 12f)
                path.lineTo(12f, 12f)
                path.moveTo(4.5f, 17f)
                path.lineTo(9f, 17f)
                path.moveTo(17.5f, 5.5f)
                path.lineTo(17.5f, 18.5f)
                path.moveTo(15f, 16f)
                path.lineTo(17.5f, 18.5f)
                path.lineTo(20f, 16f)
                canvas.drawPath(path, paint)
            }
            Shape.MAIL -> {
                paint.style = Paint.Style.STROKE
                path.addRoundRect(3.5f, 5.8f, 20.5f, 18.2f, 1.6f, 1.6f, Path.Direction.CW)
                path.moveTo(3.5f, 7.4f)
                path.lineTo(12f, 13.6f)
                path.lineTo(20.5f, 7.4f)
                canvas.drawPath(path, paint)
            }
            Shape.BUG -> {
                paint.style = Paint.Style.STROKE
                path.addRoundRect(7f, 7.6f, 17f, 19.6f, 5f, 5f, Path.Direction.CW)
                path.moveTo(9.2f, 7.6f)
                path.cubicTo(9.2f, 4.6f, 14.8f, 4.6f, 14.8f, 7.6f)
                path.moveTo(9.6f, 4.6f)
                path.lineTo(10.8f, 6.4f)
                path.moveTo(14.4f, 4.6f)
                path.lineTo(13.2f, 6.4f)
                path.moveTo(7f, 11.4f)
                path.lineTo(4f, 9.6f)
                path.moveTo(7f, 14f)
                path.lineTo(3.6f, 14f)
                path.moveTo(7f, 16.6f)
                path.lineTo(4f, 18.4f)
                path.moveTo(17f, 11.4f)
                path.lineTo(20f, 9.6f)
                path.moveTo(17f, 14f)
                path.lineTo(20.4f, 14f)
                path.moveTo(17f, 16.6f)
                path.lineTo(20f, 18.4f)
                path.moveTo(12f, 9.6f)
                path.lineTo(12f, 19.4f)
                canvas.drawPath(path, paint)
            }
            Shape.BULB -> {
                paint.style = Paint.Style.STROKE
                path.moveTo(9f, 16.6f)
                path.cubicTo(9f, 14.4f, 5.8f, 13.2f, 5.8f, 9.8f)
                path.cubicTo(5.8f, 6.4f, 8.6f, 3.6f, 12f, 3.6f)
                path.cubicTo(15.4f, 3.6f, 18.2f, 6.4f, 18.2f, 9.8f)
                path.cubicTo(18.2f, 13.2f, 15f, 14.4f, 15f, 16.6f)
                path.close()
                path.moveTo(9.4f, 18.8f)
                path.lineTo(14.6f, 18.8f)
                path.moveTo(10.4f, 21f)
                path.lineTo(13.6f, 21f)
                canvas.drawPath(path, paint)
            }
            Shape.HEART_FILL -> {
                paint.style = Paint.Style.FILL
                path.moveTo(12f, 20.8f)
                path.cubicTo(1.6f, 13.9f, 3.8f, 4.6f, 9.2f, 4.6f)
                path.cubicTo(10.9f, 4.6f, 12f, 5.9f, 12f, 7.2f)
                path.cubicTo(12f, 5.9f, 13.1f, 4.6f, 14.8f, 4.6f)
                path.cubicTo(20.2f, 4.6f, 22.4f, 13.9f, 12f, 20.8f)
                path.close()
                canvas.drawPath(path, paint)
            }
            Shape.SWAP -> {
                paint.style = Paint.Style.STROKE
                path.moveTo(4f, 9f)
                path.lineTo(19f, 9f)
                path.moveTo(15.5f, 5.5f)
                path.lineTo(19f, 9f)
                path.lineTo(15.5f, 12.5f)
                path.moveTo(20f, 15f)
                path.lineTo(5f, 15f)
                path.moveTo(8.5f, 11.5f)
                path.lineTo(5f, 15f)
                path.lineTo(8.5f, 18.5f)
                canvas.drawPath(path, paint)
            }
            Shape.UNDO -> {
                paint.style = Paint.Style.STROKE
                path.moveTo(9.5f, 5f)
                path.lineTo(5f, 9.5f)
                path.lineTo(9.5f, 14f)
                path.moveTo(5f, 9.5f)
                path.lineTo(14.5f, 9.5f)
                path.cubicTo(18.5f, 9.5f, 20.5f, 12f, 20.5f, 14.8f)
                path.cubicTo(20.5f, 17.6f, 18.5f, 19.5f, 15f, 19.5f)
                path.lineTo(10.5f, 19.5f)
                canvas.drawPath(path, paint)
            }
            Shape.REDO -> {
                paint.style = Paint.Style.STROKE
                path.moveTo(14.5f, 5f)
                path.lineTo(19f, 9.5f)
                path.lineTo(14.5f, 14f)
                path.moveTo(19f, 9.5f)
                path.lineTo(9.5f, 9.5f)
                path.cubicTo(5.5f, 9.5f, 3.5f, 12f, 3.5f, 14.8f)
                path.cubicTo(3.5f, 17.6f, 5.5f, 19.5f, 9f, 19.5f)
                path.lineTo(13.5f, 19.5f)
                canvas.drawPath(path, paint)
            }
            Shape.BOLT -> {
                paint.style = Paint.Style.FILL
                path.moveTo(13.6f, 3.4f)
                path.lineTo(6f, 13.6f)
                path.lineTo(11f, 13.6f)
                path.lineTo(10f, 20.6f)
                path.lineTo(18f, 10.2f)
                path.lineTo(13f, 10.2f)
                path.close()
                canvas.drawPath(path, paint)
            }
            else -> Unit
        }
        canvas.restore()
    }

    override fun setAlpha(alpha: Int) {
        paint.alpha = alpha
    }

    override fun setColorFilter(filter: ColorFilter?) {
        given = filter
        paint.colorFilter = filter
    }

    @Deprecated("The platform asks for this and nothing reads it.")
    override fun getOpacity(): Int = PixelFormat.TRANSLUCENT
}

internal class Filling(private val colour: Int, private val radius: Float) : Drawable() {

    private val paint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val shape = RectF()

    var much = 0f
        set(value) {
            field = value.coerceIn(0f, 1f)
            invalidateSelf()
        }

    override fun draw(canvas: Canvas) {
        if (much <= 0f || bounds.width() <= 0 || bounds.height() <= 0) return
        paint.color = colour
        shape.set(
            bounds.left.toFloat(),
            bounds.top.toFloat(),
            bounds.left + bounds.width() * much,
            bounds.bottom.toFloat(),
        )
        canvas.drawRoundRect(shape, radius, radius, paint)
    }

    override fun setAlpha(alpha: Int) {
        paint.alpha = alpha
    }

    override fun setColorFilter(filter: ColorFilter?) {
        paint.colorFilter = filter
    }

    @Deprecated("The platform asks for this and nothing reads it.")
    override fun getOpacity(): Int = PixelFormat.TRANSLUCENT
}

class FlowLayout(
    context: Context,
    private val betweenX: Int,
    private val betweenY: Int,
) : ViewGroup(context) {

    private fun measureRows(width: Int, place: Boolean): Int {
        val mirrored = layoutDirection == LAYOUT_DIRECTION_RTL
        val limit = width - paddingStart - paddingEnd
        var x = 0
        var y = paddingTop
        var tallest = 0
        for (index in 0 until childCount) {
            val child = getChildAt(index)
            if (child.visibility == GONE) {
                continue
            }
            if (x > 0 && x + child.measuredWidth > limit) {
                x = 0
                y += tallest + betweenY
                tallest = 0
            }
            if (place) {
                val from = if (mirrored) {
                    width - paddingStart - x - child.measuredWidth
                } else {
                    paddingStart + x
                }
                child.layout(from, y, from + child.measuredWidth, y + child.measuredHeight)
            }
            x += child.measuredWidth + betweenX
            tallest = maxOf(tallest, child.measuredHeight)
        }
        return y + tallest + paddingBottom
    }

    override fun onMeasure(widthSpec: Int, heightSpec: Int) {
        val width = MeasureSpec.getSize(widthSpec)
        val room = MeasureSpec.makeMeasureSpec(
            (width - paddingStart - paddingEnd).coerceAtLeast(0),
            MeasureSpec.AT_MOST,
        )
        for (index in 0 until childCount) {
            val child = getChildAt(index)
            if (child.visibility != GONE) {
                child.measure(room, MeasureSpec.makeMeasureSpec(0, MeasureSpec.UNSPECIFIED))
            }
        }
        setMeasuredDimension(width, measureRows(width, false))
    }

    override fun onLayout(changed: Boolean, left: Int, top: Int, right: Int, bottom: Int) {
        measureRows(right - left, true)
    }
}

private enum class Order(val label: Int, val mark: Mark.Shape) {
    NAME(R.string.omni_files_by_name, Mark.Shape.SORTED_UP),
    NAME_BACK(R.string.omni_files_by_name_back, Mark.Shape.SORTED_DOWN),
    KIND(R.string.omni_files_by_kind, Mark.Shape.NOTE),
    CHANGED(R.string.omni_files_by_changed, Mark.Shape.CLOCK),
    SIZE(R.string.omni_files_by_size, Mark.Shape.CHART),
}

private sealed interface Screen {

    data object Projects : Screen


    data object NewProject : Screen

    data object WebToApk : Screen

    data class Files(val root: String, val folder: String) : Screen

    data class Editor(val root: String, val path: String) : Screen

    data class Picture(val root: String, val path: String) : Screen

    data class Build(val root: String) : Screen

    data object Repos : Screen

    data object Trash : Screen

    data object Settings : Screen

    data object Keys : Screen

    data object NewKey : Screen

    data object Logcat : Screen

    data object Terminal : Screen

    data object Assistant : Screen

    data object Theme : Screen

    data object Languages : Screen

    data object Feedback : Screen

    data object Rate : Screen

    data object About : Screen

    data object Help : Screen

    data object HelpAdb : Screen

    data object HelpTalk : Screen

    data object HelpDocs : Screen

    data object HelpFaq : Screen

    data class Legal(val terms: Boolean) : Screen
}

class BuilderActivity : Activity() {

    private companion object {
        const val IMAGE_REQUEST = 2

        const val LOOK_MILLIS = 60L

        const val WATCH_MILLIS = 110L

        const val TOUCH_UNITS = Motion.TOUCH_UNITS

        const val WIDE_DP = 600

        const val COLUMN_DP = 640

        const val DRAWER_DP = 300

        const val SHOT_REQUEST = 3

        const val ARCHIVE_REQUEST = 4

        const val NOTE_REQUEST = 5

        const val PAIRING_SERVICE = "_adb-tls-pairing._tcp."

        const val TRASH_FOLDER = "Trash"

        val PACK_FORMATS = listOf("zip", "tar")

        const val MOST_SHOTS = 4

        const val SHOT_LARGEST_MB = 5

        const val LARGEST_SHOT_BYTES = SHOT_LARGEST_MB * 1024L * 1024L

        const val FEEDBACK_REST_MILLIS = 5L * 60L * 1000L

        const val SHOTS_FOLDER = "Shots"

        const val OUTBOX_FOLDER = "Outbox"

        const val REPORT_FILE = "Report.json"

        const val LOG_HELD = 600

        val TILE_UNTOUCHED = 0xFFF7F8FA.toInt()
        val LABEL_PILL = 0xFF33363A.toInt()

        const val CHARSET_SHOWN = "UTF-8"


        val EDITOR_SYMBOLS = listOf(
            "{", "}", "(", ")", "[", "]", ";", ",", ".", "=", "->", "\u03bb",
            "<", ">", "@", "\"",
        )

        const val LOG_TICK_MILLIS = 400L

        val NATIVE_TILES = listOf(
            Triple(R.drawable.omni_tech_kotlin, "Kotlin", 0xFF642497.toInt()),
            Triple(R.drawable.omni_tech_rust, "Rust", 0xFF975637.toInt()),
            Triple(R.drawable.omni_tech_cpp, "C++", 0xFF1784BD.toInt()),
            Triple(R.drawable.omni_tech_java, "Java", 0xFF692793.toInt()),
        )

        val WEB_TILES = listOf(
            Triple(R.drawable.omni_tech_html, "HTML", 0),
            Triple(R.drawable.omni_tech_css, "CSS", 0),
            Triple(R.drawable.omni_tech_js, "JavaScript", 0),
        )

        val WEB_PARTS = listOf("script", "storage", "pictures", "whole", "cached")

        val WEB_PARTS_ON = listOf(true, true, true, true, false)

        val CLONE_KINDS = listOf("Git")

        const val CLONE_TELL_MILLIS = 250L

        val WEB_PART_NAMES = listOf(
            R.string.omni_web_script,
            R.string.omni_web_storage,
            R.string.omni_web_pictures,
            R.string.omni_web_whole,
            R.string.omni_web_cached,
        )

        val WEB_PART_MARKS = listOf(
            Mark.Shape.CODE,
            Mark.Shape.TABLE,
            Mark.Shape.IMAGE,
            Mark.Shape.WIDEN,
            Mark.Shape.GLOBE,
        )

        val WEB_PART_NOTES = listOf(
            R.string.omni_web_script_note,
            R.string.omni_web_storage_note,
            R.string.omni_web_pictures_note,
            R.string.omni_web_whole_note,
            R.string.omni_web_cached_note,
        )

        const val SUBJECT_LIMIT = 100

        const val DETAIL_LIMIT = 1_000

        val FEEDBACK_KINDS = listOf(
            R.string.omni_feedback_kind_fault,
            R.string.omni_feedback_kind_idea,
            R.string.omni_feedback_kind_question,
            R.string.omni_feedback_kind_other,
        )

        val HELP_BAND_NAMES = listOf(
            R.string.omni_help_band_all,
            R.string.omni_help_band_start,
            R.string.omni_help_band_making,
            R.string.omni_help_band_files,
            R.string.omni_help_band_common,
            R.string.omni_help_band_web,
            R.string.omni_help_band_trouble,
        )

        val HELP_BANDS = listOf(3, 4, 1, 1, 2, 2, 6, 2, 2, 5, 3)

        val TALK_STEPS = listOf(
            R.string.omni_talk_step_ask to R.string.omni_talk_step_ask_note,
            R.string.omni_talk_folder to R.string.omni_talk_folder_note,
            R.string.omni_talk_step_add to R.string.omni_talk_step_add_note,
            R.string.omni_talk_switch to R.string.omni_talk_step_switch_note,
            R.string.omni_talk_step_keep to R.string.omni_talk_step_keep_note,
            R.string.omni_talk_step_word to R.string.omni_talk_step_word_note,
            R.string.omni_talk_step_write to R.string.omni_talk_step_write_note,
            R.string.omni_talk_step_fix to R.string.omni_talk_step_fix_note,
            R.string.omni_talk_step_private to R.string.omni_talk_step_private_note,
            R.string.omni_talk_step_tips to R.string.omni_talk_step_tips_note,
        )

        val OS_TABS = listOf(
            R.string.omni_os_windows to R.drawable.omni_logo_windows,
            R.string.omni_os_mac to R.drawable.omni_logo_apple,
            R.string.omni_os_linux to R.drawable.omni_logo_linux,
        )

        val OS_STEPS = listOf(
            listOf(
                R.string.omni_os_win_one,
                R.string.omni_os_win_two,
                R.string.omni_os_win_three,
            ),
            listOf(
                R.string.omni_os_mac_one,
                R.string.omni_os_mac_two,
                R.string.omni_os_mac_three,
            ),
            listOf(
                R.string.omni_os_linux_one,
                R.string.omni_os_linux_two,
                R.string.omni_os_linux_three,
            ),
        )

        val OS_FILES = listOf(
            listOf("adb.exe", "AdbWinApi.dll", "AdbWinUsbApi.dll", "fastboot.exe"),
            listOf("adb", "fastboot", "etc1tool", "sqlite3"),
            listOf("adb", "fastboot", "etc1tool", "sqlite3"),
        )

        val ADB_ALONE = listOf(
            R.string.omni_adb_a_one to R.string.omni_adb_a_one_note,
            R.string.omni_adb_a_two to R.string.omni_adb_a_two_note,
            R.string.omni_adb_a_three to R.string.omni_adb_a_three_note,
        )

        val ADB_DESK = listOf(
            R.string.omni_adb_d_one to R.string.omni_adb_d_one_note,
            R.string.omni_adb_d_setup to R.string.omni_adb_d_setup_note,
            R.string.omni_adb_d_two to R.string.omni_adb_d_two_note,
            R.string.omni_adb_d_three to R.string.omni_adb_d_three_note,
        )

        val ADB_ALONE_ART = listOf(
            R.drawable.omni_adb_wifi_one,
            R.drawable.omni_adb_wifi_two,
            R.drawable.omni_adb_wifi_three,
        )

        val ADB_DESK_ART = listOf(
            0,
            0,
            R.drawable.omni_adb_usb_two,
            R.drawable.omni_adb_usb_three,
        )

        val HELP_TOPICS = listOf(
            Triple(
                R.string.omni_ask_tree,
                R.string.omni_ask_tree_note,
                R.string.omni_ask_tree_body,
            ),
            Triple(
                R.string.omni_help_first,
                R.string.omni_help_first_note,
                R.string.omni_help_first_body,
            ),
            Triple(
                R.string.omni_help_project,
                R.string.omni_help_project_note,
                R.string.omni_help_project_body,
            ),
            Triple(
                R.string.omni_help_write,
                R.string.omni_help_write_note,
                R.string.omni_help_write_body,
            ),
            Triple(
                R.string.omni_help_package,
                R.string.omni_help_package_note,
                R.string.omni_help_package_body,
            ),
            Triple(
                R.string.omni_help_sign,
                R.string.omni_help_sign_note,
                R.string.omni_help_sign_body,
            ),
            Triple(
                R.string.omni_help_trouble,
                R.string.omni_help_trouble_note,
                R.string.omni_help_trouble_body,
            ),
            Triple(
                R.string.omni_ask_gradle,
                R.string.omni_ask_gradle_note,
                R.string.omni_ask_gradle_note,
            ),
            Triple(
                R.string.omni_ask_small,
                R.string.omni_ask_small_note,
                R.string.omni_ask_small_note,
            ),
            Triple(
                R.string.omni_ask_web,
                R.string.omni_ask_web_note,
                R.string.omni_ask_web_note,
            ),
            Triple(
                R.string.omni_ask_move,
                R.string.omni_ask_move_note,
                R.string.omni_ask_move_note,
            ),
        )

        val FEEDBACK_MARKS = listOf(
            Mark.Shape.BUG,
            Mark.Shape.BULB,
            Mark.Shape.STAR_LINE,
            Mark.Shape.NOTE,
        )

        val FEEDBACK_NOTES = listOf(
            R.string.omni_feedback_kind_fault_note,
            R.string.omni_feedback_kind_idea_note,
            R.string.omni_feedback_kind_question_note,
            R.string.omni_feedback_kind_other_note,
        )

        val GUIDE_TINTS = listOf(
            0xFF2C7BE5.toInt(),
            0xFF1FC77A.toInt(),
            0xFF8B5CF6.toInt(),
            0xFFF08A24.toInt(),
            0xFFEC4899.toInt(),
            0xFF06B6D4.toInt(),
        )

        val HELP_POPULAR = listOf(
            R.string.omni_help_band_adb,
            R.string.omni_menu_assistant,
            R.string.omni_help_band_terminal,
            R.string.omni_help_band_fixing,
        )

        const val WATCH_CODE_MILLIS = 1_200L

        const val OFFER_MILLIS = 220L

        const val OFFER_LEAST = 2

        const val OFFER_MOST = 6

        val STAGE_NAMES = listOf(
            R.string.omni_stage_project,
            R.string.omni_stage_resources,
            R.string.omni_stage_java,
            R.string.omni_stage_manifest,
            R.string.omni_stage_dex,
            R.string.omni_stage_package,
            R.string.omni_stage_signing,
            R.string.omni_stage_bundle,
            R.string.omni_stage_verify,
        )
        const val RISE_DP = 18

        const val DEFAULT_PACKAGE = "com.my.app"
        const val DEFAULT_LABEL = "My App"
        const val BASE_LOCALE = "en"

        const val DEFAULT_KEY_ALIAS = "My_Key"
        const val DEFAULT_KEY_COMMON_NAME = "Builder"
        const val DEFAULT_KEY_ORGANISATION = "My_App"
        const val DEFAULT_KEY_COUNTRY = "EN"
        const val DEFAULT_KEY_YEARS = 10
        const val DEFAULT_KEY_PASSWORD = "My_App_Builder"

        val ONLY_ABI = listOf("arm64-v8a")

        const val OLDEST_PLATFORM = 30
        const val NEWEST_PLATFORM = 37

        val ANDROID_RELEASES = listOf(
            30 to "11", 31 to "12", 32 to "12L", 33 to "13",
            34 to "14", 35 to "15", 36 to "16", 37 to "17",
        )

        val LANGUAGE_CHOICES = listOf(Triple("java", "Java", true))

        val COMMON_PERMISSIONS = listOf(
            "android.permission.INTERNET",
            "android.permission.ACCESS_NETWORK_STATE",
            "android.permission.CAMERA",
            "android.permission.RECORD_AUDIO",
            "android.permission.POST_NOTIFICATIONS",
            "android.permission.VIBRATE",
            "android.permission.ACCESS_FINE_LOCATION",
            "android.permission.READ_MEDIA_IMAGES",
            "android.permission.WAKE_LOCK",
        )

        const val LARGEST_ICON_EDGE = 512

        const val PROJECT_RES = "Res"
        const val PROJECT_ICON = "Icon.png"
        const val PROJECT_FILE = "Builder.json"

        const val OUTPUT_STAMP = "yyyy-MM-dd_HH-mm-ss"

        const val ROW_UNITS = 10

        const val MOST_SHELLS = 4

        const val SHELL_NAME_MOST = 14

        const val ASSISTANT_PATHS = 400

        const val AGENT_ROUNDS = 12

        const val LIBRARY_ROUNDS = 12

        const val LIBRARY_SHOWN = 12

        const val LIBRARY_OPEN_MILLIS = 20_000

        const val LIBRARY_READ_MILLIS = 60_000

        const val LIBRARY_MOST_BYTES = 96L * 1024L * 1024L

        const val AGENT_WRITE = "<<<OMNI WRITE "

        const val AGENT_DELETE = "<<<OMNI DELETE "

        const val AGENT_END = ">>>"

        const val AGENT_DONE = "<<<OMNI DONE>>>"

        const val AGENT_CARRY = "Those are written. Carry on with whatever is still " +
            "missing, or end with the done line if the project now stands on its own."

        val AGENT_RULES = """
            You are the hand as well as the voice: you write the files yourself, and
            nothing you say outside a block is written. Answer with blocks and only
            blocks when there is work to do, one block for each file, the whole file
            every time, never a patch and never an ellipsis.

            To write a file:
            <<<OMNI WRITE Java/com/tr/yt/App.java
            the whole file, from its first line to its last
            >>>

            To remove one:
            <<<OMNI DELETE Java/com/tr/yt/Old.java>>>

            Paths are relative to the project root and never climb out of it. The
            manifest is AndroidManifest.xml, Java classes live under Java, Kotlin
            under Kotlin, C and C++ under Native, Rust under Rust, pictures and
            layouts under Res, and the settings live in Builder.json. When the
            project stands on its own and builds, say what you made in a sentence or
            two and end your answer with this line and nothing after it:
            <<<OMNI DONE>>>
        """.trimIndent()

        val LOCKED_THEMES = setOf("hacker", "anime")

        val SHELL_BAR = 0xFF000000.toInt()

        val PICTURE_SUFFIXES = listOf(".png", ".jpg", ".jpeg", ".webp", ".gif", ".bmp")

        val CODE_SUFFIXES = listOf(
            ".java", ".kt", ".kts", ".rs", ".cpp", ".hpp", ".c", ".h", ".py", ".gradle",
        )

        val PACKAGE_SUFFIXES = listOf(".apk", ".aab")

        val ARCHIVE_SUFFIXES = listOf(".jar", ".aar", ".zip", ".tar", ".gz", ".7z", ".rar")

        val SOUND_SUFFIXES = listOf(".mp3", ".ogg", ".wav", ".m4a", ".flac", ".aac", ".opus")

        val FILM_SUFFIXES = listOf(".mp4", ".mkv", ".webm", ".avi", ".mov", ".3gp")

        val BINARY_SUFFIXES = listOf(".so", ".dex", ".bin", ".arsc", ".o", ".a", ".keystore")

        val FONT_SUFFIXES = listOf(".ttf", ".otf", ".ttc")

        val TABLE_SUFFIXES = listOf(".json", ".csv", ".db", ".sqlite", ".toml", ".yaml", ".yml")

        val NOTE_SUFFIXES = listOf(
            ".txt", ".md", ".properties", ".cfg", ".pro", ".ini", ".log",
        )

        val KEY_SIZES = listOf(2048, 3072, 4096)
        val VALIDITY_YEARS = listOf(1, 5, 10, 25, 100)
    }

    private lateinit var palette: Palette
    private lateinit var aurora: AuroraView
    private lateinit var veil: BinaryVeil
    private lateinit var ceremony: FrameLayout
    private lateinit var drawer: LinearLayout
    private lateinit var shade: View
    private lateinit var crown: LinearLayout
    private lateinit var footer: LinearLayout
    private lateinit var raise: LinearLayout
    private lateinit var scroller: ScrollView
    private lateinit var content: LinearLayout
    private lateinit var results: LinearLayout
    private lateinit var banner: LinearLayout

    private var logPort = 0
    private var logWireless = false
    private var logWired = false
    private var logSeeking = false
    private var showing = false
    private var logPairHost = ""
    private var logPairPort = 0
    private var listening: NsdManager.DiscoveryListener? = null

    private var screen: Screen = Screen.Projects
    private var navigating = false
    private var building = false
    private var standing = "UNKNOWN"
    private var openProject: String? = null

    private var formPackage = DEFAULT_PACKAGE
    private var formLabel = DEFAULT_LABEL
    private var formVersionName = "1.0.0"
    private var formVersionCode = "1"
    private var formMinSdk = OLDEST_PLATFORM
    private var formTargetSdk = NEWEST_PLATFORM
    private val formLanguages = linkedSetOf("java")
    private val formLocales = linkedSetOf<String>()
    private var formImage: String? = null
    private var imageForProject: String? = null
    private var secondFolder: String? = null
    private var fileClip: FileClip? = null
    private var paneRight = false
    private val besideBefore = mutableListOf<String>()
    private val besideAfter = mutableListOf<String>()
    private var leftBox: View? = null
    private var rightBox: View? = null
    private var filesOrder = Order.NAME
    private val pickedIn = mutableMapOf<String, MutableSet<String>>()

    private var keyAlias = DEFAULT_KEY_ALIAS
    private var keyCommonName = DEFAULT_KEY_COMMON_NAME
    private var keyOrganisation = DEFAULT_KEY_ORGANISATION
    private var keyCountry = DEFAULT_KEY_COUNTRY
    private var keyYears = VALIDITY_YEARS.indexOf(DEFAULT_KEY_YEARS)
    private var keyBits = KEY_SIZES.indexOf(4096)
    private var keyPasswordView: EditText? = null
    private var keyPasswordAgainView: EditText? = null
    private var buildPasswordView: EditText? = null
    private var buildSecret: String? = null

    private class Shell(
        val view: TerminalView,
        val folder: String,
        var named: String,
        var handle: Int = -1,
        var reader: Thread? = null,
    )

    private val shells = mutableListOf<Shell>()
    private var shellAt = 0
    private var shellControl = false

    private var logReading = false
    private var logProcess: java.lang.Process? = null
    private val logHeld = ArrayDeque<String>()
    private var logRefused = ""
    private var logView: TextView? = null
    private var logCount: TextView? = null
    private var logHollow: View? = null
    private var logSieve = ""
    private var logArgs = ""
    private var logPackage = ""
    private var logPackageShown = ""

    private var webAddress = ""
    private val webParts = WEB_PARTS_ON.toMutableList()

    private var feedbackKind = 0
    private var feedbackSubject = ""
    private var feedbackDetail = ""
    private val feedbackShots = mutableListOf<String>()
    private var helpFor = ""
    private var helpBand = 0
    private var guideSection = 0
    private var guideOpen = -1
    private var guideHeld: Pair<String, List<GuideSection>>? = null
    private var helpOpen = -1
    private var adbTab = 0
    private var osTab = 0

    private val talkHeld = mutableListOf<Pair<Boolean, String>>()
    private var talkDraft = ""
    private var talkAsking = false
    private var talkWindowUsed = 0L
    private var talkSpentIn = 0L
    private var talkSpentOut = 0L
    private var talkLeftRequests = -1L
    private var talkLeftTokens = -1L
    private var talkResetsAt = ""
    private var talkPartial = ""
    private var talkLive: TextView? = null
    private val talkStopping = java.util.concurrent.atomic.AtomicBoolean(false)
    private var agentRound = 0
    private var agentWrote = mutableListOf<String>()
    private var awaitingInstall: File? = null
    private val signingIn = java.util.concurrent.atomic.AtomicBoolean(false)
    private var cloneAddress = ""

    private var cloneToken = ""

    private var cloning = false

    private var reposQuery = ""

    private var reposFound: List<Forge.Repository> = emptyList()

    private var raised = false
    private val wentBefore = mutableListOf<String>()
    private var cameBefore: Screen? = null
    private var talkRoot: String? = null
    private val wentAfter = mutableListOf<String>()

    private var searchFor = ""
    private var searchHits: List<SearchHit> = emptyList()
    private var searchAt = -1
    private var editorColumn = 0
    private var editorPick = 0

    private var editorText = ""
    private var editor: CodeEditor? = null

    private var editorAt: Pair<String, String>? = null

    private var editorOnDisk = ""

    private var editorLine = 0

    private var editorSaveMark: View? = null

    private var editorSpot: TextView? = null

    private var editorUndoMark: View? = null

    private var editorRedoMark: View? = null

    private var editorSave: (() -> Unit)? = null

    private var editorUndo: (() -> Unit)? = null

    private var editorRedo: (() -> Unit)? = null

    private var editorMore: ((View) -> Unit)? = null

    private var editorDirty = false
    private var findHere = ""
    private var checkingCode = false
    private var offering = false
    private var offerRoot: String? = null
    private var offers: android.widget.PopupWindow? = null

    private var menu: android.widget.PopupWindow? = null

    override fun attachBaseContext(base: Context) {

        val tag = Preferences.language(base)
        super.attachBaseContext(if (tag.isEmpty()) base else localised(base, tag))
    }

    private fun localised(base: Context, tag: String): Context {
        val locale = Locale.forLanguageTag(tag)
        val configuration = Configuration(base.resources.configuration)
        configuration.setLocale(locale)
        configuration.setLayoutDirection(locale)
        return base.createConfigurationContext(configuration)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        OmniLog.event(LogLevel.INFO, "lifecycle", "Activity created.")

        Motion.readWhatTheDeviceAsksFor(this)
        speakTheDeviceLanguage()
        palette = Preferences.palette(this)
        aurora = AuroraView(this, palette)
        content = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(4), gap(3), gap(4), gap(8))
        }
        results = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            layoutTransition = LayoutTransition()
        }
        scroller = ScrollView(this).apply {
            isFillViewport = true
            clipToPadding = false
            overScrollMode = View.OVER_SCROLL_IF_CONTENT_SCROLLS
            addView(content, MATCH_PARENT, WRAP_CONTENT)
        }

        drawer = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setBackgroundColor(palette.surface)
        }
        shade = View(this).apply {
            setBackgroundColor(Ink.fade(Color.BLACK, 0.72f))
            visibility = View.GONE
            isClickable = true
            setOnClickListener { closeTheDrawer() }
        }

        scroller.addOnLayoutChangeListener { _, left, _, right, _, was, _, until, _ ->
            if (right - left != until - was) {
                fitTheColumn()
            }
        }

        crown = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(gap(2), gap(1), gap(2), gap(1))
            setBackgroundColor(palette.surface)
        }
        footer = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setBackgroundColor(palette.background)
        }
        raise = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.END
        }

        banner = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            visibility = View.GONE
            setPadding(gap(4), gap(3), gap(4), gap(3))
        }

        val roof = View(this)
        val shell = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            addView(roof, LinearLayout.LayoutParams(MATCH_PARENT, 0))
            addView(crown, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            addView(rule(), LinearLayout.LayoutParams(MATCH_PARENT, 1))
            addView(banner, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            addView(scroller, LinearLayout.LayoutParams(MATCH_PARENT, 0, 1f))
            addView(footer, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
        }

        ceremony = FrameLayout(this).apply {
            visibility = View.GONE
            isClickable = true
            setBackgroundColor(palette.background)
        }
        veil = BinaryVeil(this).apply { visibility = View.GONE }

        val stack = FrameLayout(this).apply {
            setBackgroundColor(palette.background)
            addView(aurora, FrameLayout.LayoutParams(MATCH_PARENT, MATCH_PARENT))
            addView(shell, FrameLayout.LayoutParams(MATCH_PARENT, MATCH_PARENT))
            addView(
                raise,
                FrameLayout.LayoutParams(
                    WRAP_CONTENT,
                    WRAP_CONTENT,
                    Gravity.END or Gravity.BOTTOM,
                ).apply { setMargins(gap(4), gap(4), gap(4), gap(6)) },
            )
            addView(shade, FrameLayout.LayoutParams(MATCH_PARENT, MATCH_PARENT))
            addView(
                drawer,
                FrameLayout.LayoutParams(gap(DRAWER_DP / 4), MATCH_PARENT, Gravity.START),
            )
            addView(ceremony, FrameLayout.LayoutParams(MATCH_PARENT, MATCH_PARENT))
            addView(veil, FrameLayout.LayoutParams(MATCH_PARENT, MATCH_PARENT))
        }
        setContentView(stack)
        fitAroundTheSystemBars(stack, roof)
        hideTheNavigationBar()

        drawer.translationX = -gap(DRAWER_DP / 4).toFloat()
        standing = examine()
        provisionSharedKey()
        answerBackTheWayThisScreenWants()
        fitTheColumn()
        render(false)
    }

    private fun hideTheNavigationBar() {
        val pale = Color.luminance(palette.background) > 0.5f
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.VANILLA_ICE_CREAM) {
                @Suppress("DEPRECATION")
                window.setDecorFitsSystemWindows(false)
            }
            window.insetsController?.let { controller ->
                controller.hide(WindowInsets.Type.navigationBars())
                controller.systemBarsBehavior =
                    WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
                controller.setSystemBarsAppearance(
                    if (pale) WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS else 0,
                    WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS,
                )
            }
            return
        }
        @Suppress("DEPRECATION")
        window.decorView.systemUiVisibility =
            View.SYSTEM_UI_FLAG_LAYOUT_STABLE or
                View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION or
                View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN or
                View.SYSTEM_UI_FLAG_HIDE_NAVIGATION or
                View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY or
                if (pale) View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR else 0
    }

    private fun fitAroundTheSystemBars(stack: View, roof: View) {
        stack.setOnApplyWindowInsetsListener { _, insets ->
            val top: Int
            val bottom: Int
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                val around = insets.getInsets(
                    WindowInsets.Type.statusBars() or WindowInsets.Type.displayCutout()
                )
                top = around.top
                bottom = insets.getInsets(WindowInsets.Type.navigationBars()).bottom
            } else {
                @Suppress("DEPRECATION")
                top = insets.systemWindowInsetTop
                @Suppress("DEPRECATION")
                bottom = insets.systemWindowInsetBottom
            }
            val keyboard = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                insets.getInsets(WindowInsets.Type.ime()).bottom
            } else {
                0
            }
            val above = roof.layoutParams as LinearLayout.LayoutParams
            if (above.height != top) {
                above.height = top
                roof.layoutParams = above
            }
            drawer.setPadding(0, top, 0, bottom)
            footer.setPadding(gap(2), gap(1), gap(2), gap(1) + bottom)
            footer.visibility = if (keyboard > 0) View.GONE else footer.visibility
            scroller.setPadding(0, 0, 0, keyboard)
            insets
        }
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus) {
            hideTheNavigationBar()
        }
    }

    private fun provisionSharedKey() {
        if (Preferences.signingKey(this).isNotEmpty()) {
            return
        }
        val folder = keysFolder().absolutePath
        Thread {
            val answer = runCatching { Builder.nativeDefaultKey(folder) }.getOrNull()
            val key = answer
                ?.let { runCatching { JSONObject(it) }.getOrNull() }
                ?.takeIf { it.optBoolean("ready", false) }
                ?.optJSONObject("key")
                ?: return@Thread
            runOnUiThread {
                if (isFinishing || isDestroyed) {
                    return@runOnUiThread
                }
                Preferences.setSigningKey(this, key.optString("path"))
                OmniLog.event(
                    LogLevel.INFO,
                    "keystore",
                    "Shared key ${key.optString("alias")} ready, ${key.optString("fingerprint")}",
                )
                render(false)
            }
        }.start()
    }

    override fun onResume() {
        super.onResume()
        showing = true
        Bell.hush(this, Bell.BUILD_NOTE)
        aurora.resumeDrawing()
        awaitingInstall?.let { waiting ->
            if (packageManager.canRequestPackageInstalls()) {
                awaitingInstall = null
                if (waiting.isFile) {
                    openOutside(waiting)
                }
            }
        }
        if (Sentry.refused(this)) {
            if (standing != "TAMPERED") {
                standing = "TAMPERED"
                render(false)
            }
            return
        }
        Thread {
            val found = runCatching { Sentry.check(this) }.getOrDefault("UNKNOWN")
            runOnUiThread {
                if (!isFinishing && !isDestroyed && found != standing) {
                    standing = found
                    render(false)
                }
            }
        }.start()
    }

    private fun keepTheDraft() {
        val open = editor ?: return
        val (root, path) = editorAt ?: return
        val held = open.text?.toString().orEmpty()
        if (held != editorOnDisk) {
            Drafts.write(this, root, path, held)
        }
    }

    override fun onPause() {
        showing = false
        hideOffers()
        keepTheDraft()
        stopTheLog()
        aurora.pauseDrawing()
        super.onPause()
        OmniLog.flushSession()
    }

    override fun onDestroy() {
        keepTheDraft()
        stopTheBuild()
        stopTheLog()
        stopListeningForPairing()
        closeTheShell()
        OmniLog.flushSession()
        super.onDestroy()
    }

    private fun examine(): String =
        if (Sentry.refused(this)) "TAMPERED" else Sentry.check(this)

    private fun go(next: Screen) {
        if (next == screen && !navigating) {
            return
        }
        if (next is Screen.Help && !inHelp(screen)) {
            cameBefore = screen
        }
        rememberWalk(next)
        raised = false
        screen = next
        closeTheDrawer()
        answerBackTheWayThisScreenWants()
        if (navigating) {
            return
        }
        navigating = true
        content.animate()
            .alpha(0f)
            .translationY(-gap(RISE_DP / 6).toFloat())
            .setDuration(Motion.of(Motion.LEAVE))
            .setInterpolator(AccelerateInterpolator())
            .start()
        veil.sweep(palette.accent) {
            navigating = false
            if (isFinishing || isDestroyed) {
                return@sweep
            }
            render(true)
            scroller.scrollTo(0, 0)
        }
    }

    private fun showCeremony(view: View, stop: (() -> Unit)? = null) {
        ceremony.removeAllViews()
        ceremony.addView(view, FrameLayout.LayoutParams(MATCH_PARENT, MATCH_PARENT))
        if (stop != null) {
            ceremony.addView(
                stopControl(stop),
                FrameLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT).apply {
                    gravity = Gravity.BOTTOM or Gravity.CENTER_HORIZONTAL
                    bottomMargin = gap(8)
                },
            )
        }
        ceremony.alpha = 0f
        ceremony.visibility = View.VISIBLE
        ceremony.animate().alpha(1f).setDuration(Motion.of(Motion.ENTER)).withEndAction {
            aurora.pauseDrawing()
        }.start()
    }

    private fun hideCeremony(then: () -> Unit) {
        ceremony.animate()
            .alpha(0f)
            .setDuration(Motion.of(Motion.LEAVE))
            .withEndAction {
                ceremony.removeAllViews()
                ceremony.visibility = View.GONE
                aurora.resumeDrawing()
                then()
            }
            .start()
    }

    private fun ceremonyIsUp(): Boolean = ceremony.visibility == View.VISIBLE

    private fun stopControl(stop: () -> Unit) = Button(this).apply {
        text = getString(R.string.omni_stage_stop)
        isAllCaps = false
        stateListAnimator = null
        setTextColor(stated(palette.foreground))
        typeface = Type.strong
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
        letterSpacing = Type.TRACKING
        minimumWidth = gap(30)
        minimumHeight = gap(TOUCH_UNITS)
        setPadding(gap(6), gap(3), gap(6), gap(3))
        background = reacting(palette.raised, gap(5).toFloat(), palette.error)
        setOnClickListener {
            isEnabled = false
            alpha = 0.5f
            stop()
        }
    }

    private fun render(animated: Boolean) {
        keepTheDraft()
        hideOffers()
        hideMenus()
        hideBanner()
        editor = null
        editorAt = null
        editorSaveMark = null
        editorSpot = null
        editorUndoMark = null
        editorRedoMark = null
        editorSave = null
        editorUndo = null
        editorRedo = null
        editorMore = null
        logView = null
        logCount = null
        logHollow = null
        leftBox = null
        rightBox = null
        if (screen !is Screen.Logcat) {
            stopTheLog()
            stopListeningForPairing()
        }
        if (screen !is Screen.Terminal) {
            closeTheShell()
        }
        content.removeAllViews()
        results.removeAllViews()
        fitTheColumn()

        when (val load = Builder.load()) {
            is Builder.LoadState.Failed -> {
                content.addView(notice(load.reason, palette.error))
                content.alpha = 1f
                content.translationY = 0f
                return
            }
            is Builder.LoadState.Loaded -> Unit
        }

        if (standing == "TAMPERED") {
            drawer.removeAllViews()
            footer.visibility = View.GONE
            raise.visibility = View.GONE
            crown.visibility = View.GONE
            content.addView(notice(getString(R.string.omni_integrity_refused_title), palette.error))
            content.addView(body(getString(R.string.omni_integrity_refused_body)))
            content.addView(quiet(getString(R.string.omni_integrity_checked)))
            content.alpha = 1f
            content.translationY = 0f
            return
        }

        crown.visibility = if (screen is Screen.Terminal) View.GONE else View.VISIBLE
        drawCrown()
        drawFooter()
        drawRaise()
        drawTheDrawer()

        when (val here = screen) {
            is Screen.Projects -> renderProjects()
            is Screen.NewProject -> renderNewProject()
            is Screen.WebToApk -> renderWebToApk()
            is Screen.Files -> renderFiles(here.root, here.folder)
            is Screen.Editor -> renderEditor(here.root, here.path)
            is Screen.Picture -> renderPicture(here.root, here.path)
            is Screen.Build -> renderBuild(here.root)
            is Screen.Repos -> renderRepos()
            is Screen.Trash -> renderTrash()
            is Screen.Settings -> renderSettings()
            is Screen.Keys -> renderKeys()
            is Screen.NewKey -> renderNewKey()
            is Screen.Logcat -> renderLogcat()
            is Screen.Terminal -> renderTerminal()
            is Screen.Assistant -> renderAssistant()
            is Screen.Theme -> renderTheme()
            is Screen.Languages -> renderLanguages()
            is Screen.Feedback -> renderFeedback()
            is Screen.Rate -> renderRate()
            is Screen.About -> renderAbout()
            is Screen.Legal -> renderLegal(here.terms)
            is Screen.Help -> renderHelp()
            is Screen.HelpAdb -> renderHelpAdb()
            is Screen.HelpTalk -> renderHelpTalk()
            is Screen.HelpDocs -> renderHelpDocs()
            is Screen.HelpFaq -> renderHelpFaq()
        }
        content.addView(results)

        if (animated) {
            content.alpha = 0f
            content.translationY = gap(RISE_DP / 6).toFloat()
            content.animate()
                .alpha(1f)
                .translationY(0f)
                .setDuration(Motion.of(Motion.ENTER))
                .setInterpolator(DecelerateInterpolator())
                .start()
        } else {
            content.alpha = 1f
            content.translationY = 0f
        }
    }

    @Suppress("DEPRECATION", "OVERRIDE_DEPRECATION")
    override fun onBackPressed() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            super.onBackPressed()
            return
        }
        stepBack()
    }

    private var backHere: Any? = null
    private var backRegistered = false

    private fun answerBackTheWayThisScreenWants() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            return
        }
        val wanted = screen !is Screen.Projects || drawerIsOpen()
        if (wanted == backRegistered) {
            return
        }
        val callback = backHere as? OnBackInvokedCallback
            ?: OnBackInvokedCallback { stepBack() }.also { backHere = it }
        if (wanted) {
            onBackInvokedDispatcher.registerOnBackInvokedCallback(
                OnBackInvokedDispatcher.PRIORITY_DEFAULT,
                callback,
            )
        } else {
            onBackInvokedDispatcher.unregisterOnBackInvokedCallback(callback)
        }
        backRegistered = wanted
    }

    private fun stepBack() {
        if (ceremonyIsUp()) {
            return
        }
        if (drawerIsOpen()) {
            closeTheDrawer()
            return
        }
        when (val here = screen) {
            is Screen.Projects -> finish()
            is Screen.NewProject -> go(Screen.Projects)
            is Screen.WebToApk -> go(Screen.Projects)
            is Screen.Help -> go(cameBefore ?: Screen.Projects)
            is Screen.HelpAdb -> go(Screen.Help)
            is Screen.HelpTalk -> go(Screen.Help)
            is Screen.HelpDocs -> go(Screen.Help)
            is Screen.HelpFaq -> go(Screen.Help)
            is Screen.Editor -> go(Screen.Files(here.root, here.path.substringBeforeLast('/', "")))
            is Screen.Picture -> go(Screen.Files(here.root, here.path.substringBeforeLast('/', "")))
            is Screen.Files -> {
                val beside = secondFolder.orEmpty()
                if (paneRight && beside.isNotEmpty()) {
                    stepPane(here.root, true, beside, beside.substringBeforeLast('/', ""))
                } else if (here.folder.isEmpty()) {
                    go(Screen.Projects)
                } else {
                    go(Screen.Files(here.root, here.folder.substringBeforeLast('/', "")))
                }
            }
            is Screen.Build -> go(Screen.Files(here.root, ""))
            is Screen.Keys -> go(Screen.Settings)
            is Screen.NewKey -> go(Screen.Keys)
            is Screen.Legal -> go(Screen.About)
            else -> go(Screen.Projects)
        }
    }

    private val bannerAway = Runnable { hideBanner() }

    private var bannerTurn = 0

    private fun flash(
        said: String,
        tint: Int,
        mark: Mark.Shape = Mark.Shape.INFO,
        holding: Long = 2600L,
    ) {
        banner.removeCallbacks(bannerAway)
        bannerTurn += 1
        banner.animate().cancel()
        banner.removeAllViews()
        banner.setBackgroundColor(Ink.fade(tint, 0.16f))
        banner.addView(
            View(this).apply { foreground = Mark(mark, tint) },
            LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(3) },
        )
        banner.addView(
            TextView(this).apply {
                text = said
                setTextColor(palette.foreground)
                typeface = Type.strong
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                setLineSpacing(gap(1).toFloat(), 1f)
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
        )
        banner.visibility = View.VISIBLE
        banner.alpha = 0f
        banner.translationY = -gap(4).toFloat()
        banner.animate().alpha(1f).translationY(0f).setDuration(LOOK_MILLIS * 3).start()
        if (holding > 0L) {
            banner.postDelayed(bannerAway, holding)
        }
    }

    private fun hideBanner() {
        banner.removeCallbacks(bannerAway)
        if (banner.visibility != View.VISIBLE) {
            return
        }
        bannerTurn += 1
        val mine = bannerTurn
        banner.animate()
            .alpha(0f)
            .translationY(-gap(4).toFloat())
            .setDuration(LOOK_MILLIS * 2)
            .withEndAction { if (mine == bannerTurn) banner.visibility = View.GONE }
            .start()
    }

    private fun markControl(shape: Mark.Shape, named: Int, onPress: () -> Unit) =
        View(this).apply {
            contentDescription = getString(named)
            isClickable = true
            isFocusable = true
            readAs(Button::class.java.name)
            background = reacting(Color.TRANSPARENT, gap(6).toFloat(), palette.accent)
            foreground = Mark(shape, palette.foreground)
            val squeeze = gap(3) / 2
            setPadding(squeeze, squeeze, squeeze, squeeze)
            setOnClickListener { onPress() }
        }

    private fun versionShown(): String = getString(R.string.omni_version_shown)

    private fun screenNamed(here: Screen): String = getString(
        when (here) {
            is Screen.WebToApk -> R.string.omni_web_title
            is Screen.Help -> R.string.omni_help_title
            is Screen.HelpDocs -> R.string.omni_help_terminal
            is Screen.HelpFaq -> R.string.omni_help_faq
            is Screen.Settings -> R.string.omni_tab_settings
            is Screen.Keys -> R.string.omni_keys_title
            is Screen.NewKey -> R.string.omni_keys_new
            is Screen.Repos -> R.string.omni_repos_title
            is Screen.Trash -> R.string.omni_trash_title
            is Screen.Build -> R.string.omni_tab_build
            is Screen.Legal ->
                if (here.terms) R.string.omni_about_terms else R.string.omni_about_policy
            else -> R.string.omni_brand_note
        }
    )

    private fun inHelp(here: Screen): Boolean =
        here is Screen.Help || here is Screen.HelpAdb || here is Screen.HelpTalk ||
            here is Screen.HelpDocs || here is Screen.HelpFaq

    private fun homeLike(here: Screen): Boolean =
        here is Screen.Projects || here is Screen.Files || here is Screen.NewProject

    private fun brandMark(drawn: Int, round: Int, inset: Int = round / 2): View =
        ImageView(this).apply {
            setImageResource(drawn)
            scaleType = ImageView.ScaleType.FIT_CENTER
            setPadding(inset, inset, inset, inset)
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = round.toFloat()
                setColor(Ink.fade(Ink.hot(palette.accent, 0.25f), 0.28f))
            }
        }

    private fun whereWeAre(named: String, said: String): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(2), 0, gap(2), 0)
            addView(
                TextView(context).apply {
                    text = named
                    setTextColor(palette.foreground)
                    typeface = Type.heading
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 17f)
                    maxLines = 1
                    ellipsize = android.text.TextUtils.TruncateAt.END
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
            addView(
                TextView(context).apply {
                    text = said
                    setTextColor(palette.marker)
                    typeface = Type.data
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                    maxLines = 1
                    ellipsize = android.text.TextUtils.TruncateAt.START
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
        }

    private fun brandBlock(named: String, full: Boolean, note: String = ""): View =
        LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER_VERTICAL
        if (full) {
            addView(
                brandMark(R.drawable.omni_brand, gap(2), gap(3) / 2),
                LinearLayout.LayoutParams(gap(9), gap(9)).apply {
                    marginStart = gap(2)
                    marginEnd = gap(3)
                },
            )
        }
        addView(
            LinearLayout(context).apply {
                orientation = LinearLayout.VERTICAL
                addView(
                    TextView(context).apply {
                        text = named
                        setTextColor(palette.foreground)
                        typeface = if (full) Type.heading else Type.body
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, if (full) 15f else 20f)
                        maxLines = 1
                    }
                )
                if (full) {
                    addView(
                        TextView(context).apply {
                            text = note.ifEmpty { getString(R.string.omni_brand_note) }
                            setTextColor(palette.muted)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                            maxLines = 1
                        }
                    )
                }
            }
        )
    }

    private fun guideCrown(mark: Mark.Shape, tint: Int, title: String): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            addView(
                filledMark(mark, Ink.fade(tint, 0.85f), gap(2)),
                LinearLayout.LayoutParams(gap(9), gap(9)).apply {
                    marginStart = gap(2)
                    marginEnd = gap(3)
                },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = title
                            setTextColor(palette.foreground)
                            typeface = Type.heading
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                            maxLines = 1
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = getString(R.string.omni_app_name)
                            setTextColor(palette.muted)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                            maxLines = 1
                        }
                    )
                }
            )
        }

    private fun crownTitle(title: String, note: String): View = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(gap(2), 0, gap(2), 0)
        addView(
            TextView(context).apply {
                text = title
                setTextColor(palette.foreground)
                typeface = Type.heading
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 19f)
                maxLines = 1
            }
        )
        addView(
            TextView(context).apply {
                text = note
                setTextColor(palette.muted)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                maxLines = 1
            }
        )
    }

    private fun crownAction(
        mark: Mark.Shape,
        named: Int,
        tint: Int,
        edged: Boolean = false,
        onPress: () -> Unit,
    ): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            minimumHeight = gap(10)
            setPadding(gap(2), gap(1), gap(2), gap(1))
            isClickable = true
            isFocusable = true
            contentDescription = getString(named)
            readAs(Button::class.java.name)
            background = touchable(
                GradientDrawable().apply {
                    shape = GradientDrawable.RECTANGLE
                    cornerRadius = gap(2).toFloat()
                    setColor(Color.TRANSPARENT)
                    if (edged) {
                        setStroke(gap(1) / 2, tint)
                    }
                },
                palette.accent,
            )
            setOnClickListener { onPress() }
            addView(
                View(context).apply { foreground = Mark(mark, tint) },
                LinearLayout.LayoutParams(gap(6), gap(6)),
            )
            if (wideEnough()) {
                addView(
                    TextView(context).apply {
                        text = getString(named)
                        setTextColor(palette.foreground)
                        typeface = Type.strong
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                        maxLines = 1
                        setPadding(gap(2), 0, 0, 0)
                    }
                )
            }
        }

    private fun drawCrown() {
        crown.removeAllViews()
        val here = screen
        val each = LinearLayout.LayoutParams(gap(10), gap(10))
        if (homeLike(here)) {
            crown.addView(
                markControl(Mark.Shape.MENU, R.string.omni_menu_title) { openTheDrawer() },
                each,
            )
            val root = openProject
            val listing = here is Screen.Files
            if (listing && root != null) {
                crown.addView(
                    whereWeAre(File(root).name, "/" + here.folder),
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            } else {
                crown.addView(
                    brandBlock(getString(R.string.omni_app_name), wideEnough()),
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            }
            if (root != null && listing) {
                crown.addView(
                    if (building) {
                        crownAction(
                            Mark.Shape.STOP,
                            R.string.omni_stage_stop_build,
                            palette.accent,
                            edged = true,
                        ) { go(Screen.Build(root)) }
                    } else {
                        crownAction(Mark.Shape.PLAY, R.string.omni_tab_build, palette.foreground) {
                            go(Screen.Build(root))
                        }
                    }
                )
            }
            crown.addView(
                crownAction(Mark.Shape.LOG, R.string.omni_tab_shell, palette.foreground) {
                    go(Screen.Terminal)
                }
            )
            if (here is Screen.NewProject) {
                crown.addView(
                    markControl(Mark.Shape.HELP, R.string.omni_help_title) { go(Screen.Help) },
                    each,
                )
            } else {
                val more = markControl(Mark.Shape.MORE, R.string.omni_action_more) { }
                more.setOnClickListener { homeMenu(more) }
                crown.addView(more, each)
            }
        } else {
            crown.addView(
                markControl(Mark.Shape.CHEVRON_LEFT, R.string.omni_action_back) { stepBack() },
                each,
            )
            if (here is Screen.Terminal) {
                crown.addView(
                    crownTitle(
                        getString(R.string.omni_tab_shell),
                        getString(R.string.omni_brand_note),
                    ),
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                crown.addView(
                    markControl(Mark.Shape.HELP, R.string.omni_help_title) { go(Screen.Help) },
                    each,
                )
                return
            }
            if (here is Screen.Logcat) {
                crown.addView(
                    crownTitle(
                        getString(R.string.omni_menu_logcat),
                        getString(R.string.omni_log_note),
                    ),
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                crown.addView(
                    markControl(Mark.Shape.HELP, R.string.omni_help_title) { go(Screen.Help) },
                    each,
                )
                return
            }
            if (here is Screen.Assistant) {
                crown.addView(
                    crownTitle(
                        getString(R.string.omni_menu_assistant),
                        getString(R.string.omni_app_name),
                    ),
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                crown.addView(
                    markControl(Mark.Shape.CLOCK, R.string.omni_talk_export) {
                        copyOut(
                            talkHeld.joinToString("\n\n") { it.second },
                            getString(R.string.omni_menu_assistant),
                        )
                    },
                    each,
                )
                crown.addView(
                    markControl(Mark.Shape.TRASH, R.string.omni_talk_clear) {
                        talkHeld.clear()
                        render(false)
                    },
                    each,
                )
                val talkMore = markControl(Mark.Shape.MORE, R.string.omni_action_more) { }
                talkMore.setOnClickListener { talkMenu(talkMore) }
                crown.addView(talkMore, each)
                return
            }
            if (here is Screen.Editor) {
                val spot = TextView(this).apply {
                    setTextColor(palette.marker)
                    typeface = Type.data
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                    maxLines = 1
                    ellipsize = android.text.TextUtils.TruncateAt.START
                }
                editorSpot = spot
                crown.addView(
                    editorHead(here.root, spot),
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                val small = LinearLayout.LayoutParams(gap(9), gap(9))
                val stepBackward = markControl(Mark.Shape.UNDO, R.string.omni_editor_undo) {
                    editorUndo?.invoke()
                }
                val stepForward = markControl(Mark.Shape.REDO, R.string.omni_editor_redo) {
                    editorRedo?.invoke()
                }
                editorUndoMark = stepBackward
                editorRedoMark = stepForward
                crown.addView(stepBackward, small)
                crown.addView(stepForward, small)
                val saving = markControl(Mark.Shape.SAVE, R.string.omni_action_save) {
                    editorSave?.invoke()
                }
                editorSaveMark = saving
                crown.addView(saving, small)
                val opening = markControl(Mark.Shape.MORE, R.string.omni_action_more) { }
                opening.setOnClickListener { editorMore?.invoke(opening) }
                crown.addView(opening, small)
                paintTheSave()
                paintTheSteps()
                return
            }
            if (here is Screen.HelpAdb || here is Screen.HelpTalk) {
                val guided = here is Screen.HelpAdb
                crown.addView(
                    guideCrown(
                        if (guided) Mark.Shape.LOG else Mark.Shape.BRAIN,
                        if (guided) palette.accent else 0xFFB06BFF.toInt(),
                        getString(
                            if (guided) R.string.omni_help_adb
                            else R.string.omni_help_talk_title
                        ),
                    ),
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                crown.addView(
                    markControl(Mark.Shape.FIND, R.string.omni_help_search) {
                        go(Screen.HelpFaq)
                    },
                    each,
                )
                val guideMore = markControl(Mark.Shape.MORE, R.string.omni_action_more) { }
                guideMore.setOnClickListener { homeMenu(guideMore) }
                crown.addView(guideMore, each)
                return
            }
            crown.addView(
                brandBlock(
                    getString(R.string.omni_app_shown, versionShown()),
                    true,
                    screenNamed(here),
                ),
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            if (here is Screen.WebToApk) {
                crown.addView(
                    markControl(Mark.Shape.HELP, R.string.omni_help_title) { go(Screen.Help) },
                    each,
                )
            }
        }
    }

    private fun paintTheSave() {
        editorSaveMark?.foreground = Mark(
            Mark.Shape.SAVE,
            if (editorDirty) palette.warning else palette.foreground,
        )
    }

    private fun paintTheSteps() {
        val held = editor
        val backward = held?.canUndo() ?: false
        val forward = held?.canRedo() ?: false
        editorUndoMark?.let {
            it.isEnabled = backward
            it.foreground = Mark(
                Mark.Shape.UNDO,
                if (backward) palette.foreground else palette.divider,
            )
        }
        editorRedoMark?.let {
            it.isEnabled = forward
            it.foreground = Mark(
                Mark.Shape.REDO,
                if (forward) palette.foreground else palette.divider,
            )
        }
    }

    private fun editorHead(root: String, spot: TextView): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(2), 0, gap(2), 0)
            addView(
                TextView(context).apply {
                    text = File(root).name
                    setTextColor(palette.foreground)
                    typeface = Type.heading
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 17f)
                    maxLines = 1
                    ellipsize = android.text.TextUtils.TruncateAt.END
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
            addView(spot, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
        }

    private fun footControl(mark: Mark.Shape, named: Int, on: Boolean, onPress: () -> Unit): View =
        View(this).apply {
            contentDescription = getString(named)
            isClickable = on
            isFocusable = on
            readAs(Button::class.java.name)
            background = reacting(Color.TRANSPARENT, gap(5).toFloat(), palette.accent)
            foreground = Mark(mark, if (on) palette.foreground else palette.divider)
            val squeeze = gap(5) / 2
            setPadding(squeeze, squeeze, squeeze, squeeze)
            setOnClickListener { if (on) onPress() }
        }

    private fun paneFolder(): String {
        val here = screen as? Screen.Files ?: return ""
        return if (paneRight) secondFolder ?: here.folder else here.folder
    }

    private fun walkPane(to: String) {
        val here = screen as? Screen.Files ?: return
        if (paneRight) {
            besideBefore.add(secondFolder ?: here.folder)
            besideAfter.clear()
            secondFolder = to
            render(true)
        } else {
            go(Screen.Files(here.root, to))
        }
    }

    private fun stepPane(by: Int) {
        if (!paneRight) {
            stepWalked(by)
            return
        }
        val here = screen as? Screen.Files ?: return
        val now = secondFolder ?: here.folder
        if (by < 0) {
            val back = besideBefore.removeLastOrNull() ?: return
            besideAfter.add(now)
            secondFolder = back
        } else {
            val on = besideAfter.removeLastOrNull() ?: return
            besideBefore.add(now)
            secondFolder = on
        }
        render(true)
    }

    private fun askToCreate(root: String, folder: String) {
        val editor = styledField("", getString(R.string.omni_files_needs_name))
        fun named(): String = editor.text.toString().trim()
        dialogShell(
            getString(R.string.omni_action_create),
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(0, gap(1), 0, gap(2))
                addView(
                    TextView(context).apply {
                        text = if (folder.isEmpty()) root else "$root/$folder"
                        setTextColor(palette.muted)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                        maxLines = 1
                        ellipsize = android.text.TextUtils.TruncateAt.START
                        setPadding(0, 0, 0, gap(2))
                    }
                )
                addView(editor, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            },
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() },
                getString(R.string.omni_files_file) to { it: AlertDialog ->
                    val chosen = named()
                    if (chosen.isEmpty()) {
                        results.removeAllViews()
                        results.addView(
                            notice(getString(R.string.omni_files_needs_name), palette.warning)
                        )
                    } else {
                        it.dismiss()
                        act(
                            Builder.nativeWriteFile(root, joined(folder, chosen), ""),
                            "saved",
                        )
                    }
                },
                getString(R.string.omni_files_folder) to { it: AlertDialog ->
                    val chosen = named()
                    if (chosen.isEmpty()) {
                        results.removeAllViews()
                        results.addView(
                            notice(getString(R.string.omni_files_needs_name), palette.warning)
                        )
                    } else {
                        it.dismiss()
                        act(Builder.nativeNewFolder(root, joined(folder, chosen)), "made")
                    }
                },
            ),
        )
        editor.requestFocus()
    }

    private fun drawFooter() {
        footer.removeAllViews()
        val here = screen
        if (here !is Screen.Files) {
            footer.visibility = View.GONE
            return
        }
        footer.visibility = View.VISIBLE
        val holds = LinearLayout.LayoutParams(0, gap(10), 1f)
        val backable = if (paneRight) besideBefore.isNotEmpty() else wentBefore.isNotEmpty()
        val onward = if (paneRight) besideAfter.isNotEmpty() else wentAfter.isNotEmpty()
        footer.addView(
            footControl(Mark.Shape.CHEVRON_LEFT, R.string.omni_action_back, backable) { stepPane(-1) },
            holds,
        )
        footer.addView(
            footControl(Mark.Shape.CHEVRON_RIGHT, R.string.omni_action_forward, onward) { stepPane(1) },
            LinearLayout.LayoutParams(0, gap(10), 1f),
        )
        footer.addView(
            footControl(Mark.Shape.CHEVRON_UP, R.string.omni_files_up, paneFolder().isNotEmpty()) {
                walkPane(paneFolder().substringBeforeLast('/', ""))
            },
            LinearLayout.LayoutParams(0, gap(10), 1f),
        )
        footer.addView(
            footControl(Mark.Shape.PAGE_ADD, R.string.omni_action_create, true) {
                askToCreate(here.root, paneFolder())
            },
            LinearLayout.LayoutParams(0, gap(10), 1f),
        )
        footer.addView(
            footControl(Mark.Shape.CLIP, R.string.omni_action_paste, canPaste(here.root, paneFolder())) {
                pasteInto(here.root, paneFolder())
            },
            LinearLayout.LayoutParams(0, gap(10), 1f),
        )
        footer.addView(
            footControl(Mark.Shape.SWAP, R.string.omni_files_mirror, true) {
                val mine = paneFolder()
                if (paneRight) {
                    go(Screen.Files(here.root, mine))
                } else {
                    besideBefore.add(secondFolder ?: here.folder)
                    besideAfter.clear()
                    secondFolder = mine
                    render(true)
                }
            },
            LinearLayout.LayoutParams(0, gap(10), 1f),
        )
    }

    private fun raiseAction(named: Int, mark: Mark.Shape, onPress: () -> Unit): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            layoutParams = LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT).apply {
                gravity = Gravity.END
                bottomMargin = gap(3)
            }
            addView(
                TextView(context).apply {
                    text = getString(named)
                    setTextColor(palette.foreground)
                    typeface = Type.strong
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                    setPadding(gap(5), gap(2), gap(5), gap(2))
                    background = pill(LABEL_PILL, gap(6).toFloat())
                }
            )
            addView(
                View(context).apply {
                    background = GradientDrawable().apply {
                        this.shape = GradientDrawable.OVAL
                        setColor(Ink.hot(palette.raised, 0.12f))
                    }
                    foreground = Mark(mark, palette.foreground)
                    isClickable = true
                    isFocusable = true
                    contentDescription = getString(named)
                    readAs(Button::class.java.name)
                    setOnClickListener { onPress() }
                },
                LinearLayout.LayoutParams(gap(12), gap(12)).apply { marginStart = gap(3) },
            )
            setOnClickListener { onPress() }
            isClickable = true
        }

    private fun drawRaise() {
        raise.removeAllViews()
        if (screen !is Screen.Projects) {
            raise.visibility = View.GONE
            return
        }
        raise.visibility = View.VISIBLE
        if (raised) {
            raise.addView(
                raiseAction(R.string.omni_projects_new, Mark.Shape.PAGE_ADD) {
                    raised = false
                    showKinds()
                }
            )
            raise.addView(
                raiseAction(R.string.omni_raise_open, Mark.Shape.FOLDER_OPEN) {
                    raised = false
                    drawRaise()
                    askForAProjectArchive()
                }
            )
            raise.addView(
                raiseAction(R.string.omni_clone_title, Mark.Shape.BRANCH) {
                    raised = false
                    drawRaise()
                    askToClone()
                }
            )
        }
        raise.addView(
            View(this).apply {
                background = GradientDrawable().apply {
                    shape = GradientDrawable.OVAL
                    setColor(Ink.hot(palette.raised, 0.12f))
                }
                foreground = Mark(
                    if (raised) Mark.Shape.CLOSE else Mark.Shape.ADD,
                    palette.foreground,
                )
                val squeeze = gap(3)
                setPadding(squeeze, squeeze, squeeze, squeeze)
                isClickable = true
                isFocusable = true
                contentDescription = getString(R.string.omni_raise_more)
                readAs(Button::class.java.name)
                setOnClickListener {
                    raised = !raised
                    drawRaise()
                }
            },
            LinearLayout.LayoutParams(gap(27) / 2, gap(27) / 2).apply {
                gravity = Gravity.END
            },
        )
    }

    private fun askToSearch() {
        val root = openProject
        results.removeAllViews()
        if (root == null) {
            results.addView(notice(getString(R.string.omni_projects_none), palette.warning))
            return
        }
        askToFind(getString(R.string.omni_find_project), searchFor) { word ->
            if (word.isBlank()) {
                return@askToFind
            }
            working({ Builder.nativeSearchProject(root, word, false, false) })
            finished@{ answer ->
                val document = runCatching { JSONObject(answer) }.getOrNull() ?: return@finished
                if (!document.optBoolean("searched", false)) {
                    showRefusal(Refusal.parse(document), results)
                    return@finished
                }
                searchFor = word
                showFound(root, document.optJSONObject("result"))
            }
        }
    }

    private fun homeMenu(under: View) {
        val listing = screen is Screen.Files
        val rows = mutableListOf<MenuChoice>()
        if (listing) {
            rows += MenuChoice(Mark.Shape.FIND, getString(R.string.omni_find_project)) {
                askToSearch()
            }
        }
        rows += MenuChoice(
            Mark.Shape.LOG,
            getString(R.string.omni_menu_logcat),
            parted = true,
        ) { go(Screen.Logcat) }
        rows += MenuChoice(Mark.Shape.SPARK, getString(R.string.omni_menu_assistant)) {
            go(Screen.Assistant)
        }
        if (listing) {
            rows += MenuChoice(
                Mark.Shape.SORT,
                getString(R.string.omni_files_sorting),
                onward = true,
            ) { sortMenu(under) }
        }
        rows += MenuChoice(
            Mark.Shape.HELP,
            getString(R.string.omni_menu_help),
            parted = true,
        ) { go(Screen.Help) }
        anchoredMenu(under, rows)
    }

    private fun sortMenu(under: View) {
        anchoredMenu(
            under,
            Order.entries.map { order ->
                MenuChoice(order.mark, getString(order.label), picked = order == filesOrder) {
                    filesOrder = order
                    render(false)
                }
            },
        )
    }

    private class MenuChoice(
        val mark: Mark.Shape,
        val named: String,
        val onward: Boolean = false,
        val picked: Boolean? = null,
        val tint: Int? = null,
        val parted: Boolean = false,
        val enabled: Boolean = true,
        val onPress: () -> Unit,
    )

    private data class FileClip(val root: String, val path: String, val cut: Boolean)

    private fun sheetCell(row: MenuChoice, shown: () -> AlertDialog?): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            minimumHeight = gap(13)
            setPadding(gap(3), gap(2), gap(3), gap(2))
            val live = row.picked != false
            isClickable = live
            isFocusable = live
            alpha = if (live) 1f else 0.38f
            contentDescription = row.named
            readAs(Button::class.java.name)
            if (live) {
                pressable()
                setOnClickListener {
                    shown()?.dismiss()
                    row.onPress()
                }
            }
            addView(
                View(context).apply {
                    foreground = Mark(row.mark, row.tint ?: palette.foreground)
                    importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                },
                LinearLayout.LayoutParams(gap(7), gap(7)).apply { marginEnd = gap(3) },
            )
            addView(
                TextView(context).apply {
                    text = row.named
                    setTextColor(row.tint ?: palette.foreground)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                    maxLines = 1
                    ellipsize = android.text.TextUtils.TruncateAt.END
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
        }

    private fun actionSheet(titled: String, rows: List<MenuChoice>) {
        var shown: AlertDialog? = null
        val grid = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(0, gap(1), 0, gap(2))
        }
        var at = 0
        while (at < rows.size) {
            val left = rows[at]
            val right = rows.getOrNull(at + 1)
            grid.addView(
                LinearLayout(this).apply {
                    orientation = LinearLayout.HORIZONTAL
                    addView(
                        sheetCell(left) { shown },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                    if (right != null) {
                        addView(
                            sheetCell(right) { shown },
                            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                        )
                    } else {
                        addView(View(context), LinearLayout.LayoutParams(0, 1, 1f))
                    }
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
            at += 2
        }
        shown = dialogShell(titled, grid, emptyList())
    }

    private fun anchoredMenu(under: View, rows: List<MenuChoice>) {
        val list = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(1), gap(2), gap(1), gap(2))
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(3).toFloat()
                setColor(palette.raised)
                setStroke(gap(1) / 2, palette.divider)
            }
        }
        hideMenus()
        val ruler = TextView(this).apply {
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
        }
        val widest = rows.maxOfOrNull { ruler.paint.measureText(it.named) } ?: 0f
        val trailing = if (rows.any { it.onward || it.picked != null }) gap(9) else 0
        val wide = (widest.toInt() + gap(3) + gap(6) + gap(3) + trailing + gap(4) + gap(2))
        val window = android.widget.PopupWindow(list, wide, WRAP_CONTENT, true).apply {
            isOutsideTouchable = true
            elevation = gap(3).toFloat()
        }
        menu = window
        rows.forEach { row ->
            if (row.parted) {
                list.addView(
                    View(this).apply {
                        setBackgroundColor(Ink.hot(palette.divider, 0.25f))
                    },
                    LinearLayout.LayoutParams(MATCH_PARENT, 1).apply {
                        topMargin = gap(1)
                        bottomMargin = gap(1)
                    },
                )
            }
            list.addView(
                LinearLayout(this).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    minimumHeight = gap(10)
                    minimumWidth = gap(44)
                    setPadding(gap(3), gap(1), gap(4), gap(1))
                    isClickable = row.enabled
                    isFocusable = row.enabled
                    isEnabled = row.enabled
                    alpha = if (row.enabled) 1f else 0.38f
                    readAs(Button::class.java.name)
                    background = reacting(Color.TRANSPARENT, gap(2).toFloat(), palette.accent)
                    setOnClickListener {
                        if (row.enabled) {
                            hideMenus()
                            row.onPress()
                        }
                    }
                    addView(
                        View(context).apply {
                            foreground = Mark(row.mark, row.tint ?: palette.foreground)
                            importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                        },
                        LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(3) },
                    )
                    addView(
                        TextView(context).apply {
                            text = row.named
                            setTextColor(palette.foreground)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                            maxLines = 1
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                    if (row.onward) {
                        addView(
                            View(context).apply {
                                foreground = Mark(Mark.Shape.CHEVRON_RIGHT, palette.muted)
                                importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                            },
                            LinearLayout.LayoutParams(gap(6), gap(6)).apply {
                                marginStart = gap(3)
                            },
                        )
                    }
                    val picked = row.picked
                    if (picked != null) {
                        addView(
                            View(context).apply {
                                background = GradientDrawable().apply {
                                    shape = GradientDrawable.OVAL
                                    setColor(Color.TRANSPARENT)
                                    setStroke(
                                        gap(1) / 2,
                                        if (picked) palette.accent else palette.muted,
                                    )
                                }
                                foreground = if (picked) {
                                    Mark(Mark.Shape.TICK, palette.accent)
                                } else {
                                    null
                                }
                            },
                            LinearLayout.LayoutParams(gap(6), gap(6)).apply {
                                marginStart = gap(3)
                            },
                        )
                    }
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
        }
        runCatching { window.showAsDropDown(under, under.width - wide, 0) }
            .onFailure { menu = null }
    }

    private fun hideMenus() {
        menu?.let { runCatching { it.dismiss() } }
        menu = null
    }

    private fun rememberWalk(next: Screen) {
        val here = screen
        if (here is Screen.Files && next is Screen.Files && here.folder != next.folder) {
            wentBefore.add(here.folder)
            wentAfter.clear()
        }
    }

    private fun stepWalked(by: Int) {
        val here = screen as? Screen.Files ?: return
        if (by < 0) {
            val back = wentBefore.removeLastOrNull() ?: return
            wentAfter.add(here.folder)
            screen = Screen.Files(here.root, back)
        } else {
            val on = wentAfter.removeLastOrNull() ?: return
            wentBefore.add(here.folder)
            screen = Screen.Files(here.root, on)
        }
        render(true)
    }

    private fun wideEnough(): Boolean =
        resources.configuration.screenWidthDp >= WIDE_DP

    private fun fitTheColumn() {
        val room = scroller.width.takeIf { it > 0 } ?: resources.displayMetrics.widthPixels
        val most = (COLUMN_DP * resources.displayMetrics.density).toInt()
        val bleeds = screen is Screen.Files || screen is Screen.Terminal
        val side = if (bleeds) {
            if (screen is Screen.Terminal) 0 else gap(1)
        } else {
            ((room - most) / 2).coerceAtLeast(0) + gap(7) / 2
        }
        val top = if (bleeds) gap(1) else gap(2)
        if (content.paddingLeft != side || content.paddingTop != top) {
            content.setPadding(side, top, side, if (bleeds) 0 else gap(8))
        }
    }

    private fun drawerIsOpen(): Boolean = shade.visibility == View.VISIBLE

    private fun openTheDrawer() {
        if (drawerIsOpen()) {
            return
        }
        shade.alpha = 0f
        shade.visibility = View.VISIBLE
        shade.animate().alpha(1f).setDuration(Motion.of(Motion.ENTER)).start()
        drawer.animate()
            .translationX(0f)
            .setDuration(Motion.of(Motion.ENTER))
            .setInterpolator(DecelerateInterpolator())
            .start()
        answerBackTheWayThisScreenWants()
    }

    private fun closeTheDrawer() {
        if (!drawerIsOpen()) {
            return
        }
        shade.animate()
            .alpha(0f)
            .setDuration(Motion.of(Motion.LEAVE))
            .withEndAction { shade.visibility = View.GONE }
            .start()
        drawer.animate()
            .translationX(-drawer.width.toFloat())
            .setDuration(Motion.of(Motion.LEAVE))
            .setInterpolator(AccelerateInterpolator())
            .start()
        answerBackTheWayThisScreenWants()
    }

    private fun leaveFor(next: Screen) {
        closeTheDrawer()
        drawer.postDelayed({ go(next) }, Motion.of(Motion.LEAVE))
    }

    private fun drawerHeader(): View = LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER_VERTICAL
        setPadding(gap(4), gap(4), gap(4), gap(4))
        addView(
            brandMark(R.drawable.omni_brand, gap(2)),
            LinearLayout.LayoutParams(gap(9), gap(9)).apply { marginEnd = gap(4) },
        )
        addView(
            LinearLayout(context).apply {
                orientation = LinearLayout.VERTICAL
                addView(
                    TextView(context).apply {
                        text = getString(R.string.omni_app_shown, versionShown())
                        setTextColor(palette.foreground)
                        typeface = Type.heading
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                    }
                )
                addView(
                    TextView(context).apply {
                        text = getString(R.string.omni_brand_note)
                        setTextColor(palette.muted)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                    }
                )
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
        )
    }

    private fun languageShown(): String {
        val tag = Preferences.language(this)
        if (tag.isEmpty()) {
            return getString(R.string.omni_language_auto)
        }
        return Preferences.LANGUAGES.firstOrNull { it.first == tag }?.second ?: tag
    }

    private fun drawerSection(value: String): View = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        addView(
            rule(),
            LinearLayout.LayoutParams(MATCH_PARENT, 1).apply {
                marginStart = gap(2)
                marginEnd = gap(2)
                topMargin = gap(2)
            },
        )
        addView(
            TextView(context).apply {
                text = value.uppercase(Locale.getDefault())
                setTextColor(palette.muted)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                letterSpacing = 0.06f
                setPadding(gap(5), gap(3), gap(4), gap(2))
            }
        )
    }

    private fun drawerRow(
        shape: Mark.Shape,
        named: Int,
        active: Boolean,
        under: String = "",
        onPress: () -> Unit,
    ) =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            minimumHeight = gap(10)
            setPadding(gap(5), gap(2), gap(4), gap(2))
            isClickable = true
            isFocusable = true
            contentDescription = said(getString(named), under)
            readAs(Button::class.java.name)
            background = touchable(
                pill(if (active) palette.raised else Color.TRANSPARENT, gap(2).toFloat()),
                palette.accent,
            )
            setOnClickListener { onPress() }
            addView(
                View(context).apply { foreground = Mark(shape, palette.foreground) },
                LinearLayout.LayoutParams(gap(7), gap(7)).apply { marginEnd = gap(4) },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = getString(named)
                            setTextColor(palette.foreground)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                        }
                    )
                    if (under.isNotEmpty()) {
                        addView(
                            TextView(context).apply {
                                text = under
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                            }
                        )
                    }
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                marginStart = gap(2)
                marginEnd = gap(2)
            }
        }

    private fun drawTheDrawer() {
        drawer.removeAllViews()
        val here = screen
        drawer.addView(drawerHeader())
        val list = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        list.addView(
            drawerRow(Mark.Shape.HOME, R.string.omni_menu_home, here is Screen.Projects) {
                leaveFor(Screen.Projects)
            }
        )
        list.addView(drawerSection(getString(R.string.omni_menu_making)))
        list.addView(
            drawerRow(Mark.Shape.SPARK, R.string.omni_menu_assistant, here is Screen.Assistant) {
                leaveFor(Screen.Assistant)
            }
        )
        list.addView(
            drawerRow(Mark.Shape.LOG, R.string.omni_tab_shell, here is Screen.Terminal) {
                leaveFor(Screen.Terminal)
            }
        )
        list.addView(
            drawerRow(Mark.Shape.BRANCH, R.string.omni_repos_title, here is Screen.Repos) {
                leaveFor(Screen.Repos)
            }
        )
        list.addView(
            drawerRow(Mark.Shape.NOTE, R.string.omni_menu_logcat, here is Screen.Logcat) {
                leaveFor(Screen.Logcat)
            }
        )
        list.addView(drawerSection(getString(R.string.omni_menu_other)))
        list.addView(
            drawerRow(Mark.Shape.BRUSH, R.string.omni_settings_theme, here is Screen.Theme) {
                leaveFor(Screen.Theme)
            }
        )
        list.addView(
            drawerRow(
                Mark.Shape.GLOBE,
                R.string.omni_settings_language,
                here is Screen.Languages,
                getString(R.string.omni_language_default, languageShown()),
            ) { leaveFor(Screen.Languages) }
        )
        list.addView(
            drawerRow(Mark.Shape.TRASH, R.string.omni_trash_title, here is Screen.Trash) {
                leaveFor(Screen.Trash)
            }
        )
        list.addView(
            drawerRow(Mark.Shape.SPEECH, R.string.omni_feedback_title, here is Screen.Feedback) {
                leaveFor(Screen.Feedback)
            }
        )
        list.addView(
            drawerRow(Mark.Shape.HEART, R.string.omni_rate_title, here is Screen.Rate) {
                leaveFor(Screen.Rate)
            }
        )
        list.addView(
            drawerRow(Mark.Shape.INFO, R.string.omni_settings_about, here is Screen.About) {
                leaveFor(Screen.About)
            }
        )
        drawer.addView(
            ScrollView(this).apply {
                isFillViewport = false
                overScrollMode = View.OVER_SCROLL_IF_CONTENT_SCROLLS
                addView(list, MATCH_PARENT, WRAP_CONTENT)
            },
            LinearLayout.LayoutParams(MATCH_PARENT, 0, 1f),
        )
    }

    private fun projectsFolder() = File(getExternalFilesDir(null) ?: filesDir, "Projects")

    private fun builtFolder() = PackageProvider.folder(this).also { it.mkdirs() }

    private fun outputsFolder(root: String) =
        File(root, PackageProvider.OUTPUTS).also { it.mkdirs() }

    private fun freshOutput(root: String): File {
        val stamp = SimpleDateFormat(OUTPUT_STAMP, Locale.US).format(Date())
        val here = File(outputsFolder(root), stamp)
        here.mkdirs()
        return here
    }

    private fun trashFolder() = File(getExternalFilesDir(null) ?: filesDir, "Trash").absolutePath

    private fun openHere(root: String) {
        if (openProject != root) {
            secondFolder = ""
            paneRight = false
            wentBefore.clear()
            wentAfter.clear()
            besideBefore.clear()
            besideAfter.clear()
            pickedIn.clear()
        }
        openProject = root
        go(Screen.Files(root, ""))
    }

    private fun keysFolder() = File(filesDir, "Keys")

    private fun renderProjects() {
        val projects = runCatching {
            ProjectSummary.list(Builder.nativeListProjects(projectsFolder().absolutePath))
        }.getOrDefault(emptyList())

        if (projects.isEmpty()) {
            content.addView(quiet(getString(R.string.omni_projects_none)))
            return
        }
        projects.forEach { project ->
            content.addView(projectRow(project))
        }
    }

    private fun projectRow(project: ProjectSummary): View = LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER_VERTICAL
        minimumHeight = gap(TOUCH_UNITS)
        setPadding(gap(2), gap(2), gap(2), gap(2))
        isClickable = true
        isFocusable = true
        contentDescription = project.label.ifEmpty { project.name }
        readAs(Button::class.java.name)
        background = touchable(pill(Color.TRANSPARENT, gap(2).toFloat()), palette.accent)
        setOnClickListener { openHere(project.root) }
        addView(
            thumbnail(
                File(project.root, "$PROJECT_RES/$PROJECT_ICON").absolutePath,
                gap(11),
            ),
            LinearLayout.LayoutParams(gap(11), gap(11)).apply { marginEnd = gap(3) },
        )
        addView(
            LinearLayout(context).apply {
                orientation = LinearLayout.VERTICAL
                addView(
                    TextView(context).apply {
                        text = project.label.ifEmpty { project.name }
                        setTextColor(palette.foreground)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                        maxLines = 1
                    }
                )
                addView(
                    TextView(context).apply {
                        text = project.root
                        setTextColor(palette.muted)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                        maxLines = 1
                        ellipsize = android.text.TextUtils.TruncateAt.MIDDLE
                    }
                )
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
        )
        addView(
            projectMore(project.root),
            LinearLayout.LayoutParams(gap(9), gap(9)),
        )
    }

    private fun renderNewProject() {
        val head = card()
        head.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, gap(1), 0, gap(1))
                addView(
                    thumbnail(formImage.orEmpty(), gap(18)),
                    LinearLayout.LayoutParams(gap(18), gap(18)).apply { marginEnd = gap(4) },
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_projects_new)
                                setTextColor(palette.foreground)
                                typeface = Type.heading
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 24f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_form_note)
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                                setLineSpacing(gap(1).toFloat(), 1f)
                                setPadding(0, gap(1), 0, 0)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            }
        )
        content.addView(head)

        content.addView(field(getString(R.string.omni_form_label), formLabel) { formLabel = it })
        content.addView(
            field(getString(R.string.omni_form_package), formPackage) { formPackage = it }
        )
        val versions = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL }
        versions.addView(
            field(getString(R.string.omni_form_version_name), formVersionName) {
                formVersionName = it
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply { marginEnd = gap(2) },
        )
        versions.addView(
            field(getString(R.string.omni_form_version_code), formVersionCode) {
                formVersionCode = it
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply { marginStart = gap(2) },
        )
        content.addView(versions)

        content.addView(label(getString(R.string.omni_form_image)))
        content.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(gap(4), gap(4), gap(4), gap(4))
                background = GradientDrawable().apply {
                    this.shape = GradientDrawable.RECTANGLE
                    cornerRadius = gap(4).toFloat()
                    setColor(Color.TRANSPARENT)
                    setStroke(gap(1) / 2, palette.divider, gap(2).toFloat(), gap(2).toFloat())
                }
                addView(
                    LauncherPreview(context, palette).apply { show(formImage) },
                    LinearLayout.LayoutParams(gap(20), gap(20)).apply { marginEnd = gap(4) },
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            markedButton(
                                Mark.Shape.IMAGE,
                                getString(R.string.omni_form_image_choose),
                                palette.accent,
                                true,
                            ) { chooseImage(null) },
                            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
                        )
                        addView(
                            markedButton(
                                Mark.Shape.TRASH,
                                getString(R.string.omni_action_clear),
                                palette.foreground,
                                false,
                            ) {
                                formImage = null
                                render(false)
                            },
                            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                                topMargin = gap(2)
                            },
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_form_image_note)
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                                gravity = Gravity.CENTER
                                setPadding(0, gap(2), 0, 0)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            },
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply { topMargin = gap(1) },
        )

        val told = card()
        told.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                setPadding(0, gap(1), 0, gap(1))
                addView(
                    View(context).apply { foreground = Mark(Mark.Shape.INFO, palette.accent) },
                    LinearLayout.LayoutParams(gap(7), gap(7)).apply { marginEnd = gap(3) },
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_form_platforms)
                                setTextColor(palette.accent)
                                typeface = Type.strong
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(
                                    R.string.omni_form_platforms_note,
                                    releaseNamed(formMinSdk),
                                    releaseNamed(formTargetSdk),
                                )
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                                setLineSpacing(gap(1).toFloat(), 1f)
                                setPadding(0, gap(2), 0, 0)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_form_platforms_bits)
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                                setLineSpacing(gap(1).toFloat(), 1f)
                                setPadding(0, gap(3), 0, 0)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            }
        )
        content.addView(told)

        content.addView(primary(getString(R.string.omni_action_create)) { createProject() })
        content.addView(
            subtle(getString(R.string.omni_action_cancel), palette.muted) {
                go(Screen.Projects)
            }
        )
    }

    private fun releaseNamed(api: Int): String =
        ANDROID_RELEASES.firstOrNull { it.first == api }?.second ?: api.toString()

    private fun paneHead(root: String, folder: String, side: Boolean): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(gap(2), gap(2), gap(1), gap(2))
            addView(
                View(context).apply {
                    foreground = Mark(
                        if (folder.isEmpty()) Mark.Shape.HOME else Mark.Shape.FOLDER_OPEN,
                        palette.marker,
                    )
                },
                LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(2) },
            )
            addView(
                TextView(context).apply {
                    text = "/" + folder
                    setTextColor(palette.marker)
                    typeface = Type.data
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                    maxLines = 1
                    ellipsize = android.text.TextUtils.TruncateAt.START
                    isClickable = true
                    isFocusable = true
                    contentDescription = getString(R.string.omni_action_copy_path)
                    readAs(Button::class.java.name)
                    pressable()
                    setOnClickListener {
                        copyOut(File(File(root), folder).absolutePath, File(root).name)
                    }
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            addView(
                paneMore(root, folder, side),
                LinearLayout.LayoutParams(gap(8), gap(8)),
            )
        }

    private fun paneMore(root: String, folder: String, side: Boolean): View {
        val more = markControl(Mark.Shape.MORE, R.string.omni_action_more) { }
        more.setOnClickListener {
            paneRight = side
            markPanes()
            projectMenu(root, more)
        }
        return more
    }

    private fun projectMore(root: String): View {
        val more = markControl(Mark.Shape.MORE, R.string.omni_action_more) { }
        more.setOnClickListener { projectMenu(root, more) }
        return more
    }

    private fun treeMark(entry: FileEntry): View {
        val (shape, tint) = markFor(entry)
        if (entry.folder) {
            return View(this).apply {
                foreground = Mark(shape, palette.marker)
                importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
            }
        }
        return FrameLayout(this).apply {
            background = pill(TILE_UNTOUCHED, gap(1).toFloat())
            importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
            addView(
                View(context).apply { foreground = Mark(shape, tint) },
                FrameLayout.LayoutParams(gap(4), gap(4), Gravity.CENTER),
            )
        }
    }

    private fun renderFiles(root: String, folder: String) {
        val entries = runCatching {
            FileEntry.list(Builder.nativeProjectTree(root))
        }.getOrDefault(emptyList())

        if (secondFolder == null) {
            secondFolder = folder
        }
        val beside = secondFolder ?: folder
        val panes = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
        }
        val left = PaneBox(false).apply { addView(onePane(root, folder, beside, entries, false)) }
        val right = PaneBox(true).apply { addView(onePane(root, beside, folder, entries, true)) }
        panes.addView(
            left,
            LinearLayout.LayoutParams(0, MATCH_PARENT, 1f).apply { marginEnd = gap(1) / 2 },
        )
        panes.addView(
            right,
            LinearLayout.LayoutParams(0, MATCH_PARENT, 1f).apply { marginStart = gap(1) / 2 },
        )
        leftBox = left
        rightBox = right
        markPanes()
        content.addView(
            panes,
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply { weight = 1f },
        )
    }

    private fun onePane(
        root: String,
        folder: String,
        other: String,
        entries: List<FileEntry>,
        side: Boolean,
    ): View = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        addView(paneHead(root, folder, side))
        addView(rule(), LinearLayout.LayoutParams(MATCH_PARENT, 1))
        addView(filePane(root, folder, entries, other, side))
    }

    private fun paneSkin(chosen: Boolean): Drawable = GradientDrawable().apply {
        shape = GradientDrawable.RECTANGLE
        cornerRadius = gap(4).toFloat()
        setColor(if (chosen) palette.surface else palette.background)
        setStroke(if (chosen) gap(1) / 2 else 1, if (chosen) palette.accent else palette.divider)
    }

    private fun markPanes() {
        leftBox?.background = paneSkin(!paneRight)
        rightBox?.background = paneSkin(paneRight)
    }

    private fun choosePane(side: Boolean) {
        if (paneRight == side) return
        paneRight = side
        markPanes()
        drawFooter()
    }

    private inner class PaneBox(private val side: Boolean) : FrameLayout(this@BuilderActivity) {
        private var fromX = 0f
        private var fromY = 0f

        override fun onInterceptTouchEvent(event: MotionEvent): Boolean {
            when (event.actionMasked) {
                MotionEvent.ACTION_DOWN -> {
                    fromX = event.x
                    fromY = event.y
                    choosePane(side)
                }
                MotionEvent.ACTION_MOVE -> {
                    val slid = event.x - fromX
                    if (abs(slid) > gap(6) && abs(slid) > abs(event.y - fromY)) {
                        choosePane(slid < 0f)
                    }
                }
            }
            return false
        }
    }

    private fun renderLegal(terms: Boolean) {
        val sections = if (terms) {
            listOf(
                R.string.omni_terms_one_head to R.string.omni_terms_one_body,
                R.string.omni_terms_two_head to R.string.omni_terms_two_body,
                R.string.omni_terms_three_head to R.string.omni_terms_three_body,
                R.string.omni_terms_four_head to R.string.omni_terms_four_body,
                R.string.omni_terms_five_head to R.string.omni_terms_five_body,
                R.string.omni_terms_six_head to R.string.omni_terms_six_body,
            )
        } else {
            listOf(
                R.string.omni_policy_one_head to R.string.omni_policy_one_body,
                R.string.omni_policy_two_head to R.string.omni_policy_two_body,
                R.string.omni_policy_three_head to R.string.omni_policy_three_body,
                R.string.omni_policy_four_head to R.string.omni_policy_four_body,
                R.string.omni_policy_five_head to R.string.omni_policy_five_body,
                R.string.omni_policy_six_head to R.string.omni_policy_six_body,
            )
        }
        val lead = if (terms) R.string.omni_about_terms else R.string.omni_about_policy
        val under = if (terms) R.string.omni_about_terms_note else R.string.omni_about_policy_note

        content.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, gap(2), 0, gap(1))
                addView(
                    View(context).apply {
                        foreground = Mark(
                            if (terms) Mark.Shape.NOTE_FILL else Mark.Shape.SHIELD,
                            if (terms) 0xFFB06BFF.toInt() else palette.accent,
                        )
                    },
                    LinearLayout.LayoutParams(gap(11), gap(11)).apply { marginEnd = gap(4) },
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = getString(lead)
                                setTextColor(palette.foreground)
                                typeface = Type.heading
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 20f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(under)
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            }
        )

        sections.forEachIndexed { at, (head, said) ->
            val block = card()
            block.addView(
                LinearLayout(this).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    addView(
                        TextView(context).apply {
                            text = (at + 1).toString()
                            setTextColor(palette.marker)
                            typeface = Type.data
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                            gravity = Gravity.CENTER
                            background = pill(Ink.fade(palette.accent, 0.16f), gap(5) / 2f)
                        },
                        LinearLayout.LayoutParams(gap(7), gap(7)).apply { marginEnd = gap(3) },
                    )
                    addView(
                        TextView(context).apply {
                            text = getString(head)
                            setTextColor(palette.foreground)
                            typeface = Type.strong
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                }
            )
            block.addView(
                TextView(this).apply {
                    text = getString(said)
                    setTextColor(palette.muted)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                    setLineSpacing(gap(3) / 2f, 1f)
                    setPadding(0, gap(2), 0, gap(2))
                }
            )
            content.addView(block)
        }

        content.addView(
            quiet(
                getString(
                    R.string.omni_legal_stamp,
                    versionShown(),
                    Calendar.getInstance().get(Calendar.YEAR).toString(),
                )
            ).apply { setPadding(gap(1), gap(3), gap(1), gap(6)) }
        )
    }

    private fun inspectPackage(path: String) {
        working({ Builder.nativeInspectPackage(path) }) finished@{ answer ->
            val document = runCatching { JSONObject(answer) }.getOrNull() ?: return@finished
            if (!document.optBoolean("opened", false)) {
                showRefusal(Refusal.parse(document), results)
                return@finished
            }
            val found = document.optJSONObject("package") ?: return@finished

            val sound = found.optBoolean("sound", false)
            results.addView(
                notice(
                    getString(
                        if (sound) R.string.omni_open_sound else R.string.omni_open_unsound
                    ),
                    if (sound) palette.ok else palette.error,
                )
            )

            val facts = card()
            facts.addView(
                keyValue(
                    found.optString("package"),
                    found.optString("label"),
                    found.optString("versionName") + " (" +
                        found.optString("versionCode") + ")",
                    palette.foreground,
                )
            )
            facts.addView(
                keyValue(
                    getString(R.string.omni_open_platforms),
                    joined(found, "abis"),
                    "API " + found.optString("minSdk") + "–" + found.optString("targetSdk"),
                    palette.muted,
                )
            )
            facts.addView(
                keyValue(
                    getString(R.string.omni_open_code),
                    getString(
                        R.string.omni_open_code_detail,
                        found.optInt("classes"),
                        found.optInt("methods"),
                    ),
                    found.optInt("dexFiles").toString() + " dex",
                    palette.muted,
                )
            )
            facts.addView(
                keyValue(
                    getString(R.string.omni_result_signature),
                    joined(found, "signedBy"),
                    joined(found, "schemes").replace(", ", " + ").ifEmpty { "?" },
                    if (sound) palette.ok else palette.error,
                )
            )
            found.optJSONArray("fingerprints")?.let { array ->
                for (index in 0 until array.length()) {
                    facts.addView(quiet(array.getString(index)))
                }
            }
            facts.addView(
                keyValue(
                    getString(R.string.omni_open_entries),
                    "",
                    found.optInt("entries").toString(),
                    palette.muted,
                )
            )
            results.addView(facts)

            listOf(
                R.string.omni_open_permissions to "permissions",
                R.string.omni_open_activities to "activities",
            ).forEach { (label, key) ->
                val array = found.optJSONArray(key) ?: return@forEach
                if (array.length() == 0) return@forEach
                results.addView(heading(getString(label)))
                val listed = card()
                for (index in 0 until array.length()) {
                    listed.addView(quiet(array.getString(index)))
                }
                results.addView(listed)
            }

            found.optJSONArray("held")?.let { array ->
                if (array.length() == 0) return@let
                results.addView(heading(getString(R.string.omni_open_largest)))
                val listed = card()
                for (index in 0 until minOf(array.length(), 20)) {
                    val one = array.optJSONObject(index) ?: continue
                    listed.addView(
                        keyValue(
                            one.optString("name"),
                            "",
                            size(one.optLong("stored")),
                            palette.muted,
                        )
                    )
                }
                results.addView(listed)
            }

            found.optJSONArray("notes")?.let { array ->
                for (index in 0 until array.length()) {
                    results.addView(notice(array.getString(index), palette.warning))
                }
            }
        }
    }

    private fun joined(holder: JSONObject, key: String): String {
        val array = holder.optJSONArray(key) ?: return ""
        return (0 until array.length()).joinToString(", ") { array.getString(it) }
    }

    private class SearchHit(val path: String, val line: Int, val column: Int)

    private fun goToHit(root: String, one: SearchHit) {
        val here = screen
        if (here is Screen.Editor && here.root == root && here.path == one.path) {
            editor?.showAt(one.line, one.column, searchFor.length)
            return
        }
        editorLine = one.line
        editorColumn = one.column
        editorPick = searchFor.length
        go(Screen.Editor(root, one.path))
    }

    private fun stepToHit(root: String, by: Int) {
        if (searchHits.isEmpty()) {
            return
        }
        val many = searchHits.size
        searchAt = (((searchAt + by) % many) + many) % many
        goToHit(root, searchHits[searchAt])
    }

    private fun walkFound(root: String, count: Int): View {
        val counter = TextView(this).apply {
            setTextColor(palette.muted)
            typeface = Type.data
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
            gravity = Gravity.CENTER
        }
        fun say() {
            counter.text = (searchAt + 1).coerceAtLeast(0).toString() + " / " + count
        }
        say()
        fun stepper(mark: Mark.Shape, named: Int, by: Int): View =
            markControl(mark, named) {
                stepToHit(root, by)
                say()
            }
        return LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(0, gap(1), 0, gap(1))
            addView(
                TextView(context).apply {
                    text = searchFor
                    setTextColor(palette.accent)
                    typeface = Type.strong
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                    maxLines = 1
                    ellipsize = android.text.TextUtils.TruncateAt.END
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            addView(
                stepper(Mark.Shape.CHEVRON_UP, R.string.omni_editor_find_back, -1),
                LinearLayout.LayoutParams(gap(9), gap(9)),
            )
            addView(counter, LinearLayout.LayoutParams(gap(16), WRAP_CONTENT))
            addView(
                stepper(Mark.Shape.CHEVRON_DOWN, R.string.omni_editor_find_next, 1),
                LinearLayout.LayoutParams(gap(9), gap(9)),
            )
        }
    }

    private fun showFound(root: String, result: JSONObject?) {
        if (result == null) {
            return
        }
        val found = result.optJSONArray("found")
        val count = found?.length() ?: 0
        if (count == 0) {
            results.addView(notice(getString(R.string.omni_search_none), palette.muted))
        } else {
            results.addView(
                notice(
                    getString(
                        if (result.optBoolean("stoppedEarly", false)) {
                            R.string.omni_search_capped
                        } else {
                            R.string.omni_search_found
                        },
                        count,
                        result.optInt("filesSearched"),
                    ),
                    palette.ok,
                )
            )
        }

        val hits = mutableListOf<SearchHit>()
        for (index in 0 until count) {
            val one = found?.optJSONObject(index) ?: continue
            hits.add(
                SearchHit(one.optString("path"), one.optInt("line"), one.optInt("column"))
            )
        }
        searchHits = hits
        searchAt = -1
        if (hits.isNotEmpty()) {
            results.addView(walkFound(root, hits.size))
        }

        var lastPath = ""
        var card: LinearLayout? = null
        for (index in 0 until count) {
            val one = found?.optJSONObject(index) ?: continue
            val path = one.optString("path")
            if (path != lastPath) {
                lastPath = path
                card = card()
                card.addView(quiet(path))
                results.addView(card)
            }
            card?.addView(
                row(
                    one.optString("text").trim().take(120),
                    "",
                    one.optInt("line").toString() + ":" + one.optInt("column"),
                    palette.accent,
                ) {
                    searchAt = index
                    goToHit(root, hits[index])
                }
            )
        }

        val quiet = listOfNotNull(
            result.optInt("filesTooLarge").takeIf { it > 0 }
                ?.let { getString(R.string.omni_search_too_large, it) },
            result.optInt("filesNotText").takeIf { it > 0 }
                ?.let { getString(R.string.omni_search_not_text, it) },
        )
        quiet.forEach { results.addView(quiet(it)) }
    }

    private fun joined(folder: String, name: String): String =
        if (folder.isEmpty()) name else "$folder/$name"

    private fun picks(folder: String): MutableSet<String> =
        pickedIn.getOrPut(folder) { mutableSetOf() }

    private fun carryOver(root: String, from: String, into: String, moving: Boolean) {
        val chosen = picks(from).toList()
        if (chosen.isEmpty()) {
            return
        }
        var done = 0
        var refused: Refusal? = null
        for (path in chosen) {
            val name = path.substringAfterLast('/')
            val target = joined(into, name)
            if (target == path) {
                continue
            }
            val answer = runCatching {
                if (moving) {
                    Builder.nativeRenamePath(root, path, target)
                } else {
                    Builder.nativeCopyPath(root, path, target)
                }
            }.getOrNull()
            val read = answer?.let { runCatching { JSONObject(it) }.getOrNull() }
            if (read != null && read.optBoolean(if (moving) "moved" else "copied", false)) {
                done += 1
            } else if (refused == null && read != null) {
                refused = Refusal.parse(read)
            }
        }
        picks(from).clear()
        render(false)
        if (refused != null) {
            showRefusal(refused, results)
        } else {
            results.addView(
                notice(getString(R.string.omni_files_carried, done), palette.ok)
            )
        }
    }

    private fun transferRow(root: String, from: String, into: String): View {
        val many = picks(from).size
        val clash = picks(from).any { path ->
            val target = joined(into, path.substringAfterLast('/'))
            target != path && File(root, target).exists()
        }
        val row = FlowLayout(this, gap(2), gap(2)).apply {
            setPadding(0, gap(1), 0, gap(1))
        }
        row.addView(
            TextView(this).apply {
                text = getString(R.string.omni_files_selected, many)
                typeface = Type.label
                setTextColor(palette.muted)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                setPadding(gap(2), gap(2), gap(2), gap(2))
            }
        )
        row.addView(
            tool(getString(R.string.omni_files_copy_over), palette.accent) {
                carryOver(root, from, into, false)
            }.apply { isEnabled = !clash }
        )
        row.addView(
            tool(getString(R.string.omni_files_move_over), palette.warning) {
                carryOver(root, from, into, true)
            }.apply { isEnabled = !clash }
        )
        row.addView(
            tool(getString(R.string.omni_files_clear_picks), palette.muted) {
                picks(from).clear()
                render(false)
            }
        )
        return row
    }

    private fun besideUp(root: String, folder: String, side: Boolean): View =
        LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER_VERTICAL
        minimumHeight = gap(ROW_UNITS)
        setPadding(gap(3), gap(1), gap(3), gap(1))
        isClickable = true
        isFocusable = true
        contentDescription = ".."
        readAs(Button::class.java.name)
        pressable()
        setOnClickListener {
            if (folder.isNotEmpty()) {
                stepPane(root, side, folder, folder.substringBeforeLast('/', ""))
            }
        }
        addView(
            View(context).apply { foreground = Mark(Mark.Shape.CHEVRON_LEFT, palette.marker) },
            LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(3) },
        )
        addView(
            TextView(context).apply {
                text = ".."
                setTextColor(palette.foreground)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
            }
        )
    }

    private fun filePane(
        root: String,
        folder: String,
        entries: List<FileEntry>,
        other: String?,
        side: Boolean,
    ): View {
        val holder = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(0, gap(2), 0, gap(2))
        }
        if (folder.isNotEmpty()) {
            holder.addView(besideUp(root, folder, side))
        }
        val here = arranged(entries.filter { it.path.substringBeforeLast('/', "") == folder })
        if (here.isEmpty()) {
            holder.addView(quiet(getString(R.string.omni_files_empty)).apply {
                setPadding(gap(6), gap(4), gap(4), gap(4))
            })
        }
        here.forEach { entry ->
            holder.addView(fileRow(root, folder, entry, entries, side))
        }
        if (other != null && picks(folder).isNotEmpty()) {
            holder.addView(transferRow(root, folder, other))
        }
        return holder
    }

    private fun stepPane(root: String, side: Boolean, from: String, into: String) {
        if (side) {
            besideBefore.add(from)
            besideAfter.clear()
            secondFolder = into
            paneRight = true
            render(false)
        } else {
            paneRight = false
            go(Screen.Files(root, into))
        }
    }

    private fun markFor(entry: FileEntry): Pair<Mark.Shape, Int> {
        if (entry.folder) {
            return Mark.Shape.FOLDER to palette.accent
        }
        val name = entry.path.substringAfterLast('/').lowercase(Locale.US)
        return when {
            PACKAGE_SUFFIXES.any { name.endsWith(it) } ->
                Mark.Shape.PACKAGE to Ink.hot(palette.ok, 0.15f)
            ARCHIVE_SUFFIXES.any { name.endsWith(it) } ->
                Mark.Shape.ARCHIVE to palette.warning
            name.endsWith(".java") -> Mark.Shape.CUP to Ink.hot(palette.error, 0.25f)
            CODE_SUFFIXES.any { name.endsWith(it) } -> Mark.Shape.CODE to palette.accent
            name.endsWith(".xml") -> Mark.Shape.MARKUP to Ink.hot(palette.warning, -0.1f)
            PICTURE_SUFFIXES.any { name.endsWith(it) } ->
                Mark.Shape.IMAGE to Ink.hot(palette.error, 0.3f)
            SOUND_SUFFIXES.any { name.endsWith(it) } -> Mark.Shape.SOUND to palette.ok
            FILM_SUFFIXES.any { name.endsWith(it) } ->
                Mark.Shape.FILM to Ink.hot(palette.glowThird, 0.55f)
            FONT_SUFFIXES.any { name.endsWith(it) } ->
                Mark.Shape.FONT to Ink.hot(palette.accent, -0.2f)
            BINARY_SUFFIXES.any { name.endsWith(it) } -> Mark.Shape.BINARY to palette.error
            name.endsWith(".json") -> Mark.Shape.BRACES to Ink.hot(palette.glowFirst, 0.75f)
            TABLE_SUFFIXES.any { name.endsWith(it) } ->
                Mark.Shape.TABLE to Ink.hot(palette.ok, -0.25f)
            NOTE_SUFFIXES.any { name.endsWith(it) } -> Mark.Shape.NOTE to palette.muted
            else -> Mark.Shape.FILE to palette.muted
        }
    }

    private fun kindOrder(entry: FileEntry): Int {
        if (entry.folder) {
            return 0
        }
        val name = entry.path.substringAfterLast('/').lowercase(Locale.US)
        return when {
            PACKAGE_SUFFIXES.any { name.endsWith(it) } -> 1
            ARCHIVE_SUFFIXES.any { name.endsWith(it) } -> 2
            CODE_SUFFIXES.any { name.endsWith(it) } -> 3
            name.endsWith(".xml") -> 4
            PICTURE_SUFFIXES.any { name.endsWith(it) } -> 5
            SOUND_SUFFIXES.any { name.endsWith(it) } -> 6
            FILM_SUFFIXES.any { name.endsWith(it) } -> 7
            FONT_SUFFIXES.any { name.endsWith(it) } -> 8
            BINARY_SUFFIXES.any { name.endsWith(it) } -> 9
            TABLE_SUFFIXES.any { name.endsWith(it) } -> 10
            NOTE_SUFFIXES.any { name.endsWith(it) } -> 11
            else -> 12
        }
    }

    private fun arranged(here: List<FileEntry>): List<FileEntry> {
        val byName = compareBy<FileEntry> { it.path.lowercase(Locale.getDefault()) }
        val within: Comparator<FileEntry> = when (filesOrder) {
            Order.NAME -> byName
            Order.NAME_BACK -> byName.reversed()
            Order.SIZE -> compareByDescending<FileEntry> { it.bytes }.then(byName)
            Order.CHANGED -> compareByDescending<FileEntry> { it.changed }.then(byName)
            Order.KIND -> compareBy<FileEntry> { kindOrder(it) }.then(byName)
        }
        return here.sortedWith(compareBy<FileEntry> { !it.folder }.then(within))
    }

    private fun looksLikeAPicture(path: String): Boolean {
        val lower = path.lowercase(Locale.US)
        return PICTURE_SUFFIXES.any { lower.endsWith(it) }
    }

    private fun thumbnail(path: String, edge: Int): View {
        val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        runCatching { BitmapFactory.decodeFile(path, bounds) }
        if (bounds.outWidth > 0 && bounds.outHeight > 0) {
            val options = BitmapFactory.Options()
            var step = 1
            while (bounds.outWidth / (step * 2) >= edge && bounds.outHeight / (step * 2) >= edge) {
                step *= 2
            }
            options.inSampleSize = step
            val decoded = runCatching { BitmapFactory.decodeFile(path, options) }.getOrNull()
            if (decoded != null) {
                return ImageView(this).apply {
                    setImageBitmap(decoded)
                    scaleType = ImageView.ScaleType.FIT_CENTER
                    background = pill(palette.raised, gap(2).toFloat())
                    setPadding(gap(1))
                }
            }
        }
        return plainLauncher()
    }

    private fun plainLauncher(): View = ImageView(this).apply {
        setImageResource(R.drawable.omni_art_project)
        scaleType = ImageView.ScaleType.CENTER_CROP
        clipToOutline = true
        background = GradientDrawable().apply {
            shape = GradientDrawable.RECTANGLE
            cornerRadius = gap(3).toFloat()
            setColor(palette.raised)
        }
    }

    private fun fileRow(
        root: String,
        folder: String,
        entry: FileEntry,
        all: List<FileEntry>,
        side: Boolean = false,
    ): View {
        val name = entry.path.substringAfterLast('/')
        val absolute = File(root, entry.path).absolutePath
        val picture = !entry.folder && looksLikeAPicture(entry.path)

        return LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            minimumHeight = gap(ROW_UNITS)
            setPadding(gap(3), gap(1), gap(3), gap(1))
            isClickable = true
            isFocusable = true
            contentDescription = name
            readAs(Button::class.java.name)
            pressable(onHeld = { showActions(root, folder, entry) })
            setOnClickListener {
                if (entry.folder) {
                    stepPane(root, side, folder, entry.path)
                } else {
                    openEntry(root, entry)
                }
            }

            val chosen = picks(folder).contains(entry.path)
            addView(
                FrameLayout(context).apply {
                    addView(
                        if (picture) {
                            thumbnail(absolute, gap(6))
                        } else {
                            treeMark(entry)
                        },
                        FrameLayout.LayoutParams(gap(6), gap(6)),
                    )
                    if (chosen) {
                        addView(
                            View(context).apply {
                                background = pill(Ink.hot(palette.accent, 0.55f), gap(1).toFloat())
                                foreground = Mark(Mark.Shape.TICK, palette.background)
                                importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                            },
                            FrameLayout.LayoutParams(gap(6), gap(6)),
                        )
                    }
                    contentDescription = name
                    isClickable = true
                    isFocusable = true
                    readAs(Button::class.java.name)
                    setOnClickListener {
                        val held = picks(folder)
                        if (!held.remove(entry.path)) {
                            held.add(entry.path)
                        }
                        render(false)
                    }
                },
                LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(3) },
            )

            addView(
                TextView(context).apply {
                    text = name
                    setTextColor(palette.foreground)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                    maxLines = 1
                    ellipsize = android.text.TextUtils.TruncateAt.MIDDLE
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
        }
    }

    private fun openEntry(root: String, entry: FileEntry) {
        val name = entry.path.lowercase(Locale.US)
        when {
            entry.folder -> go(Screen.Files(root, entry.path))
            looksLikeAPicture(entry.path) -> go(Screen.Picture(root, entry.path))
            PACKAGE_SUFFIXES.any { name.endsWith(it) } -> handPackage(root, entry)
            else -> {
                editorText = ""
                go(Screen.Editor(root, entry.path))
            }
        }
    }

    private fun handPackage(root: String, entry: FileEntry) {
        val file = File(root, entry.path)
        val named = entry.path.substringAfterLast('/')
        val actions = mutableListOf<Pair<String, () -> Unit>>()
        if (named.lowercase(Locale.US).endsWith(".apk")) {
            actions += getString(R.string.omni_install_title) to { installPackage(file) }
        }
        actions += getString(R.string.omni_action_open) to { openOutside(file) }
        actions += getString(R.string.omni_action_share) to { shareOutside(file) }
        actions += getString(R.string.omni_open_title) to {
            results.removeAllViews()
            inspectPackage(file.absolutePath)
            scroller.post { scroller.fullScroll(View.FOCUS_DOWN) }
        }
        pickOne(named, actions.map { it.first }) { which -> actions[which].second() }
    }

    private fun installPackage(file: File) {
        if (!packageManager.canRequestPackageInstalls()) {
            awaitingInstall = file
            askToAllowInstalling()
            return
        }
        awaitingInstall = null
        openOutside(file)
    }

    private fun askToAllowInstalling() {
        dialogShell(
            getString(R.string.omni_install_needed),
            body(getString(R.string.omni_install_needed_note)),
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog ->
                    awaitingInstall = null
                    it.dismiss()
                },
                getString(R.string.omni_action_confirm) to { it: AlertDialog ->
                    it.dismiss()
                    hand(
                        Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES)
                            .setData(Uri.parse("package:$packageName"))
                            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    )
                },
            ),
        )
    }

    private fun holdForPaste(root: String, entry: FileEntry, cut: Boolean) {
        fileClip = FileClip(root, entry.path, cut)
        flash(
            getString(R.string.omni_files_held, entry.path.substringAfterLast('/')),
            palette.ok,
            if (cut) Mark.Shape.EXPORT else Mark.Shape.COPY,
        )
        drawFooter()
    }

    private fun canPaste(root: String, into: String): Boolean {
        val clip = fileClip ?: return false
        if (clip.root != root || !File(root, clip.path).exists()) {
            return false
        }
        if (into == clip.path || into.startsWith(clip.path + "/")) {
            return false
        }
        return !File(File(root, into), clip.path.substringAfterLast('/')).exists()
    }

    private fun pasteInto(root: String, into: String) {
        val clip = fileClip ?: return
        if (!canPaste(root, into)) {
            return
        }
        val target = joined(into, clip.path.substringAfterLast('/'))
        if (clip.cut) {
            fileClip = null
            act(Builder.nativeRenamePath(root, clip.path, target), "moved")
        } else {
            act(Builder.nativeCopyPath(root, clip.path, target), "copied")
        }
    }

    private fun showActions(root: String, folder: String, entry: FileEntry) {
        val name = entry.path.substringAfterLast('/')
        val here = File(root, entry.path)
        val pasteTo = if (entry.folder) entry.path else folder
        val rows = listOf(
            MenuChoice(Mark.Shape.COPY, getString(R.string.omni_action_copy)) {
                holdForPaste(root, entry, false)
            },
            MenuChoice(Mark.Shape.EXPORT, getString(R.string.omni_action_cut)) {
                holdForPaste(root, entry, true)
            },
            MenuChoice(
                Mark.Shape.CLIP,
                getString(R.string.omni_action_paste),
                enabled = canPaste(root, pasteTo),
            ) {
                pasteInto(root, pasteTo)
            },
            MenuChoice(
                Mark.Shape.TRASH,
                getString(
                    if (entry.path == TRASH_FOLDER) {
                        R.string.omni_action_empty
                    } else {
                        R.string.omni_action_delete
                    }
                ),
                tint = palette.error,
            ) {
                if (entry.path == TRASH_FOLDER) {
                    results.removeAllViews()
                    runCatching { Builder.nativeTrashEmpty(projectTrash(root)) }
                    render(false)
                } else {
                    act(Builder.nativeRemovePath(root, entry.path, projectTrash(root)), "removed")
                }
            },
            MenuChoice(Mark.Shape.PEN, getString(R.string.omni_action_rename)) {
                askForName(getString(R.string.omni_action_rename), name) { chosen ->
                    act(Builder.nativeRenamePath(root, entry.path, joined(folder, chosen)), "moved")
                }
            },
            MenuChoice(Mark.Shape.GEAR, getString(R.string.omni_action_tools)) {
                toolsFor(root, entry)
            },
            MenuChoice(Mark.Shape.ARCHIVE, getString(R.string.omni_action_compress)) {
                askToCompress(root, folder, entry)
            },
            MenuChoice(Mark.Shape.INFO_FILL, getString(R.string.omni_action_properties)) {
                showProperties(here, entry)
            },
            MenuChoice(
                Mark.Shape.SHARE,
                getString(R.string.omni_action_share),
                enabled = !entry.folder,
            ) {
                if (!entry.folder) {
                    shareOutside(here)
                }
            },
        )
        actionSheet(if (entry.folder) "$name/" else name, rows)
    }

    private fun projectTrash(root: String): String =
        File(root, TRASH_FOLDER).absolutePath

    private fun permissionsOf(file: File): String {
        val held = runCatching {
            Files.readAttributes(file.toPath(), PosixFileAttributes::class.java)
        }.getOrNull() ?: return ""
        val order = listOf(
            PosixFilePermission.OWNER_READ to "r",
            PosixFilePermission.OWNER_WRITE to "w",
            PosixFilePermission.OWNER_EXECUTE to "x",
            PosixFilePermission.GROUP_READ to "r",
            PosixFilePermission.GROUP_WRITE to "w",
            PosixFilePermission.GROUP_EXECUTE to "x",
            PosixFilePermission.OTHERS_READ to "r",
            PosixFilePermission.OTHERS_WRITE to "w",
            PosixFilePermission.OTHERS_EXECUTE to "x",
        )
        val drawn = order.joinToString("") { (one, letter) ->
            if (held.permissions().contains(one)) letter else "-"
        }
        var octal = 0
        order.forEachIndexed { at, (one, _) ->
            if (held.permissions().contains(one)) {
                octal = octal or (1 shl (8 - at))
            }
        }
        val head = if (file.isDirectory) "d" else "-"
        return head + drawn + "(" + Integer.toOctalString(octal).padStart(3, '0') + ")"
    }

    private fun ownerOf(file: File): Pair<String, String> {
        val held = runCatching {
            Files.readAttributes(file.toPath(), PosixFileAttributes::class.java)
        }.getOrNull() ?: return "" to ""
        val owner = runCatching { held.owner().name }.getOrDefault("")
        val group = runCatching { held.group().name }.getOrDefault("")
        return owner to group
    }

    private fun weighed(file: File): Long =
        if (file.isDirectory) {
            file.walkTopDown().filter { it.isFile }.sumOf { it.length() }
        } else {
            file.length()
        }

    private fun showProperties(here: File, entry: FileEntry) {
        val facts = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(0, gap(1), 0, gap(2))
        }
        val carried = weighed(here)
        val (owner, group) = ownerOf(here)
        val rows = listOf(
            getString(R.string.omni_property_name) to here.name,
            getString(R.string.omni_property_parent) to (here.parent ?: ""),
            getString(R.string.omni_property_kind) to getString(
                if (entry.folder) R.string.omni_files_folder else R.string.omni_files_file
            ),
            getString(R.string.omni_property_size) to
                size(carried) + "  (" + carried + ")",
            getString(R.string.omni_property_changed) to
                SimpleDateFormat("yyyy-MM-dd HH:mm:ss", Locale.US).format(Date(here.lastModified())),
            getString(R.string.omni_property_rights) to permissionsOf(here),
            getString(R.string.omni_property_owner) to owner,
            getString(R.string.omni_property_group) to group,
        )
        rows.forEach { (named, said) ->
            if (said.isEmpty()) {
                return@forEach
            }
            facts.addView(
                LinearLayout(this).apply {
                    orientation = LinearLayout.HORIZONTAL
                    setPadding(0, gap(1), 0, gap(1))
                    addView(
                        TextView(context).apply {
                            text = named
                            setTextColor(palette.muted)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                        },
                        LinearLayout.LayoutParams(gap(28), WRAP_CONTENT),
                    )
                    addView(
                        TextView(context).apply {
                            text = said
                            setTextColor(palette.foreground)
                            typeface = Type.data
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
        }
        val summed = TextView(this).apply {
            setTextColor(palette.marker)
            typeface = Type.data
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
            setPadding(0, gap(2), 0, 0)
        }
        facts.addView(summed, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
        dialogShell(
            getString(R.string.omni_action_properties),
            facts,
            listOf(
                getString(R.string.omni_property_sum) to { _: AlertDialog ->
                    if (here.isFile) {
                        summed.text = getString(R.string.omni_property_summing)
                        working({ Builder.nativeDigestFile(here.absolutePath) }) { answer ->
                            val held = runCatching { JSONObject(answer) }.getOrNull()
                            summed.text = if (held?.optBoolean("summed", false) == true) {
                                "SHA-256  " + held.optString("sha256")
                            } else {
                                getString(R.string.omni_refused)
                            }
                        }
                    }
                },
                getString(R.string.omni_action_confirm) to { it: AlertDialog -> it.dismiss() },
            ),
        )
    }

    private fun askToCompress(root: String, folder: String, entry: FileEntry) {
        val name = entry.path.substringAfterLast('/')
        val stem = if (entry.folder) name else name.substringBeforeLast('.', name)
        var format = PACK_FORMATS.first()

        val named = styledField(stem, getString(R.string.omni_pack_name))
        val ending = TextView(this).apply {
            text = "." + format
            setTextColor(palette.muted)
            typeface = Type.data
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
        }
        val secret = styledField("", getString(R.string.omni_pack_password), secret = true)
        val shapes = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            setPadding(0, gap(1), 0, gap(1))
        }
        fun paintShapes() {
            for (at in 0 until shapes.childCount) {
                val chip = shapes.getChildAt(at) as TextView
                val on = PACK_FORMATS[at] == format
                chip.setTextColor(if (on) Color.WHITE else palette.foreground)
                chip.typeface = if (on) Type.strong else Type.body
                chip.background = GradientDrawable().apply {
                    shape = GradientDrawable.RECTANGLE
                    cornerRadius = gap(5).toFloat()
                    setColor(if (on) palette.accent else Ink.fade(palette.accent, 0.1f))
                    setStroke(1, if (on) palette.accent else Ink.fade(palette.accent, 0.35f))
                }
            }
            ending.text = "." + format
            secret.isEnabled = format == "zip"
            secret.alpha = if (secret.isEnabled) 1f else 0.38f
        }
        PACK_FORMATS.forEach { one ->
            shapes.addView(
                TextView(this).apply {
                    text = "." + one
                    typeface = Type.data
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                    gravity = Gravity.CENTER
                    minimumHeight = gap(9)
                    setPadding(gap(5), gap(2), gap(5), gap(2))
                    isClickable = true
                    isFocusable = true
                    contentDescription = getString(R.string.omni_pack_format) + " " + one
                    readAs(Button::class.java.name)
                    setOnClickListener {
                        format = one
                        paintShapes()
                    }
                },
                LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT).apply { marginEnd = gap(2) },
            )
        }
        paintShapes()

        dialogShell(
            getString(R.string.omni_action_compress),
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(0, gap(1), 0, gap(2))
                addView(label(getString(R.string.omni_pack_name)))
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.HORIZONTAL
                        gravity = Gravity.CENTER_VERTICAL
                        addView(named, LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f))
                        addView(
                            ending,
                            LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT).apply {
                                marginStart = gap(2)
                            },
                        )
                    },
                    LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
                )
                addView(label(getString(R.string.omni_pack_format)))
                addView(shapes, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
                addView(label(getString(R.string.omni_pack_password)))
                addView(secret, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
                addView(quiet(getString(R.string.omni_pack_password_note)))
            },
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() },
                getString(R.string.omni_action_confirm) to { it: AlertDialog ->
                    val chosen = named.text.toString().trim()
                    results.removeAllViews()
                    if (chosen.isEmpty()) {
                        results.addView(
                            notice(getString(R.string.omni_files_needs_name), palette.warning)
                        )
                    } else {
                        it.dismiss()
                        packInto(root, folder, entry, chosen + "." + format, format, secret)
                    }
                },
            ),
        )
        named.requestFocus()
    }

    private fun packInto(
        root: String,
        folder: String,
        entry: FileEntry,
        named: String,
        format: String,
        secret: EditText,
    ) {
        val held = readSecret(secret)
        val password = String(held)
        held.fill(' ')
        secret.text?.clear()
        val into = File(File(root, folder), named).absolutePath
        working({
            Builder.nativePackArchive(root, entry.path, into, format, password)
        }) { answer ->
            val document = runCatching { JSONObject(answer) }.getOrNull() ?: return@working
            if (!document.optBoolean("packed", false)) {
                showRefusal(Refusal.parse(document), results)
                return@working
            }
            flash(
                getString(
                    R.string.omni_pack_done,
                    document.optInt("files"),
                    size(document.optLong("written")),
                ),
                palette.ok,
                Mark.Shape.ARCHIVE,
            )
            render(false)
        }
    }

    private fun toolsFor(root: String, entry: FileEntry) {
        val here = File(root, entry.path)
        val rows = mutableListOf<MenuChoice>()
        if (!entry.folder) {
            rows += MenuChoice(Mark.Shape.CODE, getString(R.string.omni_action_open_text)) {
                editorText = ""
                go(Screen.Editor(root, entry.path))
            }
        }
        rows += MenuChoice(Mark.Shape.LINK, getString(R.string.omni_action_copy_path)) {
            copyOut(here.absolutePath, entry.path.substringAfterLast('/'))
        }
        rows += MenuChoice(Mark.Shape.PAGE_ADD, getString(R.string.omni_action_create)) {
            askToCreate(root, if (entry.folder) entry.path else entry.path.substringBeforeLast('/', ""))
        }
        actionSheet(getString(R.string.omni_action_tools), rows)
    }

    private fun askForSecret(title: String, onChosen: (CharArray) -> Unit) {
        val field = styledField("", title, secret = true)
        dialogShell(
            title,
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(0, gap(1), 0, gap(2))
                addView(field, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            },
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() },
                getString(R.string.omni_action_confirm) to { it: AlertDialog ->
                    val held = CharArray(field.text.length)
                    android.text.TextUtils.getChars(field.text, 0, field.text.length, held, 0)
                    field.text.clear()
                    it.dismiss()
                    onChosen(held)
                },
            ),
        )
        field.requestFocus()
    }

    private fun askToFind(title: String, initial: String, onChosen: (String) -> Unit) {
        val editor = styledField(initial, title)
        val past = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            visibility = View.GONE
        }
        val keeper = markControl(Mark.Shape.CLOCK, R.string.omni_find_past) { }
        fun fillPast() {
            past.removeAllViews()
            val kept = Preferences.searches(this)
            if (kept.isEmpty()) {
                past.addView(quiet(getString(R.string.omni_find_past_none)))
                return
            }
            kept.forEach { word ->
                past.addView(
                    TextView(this).apply {
                        text = word
                        setTextColor(palette.foreground)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                        minimumHeight = gap(9)
                        gravity = Gravity.CENTER_VERTICAL
                        maxLines = 1
                        ellipsize = android.text.TextUtils.TruncateAt.END
                        setPadding(gap(2), gap(1), gap(2), gap(1))
                        isClickable = true
                        isFocusable = true
                        readAs(Button::class.java.name)
                        background = reacting(Color.TRANSPARENT, gap(2).toFloat(), palette.accent)
                        setOnClickListener {
                            editor.setText(word)
                            editor.setSelection(word.length)
                            past.visibility = View.GONE
                        }
                    },
                    LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
                )
            }
            past.addView(
                subtle(getString(R.string.omni_find_past_clear), palette.muted) {
                    Preferences.forgetSearches(this)
                    fillPast()
                }
            )
        }
        keeper.setOnClickListener {
            if (past.visibility == View.VISIBLE) {
                past.visibility = View.GONE
            } else {
                fillPast()
                past.visibility = View.VISIBLE
            }
        }
        dialogShell(
            title,
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(0, gap(1), 0, gap(2))
                addView(editor, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
                addView(past, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            },
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() },
                getString(R.string.omni_action_confirm) to { it: AlertDialog ->
                    val chosen = editor.text.toString().trim()
                    if (chosen.isEmpty()) {
                        results.removeAllViews()
                        results.addView(
                            notice(getString(R.string.omni_files_needs_name), palette.warning)
                        )
                    } else {
                        it.dismiss()
                        Preferences.remember(this, chosen)
                        onChosen(chosen)
                    }
                },
            ),
            keeper,
        )
        editor.requestFocus()
    }

    private fun askForTheAssistantKey(vendor: Brain.Vendor) {
        val editor = styledField("", getString(R.string.omni_talk_key)).apply {
            inputType = android.text.InputType.TYPE_CLASS_TEXT or
                android.text.InputType.TYPE_TEXT_VARIATION_PASSWORD
            typeface = Type.data
        }
        dialogShell(
            vendor.shown,
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(0, gap(1), 0, gap(2))
                addView(editor, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
                addView(quiet(getString(R.string.omni_talk_key_where, vendor.where)))
                addView(quiet(getString(R.string.omni_talk_key_note)))
            },
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() },
                getString(R.string.omni_action_confirm) to { it: AlertDialog ->
                    val held = readSecret(editor)
                    results.removeAllViews()
                    if (held.isEmpty()) {
                        results.addView(
                            notice(getString(R.string.omni_talk_no_key), palette.warning)
                        )
                    } else {
                        val locked = Vault.seal(held)
                        held.fill(' ')
                        editor.text?.clear()
                        if (locked == null) {
                            results.addView(
                                notice(getString(R.string.omni_device_refused), palette.error)
                            )
                        } else {
                            Preferences.setVendorKey(this, vendor.id, locked)
                            Preferences.setFaults(this, vendor.id, 0)
                            Preferences.setRestingUntil(this, vendor.id, 0L)
                            results.addView(
                                notice(getString(R.string.omni_talk_key_set), palette.ok)
                            )
                        }
                        it.dismiss()
                        render(false)
                    }
                },
            ),
        )
        editor.requestFocus()
    }

    private fun sayWhy(why: Throwable): String {
        val refused = why as? Brain.Refused
        if (refused == null) {
            return why.message ?: why.javaClass.simpleName
        }
        val head = getString(refused.said)
        return if (refused.detail.isEmpty()) head else head + "  (" + refused.detail + ")"
    }

    private fun askToSignIn(vendor: Brain.Vendor) {
        if (vendor.copilot) {
            signIntoGitHub(vendor)
        } else {
            askForTheAssistantKey(vendor)
        }
    }

    private fun signIntoGitHub(vendor: Brain.Vendor) {
        results.removeAllViews()
        results.addView(notice(getString(R.string.omni_working), palette.accent))
        Thread {
            val answer = runCatching { Copilot.begin() }
            runOnUiThread {
                if (isFinishing || isDestroyed) {
                    return@runOnUiThread
                }
                results.removeAllViews()
                answer.fold({ coming -> showTheGitHubCode(vendor, coming) }) { why ->
                    results.addView(notice(sayWhy(why), palette.error))
                }
            }
        }.start()
    }

    private fun showTheGitHubCode(vendor: Brain.Vendor, coming: Copilot.Coming) {
        val body = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(0, gap(1), 0, gap(2))
            addView(
                TextView(context).apply {
                    text = coming.shown
                    setTextColor(palette.accent)
                    typeface = Type.data
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 26f)
                    gravity = Gravity.CENTER
                    setPadding(0, gap(2), 0, gap(2))
                }
            )
            addView(quiet(getString(R.string.omni_copilot_where, coming.where)))
            addView(
                subtle(getString(R.string.omni_action_copy), palette.accent) {
                    copyOut(coming.shown, vendor.shown)
                }
            )
            addView(
                subtle(getString(R.string.omni_action_open), palette.ok) {
                    hand(
                        Intent(Intent.ACTION_VIEW, Uri.parse(coming.where))
                            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    )
                }
            )
            addView(quiet(getString(R.string.omni_copilot_waiting)))
        }
        val shown = dialogShell(
            getString(R.string.omni_copilot_title),
            body,
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog ->
                    signingIn.set(false)
                    it.dismiss()
                },
            ),
        )
        signingIn.set(true)
        Thread {
            var held = ""
            var why: Throwable? = null
            while (signingIn.get() && System.currentTimeMillis() < coming.until) {
                runCatching { Thread.sleep(coming.every * 1000L) }
                if (!signingIn.get()) {
                    break
                }
                val outcome = runCatching { Copilot.granted(coming) }
                if (outcome.isSuccess) {
                    held = outcome.getOrNull().orEmpty()
                    break
                }
                val fault = outcome.exceptionOrNull()
                if (fault !is Copilot.Waiting) {
                    why = fault
                    break
                }
            }
            val given = held
            val fault = why
            runOnUiThread {
                signingIn.set(false)
                runCatching { shown.dismiss() }
                if (isFinishing || isDestroyed) {
                    return@runOnUiThread
                }
                results.removeAllViews()
                theGitHubAnswer(vendor, given, fault)
            }
        }.start()
    }

    private fun theGitHubAnswer(vendor: Brain.Vendor, given: String, why: Throwable?) {
        if (given.isEmpty()) {
            results.addView(
                notice(
                    why?.let { sayWhy(it) } ?: getString(R.string.omni_copilot_refused),
                    palette.error,
                )
            )
            return
        }
        val holding = given.toCharArray()
        val locked = Vault.seal(holding)
        holding.fill(' ')
        if (locked == null) {
            results.addView(notice(getString(R.string.omni_device_refused), palette.error))
            return
        }
        Preferences.setVendorKey(this, vendor.id, locked)
        Preferences.setFaults(this, vendor.id, 0)
        Preferences.setRestingUntil(this, vendor.id, 0L)
        Preferences.setAssistantVendor(this, vendor.id)
        Copilot.forget()
        results.addView(notice(getString(R.string.omni_copilot_done), palette.ok))
        render(false)
    }

    private fun vendorRow(vendor: Brain.Vendor, onPicked: () -> Unit): View {
        val here = Preferences.assistantVendor(this)
        val keyed = vendor.keyed(this)
        val resting = Preferences.restingUntil(this, vendor.id) - System.currentTimeMillis()
        return LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            minimumHeight = gap(ROW_UNITS)
            setPadding(gap(4), gap(2), gap(4), gap(2))
            isClickable = true
            isFocusable = true
            contentDescription = vendor.shown
            readAs(Button::class.java.name)
            pressable(onHeld = { askToSignIn(vendor) })
            setOnClickListener {
                if (keyed) {
                    Preferences.setAssistantVendor(this@BuilderActivity, vendor.id)
                    onPicked()
                } else {
                    askToSignIn(vendor)
                }
            }
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = vendor.shown
                            setTextColor(palette.foreground)
                            typeface = Type.strong
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = buildString {
                                append(
                                    when {
                                        vendor.keyless ->
                                            getString(R.string.omni_talk_keyless)
                                        keyed -> getString(R.string.omni_talk_key_set)
                                        else -> getString(R.string.omni_talk_no_key)
                                    }
                                )
                                if (vendor.free) {
                                    append("  \u00b7  ").append(getString(R.string.omni_talk_free))
                                }
                                if (resting > 0L) {
                                    append("  \u00b7  ").append(
                                        getString(
                                            R.string.omni_talk_resting,
                                            ((resting + 59_999L) / 60_000L).toInt(),
                                        )
                                    )
                                }
                            }
                            setTextColor(
                                when {
                                    resting > 0L -> palette.warning
                                    keyed -> palette.ok
                                    else -> palette.muted
                                }
                            )
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                            maxLines = 1
                        }
                    )
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            addView(
                View(context).apply { background = radioMark(vendor.id == here && keyed) },
                LinearLayout.LayoutParams(gap(5), gap(5)),
            )
        }
    }

    private fun chooseTheVendor() {
        var shown: AlertDialog? = null
        val list = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        Brain.VENDORS.forEachIndexed { at, one ->
            if (at > 0) {
                list.addView(rule(), LinearLayout.LayoutParams(MATCH_PARENT, 1))
            }
            list.addView(
                vendorRow(one) {
                    shown?.dismiss()
                    render(false)
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
        }
        shown = dialogShell(
            getString(R.string.omni_talk_providers),
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                addView(quiet(getString(R.string.omni_talk_providers_note)))
                addView(list, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            },
            listOf(getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() }),
        )
    }

    private fun askForName(title: String, initial: String, onChosen: (String) -> Unit) {
        val editor = styledField(initial, title)
        dialogShell(
            title,
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(0, gap(1), 0, gap(2))
                addView(editor, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            },
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() },
                getString(R.string.omni_action_confirm) to { it: AlertDialog ->
                    val chosen = editor.text.toString().trim()
                    if (chosen.isEmpty()) {
                        results.removeAllViews()
                        results.addView(
                            notice(getString(R.string.omni_files_needs_name), palette.warning)
                        )
                    } else {
                        it.dismiss()
                        onChosen(chosen)
                    }
                },
            ),
        )
        editor.requestFocus()
    }

    private fun renderPicture(root: String, path: String) {
        content.addView(quiet(path))

        val file = File(root, path)
        val sheet = card()
        sheet.addView(
            thumbnail(file.absolutePath, gap(64)),
            LinearLayout.LayoutParams(MATCH_PARENT, gap(64)).apply {
                topMargin = gap(2)
                bottomMargin = gap(2)
            },
        )

        val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        runCatching { BitmapFactory.decodeFile(file.absolutePath, bounds) }
        sheet.addView(
            keyValue(
                getString(R.string.omni_picture_size),
                if (bounds.outWidth > 0) "${bounds.outWidth} × ${bounds.outHeight}" else "",
                size(file.length()),
                palette.muted,
            )
        )
        content.addView(sheet)

        content.addView(subtle(getString(R.string.omni_action_share), palette.accent) {
            shareOutside(file)
        })
        content.addView(subtle(getString(R.string.omni_action_delete), palette.error) {
            results.removeAllViews()
            val thrown = runCatching {
                JSONObject(Builder.nativeRemovePath(root, path, trashFolder()))
            }.getOrNull()
            if (thrown != null && thrown.optBoolean("removed", false)) {
                go(Screen.Files(root, path.substringBeforeLast('/', "")))
            } else {
                thrown?.let { showRefusal(Refusal.parse(it), results) }
            }
        })
        content.addView(subtle(getString(R.string.omni_action_back), palette.muted) {
            go(Screen.Files(root, path.substringBeforeLast('/', "")))
        })
    }

    private fun renderEditor(root: String, path: String) {
        val answer = runCatching { JSONObject(Builder.nativeReadFile(root, path)) }.getOrNull()
        if (answer == null || !answer.optBoolean("read", false)) {
            answer?.let { showRefusal(Refusal.parse(it), content) }
            content.addView(subtle(getString(R.string.omni_action_back), palette.muted) {
                go(Screen.Files(root, path.substringBeforeLast('/', "")))
            })
            return
        }
        val onDisk = answer.optString("text")
        editorOnDisk = onDisk

        val draft = Drafts.read(this, root, path)?.takeIf { it != onDisk }
        editorText = draft ?: onDisk

        val editor = CodeEditor(this, palette).apply {
            setPadding(gap(9), gap(2), gap(3), gap(2))
            readAt(Preferences.reading(this@BuilderActivity))
            wrapAt(Preferences.wrapping(this@BuilderActivity))
            open(path.substringAfterLast('/'), editorText)
            onResized = { size -> Preferences.setReading(this@BuilderActivity, size) }
        }
        this.editor = editor
        this.editorAt = root to path
        if (editorLine > 0) {
            if (editorPick > 0) {
                editor.showAt(editorLine, editorColumn, editorPick)
            } else {
                editor.showLine(editorLine)
            }
            editorLine = 0
            editorPick = 0
        }

        editorDirty = draft != null
        paintTheSave()
        var overwrite = false
        val keep = Runnable { Drafts.write(this, root, path, editorText) }

        fun save() {
            editorText = editor.text?.toString().orEmpty()
            val now = runCatching { JSONObject(Builder.nativeReadFile(root, path)) }.getOrNull()
            val elsewhere = now != null &&
                now.optBoolean("read", false) &&
                now.optString("text") != editorOnDisk
            results.removeAllViews()
            if (elsewhere && !overwrite) {
                overwrite = true
                results.addView(
                    notice(getString(R.string.omni_editor_changed_elsewhere), palette.warning)
                )
                return
            }
            val saved = runCatching { JSONObject(Builder.nativeWriteFile(root, path, editorText)) }
                .getOrNull()
            if (saved != null && saved.optBoolean("saved", false)) {
                editorDirty = false
                overwrite = false
                editorOnDisk = editorText
                paintTheSave()
                editor.removeCallbacks(keep)
                Drafts.forget(this, root, path)
            } else {
                saved?.let { showRefusal(Refusal.parse(it), results) }
            }
        }

        val watch = Runnable { checkWhileTyping(root, path, editor) }
        fun mark() {
            editorText = editor.text?.toString().orEmpty()
            val changed = editorText != onDisk
            if (changed != editorDirty) {
                editorDirty = changed
                paintTheSave()
            }
            editor.removeCallbacks(keep)
            if (changed) {
                editor.postDelayed(keep, Drafts.REST_MILLIS)
            } else {
                Drafts.forget(this, root, path)
            }
            editor.removeCallbacks(watch)
            editor.postDelayed(watch, WATCH_CODE_MILLIS)
            offerWhileTyping(root, editor)
            paintTheSteps()
        }
        editor.onChanged = ::mark

        fun sayWhere() {
            val (line, column) = editor.caretAt()
            editorSpot?.text = "/" + path + "   " +
                getString(R.string.omni_editor_spot, line, column) + "  " + CHARSET_SHOWN
        }
        sayWhere()
        editor.onCaret = { _, _ -> sayWhere() }

        editorSave = ::save
        editorUndo = {
            editor.undo()
            mark()
        }
        editorRedo = {
            editor.redo()
            mark()
        }
        editorMore = { under -> editorMenu(path, editor, under) }
        paintTheSteps()

        if (draft != null) {
            content.addView(notice(getString(R.string.omni_editor_restored), palette.warning))
        }

        content.addView(
            editor,
            LinearLayout.LayoutParams(
                MATCH_PARENT,
                (resources.displayMetrics.heightPixels * 0.78f).toInt(),
            ),
        )
        content.addView(symbolBar(editor, ::mark))
        editor.postDelayed(watch, WATCH_CODE_MILLIS)
    }

    private fun symbolBar(editor: CodeEditor, mark: () -> Unit): View {
        val row = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
        }
        for (symbol in EDITOR_SYMBOLS) {
            row.addView(
                control().apply {
                    text = symbol
                    setTextColor(palette.foreground)
                    typeface = Type.data
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                    minWidth = gap(7)
                    setPadding(gap(2), gap(3), gap(2), gap(3))
                    background = reacting(Color.TRANSPARENT, gap(2).toFloat(), palette.accent)
                    isFocusable = false
                    isFocusableInTouchMode = false
                    setOnClickListener {
                        val from = editor.selectionStart.coerceAtLeast(0)
                        val to = editor.selectionEnd.coerceAtLeast(from)
                        editor.text?.replace(from, to, symbol)
                        editor.setSelection(
                            (from + symbol.length).coerceIn(0, editor.text?.length ?: 0)
                        )
                        mark()
                        typeInto(editor)
                        press()
                    }
                },
                LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT).apply {
                    marginEnd = gap(1)
                },
            )
        }
        return HorizontalScrollView(this).apply {
            isHorizontalScrollBarEnabled = false
            contentDescription = getString(R.string.omni_editor_symbols)
            setBackgroundColor(palette.surface)
            addView(row, MATCH_PARENT, WRAP_CONTENT)
            layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT)
        }
    }

    private fun editorMenu(path: String, editor: CodeEditor, under: View) {
        anchoredMenu(
            under,
            listOf(
                MenuChoice(Mark.Shape.FIND, getString(R.string.omni_editor_find)) {
                    findInTheEditor(editor)
                },
                MenuChoice(Mark.Shape.SORTED_DOWN, getString(R.string.omni_editor_line)) {
                    askForName(getString(R.string.omni_editor_line_ask), "") { asked ->
                        results.removeAllViews()
                        val line = asked.toIntOrNull()
                        if (line == null || line < 1) {
                            results.addView(
                                notice(getString(R.string.omni_editor_line_ask), palette.warning)
                            )
                        } else {
                            editor.showLine(line)
                        }
                    }
                },
                MenuChoice(Mark.Shape.WIDEN, getString(R.string.omni_editor_size)) {
                    askForTheReadingSize(editor)
                },
                MenuChoice(
                    Mark.Shape.WIDEN,
                    getString(R.string.omni_editor_wrap),
                    picked = editor.wrapping(),
                    parted = true,
                ) {
                    val on = !editor.wrapping()
                    editor.wrapAt(on)
                    Preferences.setWrapping(this, on)
                },
                MenuChoice(
                    Mark.Shape.SPARK,
                    getString(R.string.omni_editor_complete),
                    picked = Preferences.completing(this),
                ) {
                    val on = !Preferences.completing(this)
                    Preferences.setCompleting(this, on)
                    if (!on) {
                        hideOffers()
                    }
                },
                MenuChoice(Mark.Shape.BROOM, getString(R.string.omni_editor_strip)) {
                    results.removeAllViews()
                    val removed = editor.stripComments()
                    results.addView(
                        if (removed > 0) {
                            notice(
                                getString(R.string.omni_editor_stripped, removed),
                                palette.ok,
                            )
                        } else {
                            notice(getString(R.string.omni_editor_strip_none), palette.muted)
                        }
                    )
                },
            ),
        )
    }

    private fun findInTheEditor(editor: CodeEditor) {
        val asking = styledField(findHere, getString(R.string.omni_editor_find_ask))
        val counted = quiet("")
        fun walk(onward: Boolean) {
            val needle = asking.text.toString()
            if (needle.isEmpty()) {
                counted.text = getString(R.string.omni_files_needs_name)
                return
            }
            findHere = needle
            Preferences.remember(this, needle)
            val at = if (onward) {
                editor.findFrom(needle, editor.selectionEnd)
            } else {
                editor.findBefore(needle, editor.selectionStart)
            }
            if (at < 0) {
                counted.text = getString(R.string.omni_editor_find_none)
                return
            }
            editor.show(at, at + needle.length)
            counted.text = getString(R.string.omni_editor_spot, editor.caretAt().first, at + 1)
        }
        dialogShell(
            getString(R.string.omni_editor_find),
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(0, gap(1), 0, gap(2))
                addView(asking, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
                addView(counted, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            },
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() },
                getString(R.string.omni_editor_find_back) to { _: AlertDialog -> walk(false) },
                getString(R.string.omni_editor_find_next) to { _: AlertDialog -> walk(true) },
            ),
        )
        asking.requestFocus()
    }

    private fun askForTheReadingSize(editor: CodeEditor) {
        val shown = TextView(this).apply {
            setTextColor(palette.foreground)
            typeface = Type.data
            gravity = Gravity.CENTER
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 17f)
        }
        fun say() {
            shown.text = editor.readingAt().toInt().toString()
        }
        say()
        fun step(named: String, by: Float): View = control().apply {
            text = named
            setTextColor(palette.foreground)
            typeface = Type.data
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 19f)
            minWidth = gap(12)
            minimumHeight = gap(12)
            background = reacting(palette.raised, gap(3).toFloat(), palette.accent)
            setOnClickListener {
                editor.readAt(editor.readingAt() + by)
                say()
                press()
            }
        }
        dialogShell(
            getString(R.string.omni_editor_size),
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(0, gap(2), 0, gap(2))
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.HORIZONTAL
                        gravity = Gravity.CENTER_VERTICAL
                        addView(step("\u2212", -1f))
                        addView(shown, LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f))
                        addView(step("+", 1f))
                    },
                    LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
                )
                addView(quiet(getString(R.string.omni_editor_size_note)))
            },
            listOf(
                getString(R.string.omni_action_confirm) to { it: AlertDialog -> it.dismiss() },
            ),
        )
    }

    private fun checkWhileTyping(root: String, path: String, editor: CodeEditor) {
        if (checkingCode) {
            return
        }
        checkingCode = true
        val mine = editor
        Thread {
            val answer = runCatching { Builder.nativeCheckProject(root) }.getOrNull()
            runOnUiThread {
                checkingCode = false
                if (isFinishing || isDestroyed || this.editor !== mine) {
                    return@runOnUiThread
                }
                val checked = answer
                    ?.let { runCatching { CodeCheck.parse(it) }.getOrNull() }
                    ?: return@runOnUiThread
                if (checked.clear) {
                    mine.fault(-1)
                    mine.say("")
                    return@runOnUiThread
                }
                if (checked.file == path && checked.line > 0) {
                    mine.fault(checked.line)
                    mine.say(checked.refusal?.message.orEmpty())
                } else {
                    mine.fault(-1)
                    mine.say(
                        if (checked.file.isEmpty()) {
                            checked.refusal?.message.orEmpty()
                        } else {
                            getString(
                                R.string.omni_editor_elsewhere,
                                checked.file.substringAfterLast('/'),
                                checked.line,
                            )
                        }
                    )
                }
            }
        }.start()
    }

    private fun offerWhileTyping(root: String, editor: CodeEditor) {
        editor.removeCallbacks(offerSoon)
        if (!Preferences.completing(this)) {
            hideOffers()
            return
        }
        offerRoot = root
        editor.postDelayed(offerSoon, OFFER_MILLIS)
    }

    private val offerSoon = Runnable {
        val editor = this.editor ?: return@Runnable
        val root = offerRoot ?: return@Runnable
        val held = editor.text ?: return@Runnable
        val caret = editor.selectionEnd.coerceIn(0, held.length)
        if (caret < held.length && isNamePart(held[caret])) {
            hideOffers()
            return@Runnable
        }
        var from = caret
        while (from > 0 && isNamePart(held[from - 1])) from -= 1
        val word = held.subSequence(from, caret).toString()
        if (word.length < OFFER_LEAST || !word[0].isLetter()) {
            hideOffers()
            return@Runnable
        }
        if (offering) {
            return@Runnable
        }
        offering = true
        Thread {
            val answer = runCatching { Builder.nativeSymbols(root, word) }.getOrNull()
            runOnUiThread {
                offering = false
                if (isFinishing || isDestroyed || this.editor !== editor) {
                    return@runOnUiThread
                }
                showOffers(editor, from, caret, word, answer)
            }
        }.start()
    }

    private fun showOffers(
        editor: CodeEditor,
        from: Int,
        to: Int,
        word: String,
        answer: String?,
    ) {
        val named = answer
            ?.let { runCatching { JSONObject(it) }.getOrNull() }
            ?.optJSONArray("named")
        if (named == null || named.length() == 0) {
            hideOffers()
            return
        }
        val held = editor.text ?: return
        if (editor.selectionEnd != to || to > held.length) {
            hideOffers()
            return
        }

        val list = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            background = sheet(palette.surface, palette.accent)
            setPadding(gap(1), gap(1), gap(1), gap(1))
        }
        var shown = 0
        for (index in 0 until named.length()) {
            if (shown >= OFFER_MOST) break
            val one = named.optJSONObject(index) ?: continue
            val simple = one.optString("simple")
            if (!simple.startsWith(word, ignoreCase = true)) continue
            shown += 1
            list.addView(
                TextView(this).apply {
                    text = simple
                    typeface = Type.data
                    setTextColor(palette.foreground)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                    minimumHeight = gap(9)
                    gravity = Gravity.CENTER_VERTICAL
                    setPadding(gap(3), gap(1), gap(3), gap(1))
                    isClickable = true
                    isFocusable = false
                    isFocusableInTouchMode = false
                    readAs(Button::class.java.name)
                    background = reacting(Color.TRANSPARENT, gap(2).toFloat(), palette.accent)
                    setOnClickListener {
                        hideOffers()
                        putName(editor, from, to, one)
                    }
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
        }
        if (shown == 0) {
            hideOffers()
            return
        }

        hideOffers()
        val window = android.widget.PopupWindow(
            list,
            (resources.displayMetrics.widthPixels * 0.62f).toInt(),
            WRAP_CONTENT,
            false,
        ).apply {
            isOutsideTouchable = true
            isTouchable = true
            inputMethodMode = android.widget.PopupWindow.INPUT_METHOD_NOT_NEEDED
            elevation = gap(2).toFloat()
        }
        offers = window
        val layout = editor.layout
        val row = layout?.getLineForOffset(to) ?: 0
        val x = editor.paddingLeft + (layout?.getPrimaryHorizontal(to)?.toInt() ?: 0)
        val y = (layout?.getLineBottom(row) ?: 0) - editor.scrollY + gap(1)
        runCatching {
            window.showAsDropDown(editor, x.coerceAtMost(gap(40)), y)
        }.onFailure { offers = null }
    }

    private fun typeInto(editor: EditText) {
        editor.isFocusableInTouchMode = true
        editor.requestFocus()
        runCatching {
            getSystemService(android.view.inputmethod.InputMethodManager::class.java)
                ?.showSoftInput(editor, 0)
        }
    }

    private fun hideOffers() {
        offers?.let { runCatching { it.dismiss() } }
        offers = null
    }

    private fun isNamePart(c: Char): Boolean =
        c.isLetterOrDigit() || c == '_' || c == '$'

    private fun putName(editor: CodeEditor, from: Int, to: Int, chosen: JSONObject) {
        val held = editor.text ?: return
        val simple = chosen.optString("simple")
        val qualified = chosen.optString("qualified")
        val whole = held.toString()

        val already = whole.contains("import $qualified;")
        val mine = whole.lineSequence()
            .firstOrNull { it.trimStart().startsWith("package ") }
            ?.trim()
            ?.removePrefix("package ")
            ?.removeSuffix(";")
            ?.trim()
            .orEmpty()
        val needsImport = !already &&
            qualified.contains('.') &&
            chosen.optString("package") != mine

        held.replace(from, to, simple)
        if (!needsImport) {
            editor.setSelection((from + simple.length).coerceIn(0, editor.text?.length ?: 0))
            return
        }

        val text = editor.text?.toString().orEmpty()
        var at = -1
        var line = 0
        var offset = 0
        for (one in text.lineSequence()) {
            val trimmed = one.trimStart()
            if (trimmed.startsWith("import ") || trimmed.startsWith("package ")) {
                at = offset + one.length + 1
            }
            offset += one.length + 1
            line += 1
            if (line > 200) break
        }
        val statement = "import $qualified;\n"
        val put = at.coerceIn(0, editor.text?.length ?: 0)
        editor.text?.insert(put, statement)

        val moved = if (put <= from) statement.length else 0
        editor.setSelection(
            (from + simple.length + moved).coerceIn(0, editor.text?.length ?: 0)
        )
    }

    private fun copyOut(text: String, label: String) {
        val clipboard = getSystemService(ClipboardManager::class.java) ?: return
        clipboard.setPrimaryClip(ClipData.newPlainText(label, text))
        results.removeAllViews()
        results.addView(notice(getString(R.string.omni_settings_logs_copied), palette.ok))
    }

    private fun tool(text: String, colour: Int, onPress: () -> Unit) = control().apply {
        this.text = text
        setTextColor(stated(colour))
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
        maxLines = 1
        setPadding(gap(4), gap(2), gap(4), gap(2))
        background = reacting(palette.raised, gap(4).toFloat(), colour)
        setOnClickListener {
            onPress()
            press()
        }
    }

    private fun View.press() {
        if (Motion.still()) {
            return
        }
        animate().scaleX(0.94f).scaleY(0.94f).setDuration(Motion.of(Motion.TAP)).withEndAction {
            animate().scaleX(1f).scaleY(1f).setDuration(Motion.of(Motion.REBOUND)).start()
        }.start()
    }

    private fun renderBuild(root: String) {
        content.addView(
            screenHead(
                Mark.Shape.PACKAGE,
                getString(R.string.omni_build_title),
                getString(R.string.omni_build_both_note),
            )
        )
        val summary = summaryOf(root)
        content.addView(
            quiet(summary?.label?.ifEmpty { null } ?: File(root).name)
        )

        val chosen = Preferences.signingKey(this)
        val keys = keysHere()
        val key = keys.firstOrNull { it.path == chosen }

        val signing = card()
        if (key == null) {
            signing.addView(quiet(getString(R.string.omni_build_no_key)))
        } else {
            val shared = key.alias == DEFAULT_KEY_ALIAS
            signing.addView(
                keyValue(
                    key.alias,
                    key.subject,
                    if (shared) {
                        getString(R.string.omni_keys_shared)
                    } else {
                        getString(R.string.omni_keys_yours)
                    },
                    palette.accent,
                )
            )
            val remembered = buildSecret
                ?.let { Vault.open(it) }
                ?.let { held ->
                    val said = String(held)
                    held.fill(' ')
                    said
                }
            val secret = secret(
                getString(R.string.omni_build_password),
                remembered ?: if (shared) DEFAULT_KEY_PASSWORD else "",
            )
            buildPasswordView = secret.second
            signing.addView(secret.first)
            signing.addView(
                quiet(
                    if (shared) {
                        getString(R.string.omni_build_shared_password, DEFAULT_KEY_PASSWORD)
                    } else {
                        getString(R.string.omni_build_your_password)
                    }
                )
            )
        }
        signing.addView(subtle(getString(R.string.omni_keys_title), palette.accent) {
            go(Screen.Keys)
        })
        content.addView(signing)

        val sealed = card()
        sealed.addView(
            keyValue(
                getString(R.string.omni_device_key),
                if (DeviceKey.present()) {
                    if (DeviceKey.inHardware()) {
                        getString(R.string.omni_device_hardware)
                    } else {
                        getString(R.string.omni_device_software)
                    }
                } else {
                    getString(R.string.omni_device_none)
                },
                if (DeviceKey.present() && DeviceKey.inHardware()) {
                    getString(R.string.omni_device_sealed)
                } else {
                    ""
                },
                if (DeviceKey.inHardware()) palette.ok else palette.muted,
            )
        )
        sealed.addView(quiet(getString(R.string.omni_device_note)))
        sealed.addView(
            subtle(getString(R.string.omni_device_build), palette.accent) {
                buildWithTheDeviceKey(root)
            }
        )
        content.addView(sealed)

        content.addView(
            primary(getString(R.string.omni_action_build_both)) { mendThenBuild(root) }
        )
        content.addView(quiet(getString(R.string.omni_build_both_note)))

        content.addView(heading(getString(R.string.omni_built_title)))
        val made = runCatching {
            Built.list(Builder.nativeListBuilt(outputsFolder(root).absolutePath))
        }.getOrDefault(emptyList())

        val shelf = card()
        if (made.isEmpty()) {
            shelf.addView(quiet(getString(R.string.omni_built_none)))
        }
        made.forEachIndexed { index, one ->
            if (index > 0) {
                shelf.addView(rule(), MATCH_PARENT, 1)
            }
            val file = File(one.path)
            shelf.addView(
                keyValue(
                    one.name,
                    listOf(one.folder, size(one.bytes), moment(one.writtenAt))
                        .filter { it.isNotEmpty() }
                        .joinToString("  ·  "),
                    if (one.bundle) "AAB" else "APK",
                    if (one.bundle) palette.muted else palette.ok,
                ).apply {
                    minimumHeight = gap(TOUCH_UNITS)
                    background = touchable(pill(Color.TRANSPARENT, gap(2).toFloat()), palette.accent)
                    isClickable = true
                    isFocusable = true
                    contentDescription = said(one.name, getString(R.string.omni_action_open))
                    readAs(Button::class.java.name)
                    setOnClickListener {
                        if (one.bundle) openOutside(file) else installPackage(file)
                    }
                }
            )
            actionsFor(file, shelf)
            if (!one.bundle) {
                shelf.addView(subtle(getString(R.string.omni_open_title), palette.accent) {
                    inspectPackage(one.path)
                })
            }
            shelf.addView(
                subtle(getString(R.string.omni_action_delete), palette.error) {
                    act(Builder.nativeTrashSend(trashFolder(), one.path), "removed")
                }
            )
        }
        content.addView(shelf)
        content.addView(quiet(getString(R.string.omni_trash_note)))
    }

    private fun renderRepos() {
        content.addView(
            screenHead(
                Mark.Shape.BRANCH,
                getString(R.string.omni_repos_title),
                getString(R.string.omni_repos_detail),
            )
        )
        val asking = card()
        val (queryHolder, queryField) = input(
            getString(R.string.omni_repos_query),
            reposQuery,
        )
        queryField.hint = getString(R.string.omni_repos_hint)
        asking.addView(queryHolder)
        val (tokenHolder, tokenField) = input(
            getString(R.string.omni_clone_auth_token),
            cloneToken,
        )
        asking.addView(tokenHolder)
        asking.addView(
            primary(getString(R.string.omni_repos_search)) {
                reposQuery = queryField.text?.toString().orEmpty().trim()
                cloneToken = tokenField.text?.toString().orEmpty().trim()
                lookForRepos(false)
            }
        )
        asking.addView(
            subtle(getString(R.string.omni_repos_mine), palette.accent) {
                cloneToken = tokenField.text?.toString().orEmpty().trim()
                lookForRepos(true)
            }
        )
        content.addView(asking)
        content.addView(quiet(getString(R.string.omni_repos_how)))
        if (reposFound.isNotEmpty()) {
            showRepos()
        }
    }

    private fun lookForRepos(mine: Boolean) {
        results.removeAllViews()
        results.addView(quiet(getString(R.string.omni_repos_looking)))
        val token = cloneToken
        val asked = reposQuery
        val thread = Thread {
            val answer = runCatching {
                if (mine) Forge.mine(token) else Forge.search(asked, token)
            }
            runOnUiThread {
                if (screen !is Screen.Repos) {
                    return@runOnUiThread
                }
                results.removeAllViews()
                val found = answer.getOrElse { why ->
                    results.addView(notice(refusalOf(why), palette.error))
                    return@runOnUiThread
                }
                reposFound = found
                showRepos()
            }
        }
        thread.isDaemon = true
        thread.start()
    }

    private fun showRepos() {
        if (reposFound.isEmpty()) {
            results.addView(quiet(getString(R.string.omni_repos_none)))
            return
        }
        val listed = card()
        reposFound.forEachIndexed { index, one ->
            if (index > 0) {
                listed.addView(rule(), MATCH_PARENT, 1)
            }
            val about = StringBuilder()
            if (one.about.isNotEmpty()) {
                about.append(one.about)
            }
            val facts = ArrayList<String>()
            if (one.language.isNotEmpty()) {
                facts.add(one.language)
            }
            facts.add(getString(R.string.omni_repos_stars, one.stars))
            if (one.shut) {
                facts.add(getString(R.string.omni_repos_private))
            }
            if (about.isNotEmpty()) {
                about.append('\n')
            }
            about.append(facts.joinToString(" · "))
            listed.addView(
                row(
                    one.full,
                    about.toString(),
                    getString(R.string.omni_clone_go),
                    palette.accent,
                ) {
                    cloneFrom(one.address, cloneToken, projectsFolder().absolutePath, true)
                }
            )
        }
        results.addView(listed)
    }

    private fun renderTrash() {

        val held = runCatching {
            Trashed.list(Builder.nativeTrashList(trashFolder()))
        }.getOrDefault(emptyList())

        val card = card()
        if (held.isEmpty()) {
            card.addView(quiet(getString(R.string.omni_trash_none)))
        }
        held.forEachIndexed { index, one ->
            if (index > 0) {
                card.addView(rule(), MATCH_PARENT, 1)
            }
            card.addView(
                keyValue(
                    if (one.folder) "${one.name}/" else one.name,
                    "${one.origin}\n${size(one.bytes)}",
                    left(one.secondsLeft),
                    if (one.secondsLeft < 3600) palette.warning else palette.muted,
                )
            )
            card.addView(
                row(
                    getString(R.string.omni_action_restore),
                    if (one.restorable) {
                        ""
                    } else {
                        getString(R.string.omni_trash_taken)
                    },
                    "",
                    palette.ok,
                ) {
                    act(Builder.nativeTrashRestore(trashFolder(), one.id), "restored")
                }
            )
            card.addView(
                subtle(getString(R.string.omni_action_purge), palette.error) {
                    act(Builder.nativeTrashPurge(trashFolder(), one.id), "purged")
                }
            )
        }
        content.addView(card)
        content.addView(quiet(getString(R.string.omni_trash_how)))

        if (held.isNotEmpty()) {
            content.addView(subtle(getString(R.string.omni_action_empty), palette.error) {
                results.removeAllViews()
                runCatching { Builder.nativeTrashEmpty(trashFolder()) }
                render(false)
            })
        }
    }

    private fun renderKeys() {

        val keys = keysHere()
        val chosen = Preferences.signingKey(this)

        val vault = card()
        if (keys.isEmpty()) {
            vault.addView(quiet(getString(R.string.omni_keys_none)))
        }
        keys.forEachIndexed { index, key ->
            if (index > 0) {
                vault.addView(rule(), MATCH_PARENT, 1)
            }
            val inUse = key.path == chosen
            val shared = key.alias == DEFAULT_KEY_ALIAS
            vault.addView(
                row(
                    key.alias,
                    "${key.subject}\n${getString(R.string.omni_keys_expires)} ${key.expires}  ·  " +
                        "${key.bits}\n${key.fingerprint}",
                    if (inUse) getString(R.string.omni_keys_in_use) else getString(R.string.omni_keys_use),
                    if (inUse) palette.ok else palette.accent,
                ) {
                    Preferences.setSigningKey(this, key.path)
                    render(false)
                }
            )
            vault.addView(subtle(getString(R.string.omni_keys_verify), palette.accent) {
                askForSecret(getString(R.string.omni_keys_verify)) { secret ->
                    results.removeAllViews()
                    val answer = runCatching {
                        JSONObject(Builder.nativeCheckKey(key.path, secret))
                    }.getOrNull()
                    secret.fill(' ')
                    if (answer != null && answer.optBoolean("opened", false)) {
                        results.addView(
                            notice(getString(R.string.omni_keys_verified), palette.ok)
                        )
                    } else {
                        answer?.let { showRefusal(Refusal.parse(it), results) }
                            ?: results.addView(
                                notice(getString(R.string.omni_keys_refused), palette.error)
                            )
                    }
                }
            })
            if (shared) {
                vault.addView(quiet(getString(R.string.omni_keys_shared_note, DEFAULT_KEY_PASSWORD)))
            } else {
                vault.addView(subtle(getString(R.string.omni_action_delete), palette.error) {
                    runCatching { Builder.nativeDeleteKey(key.path) }
                    if (inUse) Preferences.setSigningKey(this, "")
                    render(false)
                })
            }
        }
        content.addView(vault)
        content.addView(primary(getString(R.string.omni_keys_new)) { go(Screen.NewKey) })
        content.addView(subtle(getString(R.string.omni_action_back), palette.muted) {
            go(Screen.Settings)
        })
    }

    private fun renderNewKey() {

        val who = card()
        who.addView(field(getString(R.string.omni_key_alias), keyAlias) { keyAlias = it })
        who.addView(field(getString(R.string.omni_key_common_name), keyCommonName) { keyCommonName = it })
        who.addView(
            field(getString(R.string.omni_key_organisation), keyOrganisation) { keyOrganisation = it }
        )
        who.addView(field(getString(R.string.omni_key_country), keyCountry) { keyCountry = it })
        content.addView(who)

        content.addView(label(getString(R.string.omni_key_validity)))
        content.addView(
            chips(
                VALIDITY_YEARS.map { getString(R.string.omni_key_years, it) },
                { it == keyYears },
            ) { keyYears = it }
        )

        content.addView(label(getString(R.string.omni_key_size)))
        content.addView(chips(KEY_SIZES.map { it.toString() }, { it == keyBits }) { keyBits = it })

        val secrets = card()
        val first = secret(getString(R.string.omni_key_password))
        val again = secret(getString(R.string.omni_key_password_again))
        keyPasswordView = first.second
        keyPasswordAgainView = again.second
        secrets.addView(first.first)
        secrets.addView(again.first)
        content.addView(secrets)
        content.addView(warning(getString(R.string.omni_key_password_warning)))

        content.addView(primary(getString(R.string.omni_action_create)) { createKey() })
        content.addView(subtle(getString(R.string.omni_action_cancel), palette.muted) {
            go(Screen.Keys)
        })
    }

    private fun summaryOf(root: String): ProjectSummary? = runCatching {
        ProjectSummary.list(Builder.nativeListProjects(projectsFolder().absolutePath))
            .firstOrNull { it.root == root }
    }.getOrNull()

    private fun keysHere(): List<SigningKey> = runCatching {
        SigningKey.list(Builder.nativeListKeys(keysFolder().absolutePath))
    }.getOrDefault(emptyList())

    private fun breadcrumb(root: String, folder: String): View {
        val steps = mutableListOf(getString(R.string.omni_files_root) to "")
        var walked = ""
        for (step in folder.split('/').filter { it.isNotEmpty() }) {
            walked = if (walked.isEmpty()) step else "$walked/$step"
            steps.add(step to walked)
        }
        return chips(steps.map { it.first }, { it == steps.lastIndex }) { index ->
            go(Screen.Files(root, steps[index].second))
        }
    }

    private fun size(bytes: Long): String = when {
        bytes >= 1_048_576L -> getString(R.string.omni_size_mb, bytes / 1_048_576L)
        bytes >= 1_024L -> getString(R.string.omni_size_kb, bytes / 1_024L)
        else -> getString(R.string.omni_size_b, bytes)
    }

    private fun left(seconds: Long): String = when {
        seconds >= 3600L -> getString(R.string.omni_left_hours, seconds / 3600L)
        seconds >= 60L -> getString(R.string.omni_left_minutes, seconds / 60L)
        else -> getString(R.string.omni_left_seconds, seconds)
    }

    private fun moment(epochSeconds: Long): String =
        SimpleDateFormat("yyyy-MM-dd HH:mm", Locale.US).format(Date(epochSeconds * 1000L))

    private fun deleteProject(project: ProjectSummary) {
        results.removeAllViews()
        val thrown = runCatching {
            JSONObject(Builder.nativeTrashSend(trashFolder(), project.root))
        }.getOrNull()
        if (thrown == null || !thrown.optBoolean("removed", false)) {
            thrown?.let { showRefusal(Refusal.parse(it), results) }
            return
        }
        if (openProject == project.root) {
            openProject = null
        }
        OmniLog.event(LogLevel.INFO, "project", "Deleted ${project.root}")
        render(false)
        results.addView(notice(getString(R.string.omni_trash_sent, project.name), palette.ok))
    }

    private fun heldLog(): String = synchronized(logHeld) {
        val needle = logSieve.trim().lowercase(Locale.getDefault())
        logHeld.filter { needle.isEmpty() || it.lowercase(Locale.getDefault()).contains(needle) }
            .joinToString("\n")
    }

    private fun paintTheLog() {
        val written = heldLog()
        logView?.text = written
        logView?.visibility = if (written.isEmpty()) View.GONE else View.VISIBLE
        logHollow?.visibility = if (written.isEmpty()) View.VISIBLE else View.GONE
        logCount?.text = said(
            getString(if (logReading) R.string.omni_log_running else R.string.omni_log_stopped),
            getString(R.string.omni_log_lines, synchronized(logHeld) { logHeld.size }),
        )
    }

    private val logTick = object : Runnable {
        override fun run() {
            if (!logReading || screen !is Screen.Logcat || isFinishing || isDestroyed) {
                return
            }
            paintTheLog()
            content.postDelayed(this, LOG_TICK_MILLIS)
        }
    }

    private fun readProperty(named: String): String = runCatching {
        val asking = ProcessBuilder("getprop", named).redirectErrorStream(true).start()
        val said = asking.inputStream.bufferedReader().use { it.readText() }.trim()
        asking.waitFor()
        said
    }.getOrDefault("")

    private fun globalFlag(named: String): Boolean = runCatching {
        Settings.Global.getInt(contentResolver, named, 0)
    }.getOrDefault(0) == 1

    private fun logCommand(): List<String> {
        val said = mutableListOf("logcat", "-v", "time")
        val uid = uidOf(logPackage)
        when {
            uid > 0 -> said.add("--uid=" + uid)
            !logReadsAll() -> said.add("--pid=" + android.os.Process.myPid())
        }
        logArgs.trim().split(Regex("\\s+")).forEach { one ->
            if (one.isNotEmpty()) {
                said.add(one)
            }
        }
        return said
    }

    private fun uidOf(named: String): Int {
        if (named.isEmpty()) {
            return -1
        }
        return runCatching {
            packageManager.getApplicationInfo(named, 0).uid
        }.getOrDefault(-1)
    }

    private fun installedApps(): List<Pair<String, String>> = runCatching {
        val asked = Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_LAUNCHER)
        packageManager.queryIntentActivities(asked, 0)
            .mapNotNull { one ->
                val held = one.activityInfo?.applicationInfo ?: return@mapNotNull null
                packageManager.getApplicationLabel(held).toString() to held.packageName
            }
            .distinctBy { it.second }
            .sortedBy { it.first.lowercase(Locale.getDefault()) }
    }.getOrDefault(emptyList())

    private fun chooseTheLoggedApp() {
        val apps = installedApps()
        if (apps.isEmpty()) {
            results.removeAllViews()
            results.addView(notice(getString(R.string.omni_log_apps_none), palette.warning))
            return
        }
        val named = listOf(getString(R.string.omni_log_apps_any)) +
            apps.map { it.first + "  \u00b7  " + it.second }
        pickOne(getString(R.string.omni_log_apps), named) { which ->
            if (which == 0) {
                logPackage = ""
                logPackageShown = ""
            } else {
                logPackage = apps[which - 1].second
                logPackageShown = apps[which - 1].first
            }
            if (logReading) {
                stopTheLog()
                startTheLog()
            }
            render(false)
        }
    }

    private fun logReadsAll(): Boolean =
        checkSelfPermission(android.Manifest.permission.READ_LOGS) ==
            PackageManager.PERMISSION_GRANTED

    private fun grantLine(): String =
        "adb shell pm grant $packageName android.permission.READ_LOGS"

    private fun askToNotify() {
        if (Bell.allowed(this)) {
            return
        }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            runCatching {
                requestPermissions(
                    arrayOf(android.Manifest.permission.POST_NOTIFICATIONS),
                    NOTE_REQUEST,
                )
            }
        }
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == NOTE_REQUEST && !isFinishing && !isDestroyed) {
            render(false)
        }
    }

    private fun openDeveloperOptions() {
        hand(
            Intent(Settings.ACTION_APPLICATION_DEVELOPMENT_SETTINGS)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        )
    }

    @Suppress("DEPRECATION")
    private fun listenForPairing() {
        if (listening != null) {
            return
        }
        val manager = getSystemService(NsdManager::class.java) ?: return
        val ears = object : NsdManager.DiscoveryListener {
            override fun onDiscoveryStarted(service: String) = Unit

            override fun onDiscoveryStopped(service: String) = Unit

            override fun onStartDiscoveryFailed(service: String, why: Int) {
                runOnUiThread { stopListeningForPairing() }
            }

            override fun onStopDiscoveryFailed(service: String, why: Int) = Unit

            override fun onServiceLost(found: NsdServiceInfo) = Unit

            override fun onServiceFound(found: NsdServiceInfo) {
                runCatching {
                    manager.resolveService(
                        found,
                        object : NsdManager.ResolveListener {
                            override fun onResolveFailed(one: NsdServiceInfo, why: Int) = Unit

                            override fun onServiceResolved(one: NsdServiceInfo) {
                                val host = one.host?.hostAddress.orEmpty()
                                val port = one.port
                                runOnUiThread { pairingSeen(host, port) }
                            }
                        },
                    )
                }
            }
        }
        val started = runCatching {
            manager.discoverServices(PAIRING_SERVICE, NsdManager.PROTOCOL_DNS_SD, ears)
        }.isSuccess
        listening = if (started) ears else null
    }

    private fun stopListeningForPairing() {
        val ears = listening ?: return
        listening = null
        runCatching { getSystemService(NsdManager::class.java)?.stopServiceDiscovery(ears) }
    }

    private fun pairingSeen(host: String, port: Int) {
        if (isFinishing || isDestroyed || host.isEmpty() || port <= 0) {
            return
        }
        if (logPairHost == host && logPairPort == port) {
            return
        }
        logPairHost = host
        logPairPort = port
        Bell.say(
            this,
            Bell.PAIRING_NOTE,
            getString(R.string.omni_log_pair_found),
            getString(R.string.omni_log_pair_at, host, port),
        )
        flash(
            getString(R.string.omni_log_pair_at, host, port),
            palette.ok,
            Mark.Shape.LINK,
            4200L,
        )
        if (screen is Screen.Logcat) {
            render(false)
        }
    }

    private fun seekTheDevice() {
        if (logSeeking) {
            return
        }
        askToNotify()
        listenForPairing()
        logSeeking = true
        flash(getString(R.string.omni_log_seeking), palette.accent, Mark.Shape.LINK, 0L)
        Thread {
            val wireless = globalFlag("adb_wifi_enabled")
            val wired = globalFlag("adb_enabled")
            val port = listOf("service.adb.tls.port", "service.adb.tcp.port")
                .firstNotNullOfOrNull { readProperty(it).toIntOrNull()?.takeIf { open -> open > 0 } }
                ?: 0
            runOnUiThread {
                logSeeking = false
                if (isFinishing || isDestroyed) {
                    return@runOnUiThread
                }
                logPort = port
                logWireless = wireless
                logWired = wired
                if (screen is Screen.Logcat) {
                    render(false)
                }
                when {
                    logReadsAll() -> flash(
                        getString(R.string.omni_log_bound_note),
                        palette.ok,
                        Mark.Shape.TICK,
                    )
                    port > 0 -> flash(
                        getString(R.string.omni_log_seek_found, port.toString()),
                        palette.ok,
                        Mark.Shape.TICK,
                        4200L,
                    )
                    wireless -> flash(
                        getString(R.string.omni_log_seek_hidden),
                        palette.warning,
                        Mark.Shape.INFO,
                        4200L,
                    ).also { openDeveloperOptions() }
                    wired -> flash(
                        getString(R.string.omni_log_seek_wired),
                        palette.warning,
                        Mark.Shape.INFO,
                        4200L,
                    )
                    else -> flash(
                        getString(R.string.omni_log_seek_off),
                        palette.error,
                        Mark.Shape.INFO,
                        4200L,
                    ).also { openDeveloperOptions() }
                }
            }
        }.start()
    }

    private fun startTheLog() {
        if (logReading) {
            return
        }
        logRefused = ""
        logReading = true
        val spoken = logCommand()
        Thread {
            val reading = runCatching {
                ProcessBuilder(spoken).redirectErrorStream(true).start()
            }.getOrElse { why ->
                runOnUiThread {
                    logReading = false
                    logRefused = why.message ?: why.javaClass.simpleName
                    OmniLog.event(LogLevel.WARN, "logcat", "The device log would not open.")
                    if (!isFinishing && !isDestroyed && screen is Screen.Logcat) {
                        render(false)
                    }
                }
                return@Thread
            }
            logProcess = reading
            runCatching {
                reading.inputStream.bufferedReader().use { reader ->
                    while (logReading) {
                        val line = reader.readLine() ?: break
                        synchronized(logHeld) {
                            logHeld.addLast(line)
                            while (logHeld.size > LOG_HELD) {
                                logHeld.removeFirst()
                            }
                        }
                    }
                }
            }
            runCatching { reading.destroy() }
            logProcess = null
        }.start()
        content.postDelayed(logTick, LOG_TICK_MILLIS)
    }

    private fun stopTheLog() {
        if (!logReading) {
            return
        }
        logReading = false
        runCatching { logProcess?.destroy() }
        logProcess = null
    }

    private fun shellControl(shape: Mark.Shape, named: Int, press: () -> Unit): View =
        View(this).apply {
            foreground = Mark(shape, Color.WHITE)
            val squeeze = gap(3) / 2
            setPadding(squeeze, squeeze, squeeze, squeeze)
            isClickable = true
            isFocusable = true
            contentDescription = getString(named)
            readAs(Button::class.java.name)
            background = reacting(Color.TRANSPARENT, gap(6).toFloat(), palette.accent)
            setOnClickListener { press() }
        }

    private fun shellNow(): Shell? = shells.getOrNull(shellAt)

    private fun shellNamed(at: Int): String =
        shells.getOrNull(at)?.named.orEmpty().ifEmpty { (at + 1).toString() }

    private fun openAShell(): Shell? {
        if (shells.size >= MOST_SHELLS) {
            results.removeAllViews()
            results.addView(notice(getString(R.string.omni_shell_full), palette.warning))
            return null
        }
        val home = filesDir.absolutePath
        val folder = openProject ?: home
        val view = TerminalView(this, palette) { bytes -> sendToTheShell(bytes) }
        val shell = Shell(view, folder, (shells.size + 1).toString())
        shells.add(shell)
        shellAt = shells.size - 1
        startTheShell(shell, home)
        return shell
    }

    private fun startTheShell(shell: Shell, home: String) {
        File(home, "tmp").mkdirs()
        val handle = runCatching {
            Builder.nativeShellOpen(
                shell.folder,
                home,
                shell.view.screen.rows,
                shell.view.screen.columns,
            )
        }.getOrDefault(-1)
        if (handle < 0) {
            results.addView(notice(getString(R.string.omni_shell_refused), palette.error))
            return
        }
        shell.handle = handle
        OmniLog.event(LogLevel.INFO, "shell", "A shell was opened.")
        val reader = Thread {
            val room = ByteArray(8192)
            while (!Thread.currentThread().isInterrupted) {
                val got = runCatching { Builder.nativeShellRead(handle, room, 60) }
                    .getOrDefault(-1)
                if (got < 0) {
                    break
                }
                if (got > 0) {
                    val copy = room.copyOf(got)
                    runOnUiThread { shell.view.accept(copy, copy.size) }
                }
            }
            runOnUiThread { closeOneShell(shell) }
        }
        reader.isDaemon = true
        shell.reader = reader
        reader.start()
    }

    private fun shellTab(at: Int): View = TextView(this).apply {
        val here = at == shellAt
        text = shellNamed(at)
        setTextColor(if (here) Color.BLACK else Color.WHITE)
        typeface = Type.heading
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
        gravity = Gravity.CENTER
        maxLines = 1
        minWidth = gap(12)
        setPadding(gap(3), gap(2), gap(3), gap(2))
        isClickable = true
        isFocusable = true
        contentDescription = said(getString(R.string.omni_tab_shell), shellNamed(at))
        readAs(Button::class.java.name)
        background = touchable(
            pill(if (here) Color.WHITE else Ink.hot(Color.BLACK, 0.16f), gap(2).toFloat()),
            palette.accent,
        )
        setOnClickListener {
            if (shellAt != at) {
                shellAt = at
                render(false)
            }
        }
        setOnLongClickListener {
            renameTheShell(at)
            true
        }
    }

    private fun renameTheShell(at: Int) {
        val shell = shells.getOrNull(at) ?: return
        askForName(getString(R.string.omni_shell_rename), shell.named) { chosen ->
            shell.named = chosen.take(SHELL_NAME_MOST)
            render(false)
        }
    }

    private fun renderTerminal() {
        if (shells.isEmpty()) {
            openAShell()
        }
        if (shellAt !in shells.indices) {
            shellAt = (shells.size - 1).coerceAtLeast(0)
        }
        val shell = shellNow() ?: return
        shell.view.repaint(palette)
        val view = shell.view
        (view.parent as? ViewGroup)?.removeView(view)

        val frame = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setBackgroundColor(Color.BLACK)
        }
        frame.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                minimumHeight = gap(11)
                setPadding(gap(1), 0, gap(2), 0)
                setBackgroundColor(SHELL_BAR)
                addView(
                    View(context).apply {
                        foreground = Mark(Mark.Shape.CHEVRON_LEFT, Color.WHITE)
                        val squeeze = gap(3) / 2
                        setPadding(squeeze, squeeze, squeeze, squeeze)
                        isClickable = true
                        isFocusable = true
                        contentDescription = getString(R.string.omni_action_back)
                        readAs(Button::class.java.name)
                        background = reacting(Color.TRANSPARENT, gap(6).toFloat(), palette.accent)
                        setOnClickListener { stepBack() }
                    },
                    LinearLayout.LayoutParams(gap(10), gap(10)),
                )
                addView(
                    HorizontalScrollView(context).apply {
                        isHorizontalScrollBarEnabled = false
                        addView(
                            LinearLayout(context).apply {
                                orientation = LinearLayout.HORIZONTAL
                                gravity = Gravity.CENTER_VERTICAL
                                shells.indices.forEach { at ->
                                    addView(
                                        shellTab(at),
                                        LinearLayout.LayoutParams(
                                            WRAP_CONTENT,
                                            WRAP_CONTENT,
                                        ).apply { marginStart = gap(1) },
                                    )
                                }
                            },
                            FrameLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT),
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                if (shells.size > 1) {
                    addView(
                        shellControl(Mark.Shape.CLOSE, R.string.omni_shell_close) {
                            shellNow()?.let { closeOneShell(it) }
                            render(false)
                        },
                        LinearLayout.LayoutParams(gap(10), gap(10)),
                    )
                }
                addView(
                    shellControl(Mark.Shape.ADD, R.string.omni_shell_new) {
                        if (openAShell() != null) {
                            render(false)
                        }
                    }.apply { alpha = if (shells.size >= MOST_SHELLS) 0.35f else 1f },
                    LinearLayout.LayoutParams(gap(10), gap(10)),
                )
            },
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
        )
        frame.addView(
            view,
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply { weight = 1f },
        )
        frame.addView(shellKeys(view))
        content.addView(
            frame,
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply { weight = 1f },
        )

        view.whenResized { rows, columns ->
            val held = shellNow() ?: return@whenResized
            if (held.handle >= 0) {
                runCatching { Builder.nativeShellResize(held.handle, rows, columns) }
            }
        }
        view.setOnClickListener {
            view.requestFocus()
            view.windowInsetsController?.show(WindowInsets.Type.ime())
        }
        view.accept(byteArrayOf(), 0)
    }

    private fun writeToTheShell(bytes: ByteArray) {
        val shell = shellNow() ?: return
        if (shell.handle < 0 || bytes.isEmpty()) {
            return
        }
        runCatching { Builder.nativeShellWrite(shell.handle, bytes) }
    }

    private fun sendToTheShell(bytes: ByteArray) {
        if (bytes.isEmpty()) {
            return
        }
        val said = if (shellControl) {
            shellControl = false
            byteArrayOf((bytes[0].toInt().toChar().uppercaseChar().code and 0x1F).toByte())
        } else {
            bytes
        }
        writeToTheShell(said)
    }

    private fun closeOneShell(shell: Shell) {
        val handle = shell.handle
        shell.handle = -1
        shell.reader?.interrupt()
        shell.reader = null
        if (handle >= 0) {
            runCatching { Builder.nativeShellClose(handle) }
            OmniLog.event(LogLevel.INFO, "shell", "The shell was closed.")
        }
        val at = shells.indexOf(shell)
        if (at >= 0 && shells.size > 1) {
            shells.removeAt(at)
            if (shellAt >= shells.size) {
                shellAt = shells.size - 1
            }
        }
    }

    private fun closeTheShell() {
        shells.toList().forEach { shell ->
            val handle = shell.handle
            shell.handle = -1
            shell.reader?.interrupt()
            shell.reader = null
            if (handle >= 0) {
                runCatching { Builder.nativeShellClose(handle) }
            }
        }
        shells.clear()
        shellAt = 0
        OmniLog.event(LogLevel.INFO, "shell", "Every shell was closed.")
    }

    private fun shellKey(view: TerminalView, named: String, press: () -> Unit): View =
        TextView(this).apply {
            text = named
            typeface = Type.heading
            setTextColor(palette.foreground)
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
            gravity = Gravity.CENTER
            setPadding(0, gap(3), 0, gap(3))
            isClickable = true
            isFocusable = true
            readAs(Button::class.java.name)
            background = touchable(pill(Color.TRANSPARENT, gap(2).toFloat()), palette.accent)
            setOnClickListener {
                press()
                view.requestFocus()
            }
        }

    private fun shellKeys(view: TerminalView): View {
        val above = listOf<Pair<String, () -> Unit>>(
            "ESC" to { sendToTheShell(byteArrayOf(0x1B)) },
            "/" to { sendToTheShell("/".toByteArray()) },
            "—" to { sendToTheShell("-".toByteArray()) },
            "HOME" to { sendToTheShell("\u001B[H".toByteArray()) },
            "↑" to { sendToTheShell("\u001B[A".toByteArray()) },
            "END" to { sendToTheShell("\u001B[F".toByteArray()) },
            "PGUP" to { sendToTheShell("\u001B[5~".toByteArray()) },
        )
        val below = listOf<Pair<String, () -> Unit>>(
            "⇥" to { sendToTheShell(byteArrayOf(0x09)) },
            "←" to { sendToTheShell("\u001B[D".toByteArray()) },
            "CTRL" to { shellControl = !shellControl },
            "ALT" to { sendToTheShell(byteArrayOf(0x1B)) },
            "↓" to { sendToTheShell("\u001B[B".toByteArray()) },
            "→" to { sendToTheShell("\u001B[C".toByteArray()) },
            "PGDN" to { sendToTheShell("\u001B[6~".toByteArray()) },
        )
        val holder = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setBackgroundColor(Color.BLACK)
        }
        for (keys in listOf(above, below)) {
            holder.addView(
                LinearLayout(this).apply {
                    orientation = LinearLayout.HORIZONTAL
                    for ((named, press) in keys) {
                        addView(
                            shellKey(view, named, press),
                            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                        )
                    }
                }
            )
        }
        return holder
    }
    private fun talkBubble(mine: Boolean, said: String, lead: Boolean = false): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = if (mine) Gravity.END else Gravity.START
            setPadding(0, gap(2), 0, gap(1))
            if (!mine) {
                addView(
                    ImageView(context).apply {
                        setImageResource(R.drawable.omni_art_talk_head)
                        scaleType = ImageView.ScaleType.FIT_CENTER
                        importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                    },
                    LinearLayout.LayoutParams(gap(21), gap(21)),
                )
            }
            val cut = if (lead) said.indexOf('\n') else -1
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    setPadding(gap(3), gap(3), gap(3), gap(3))
                    background = GradientDrawable().apply {
                        shape = GradientDrawable.RECTANGLE
                        cornerRadius = gap(5) / 2f
                        setColor(if (mine) Ink.fade(palette.accent, 0.18f) else palette.raised)
                        setStroke(1, palette.divider)
                    }
                    if (cut > 0) {
                        addView(
                            TextView(context).apply {
                                text = said.substring(0, cut)
                                setTextColor(palette.foreground)
                                typeface = Type.heading
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 19f)
                                setPadding(0, 0, 0, gap(1) / 2)
                            }
                        )
                    }
                    addView(
                        TextView(context).apply {
                            text = if (cut > 0) said.substring(cut + 1) else said
                            setTextColor(Ink.hot(palette.muted, 0.34f))
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                            setLineSpacing(gap(1) / 2f, 1f)
                        }
                    )
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
        }

    private fun blocksIn(said: String): List<Pair<String, String>> {
        val out = mutableListOf<Pair<String, String>>()
        val lines = said.lines()
        var at = 0
        while (at < lines.size) {
            val line = lines[at]
            if (!line.startsWith("```")) {
                at += 1
                continue
            }
            val head = line.removePrefix("```").trim().split(Regex("\\s+"))
            val named = head.lastOrNull().orEmpty()
            var stop = at + 1
            while (stop < lines.size && !lines[stop].startsWith("```")) {
                stop += 1
            }
            if (head.size >= 2 && named.contains('/') && !named.contains("..")) {
                out.add(named to lines.subList(at + 1, stop).joinToString("\n"))
            }
            at = stop + 1
        }
        return out
    }

    private fun writeTheBlock(named: String, body: String) {
        val root = talkFolder()
        results.removeAllViews()
        if (root == null) {
            results.addView(notice(getString(R.string.omni_projects_none), palette.warning))
            return
        }
        val answer = runCatching {
            Builder.nativeWriteFile(root, named, if (body.endsWith("\n")) body else body + "\n")
        }.getOrElse {
            results.addView(notice(it.message ?: it.javaClass.simpleName, palette.error))
            return
        }
        act(answer, "written")
    }

    private class AgentStep(val path: String, val body: String?)

    private fun agentPath(named: String): Boolean =
        named.isNotEmpty() &&
            !named.contains("..") &&
            !named.startsWith("/") &&
            !named.contains('\\')

    private fun agentBody(lines: List<String>): String {
        var held = lines
        if (held.firstOrNull()?.trim()?.startsWith("```") == true &&
            held.lastOrNull { it.isNotBlank() }?.trim() == "```"
        ) {
            held = held.drop(1).dropLastWhile { it.isBlank() }.dropLast(1)
        }
        return held.joinToString("\n")
    }

    private fun agentSteps(said: String): List<AgentStep> {
        val out = mutableListOf<AgentStep>()
        val lines = said.lines()
        var at = 0
        while (at < lines.size) {
            val line = lines[at].trim()
            when {
                line.startsWith(AGENT_DELETE) -> {
                    val named = line.removePrefix(AGENT_DELETE).removeSuffix(AGENT_END).trim()
                    if (agentPath(named)) {
                        out.add(AgentStep(named, null))
                    }
                    at += 1
                }
                line.startsWith(AGENT_WRITE) -> {
                    val named = line.removePrefix(AGENT_WRITE).removeSuffix(AGENT_END).trim()
                    var stop = at + 1
                    while (stop < lines.size && lines[stop].trim() != AGENT_END) {
                        stop += 1
                    }
                    if (agentPath(named)) {
                        out.add(
                            AgentStep(named, agentBody(lines.subList(at + 1, min(stop, lines.size))))
                        )
                    }
                    at = stop + 1
                }
                else -> at += 1
            }
        }
        return out
    }

    private fun agentApply(root: String, steps: List<AgentStep>): List<String> {
        val told = mutableListOf<String>()
        steps.forEach { one ->
            val answer = runCatching {
                if (one.body == null) {
                    Builder.nativeRemovePath(root, one.path, projectTrash(root))
                } else {
                    Builder.nativeWriteFile(
                        root,
                        one.path,
                        if (one.body.endsWith("\n")) one.body else one.body + "\n",
                    )
                }
            }.getOrElse { why -> why.message ?: why.javaClass.simpleName }
            val document = runCatching { JSONObject(answer) }.getOrNull()
            val well = document?.optBoolean(if (one.body == null) "removed" else "written", false)
            if (well == true) {
                if (one.body != null && !agentWrote.contains(one.path)) {
                    agentWrote.add(one.path)
                }
                told.add(getString(R.string.omni_agent_wrote, one.path))
            } else {
                val refusal = document?.let { Refusal.parse(it) }
                told.add(
                    getString(
                        R.string.omni_agent_refused,
                        one.path,
                        listOfNotNull(refusal?.code, refusal?.message)
                            .joinToString("  ")
                            .ifEmpty { answer.take(200) },
                    )
                )
            }
        }
        return told
    }

    private fun liveBubble(): View = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        addView(
            LinearLayout(context).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.START
                setPadding(0, gap(2), 0, gap(1))
                addView(
                    ImageView(context).apply {
                        setImageResource(R.drawable.omni_art_talk_head)
                        scaleType = ImageView.ScaleType.FIT_CENTER
                        importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                    },
                    LinearLayout.LayoutParams(gap(21), gap(21)),
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        setPadding(gap(3), gap(3), gap(3), gap(3))
                        background = GradientDrawable().apply {
                            shape = GradientDrawable.RECTANGLE
                            cornerRadius = gap(5) / 2f
                            setColor(palette.raised)
                            setStroke(1, palette.divider)
                        }
                        addView(
                            TextView(context).apply {
                                text = talkPartial.ifEmpty {
                                    getString(R.string.omni_talk_thinking)
                                }
                                setTextColor(Ink.hot(palette.muted, 0.34f))
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                                setLineSpacing(gap(1) / 2f, 1f)
                                talkLive = this
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            },
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
        )
        addView(
            subtle(getString(R.string.omni_stage_stop), palette.error) { stopTheAssistant() },
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                marginStart = gap(21)
            },
        )
    }

    private fun talkFolder(): String? = talkRoot ?: openProject

    private fun chooseTalkFolder() {
        val known = runCatching {
            ProjectSummary.list(Builder.nativeListProjects(projectsFolder().absolutePath))
        }.getOrDefault(emptyList())
        if (known.isEmpty()) {
            results.removeAllViews()
            results.addView(notice(getString(R.string.omni_projects_none), palette.warning))
            return
        }
        val here = talkFolder()
        val list = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        lateinit var shown: AlertDialog
        known.forEach { one ->
            list.addView(
                LinearLayout(this).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    minimumHeight = gap(13)
                    setPadding(gap(2), gap(2), gap(2), gap(2))
                    isClickable = true
                    isFocusable = true
                    isSelected = one.root == here
                    readAs(android.widget.RadioButton::class.java.name)
                    contentDescription = said(one.label.ifEmpty { one.name }, one.root)
                    background = touchable(
                        GradientDrawable().apply {
                            shape = GradientDrawable.RECTANGLE
                            cornerRadius = gap(3).toFloat()
                            setColor(
                                if (one.root == here) Ink.fade(palette.accent, 0.12f)
                                else Color.TRANSPARENT
                            )
                            setStroke(
                                1,
                                if (one.root == here) palette.accent else Color.TRANSPARENT,
                            )
                        },
                        palette.accent,
                    )
                    setOnClickListener {
                        talkRoot = one.root
                        shown.dismiss()
                        render(false)
                    }
                    addView(
                        thumbnail(
                            File(one.root, "$PROJECT_RES/$PROJECT_ICON").absolutePath,
                            gap(9),
                        ),
                        LinearLayout.LayoutParams(gap(9), gap(9)).apply { marginEnd = gap(3) },
                    )
                    addView(
                        LinearLayout(context).apply {
                            orientation = LinearLayout.VERTICAL
                            addView(
                                TextView(context).apply {
                                    text = one.label.ifEmpty { one.name }
                                    setTextColor(palette.foreground)
                                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                                    maxLines = 1
                                }
                            )
                            addView(
                                TextView(context).apply {
                                    text = one.root
                                    setTextColor(palette.muted)
                                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                                    maxLines = 1
                                    ellipsize = android.text.TextUtils.TruncateAt.MIDDLE
                                }
                            )
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                    addView(
                        View(context).apply { background = radioMark(one.root == here) },
                        LinearLayout.LayoutParams(gap(5), gap(5)),
                    )
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
        }
        shown = dialogShell(
            getString(R.string.omni_talk_folder),
            list,
            listOf(getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() }),
        )
    }

    private fun talkMenu(under: View) {
        anchoredMenu(
            under,
            listOf(
                MenuChoice(Mark.Shape.TRASH, getString(R.string.omni_talk_clear)) {
                    talkHeld.clear()
                    render(false)
                },
                MenuChoice(Mark.Shape.SHARE, getString(R.string.omni_talk_export)) {
                    copyOut(
                        talkHeld.joinToString("\n\n") { it.second },
                        getString(R.string.omni_menu_assistant),
                    )
                },
                MenuChoice(Mark.Shape.FOLDER, getString(R.string.omni_talk_folder)) {
                    chooseTalkFolder()
                },
                MenuChoice(Mark.Shape.ROBOT, getString(R.string.omni_talk_switch)) {
                    chooseTheModel()
                },
                MenuChoice(
                    Mark.Shape.SPARK,
                    getString(R.string.omni_talk_agent),
                    picked = Preferences.assistantAgent(this),
                ) {
                    Preferences.setAssistantAgent(this, !Preferences.assistantAgent(this))
                    render(false)
                },
                MenuChoice(Mark.Shape.SHIELD, getString(R.string.omni_talk_providers)) {
                    chooseTheVendor()
                },
                MenuChoice(
                    Mark.Shape.HELP,
                    getString(R.string.omni_help_title),
                    parted = true,
                ) { go(Screen.Help) },
            ),
        )
    }

    private fun talkAttach(under: View) {
        val named = listOf(
            Mark.Shape.FILE to R.string.omni_talk_add_file,
            Mark.Shape.IMAGE to R.string.omni_talk_add_picture,
            Mark.Shape.LINK to R.string.omni_talk_add_link,
        )
        anchoredMenu(
            under,
            named.mapIndexed { at, one ->
                MenuChoice(one.first, getString(one.second)) {
                    if (at == 1) {
                        chooseShot()
                    } else {
                        askForName(getString(one.second), "") { said ->
                            if (said.isNotBlank()) {
                                talkHeld.add(true to said)
                                render(false)
                            }
                        }
                    }
                }
            },
        )
    }

    private fun renderAssistant() {
        if (talkHeld.isEmpty()) {
            talkHeld.add(false to getString(R.string.omni_talk_hello))
        }
        content.addView(assistantMeter())
        talkHeld.forEachIndexed { at, one ->
            content.addView(talkBubble(one.first, one.second, at == 0 && !one.first))
            if (!one.first) {
                blocksIn(one.second).forEach { (named, body) ->
                    content.addView(
                        subtle("${getString(R.string.omni_action_save)}  $named",
                            palette.ok) {
                            writeTheBlock(named, body)
                        }.apply {
                            layoutParams = LinearLayout.LayoutParams(
                                MATCH_PARENT,
                                WRAP_CONTENT,
                            ).apply { marginStart = gap(21) }
                        }
                    )
                }
            }
        }
        talkLive = null
        if (talkAsking) {
            content.addView(liveBubble())
        }
        content.addView(
            TextView(this).apply {
                text = SimpleDateFormat("HH:mm", Locale.getDefault()).format(Date())
                setTextColor(palette.muted)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                setPadding(gap(21), 0, 0, gap(3))
            }
        )

        val root = talkFolder()
        val where = card()
        where.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                addView(
                    filledMark(Mark.Shape.FOLDER, Ink.fade(palette.accent, 0.9f), gap(5) / 2),
                    LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(3) },
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_talk_folder)
                                setTextColor(palette.foreground)
                                typeface = Type.strong
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_talk_folder_note)
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            }
        )
        where.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(gap(3), gap(2), gap(3), gap(2))
                isClickable = true
                isFocusable = true
                readAs(Button::class.java.name)
                contentDescription = getString(R.string.omni_talk_folder)
                background = touchable(
                    GradientDrawable().apply {
                        this.shape = GradientDrawable.RECTANGLE
                        cornerRadius = gap(3).toFloat()
                        setColor(palette.raised)
                    },
                    palette.accent,
                )
                setOnClickListener { chooseTalkFolder() }
                addView(
                    thumbnail(
                        File(root.orEmpty(), "$PROJECT_RES/$PROJECT_ICON").absolutePath,
                        gap(10),
                    ),
                    LinearLayout.LayoutParams(gap(10), gap(10)).apply { marginEnd = gap(3) },
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = root?.let { File(it).name }
                                    ?: getString(R.string.omni_projects_none)
                                setTextColor(palette.foreground)
                                typeface = Type.strong
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = root.orEmpty()
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                                maxLines = 1
                                ellipsize = android.text.TextUtils.TruncateAt.MIDDLE
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                addView(
                    View(context).apply { foreground = Mark(Mark.Shape.CHEVRON_DOWN, palette.muted) },
                    LinearLayout.LayoutParams(gap(6), gap(6)),
                )
            }
        )
        content.addView(where)

        val writing = EditText(this).apply {
            setText(talkDraft)
            hint = getString(R.string.omni_talk_hint)
            setHintTextColor(palette.muted)
            setTextColor(palette.foreground)
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
            setPadding(gap(4), gap(3), gap(4), gap(3))
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(6).toFloat()
                setColor(palette.surface)
                setStroke(gap(1) / 2, palette.divider)
            }
            isSingleLine = false
            maxLines = 4
            addTextChangedListener(object : TextWatcher {
                override fun beforeTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
                override fun onTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
                override fun afterTextChanged(s: Editable?) {
                    talkDraft = s?.toString().orEmpty()
                }
            })
        }
        content.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.BOTTOM
                setPadding(0, gap(3), 0, gap(1))
                addView(
                    View(context).apply {
                        background = GradientDrawable().apply {
                            shape = GradientDrawable.OVAL
                            setColor(palette.surface)
                        }
                        foreground = Mark(Mark.Shape.CLIP, palette.foreground)
                        isClickable = true
                        isFocusable = true
                        contentDescription = getString(R.string.omni_talk_add_file)
                        readAs(Button::class.java.name)
                        setOnClickListener { talkAttach(this) }
                    },
                    LinearLayout.LayoutParams(gap(13), gap(13)).apply { marginEnd = gap(2) },
                )
                addView(writing, LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f))
                addView(
                    View(context).apply {
                        background = GradientDrawable().apply {
                            shape = GradientDrawable.OVAL
                            setColor(palette.accent)
                        }
                        foreground = Mark(Mark.Shape.SEND, palette.background)
                        isClickable = true
                        isFocusable = true
                        contentDescription = getString(R.string.omni_talk_send)
                        readAs(Button::class.java.name)
                        setOnClickListener { sendTyped() }
                    },
                    LinearLayout.LayoutParams(gap(13), gap(13)).apply { marginStart = gap(2) },
                )
            }
        )
        content.addView(
            TextView(this).apply {
                text = getString(R.string.omni_talk_warning)
                setTextColor(palette.muted)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                gravity = Gravity.CENTER
                setLineSpacing(gap(1).toFloat(), 1f)
                setPadding(gap(4), gap(2), gap(4), gap(2))
            }
        )
    }

    private fun assistantMeter(): View {
        val vendor = Brain.vendorOf(Preferences.assistantVendor(this))
        val model = Brain.modelOf(vendor, Preferences.assistantModel(this))
        val used = if (talkWindowUsed > 0L) {
            talkWindowUsed
        } else {
            Brain.roughly(
                talkHeld.drop(1).map { Brain.Turn(it.first, it.second) },
                assistantSystem(talkFolder()),
            )
        }
        val filled = if (model.window > 0L) {
            (used.toFloat() / model.window.toFloat()).coerceIn(0f, 1f)
        } else {
            0f
        }
        val tint = when {
            filled >= 0.9f -> palette.error
            filled >= 0.7f -> palette.warning
            else -> palette.ok
        }
        return card().apply {
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    addView(
                        TextView(context).apply {
                            text = vendor.shown + "  \u00b7  " + model.shown
                            setTextColor(palette.foreground)
                            typeface = Type.strong
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                            isClickable = true
                            isFocusable = true
                            contentDescription = getString(R.string.omni_talk_model)
                            readAs(Button::class.java.name)
                            setPadding(gap(2), gap(1), gap(2), gap(1))
                            maxLines = 1
                            pressable(onHeld = { chooseTheVendor() })
                            setOnClickListener { chooseTheModel() }
                        },
                        LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT),
                    )
                    addView(
                        TextView(context).apply {
                            text = buildString {
                                append(getString(R.string.omni_talk_window))
                                append("  ")
                                append(countable(used))
                                append(" / ")
                                append(countable(model.window))
                                append("  ")
                                append("%${(filled * 100f).toInt()}")
                            }
                            setTextColor(palette.muted)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                            gravity = Gravity.END
                            maxLines = 1
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
            addView(
                FrameLayout(context).apply {
                    background = pill(Ink.fade(palette.divider, 0.6f), gap(1).toFloat())
                    addView(
                        View(context).apply { background = pill(tint, gap(1).toFloat()) },
                        FrameLayout.LayoutParams(0, MATCH_PARENT).apply {
                            width = 0
                        },
                    )
                    post {
                        val bar = getChildAt(0) ?: return@post
                        bar.layoutParams = FrameLayout.LayoutParams(
                            (width * filled).toInt().coerceAtLeast(if (filled > 0f) 1 else 0),
                            MATCH_PARENT,
                        )
                        bar.requestLayout()
                    }
                    importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                },
                LinearLayout.LayoutParams(MATCH_PARENT, gap(1)).apply {
                    topMargin = gap(2)
                    bottomMargin = gap(1)
                },
            )
            val said = mutableListOf<String>()
            if (talkLeftRequests >= 0L) {
                said.add(
                    "${getString(R.string.omni_talk_left)} ${countable(talkLeftRequests)}"
                )
            }
            if (talkLeftTokens >= 0L) {
                said.add(countable(talkLeftTokens))
            }
            if (talkResetsAt.isNotEmpty()) {
                said.add(talkResetsAt)
            }
            if (talkSpentIn > 0L || talkSpentOut > 0L) {
                said.add("↑${countable(talkSpentIn)} ↓${countable(talkSpentOut)}")
            }
            if (said.isNotEmpty()) {
                addView(quiet(said.joinToString("  ·  ")))
            }
            if (talkAsking) {
                addView(quiet(getString(R.string.omni_talk_thinking)))
            }
        }
    }

    private fun chooseTheModel() {
        var shown: AlertDialog? = null
        val vendor = Brain.vendorOf(Preferences.assistantVendor(this))
        val here = Preferences.assistantModel(this)
        val fetched = Preferences.fetchedModels(this, vendor.id)
        val named = (vendor.models.map { it.id } + fetched).distinct()
        val list = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        named.forEachIndexed { at, id ->
            val one = Brain.modelOf(vendor, id)
            if (at > 0) {
                list.addView(rule(), LinearLayout.LayoutParams(MATCH_PARENT, 1))
            }
            list.addView(
                LinearLayout(this).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    minimumHeight = gap(ROW_UNITS)
                    setPadding(gap(4), gap(2), gap(4), gap(2))
                    isClickable = true
                    isFocusable = true
                    contentDescription = one.shown
                    readAs(Button::class.java.name)
                    pressable()
                    setOnClickListener {
                        Preferences.setAssistantModel(this@BuilderActivity, one.id)
                        shown?.dismiss()
                        render(false)
                    }
                    addView(
                        LinearLayout(context).apply {
                            orientation = LinearLayout.VERTICAL
                            addView(
                                TextView(context).apply {
                                    text = one.shown
                                    setTextColor(palette.foreground)
                                    typeface = Type.strong
                                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                                }
                            )
                            addView(
                                TextView(context).apply {
                                    text = "${one.id}  ·  ${countable(one.window)}"
                                    setTextColor(palette.muted)
                                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                                    maxLines = 1
                                }
                            )
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                    addView(
                        View(context).apply { background = radioMark(one.id == here) },
                        LinearLayout.LayoutParams(gap(5), gap(5)),
                    )
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
        }
        val body = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            addView(
                subtle(vendor.shown, palette.accent) {
                    shown?.dismiss()
                    chooseTheVendor()
                }
            )
            addView(list, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            if (!vendor.anthropic) {
                addView(
                    subtle(getString(R.string.omni_talk_models_fetch), palette.accent) {
                        shown?.dismiss()
                        fetchTheModels(vendor)
                    }
                )
            }
        }
        shown = dialogShell(
            getString(R.string.omni_talk_model),
            body,
            listOf(getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() }),
        )
    }

    private fun fetchTheModels(vendor: Brain.Vendor) {
        results.removeAllViews()
        val sealed = Preferences.vendorKey(this, vendor.id)
        if (sealed.isEmpty() && !vendor.keyless) {
            askToSignIn(vendor)
            return
        }
        working({
            val opened = if (sealed.isEmpty()) {
                CharArray(0)
            } else {
                Vault.open(sealed) ?: return@working ""
            }
            val found = try {
                Brain.listed(vendor, opened)
            } finally {
                opened.fill(' ')
            }
            found.joinToString("\n")
        }) finished@{ answer ->
            val found = answer.split('\n').filter { it.isNotBlank() }
            if (found.isEmpty()) {
                results.addView(notice(getString(R.string.omni_talk_models_none), palette.warning))
                return@finished
            }
            Preferences.setFetchedModels(this, vendor.id, found)
            results.addView(
                notice(getString(R.string.omni_talk_models_got, found.size), palette.ok)
            )
            chooseTheModel()
        }
    }

    private fun countable(value: Long): String = when {
        value >= 1_000_000L -> "${value / 1_000_000L}M"
        value >= 1_000L -> "${value / 1_000L}K"
        else -> value.toString()
    }

    private fun vendorsToTry(): List<Brain.Vendor> {
        val now = System.currentTimeMillis()
        val here = Brain.vendorOf(Preferences.assistantVendor(this))
        val after = Brain.VENDORS.filter {
            it.id != here.id &&
                it.keyed(this) &&
                Preferences.restingUntil(this, it.id) <= now
        }
        return listOf(here) + after
    }

    private fun vendorFaulted(vendor: Brain.Vendor) {
        val many = Preferences.faults(this, vendor.id) + 1
        if (many >= Brain.FAULTS_BEFORE_REST) {
            Preferences.setFaults(this, vendor.id, 0)
            Preferences.setRestingUntil(
                this,
                vendor.id,
                System.currentTimeMillis() + Brain.REST_MILLIS,
            )
        } else {
            Preferences.setFaults(this, vendor.id, many)
        }
    }

    private fun vendorSteady(vendor: Brain.Vendor) {
        Preferences.setFaults(this, vendor.id, 0)
        Preferences.setRestingUntil(this, vendor.id, 0L)
    }

    private fun worthAnotherVendor(why: Throwable): Boolean {
        val refused = why as? Brain.Refused ?: return false
        return refused.said == R.string.omni_talk_spent ||
            refused.said == R.string.omni_talk_busy ||
            refused.said == R.string.omni_talk_server ||
            refused.said == R.string.omni_talk_unreachable ||
            refused.said == R.string.omni_talk_no_model
    }

    private fun sendTyped() {
        agentRound = 0
        agentWrote = mutableListOf()
        sendToTheAssistant()
    }

    private fun sendToTheAssistant() {
        val said = talkDraft.trim()
        if (said.isEmpty() || talkAsking) {
            return
        }
        val tries = vendorsToTry().filter { it.keyed(this) }
        if (tries.isEmpty()) {
            talkHeld.add(true to said)
            talkHeld.add(false to getString(R.string.omni_talk_no_key))
            talkDraft = ""
            render(false)
            chooseTheVendor()
            return
        }
        talkHeld.add(true to said)
        talkDraft = ""
        talkAsking = true
        talkPartial = ""
        talkStopping.set(false)
        render(false)

        val turns = mutableListOf<Brain.Turn>()
        talkHeld.drop(1).forEach { one ->
            val last = turns.lastOrNull()
            if (last != null && last.mine == one.first) {
                turns[turns.size - 1] = Brain.Turn(one.first, last.said + "\n\n" + one.second)
            } else {
                turns.add(Brain.Turn(one.first, one.second))
            }
        }
        while (turns.isNotEmpty() && !turns.first().mine) {
            turns.removeAt(0)
        }
        val effort = Preferences.assistantEffort(this)
        val where = talkFolder()
        val chosen = tries.first().id
        val sealed = tries.associate { it.id to Preferences.vendorKey(this, it.id) }
        val models = tries.associate { it.id to Preferences.modelFor(this, it.id) }

        Thread {
            val system = assistantSystem(where)
            var held: Brain.Answer? = null
            var answered: Brain.Vendor? = null
            var why: Throwable? = null
            for (vendor in tries) {
                val locked = sealed[vendor.id].orEmpty()
                if (locked.isEmpty() && !vendor.keyless) {
                    continue
                }
                talkPartial = ""
                val outcome = runCatching {
                    val opened = if (locked.isEmpty()) {
                        CharArray(0)
                    } else {
                        Vault.open(locked)
                            ?: throw Brain.Refused(R.string.omni_talk_bad_key)
                    }
                    try {
                        Brain.stream(
                            vendor,
                            opened,
                            turns,
                            models[vendor.id].orEmpty(),
                            effort,
                            system,
                            talkStopping,
                        ) { part ->
                            runOnUiThread {
                                if (isFinishing || isDestroyed) {
                                    return@runOnUiThread
                                }
                                talkPartial += part
                                talkLive?.text = talkPartial
                            }
                        }
                    } finally {
                        opened.fill(' ')
                    }
                }
                if (outcome.isSuccess) {
                    held = outcome.getOrNull()
                    answered = vendor
                    runOnUiThread { vendorSteady(vendor) }
                    break
                }
                why = outcome.exceptionOrNull()
                runOnUiThread { vendorFaulted(vendor) }
                if (talkStopping.get() || talkPartial.isNotEmpty()) {
                    break
                }
                if (why == null || !worthAnotherVendor(why)) {
                    break
                }
            }
            runOnUiThread {
                talkAsking = false
                talkLive = null
                if (isFinishing || isDestroyed) {
                    return@runOnUiThread
                }
                var replied = ""
                if (held != null) {
                    if (answered != null && answered.id != chosen) {
                        talkHeld.add(
                            false to getString(R.string.omni_talk_instead, answered.shown)
                        )
                    }
                    val spoken = held.said.ifEmpty { talkPartial }
                    replied = spoken
                    talkHeld.add(
                        false to spoken.ifEmpty { getString(R.string.omni_refused) }
                    )
                    if (held.stopped == "max_tokens") {
                        talkHeld.add(false to getString(R.string.omni_talk_cut))
                    }
                    talkSpentIn += held.spend.input + held.spend.cacheWritten
                    talkSpentOut += held.spend.output
                    talkWindowUsed = held.spend.window() + held.spend.output
                    if (held.limits.known()) {
                        talkLeftRequests = held.limits.requestsLeft
                        talkLeftTokens = held.limits.tokensLeft
                        talkResetsAt = held.limits.resetsAt
                    }
                } else {
                    if (talkPartial.isNotEmpty()) {
                        talkHeld.add(false to talkPartial)
                    }
                    val refused = why as? Brain.Refused
                    val text = if (refused != null) {
                        val head = getString(refused.said)
                        if (refused.detail.isEmpty()) {
                            head
                        } else {
                            head + "  (" + refused.detail + ")"
                        }
                    } else {
                        why?.message ?: getString(R.string.omni_talk_refused)
                    }
                    talkHeld.add(false to text)
                }
                talkPartial = ""
                render(false)
                if (replied.isNotEmpty()) {
                    agentFollow(replied)
                }
            }
        }.start()
    }

    private fun agentFollow(said: String) {
        if (!Preferences.assistantAgent(this)) {
            return
        }
        val root = talkFolder() ?: return
        val steps = agentSteps(said)
        val told = if (steps.isEmpty()) emptyList() else agentApply(root, steps)
        val ended = said.contains(AGENT_DONE)
        agentRound += 1
        if (steps.isNotEmpty() && !ended && agentRound < AGENT_ROUNDS) {
            talkDraft = told.joinToString("\n") + "\n\n" + AGENT_CARRY
            render(false)
            sendToTheAssistant()
            return
        }
        if (told.isNotEmpty()) {
            talkHeld.add(false to told.joinToString("\n"))
        }
        if (!ended && steps.isNotEmpty()) {
            talkHeld.add(false to getString(R.string.omni_agent_stopped, agentRound))
        }
        val made = agentWrote.toList()
        agentRound = 0
        agentWrote = mutableListOf()
        render(false)
        if (made.isNotEmpty()) {
            agentOffersToBuild(root, made.size)
        }
    }

    private fun agentOffersToBuild(root: String, many: Int) {
        dialogShell(
            getString(R.string.omni_agent_ready, many),
            body(summaryOf(root)?.label?.ifEmpty { null } ?: File(root).name),
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() },
                getString(R.string.omni_action_build_both) to { it: AlertDialog ->
                    it.dismiss()
                    agentAsksForTheKey(root)
                },
            ),
        )
    }

    private fun agentAsksForTheKey(root: String) {
        val keys = keysHere()
        if (keys.isEmpty()) {
            go(Screen.NewKey)
            return
        }
        val named = keys.map { it.alias } + getString(R.string.omni_device_key)
        pickOne(getString(R.string.omni_agent_sign), named) { which ->
            if (which >= keys.size) {
                go(Screen.Build(root))
                buildWithTheDeviceKey(root)
            } else {
                Preferences.setSigningKey(this, keys[which].path)
                agentAsksForThePassword(root, keys[which])
            }
        }
    }

    private fun agentAsksForThePassword(root: String, key: SigningKey) {
        val shared = key.alias == DEFAULT_KEY_ALIAS
        val editor = styledField(
            if (shared) DEFAULT_KEY_PASSWORD else "",
            getString(R.string.omni_build_password),
            true,
        ).apply { typeface = Type.data }
        dialogShell(
            key.alias,
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(0, gap(1), 0, gap(2))
                addView(editor, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
                addView(
                    quiet(
                        if (shared) {
                            getString(R.string.omni_build_shared_password, DEFAULT_KEY_PASSWORD)
                        } else {
                            getString(R.string.omni_build_your_password)
                        }
                    )
                )
            },
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() },
                getString(R.string.omni_action_build_both) to { it: AlertDialog ->
                    val held = readSecret(editor)
                    if (held.isEmpty()) {
                        results.removeAllViews()
                        results.addView(
                            notice(getString(R.string.omni_build_needs_password), palette.warning)
                        )
                    } else {
                        buildSecret = Vault.seal(held)
                        held.fill(' ')
                        editor.text?.clear()
                        it.dismiss()
                        go(Screen.Build(root))
                        mendThenBuild(root)
                    }
                },
            ),
        )
        editor.requestFocus()
    }

    private fun stopTheAssistant() {
        if (!talkAsking) {
            return
        }
        talkStopping.set(true)
    }

    private fun assistantSystem(root: String?): String {
        if (root == null) {
            return ""
        }
        val here = File(root)
        if (!here.isDirectory) {
            return ""
        }
        val tree = runCatching {
            FileEntry.list(Builder.nativeProjectTree(root))
                .filter { !it.folder }
                .take(ASSISTANT_PATHS)
                .joinToString("\n") { it.path }
        }.getOrDefault("")
        val said = StringBuilder()
        said.append("The reader is working in an Android project named ")
            .append(here.name)
            .append(". ")
        if (Preferences.assistantAgent(this)) {
            said.append(AGENT_RULES)
        } else {
            said.append(
                "When you hand back a whole file, open its fence with the language and the " +
                    "path inside the project, like ```java Java/com/tr/yt/Main.java, so it " +
                    "can be written where it belongs."
            )
        }
        if (tree.isNotEmpty()) {
            said.append("\nThese are the files it already holds:\n").append(tree)
        }
        return said.toString()
    }

    private fun logButton(mark: Mark.Shape, named: Int, filled: Boolean, onPress: () -> Unit) =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER
            minimumHeight = gap(TOUCH_UNITS)
            setPadding(gap(2), gap(3), gap(2), gap(3))
            isClickable = true
            isFocusable = true
            contentDescription = getString(named)
            readAs(Button::class.java.name)
            background = touchable(
                GradientDrawable().apply {
                    shape = GradientDrawable.RECTANGLE
                    cornerRadius = gap(3).toFloat()
                    setColor(if (filled) palette.accent else palette.raised)
                },
                if (filled) palette.background else palette.accent,
            )
            setOnClickListener { onPress() }
            addView(
                View(context).apply {
                    foreground = Mark(mark, if (filled) palette.background else palette.foreground)
                },
                LinearLayout.LayoutParams(gap(5), gap(5)).apply { marginEnd = gap(2) },
            )
            addView(
                TextView(context).apply {
                    text = getString(named)
                    setTextColor(if (filled) palette.background else palette.foreground)
                    typeface = Type.strong
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                    maxLines = 1
                }
            )
        }

    private fun dropRow(named: String, shown: String, choices: List<String>, onPick: (Int) -> Unit): View {
        val value = TextView(this).apply {
            text = shown
            setTextColor(palette.foreground)
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
        }
        return LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(0, gap(2), 0, gap(1))
            addView(
                TextView(context).apply {
                    text = named
                    setTextColor(palette.muted)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                    setPadding(0, 0, 0, gap(1))
                }
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    setPadding(gap(3), gap(3), gap(3), gap(3))
                    isClickable = true
                    isFocusable = true
                    contentDescription = named
                    readAs(android.widget.Spinner::class.java.name)
                    background = touchable(
                        GradientDrawable().apply {
                            shape = GradientDrawable.RECTANGLE
                            cornerRadius = gap(2).toFloat()
                            setColor(palette.raised)
                        },
                        palette.accent,
                    )
                    setOnClickListener {
                        pickOne(named, choices) { which ->
                            value.text = choices[which]
                            onPick(which)
                        }
                    }
                    addView(value, LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f))
                    addView(
                        View(context).apply {
                            foreground = Mark(Mark.Shape.CHEVRON_DOWN, palette.muted)
                        },
                        LinearLayout.LayoutParams(gap(6), gap(6)),
                    )
                }
            )
        }
    }

    private fun askToClone() {
        val sheet = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(5), gap(4), gap(5), gap(2))
        }
        sheet.addView(
            dropRow(
                getString(R.string.omni_clone_kind),
                CLONE_KINDS.first(),
                CLONE_KINDS,
            ) {}
        )
        val (whereHolder, whereField) = input(
            getString(R.string.omni_clone_address),
            cloneAddress,
        )
        whereField.hint = "https://github.com/user/repo.git"
        sheet.addView(whereHolder)

        val (tokenHolder, tokenField) = input(
            getString(R.string.omni_clone_auth_token),
            cloneToken,
        )
        tokenHolder.visibility = View.GONE
        sheet.addView(
            dropRow(
                getString(R.string.omni_clone_auth),
                getString(R.string.omni_clone_auth_none),
                listOf(
                    getString(R.string.omni_clone_auth_none),
                    getString(R.string.omni_clone_auth_token),
                ),
            ) { picked -> tokenHolder.visibility = if (picked == 1) View.VISIBLE else View.GONE }
        )
        sheet.addView(tokenHolder)

        var whole = false
        sheet.addView(
            dropRow(
                getString(R.string.omni_clone_history),
                getString(R.string.omni_clone_history_last),
                listOf(
                    getString(R.string.omni_clone_history_last),
                    getString(R.string.omni_clone_history_all),
                ),
            ) { picked -> whole = picked == 1 }
        )

        val (intoHolder, intoField) = input(
            getString(R.string.omni_clone_into),
            projectsFolder().absolutePath,
        )
        sheet.addView(intoHolder)

        dialogShell(
            getString(R.string.omni_clone_title),
            sheet,
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() },
                getString(R.string.omni_clone_go) to { it: AlertDialog ->
                    cloneAddress = whereField.text?.toString().orEmpty().trim()
                    cloneToken = tokenField.text?.toString().orEmpty().trim()
                    val into = intoField.text?.toString().orEmpty().trim()
                    it.dismiss()
                    cloneFrom(cloneAddress, cloneToken, into, !whole)
                },
            ),
        )
    }

    private fun refusalOf(why: Throwable): String = when (why) {
        is Remote.Refused ->
            if (why.detail.isEmpty()) {
                getString(why.said)
            } else {
                getString(why.said) + " " + why.detail
            }
        else -> getString(R.string.omni_clone_unreachable)
    }

    private fun cloneFrom(said: String, token: String, into: String, shallow: Boolean) {
        results.removeAllViews()
        val address = try {
            Remote.address(said)
        } catch (why: Remote.Refused) {
            results.addView(notice(refusalOf(why), palette.error))
            return
        }
        val name = Remote.named(address)
        val where = if (into.isEmpty()) projectsFolder() else File(into)
        val target = File(where, name)
        if (target.isDirectory && target.list()?.isNotEmpty() == true) {
            results.addView(notice(getString(R.string.omni_clone_here), palette.warning))
            return
        }

        val told = card()
        told.addView(label(getString(R.string.omni_clone_title)))
        val telling = body(getString(R.string.omni_clone_reaching))
        told.addView(telling)
        told.addView(
            subtle(getString(R.string.omni_action_cancel), palette.error) { cloning = false }
        )
        results.addView(told)

        cloning = true
        val pack = File(cacheDir, "omni-clone.pack")
        val thread = Thread {
            var spoke = 0L
            val answer = runCatching {
                val found = Remote.advertise(address, token)
                runOnUiThread {
                    telling.text = getString(R.string.omni_clone_fetching, found.branch)
                }
                val carried = Remote.fetch(
                    address,
                    token,
                    found.head,
                    shallow && found.shallow,
                    pack,
                ) { bytes ->
                    val now = System.currentTimeMillis()
                    if (now - spoke > CLONE_TELL_MILLIS) {
                        spoke = now
                        runOnUiThread {
                            telling.text = getString(R.string.omni_clone_carried, size(bytes))
                        }
                    }
                    cloning
                }
                runOnUiThread { telling.text = getString(R.string.omni_clone_writing) }
                val report = Builder.nativeGitCheckout(
                    pack.absolutePath,
                    found.head,
                    target.absolutePath,
                )
                carried to JSONObject(report)
            }
            pack.delete()
            runOnUiThread { cloneEnded(name, target, answer) }
        }
        thread.isDaemon = true
        thread.start()
    }

    private fun cloneEnded(
        name: String,
        target: File,
        answer: Result<Pair<Long, JSONObject>>,
    ) {
        cloning = false
        results.removeAllViews()
        val got = answer.getOrElse { why ->
            target.deleteRecursively()
            OmniLog.event(LogLevel.WARN, "clone", "A clone did not finish.")
            results.addView(notice(refusalOf(why), palette.error))
            return
        }
        val report = got.second
        if (!report.optBoolean("cloned")) {
            target.deleteRecursively()
            val why = report.optString("reason").ifEmpty {
                getString(R.string.omni_clone_not_git)
            }
            results.addView(notice(why, palette.error))
            return
        }
        val files = report.optLong("files")
        val folders = report.optLong("folders")
        OmniLog.event(LogLevel.INFO, "clone", "A repository was cloned.")
        flash(
            getString(R.string.omni_clone_done, name, files, folders),
            palette.ok,
            Mark.Shape.BRANCH,
        )
        go(Screen.Files(target.absolutePath, ""))
    }

    private fun factRow(
        named: String,
        said: String,
        mark: Mark.Shape = Mark.Shape.COPY,
        spoken: Int = R.string.omni_action_copy,
        onPress: (() -> Unit)? = null,
    ): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(0, gap(2), 0, gap(2))
            addView(
                TextView(context).apply {
                    text = named
                    setTextColor(palette.muted)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                },
                LinearLayout.LayoutParams(gap(34), WRAP_CONTENT),
            )
            addView(
                TextView(context).apply {
                    text = said
                    setTextColor(palette.foreground)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            if (onPress != null) {
                addView(
                    View(context).apply {
                        foreground = Mark(mark, palette.foreground)
                        isClickable = true
                        isFocusable = true
                        contentDescription = getString(spoken)
                        readAs(Button::class.java.name)
                        background = reacting(Color.TRANSPARENT, gap(5).toFloat(), palette.accent)
                        setOnClickListener { onPress() }
                    },
                    LinearLayout.LayoutParams(gap(6), gap(6)),
                )
            }
        }

    private fun madeAt(here: File): Long = runCatching {
        java.nio.file.Files.readAttributes(
            here.toPath(),
            java.nio.file.attribute.BasicFileAttributes::class.java,
        ).creationTime().toMillis()
    }.getOrNull()?.takeIf { it > 0L } ?: here.lastModified()

    private fun countedIn(root: String): Pair<Int, Int> {
        var files = 0
        var folders = 0
        File(root).walkTopDown().forEach { if (it.isDirectory) folders += 1 else files += 1 }
        return files to (folders - 1).coerceAtLeast(0)
    }

    private fun mainActivityOf(root: String): String = runCatching {
        JSONObject(Builder.nativeManifestFacts(root))
            .optJSONObject("manifest")
            ?.optJSONArray("activities")
            ?.takeIf { it.length() > 0 }
            ?.optString(0)
            .orEmpty()
            .substringAfterLast('.')
    }.getOrDefault("")

    private fun factPair(named: String, said: String): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(0, gap(1), 0, gap(1))
            addView(
                TextView(context).apply {
                    text = named
                    setTextColor(palette.muted)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                }
            )
            addView(
                TextView(context).apply {
                    text = said
                    setTextColor(palette.foreground)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                }
            )
        }

    private fun badgedIcon(root: String, edge: Int): View = FrameLayout(this).apply {
        addView(
            thumbnail(File(root, "$PROJECT_RES/$PROJECT_ICON").absolutePath, edge),
            FrameLayout.LayoutParams(edge, edge),
        )
        addView(
            View(context).apply {
                background = GradientDrawable().apply {
                    shape = GradientDrawable.OVAL
                    setColor(palette.raised)
                    setStroke(gap(1) / 2, palette.divider)
                }
                foreground = Mark(Mark.Shape.PEN, palette.foreground)
                isClickable = true
                isFocusable = true
                contentDescription = getString(R.string.omni_project_picture)
                readAs(Button::class.java.name)
                setOnClickListener { chooseImage(root) }
            },
            FrameLayout.LayoutParams(gap(9), gap(9), Gravity.END or Gravity.BOTTOM),
        )
    }

    private fun showProjectFacts(root: String) {
        val summary = summaryOf(root)
        val here = File(root)
        val (files, folders) = countedIn(root)
        val sheet = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(5), gap(1), gap(5), gap(3))
        }
        sheet.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.TOP
                setPadding(0, gap(3), 0, gap(3))
                addView(
                    badgedIcon(root, gap(20)),
                    LinearLayout.LayoutParams(gap(20), gap(20)).apply { marginEnd = gap(5) },
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(factPair(getString(R.string.omni_facts_name), here.name))
                        addView(
                            factPair(
                                getString(R.string.omni_form_package),
                                summary?.packageName.orEmpty(),
                            )
                        )
                        val started = mainActivityOf(root)
                        if (started.isNotEmpty()) {
                            addView(
                                factPair(getString(R.string.omni_facts_activity), started)
                            )
                        }
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            }
        )
        sheet.addView(rule(), LinearLayout.LayoutParams(MATCH_PARENT, 1))
        sheet.addView(factRow(getString(R.string.omni_facts_where), root) {
            copyOut(root, getString(R.string.omni_facts_where))
        })
        sheet.addView(
            factRow(getString(R.string.omni_facts_made), moment(madeAt(here) / 1000))
        )
        sheet.addView(
            factRow(getString(R.string.omni_facts_changed), moment(here.lastModified() / 1000))
        )
        sheet.addView(
            factRow(
                getString(R.string.omni_facts_holds),
                getString(R.string.omni_facts_counted, files, folders),
            )
        )
        sheet.addView(factRow(getString(R.string.omni_facts_kind), getString(R.string.omni_facts_java)))
        sheet.addView(
            factRow(
                getString(R.string.omni_facts_made_by),
                getString(R.string.omni_app_shown, versionShown()),
            )
        )
        sheet.addView(
            factRow(
                getString(R.string.omni_facts_span),
                getString(
                    R.string.omni_form_platforms_span,
                    releaseNamed(summary?.minSdk ?: OLDEST_PLATFORM),
                    releaseNamed(summary?.targetSdk ?: NEWEST_PLATFORM),
                ),
            )
        )
        sheet.addView(
            rule(),
            LinearLayout.LayoutParams(MATCH_PARENT, 1).apply { topMargin = gap(3) },
        )
        showSheet(Mark.Shape.INFO_FILL, getString(R.string.omni_facts_title), sheet)
    }

    private fun showSheet(marked: Mark.Shape, titled: String, body: View) {
        val panel = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(4).toFloat()
                setColor(palette.background)
                setStroke(gap(1) / 2, palette.divider)
            }
        }
        panel.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(gap(5), gap(4), gap(5), gap(4))
                addView(
                    View(context).apply { foreground = Mark(marked, palette.foreground) },
                    LinearLayout.LayoutParams(gap(8), gap(8)).apply { marginEnd = gap(3) },
                )
                addView(
                    TextView(context).apply {
                        text = titled
                        setTextColor(palette.foreground)
                        typeface = Type.heading
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 19f)
                    }
                )
            }
        )
        panel.addView(rule(), LinearLayout.LayoutParams(MATCH_PARENT, 1))
        val scroll = ScrollView(this).apply { addView(body, MATCH_PARENT, WRAP_CONTENT) }
        panel.addView(
            scroll,
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply { weight = 1f },
        )
        val shown = AlertDialog.Builder(this).setView(panel).create()
        panel.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.END
                setPadding(gap(5), gap(2), gap(5), gap(4))
                addView(
                    control().apply {
                        text = getString(R.string.omni_action_confirm)
                        setTextColor(Color.WHITE)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                        setPadding(gap(7), gap(3), gap(7), gap(3))
                        background = reacting(palette.accent, gap(2).toFloat(), Color.WHITE)
                        setOnClickListener { shown.dismiss() }
                    },
                    LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT),
                )
            }
        )
        shown.window?.setBackgroundDrawable(ColorDrawable(Color.TRANSPARENT))
        shown.show()
    }

    private fun clearProjectCache(root: String) {
        val named = File(root).name
        var gone = 0
        runCatching {
            outputsFolder(root).listFiles()?.forEach { one ->
                if (one.deleteRecursively()) {
                    gone += 1
                }
            }
            builtFolder().listFiles()?.forEach { one ->
                if (one.name.startsWith(named) && one.delete()) {
                    gone += 1
                }
            }
        }
        results.removeAllViews()
        results.addView(notice(getString(R.string.omni_project_cleared, gone), palette.ok))
    }

    private fun shareProject(root: String) {
        val here = File(root)
        working({
            val into = File(cacheDir, "${here.name}.zip")
            java.util.zip.ZipOutputStream(FileOutputStream(into)).use { out ->
                here.walkTopDown().filter { it.isFile }.forEach { one ->
                    val at = one.absolutePath.removePrefix(here.absolutePath).trimStart('/')
                    out.putNextEntry(java.util.zip.ZipEntry(at))
                    one.inputStream().use { it.copyTo(out) }
                    out.closeEntry()
                }
            }
            into.absolutePath
        }) { where -> shareOutside(File(where)) }
    }

    private fun projectMenu(root: String, under: View) {
        val summary = summaryOf(root)
        anchoredMenu(
            under,
            listOf(
                MenuChoice(Mark.Shape.PEN, getString(R.string.omni_project_rename)) {
                    askForName(
                        getString(R.string.omni_project_rename),
                        File(root).name,
                    ) { asked ->
                        val into = File(File(root).parentFile, asked.trim())
                        results.removeAllViews()
                        if (asked.isBlank() || into.exists() || !File(root).renameTo(into)) {
                            results.addView(
                                notice(getString(R.string.omni_files_needs_name), palette.error)
                            )
                        } else {
                            if (openProject == root) {
                                openProject = into.absolutePath
                            }
                            go(Screen.Files(into.absolutePath, ""))
                        }
                    }
                },
                MenuChoice(Mark.Shape.IMAGE, getString(R.string.omni_project_picture)) {
                    chooseImage(root)
                },
                MenuChoice(Mark.Shape.INFO, getString(R.string.omni_facts_title)) {
                    showProjectFacts(root)
                },
                MenuChoice(
                    Mark.Shape.BROOM,
                    getString(R.string.omni_project_clear),
                    parted = true,
                ) { clearProjectCache(root) },
                MenuChoice(Mark.Shape.SHARE, getString(R.string.omni_project_share)) {
                    shareProject(root)
                },
                MenuChoice(
                    Mark.Shape.TRASH,
                    getString(R.string.omni_project_delete),
                    tint = palette.error,
                    parted = true,
                ) { summary?.let { deleteProject(it) } },
            ),
        )
    }

    private fun renderLogcat() {
        content.addView(
            rule(),
            LinearLayout.LayoutParams(MATCH_PARENT, 1).apply { bottomMargin = gap(3) },
        )

        val reading = logReadsAll()
        val bound = card()
        bound.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                addView(
                    View(context).apply {
                        foreground = Mark(
                            Mark.Shape.PHONE,
                            if (reading) palette.ok else palette.accent,
                        )
                    },
                    LinearLayout.LayoutParams(gap(9), gap(9)).apply { marginEnd = gap(3) },
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_log_device)
                                setTextColor(palette.foreground)
                                typeface = Type.strong
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                            }
                        )
                        addView(
                            LinearLayout(context).apply {
                                orientation = LinearLayout.HORIZONTAL
                                gravity = Gravity.CENTER_VERTICAL
                                addView(
                                    View(context).apply {
                                        background = GradientDrawable().apply {
                                            shape = GradientDrawable.OVAL
                                            setColor(if (reading) palette.ok else palette.error)
                                        }
                                    },
                                    LinearLayout.LayoutParams(gap(2), gap(2)).apply {
                                        marginEnd = gap(2)
                                    },
                                )
                                addView(
                                    TextView(context).apply {
                                        text = getString(
                                            if (reading) {
                                                R.string.omni_log_bound
                                            } else {
                                                R.string.omni_log_unbound
                                            }
                                        )
                                        setTextColor(if (reading) palette.ok else palette.error)
                                        typeface = Type.strong
                                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                                    }
                                )
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(
                                    if (reading) {
                                        R.string.omni_log_bound_note
                                    } else {
                                        R.string.omni_log_bind_note
                                    }
                                )
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                                setLineSpacing(gap(1).toFloat(), 1f)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                if (!reading) {
                    addView(
                        logButton(Mark.Shape.LINK, R.string.omni_log_bind, true) {
                            seekTheDevice()
                        },
                        LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT).apply {
                            marginStart = gap(2)
                        },
                    )
                }
            }
        )
        if (!reading && (logPort > 0 || logWireless || logWired)) {
            bound.addView(rule(), MATCH_PARENT, 1)
            bound.addView(
                factRow(
                    getString(R.string.omni_log_seek_port),
                    if (logPort > 0) {
                        logPort.toString()
                    } else {
                        getString(R.string.omni_log_seek_unknown)
                    },
                    onPress = null,
                )
            )
            bound.addView(
                factRow(
                    getString(R.string.omni_log_seek_switch),
                    getString(
                        when {
                            logWireless -> R.string.omni_log_seek_wifi
                            logWired -> R.string.omni_log_seek_cable
                            else -> R.string.omni_log_seek_shut
                        }
                    ),
                    onPress = null,
                )
            )
            bound.addView(
                factRow(
                    getString(R.string.omni_log_notes),
                    getString(
                        if (Bell.allowed(this)) {
                            R.string.omni_log_notes_on
                        } else {
                            R.string.omni_log_notes_off
                        }
                    ),
                    Mark.Shape.SPEECH,
                    R.string.omni_log_notes,
                ) { askToNotify() }
            )
            if (logPairHost.isNotEmpty()) {
                bound.addView(
                    factRow(
                        getString(R.string.omni_log_pair_found),
                        getString(R.string.omni_log_pair_at, logPairHost, logPairPort),
                    ) {
                        copyOut(
                            logPairHost + ":" + logPairPort,
                            getString(R.string.omni_log_pair_found),
                        )
                    }
                )
            }
            bound.addView(quiet(getString(R.string.omni_log_seek_grant)))
            bound.addView(
                LinearLayout(this).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    background = sheet(palette.raised, palette.divider)
                    setPadding(gap(3), gap(2), gap(2), gap(2))
                    addView(
                        HorizontalScrollView(context).apply {
                            isHorizontalScrollBarEnabled = false
                            addView(
                                TextView(context).apply {
                                    text = grantLine()
                                    setTextColor(palette.marker)
                                    typeface = Type.data
                                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                                    maxLines = 1
                                }
                            )
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                    addView(
                        View(context).apply {
                            foreground = Mark(Mark.Shape.COPY, palette.foreground)
                            isClickable = true
                            isFocusable = true
                            contentDescription = getString(R.string.omni_action_copy)
                            readAs(Button::class.java.name)
                            background = reacting(
                                Color.TRANSPARENT,
                                gap(5).toFloat(),
                                palette.accent,
                            )
                            setOnClickListener {
                                copyOut(grantLine(), getString(R.string.omni_log_device))
                            }
                        },
                        LinearLayout.LayoutParams(gap(7), gap(7)).apply { marginStart = gap(2) },
                    )
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                    topMargin = gap(1)
                    bottomMargin = gap(2)
                },
            )
        }
        content.addView(bound)

        content.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, gap(4), 0, gap(1))
                addView(
                    View(context).apply { foreground = Mark(Mark.Shape.LOG, palette.accent) },
                    LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(2) },
                )
                addView(
                    TextView(context).apply {
                        text = getString(R.string.omni_menu_logcat)
                        setTextColor(palette.accent)
                        typeface = Type.strong
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                    }
                )
            }
        )
        content.addView(
            View(this).apply { setBackgroundColor(palette.accent) },
            LinearLayout.LayoutParams(gap(22), gap(1) / 2),
        )
        content.addView(
            rule(),
            LinearLayout.LayoutParams(MATCH_PARENT, 1).apply { bottomMargin = gap(2) },
        )

        val sieve = card()
        sieve.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                addView(
                    View(context).apply { foreground = Mark(Mark.Shape.FIND, palette.muted) },
                    LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(2) },
                )
                addView(
                    EditText(context).apply {
                        setText(logSieve)
                        hint = getString(R.string.omni_log_sieve)
                        setHintTextColor(palette.muted)
                        setTextColor(palette.foreground)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                        setBackgroundColor(Color.TRANSPARENT)
                        isSingleLine = true
                        addTextChangedListener(object : TextWatcher {
                            override fun beforeTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
                            override fun onTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
                            override fun afterTextChanged(s: Editable?) {
                                logSieve = s?.toString().orEmpty()
                                paintTheLog()
                            }
                        })
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            }
        )
        sieve.addView(rule(), MATCH_PARENT, 1)
        sieve.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                addView(
                    View(context).apply { foreground = Mark(Mark.Shape.CODE, palette.muted) },
                    LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(2) },
                )
                addView(
                    EditText(context).apply {
                        setText(logArgs)
                        hint = getString(R.string.omni_log_args_hint)
                        setHintTextColor(palette.muted)
                        setTextColor(palette.foreground)
                        typeface = Type.data
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                        setBackgroundColor(Color.TRANSPARENT)
                        isSingleLine = true
                        addTextChangedListener(object : TextWatcher {
                            override fun beforeTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
                            override fun onTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
                            override fun afterTextChanged(s: Editable?) {
                                logArgs = s?.toString().orEmpty()
                            }
                        })
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            }
        )
        sieve.addView(rule(), MATCH_PARENT, 1)
        sieve.addView(
            factRow(
                getString(R.string.omni_log_apps),
                logPackageShown.ifEmpty { getString(R.string.omni_log_apps_any) },
                Mark.Shape.DROID,
                R.string.omni_log_apps,
            ) { chooseTheLoggedApp() }
        )
        sieve.addView(
            quiet(logCommand().joinToString(" "))
        )
        content.addView(sieve)

        val doing = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                topMargin = gap(2)
            }
        }
        doing.addView(
            logButton(Mark.Shape.PLAY, R.string.omni_log_start, !logReading) {
                startTheLog()
                render(false)
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply { marginEnd = gap(1) },
        )
        doing.addView(
            logButton(Mark.Shape.STOP, R.string.omni_log_stop, false) {
                stopTheLog()
                render(false)
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply {
                marginStart = gap(1)
                marginEnd = gap(1)
            },
        )
        doing.addView(
            logButton(Mark.Shape.TRASH, R.string.omni_log_wipe, false) {
                synchronized(logHeld) { logHeld.clear() }
                paintTheLog()
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply { marginStart = gap(1) },
        )
        content.addView(doing)

        val shown = card()
        shown.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_log_written)
                                setTextColor(palette.foreground)
                                typeface = Type.strong
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_log_scope)
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                addView(
                    markControl(Mark.Shape.COPY, R.string.omni_action_copy) {
                        copyOut(heldLog(), getString(R.string.omni_menu_logcat))
                    },
                    LinearLayout.LayoutParams(gap(8), gap(8)),
                )
                addView(
                    markControl(Mark.Shape.WIDEN, R.string.omni_log_written) {
                        showNote(getString(R.string.omni_log_written), heldLog().ifEmpty {
                            getString(R.string.omni_log_empty)
                        })
                    },
                    LinearLayout.LayoutParams(gap(8), gap(8)),
                )
            }
        )
        val counter = TextView(this).apply {
            setTextColor(palette.muted)
            typeface = Type.data
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
            setPadding(0, gap(1), 0, gap(1))
        }
        logCount = counter
        val written = EditText(this).apply {
            setTextColor(palette.foreground)
            setBackgroundColor(Color.TRANSPARENT)
            typeface = Type.data
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
            gravity = Gravity.TOP or Gravity.START
            setHorizontallyScrolling(true)
            isFocusable = true
            setTextIsSelectable(true)
            keyListener = null
            minLines = 10
            maxLines = 30
            setPadding(gap(1))
        }
        logView = written
        val hollow = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER
            setPadding(gap(4), gap(10), gap(4), gap(10))
            addView(
                View(context).apply { foreground = Mark(Mark.Shape.NOTE, palette.divider) },
                LinearLayout.LayoutParams(gap(14), gap(14)),
            )
            addView(
                TextView(context).apply {
                    text = getString(R.string.omni_log_empty)
                    setTextColor(palette.foreground)
                    typeface = Type.strong
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                    gravity = Gravity.CENTER
                    setPadding(0, gap(3), 0, gap(1))
                }
            )
            addView(
                TextView(context).apply {
                    text = getString(R.string.omni_log_empty_note)
                    setTextColor(palette.muted)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                    gravity = Gravity.CENTER
                    setLineSpacing(gap(1).toFloat(), 1f)
                }
            )
        }
        logHollow = hollow
        shown.addView(counter)
        shown.addView(hollow)
        shown.addView(
            HorizontalScrollView(this).apply {
                isFillViewport = true
                addView(written, MATCH_PARENT, WRAP_CONTENT)
            }
        )
        content.addView(shown)
        paintTheLog()
        if (logReading) {
            content.removeCallbacks(logTick)
            content.postDelayed(logTick, LOG_TICK_MILLIS)
        }
        content.addView(
            noteCard(
                getString(R.string.omni_log_about),
                getString(R.string.omni_log_note_scope),
                logButton(Mark.Shape.BOOK, R.string.omni_log_guide, false) { go(Screen.Help) },
            )
        )
    }

    private fun techTile(
        badge: Triple<Int, String, Int>,
        edge: Int,
        named: Boolean,
    ): View = LinearLayout(this).apply {
        val rim = if (badge.third == 0) Ink.fade(edge, 0.45f) else badge.third
        val wash = if (badge.third == 0) edge else badge.third
        orientation = LinearLayout.VERTICAL
        gravity = Gravity.CENTER_HORIZONTAL
        addView(
            LinearLayout(context).apply {
                orientation = LinearLayout.VERTICAL
                gravity = Gravity.CENTER_HORIZONTAL
                setPadding(gap(1), gap(2), gap(1), gap(2))
                background = GradientDrawable().apply {
                    shape = GradientDrawable.RECTANGLE
                    cornerRadius = gap(3).toFloat()
                    setColor(Ink.fade(wash, 0.08f))
                    setStroke(gap(1) / 2, rim)
                }
                addView(
                    ImageView(context).apply {
                        setImageResource(badge.first)
                        scaleType = ImageView.ScaleType.FIT_CENTER
                    },
                    LinearLayout.LayoutParams(gap(9), gap(9)),
                )
                if (named) {
                    addView(
                        TextView(context).apply {
                            text = badge.second
                            setTextColor(palette.foreground)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                            gravity = Gravity.CENTER
                            setPadding(0, gap(1), 0, 0)
                        },
                        LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
                    )
                }
            },
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
        )
        if (!named) {
            addView(
                TextView(context).apply {
                    text = badge.second
                    setTextColor(palette.foreground)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                    gravity = Gravity.CENTER
                    setPadding(0, gap(2), 0, 0)
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
        }
    }

    private fun kindCard(
        mark: Mark.Shape,
        title: String,
        detail: String,
        edge: Int,
        tiles: List<Triple<Int, String, Int>>,
        holds: Int,
        named: Boolean,
        onPress: () -> Unit,
    ): View = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(gap(4), gap(4), gap(4), gap(4))
        isClickable = true
        isFocusable = true
        contentDescription = said(title, detail)
        readAs(Button::class.java.name)
        background = touchable(
            GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(4).toFloat()
                setColor(palette.surface)
                setStroke(gap(1) / 2, edge)
            },
            edge,
        )
        setOnClickListener { onPress() }
        layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
            topMargin = gap(3)
        }
        addView(
            LinearLayout(context).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                addView(
                    badge(mark, edge),
                    LinearLayout.LayoutParams(gap(14), gap(14)).apply { marginEnd = gap(3) },
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = title
                                setTextColor(palette.foreground)
                                typeface = Type.heading
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 17f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = detail
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                                setLineSpacing(gap(1).toFloat(), 1f)
                                setPadding(0, gap(1), 0, 0)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                addView(
                    View(context).apply {
                        foreground = Mark(Mark.Shape.CHEVRON_RIGHT, palette.foreground)
                        background = GradientDrawable().apply {
                            shape = GradientDrawable.OVAL
                            setColor(Ink.fade(edge, 0.12f))
                            setStroke(gap(1) / 2, Ink.fade(edge, 0.5f))
                        }
                    },
                    LinearLayout.LayoutParams(gap(10), gap(10)).apply { marginStart = gap(2) },
                )
            }
        )
        addView(
            TextView(context).apply {
                text = getString(holds)
                setTextColor(palette.muted)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                setPadding(0, gap(4), 0, gap(1))
            }
        )
        addView(
            LinearLayout(context).apply {
                orientation = LinearLayout.HORIZONTAL
                tiles.forEach { badge ->
                    addView(
                        techTile(badge, edge, named),
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply {
                            marginStart = gap(1) / 2
                            marginEnd = gap(1) / 2
                        },
                    )
                }
            }
        )
    }

    private fun newProjectForm() {
        formLabel = DEFAULT_LABEL
        formVersionName = "1.0.0"
        formVersionCode = "1"
        formPackage = Preferences.newPackage(this, DEFAULT_PACKAGE)
        formMinSdk = Preferences.newMinSdk(this, OLDEST_PLATFORM)
        formTargetSdk = Preferences.newTargetSdk(this, NEWEST_PLATFORM)
    }

    private fun showKinds() {
        val panel = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(4), gap(3), gap(4), gap(4))
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(5).toFloat()
                setColor(palette.surface)
                setStroke(1, palette.divider)
            }
        }
        val shown = AlertDialog.Builder(this).setView(
            ScrollView(this).apply {
                setPadding(gap(2), gap(2), gap(2), gap(2))
                addView(panel, MATCH_PARENT, WRAP_CONTENT)
            }
        ).create()
        panel.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                addView(
                    brandMark(R.drawable.omni_brand, gap(2), gap(3) / 2),
                    LinearLayout.LayoutParams(gap(8), gap(8)).apply { marginEnd = gap(3) },
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_app_name)
                                setTextColor(palette.foreground)
                                typeface = Type.heading
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_kinds_hail)
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                addView(
                    View(context).apply {
                        foreground = Mark(Mark.Shape.CLOSE, palette.muted)
                        isClickable = true
                        isFocusable = true
                        contentDescription = getString(R.string.omni_action_cancel)
                        readAs(Button::class.java.name)
                        setOnClickListener { shown.dismiss() }
                    },
                    LinearLayout.LayoutParams(gap(6), gap(6)),
                )
            }
        )
        panel.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, gap(3), 0, gap(2))
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_kinds_title)
                                setTextColor(palette.foreground)
                                typeface = Type.heading
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 20f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_kinds_note)
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                                setLineSpacing(gap(1) / 2f, 1f)
                                setPadding(0, gap(1), 0, 0)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                addView(
                    ImageView(context).apply {
                        setImageResource(R.drawable.omni_art_kinds_wave)
                        scaleType = ImageView.ScaleType.FIT_CENTER
                        importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                    },
                    LinearLayout.LayoutParams(gap(30), gap(25)).apply { marginStart = gap(1) },
                )
            }
        )
        panel.addView(
            kindCard(
                Mark.Shape.DROID,
                getString(R.string.omni_kinds_native),
                getString(R.string.omni_kinds_native_note),
                palette.accent,
                NATIVE_TILES,
                R.string.omni_kinds_holds,
                false,
            ) {
                shown.dismiss()
                newProjectForm()
                go(Screen.NewProject)
            }
        )
        panel.addView(
            kindCard(
                Mark.Shape.WEB,
                getString(R.string.omni_kinds_web),
                getString(R.string.omni_kinds_web_note),
                palette.ok,
                WEB_TILES,
                R.string.omni_kinds_holds_web,
                true,
            ) {
                shown.dismiss()
                newProjectForm()
                go(Screen.WebToApk)
            }
        )
        panel.addView(
            TextView(this).apply {
                text = getString(R.string.omni_action_cancel)
                setTextColor(palette.muted)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                gravity = Gravity.CENTER
                setPadding(0, gap(3), 0, gap(3))
                isClickable = true
                isFocusable = true
                readAs(Button::class.java.name)
                background = touchable(
                    GradientDrawable().apply {
                        shape = GradientDrawable.RECTANGLE
                        cornerRadius = gap(3).toFloat()
                        setColor(Color.TRANSPARENT)
                        setStroke(1, palette.divider)
                    },
                    palette.accent,
                )
                setOnClickListener { shown.dismiss() }
            },
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply { topMargin = gap(3) },
        )
        panel.addView(
            TextView(this).apply {
                text = getString(R.string.omni_kinds_spark)
                setTextColor(palette.muted)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
                gravity = Gravity.CENTER
                setPadding(0, gap(3), 0, 0)
            }
        )
        shown.window?.setBackgroundDrawable(ColorDrawable(Color.TRANSPARENT))
        shown.show()
    }

    private fun createPage() {
        results.removeAllViews()
        if (webAddress.isBlank() || webAddress.contains(';')) {
            results.addView(notice(getString(R.string.omni_web_needs), palette.warning))
            return
        }
        val root = freeFolder(formLabel)
        working({ Builder.nativeCreateProject(root.absolutePath, pageSpec().encode()) })
        finished@{ answer ->
            val outcome = runCatching { CreateOutcome.parse(answer) }.getOrElse {
                results.addView(notice(it.message ?: it.javaClass.simpleName, palette.error))
                return@finished
            }
            if (!outcome.created) {
                showRefusal(Refusal.parse(JSONObject(answer)), results)
                return@finished
            }
            OmniLog.event(LogLevel.INFO, "project", "Created ${outcome.root}")
            results.addView(notice(getString(R.string.omni_created), palette.ok))
            openHere(root.absolutePath)
        }
    }

    private fun pageSpec() = ProjectSpec(
        packageName = formPackage.trim(),
        label = formLabel.trim(),
        versionName = formVersionName.trim(),
        versionCode = formVersionCode.trim().toIntOrNull() ?: 0,
        abis = ONLY_ABI,
        minSdk = formMinSdk,
        targetSdk = formTargetSdk,
        languages = listOf("java"),
        locales = (formLocales + BASE_LOCALE).toList(),
        page = webAddress.trim(),
        pageParts = WEB_PARTS.filterIndexed { index, _ -> webParts[index] },
    )

    private fun titledCard(mark: Mark.Shape, tint: Int, title: String, note: String): LinearLayout {
        val holder = card()
        holder.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, gap(1), 0, gap(1))
                addView(
                    badge(mark, tint),
                    LinearLayout.LayoutParams(gap(10), gap(10)).apply { marginEnd = gap(3) },
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = title
                                setTextColor(palette.foreground)
                                typeface = Type.strong
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 16f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = note
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                                setLineSpacing(gap(1).toFloat(), 1f)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            }
        )
        return holder
    }

    private fun radioMark(picked: Boolean): Drawable {
        val ring = GradientDrawable().apply {
            shape = GradientDrawable.OVAL
            setColor(Color.TRANSPARENT)
            setStroke(gap(3) / 8, if (picked) palette.accent else palette.muted)
        }
        if (!picked) {
            return ring
        }
        val dot = GradientDrawable().apply {
            shape = GradientDrawable.OVAL
            setColor(palette.accent)
        }
        return android.graphics.drawable.LayerDrawable(arrayOf(ring, dot)).apply {
            setLayerInset(1, gap(3) / 4, gap(3) / 4, gap(3) / 4, gap(3) / 4)
        }
    }

    private fun markedButton(
        mark: Mark.Shape,
        named: String,
        tint: Int,
        outlined: Boolean,
        filled: Int = 0,
        onPress: () -> Unit,
    ): View = LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER
        minimumHeight = gap(TOUCH_UNITS)
        setPadding(gap(3), gap(2), gap(3), gap(2))
        isClickable = true
        isFocusable = true
        contentDescription = named
        readAs(Button::class.java.name)
        background = touchable(
            GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(3).toFloat()
                setColor(
                    when {
                        outlined -> Color.TRANSPARENT
                        filled != 0 -> filled
                        else -> palette.raised
                    }
                )
                if (outlined) {
                    setStroke(gap(1) / 2, tint)
                }
            },
            palette.accent,
        )
        setOnClickListener { onPress() }
        addView(
            View(context).apply {
                foreground = Mark(mark, tint)
                importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
            },
            LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(2) },
        )
        addView(
            TextView(context).apply {
                text = named
                setTextColor(tint)
                typeface = Type.strong
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                maxLines = 1
            }
        )
    }

    private fun markedRow(mark: Mark.Shape, tint: Int, title: String, note: String, onPress: () -> Unit) =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            minimumHeight = gap(TOUCH_UNITS)
            setPadding(0, gap(2), 0, gap(2))
            isClickable = true
            isFocusable = true
            contentDescription = said(title, note)
            readAs(Button::class.java.name)
            background = touchable(pill(Color.TRANSPARENT, gap(2).toFloat()), palette.accent)
            setOnClickListener { onPress() }
            addView(
                badge(mark, tint),
                LinearLayout.LayoutParams(gap(9), gap(9)).apply { marginEnd = gap(3) },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = title
                            setTextColor(palette.foreground)
                            typeface = Type.strong
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = note
                            setTextColor(palette.muted)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                        }
                    )
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            addView(
                View(context).apply { foreground = Mark(Mark.Shape.CHEVRON_RIGHT, palette.muted) },
                LinearLayout.LayoutParams(gap(6), gap(6)),
            )
        }

    private fun renderWebToApk() {
        content.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, gap(1), 0, gap(2))
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                val whole = getString(R.string.omni_web_title)
                                val cut = whole.lastIndexOf(' ')
                                text = SpannableStringBuilder(whole).apply {
                                    if (cut in 0 until whole.length - 1) {
                                        setSpan(
                                            ForegroundColorSpan(palette.accent),
                                            cut + 1,
                                            whole.length,
                                            Spannable.SPAN_EXCLUSIVE_EXCLUSIVE,
                                        )
                                    }
                                }
                                setTextColor(palette.foreground)
                                typeface = Type.heading
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 30f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_web_note)
                                setTextColor(Ink.hot(palette.muted, 0.3f))
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                                setLineSpacing(gap(1) / 2f, 1f)
                                setPadding(0, gap(2), 0, 0)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                addView(
                    ImageView(context).apply {
                        setImageResource(R.drawable.omni_art_web)
                        scaleType = ImageView.ScaleType.FIT_CENTER
                        importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                    },
                    LinearLayout.LayoutParams(gap(38), gap(26)).apply { marginStart = gap(1) },
                )
            }
        )
        content.addView(
            titledCard(
                Mark.Shape.BOLT,
                palette.accent,
                getString(R.string.omni_web_tech),
                getString(R.string.omni_web_tech_note),
            )
        )

        val where = titledCard(
            Mark.Shape.LINK,
            palette.accent,
            getString(R.string.omni_web_where),
            getString(R.string.omni_web_where_note),
        )
        where.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(gap(3), gap(2), gap(3), gap(2))
                background = GradientDrawable().apply {
                    this.shape = GradientDrawable.RECTANGLE
                    cornerRadius = gap(3).toFloat()
                    setColor(palette.raised)
                    setStroke(gap(1) / 2, palette.divider)
                }
                addView(
                    View(context).apply { foreground = Mark(Mark.Shape.LINK, palette.muted) },
                    LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(2) },
                )
                addView(
                    EditText(context).apply {
                        setText(webAddress)
                        hint = "https://ornek.com"
                        setHintTextColor(palette.muted)
                        setTextColor(palette.foreground)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                        setBackgroundColor(Color.TRANSPARENT)
                        isSingleLine = true
                        inputType = InputType.TYPE_CLASS_TEXT or
                            InputType.TYPE_TEXT_VARIATION_URI
                        addTextChangedListener(object : TextWatcher {
                            override fun beforeTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
                            override fun onTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
                            override fun afterTextChanged(s: Editable?) {
                                webAddress = s?.toString().orEmpty()
                            }
                        })
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            },
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply { topMargin = gap(2) },
        )
        content.addView(where)

        val more = titledCard(
            Mark.Shape.GEAR,
            palette.muted,
            getString(R.string.omni_web_deeper),
            getString(R.string.omni_web_deeper_note),
        )
        WEB_PARTS.forEachIndexed { index, _ ->
            more.addView(rule(), MATCH_PARENT, 1)
            more.addView(
                LinearLayout(this).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    addView(
                        View(context).apply {
                            foreground = Mark(WEB_PART_MARKS[index], palette.accent)
                        },
                        LinearLayout.LayoutParams(gap(8), gap(8)).apply { marginEnd = gap(3) },
                    )
                    addView(
                        switchRow(
                            getString(WEB_PART_NAMES[index]),
                            getString(WEB_PART_NOTES[index]),
                            webParts[index],
                        ) { webParts[index] = it },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                }
            )
        }
        more.addView(rule(), MATCH_PARENT, 1)
        more.addView(
            markedRow(
                Mark.Shape.BRUSH,
                0xFFFF5C8A.toInt(),
                getString(R.string.omni_web_look),
                getString(R.string.omni_web_look_note),
            ) { go(Screen.Theme) }
        )
        more.addView(rule(), MATCH_PARENT, 1)
        more.addView(
            markedRow(
                Mark.Shape.INFO,
                palette.accent,
                getString(R.string.omni_web_facts),
                getString(R.string.omni_web_facts_note),
            ) { showWebFacts() }
        )
        content.addView(more)

        content.addView(primary(getString(R.string.omni_web_create)) { createPage() })
        content.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER
                setPadding(0, gap(2), 0, gap(3))
                addView(
                    View(context).apply { foreground = Mark(Mark.Shape.SHIELD, palette.ok) },
                    LinearLayout.LayoutParams(gap(5), gap(5)).apply { marginEnd = gap(2) },
                )
                addView(
                    TextView(context).apply {
                        text = getString(R.string.omni_web_safe)
                        setTextColor(palette.muted)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                    }
                )
            }
        )
        content.addView(quiet(getString(R.string.omni_web_note_privacy)))
    }

    private fun showWebFacts() {
        val sheet = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(5), gap(3), gap(5), gap(2))
        }
        sheet.addView(field(getString(R.string.omni_form_label), formLabel) { formLabel = it })
        sheet.addView(
            field(getString(R.string.omni_form_package), formPackage) { formPackage = it }
        )
        sheet.addView(
            field(getString(R.string.omni_form_version_name), formVersionName) {
                formVersionName = it
            }
        )
        sheet.addView(
            field(getString(R.string.omni_form_version_code), formVersionCode) {
                formVersionCode = it
            }
        )
        dialogShell(
            getString(R.string.omni_web_facts),
            sheet,
            listOf(
                getString(R.string.omni_action_confirm) to { it: AlertDialog ->
                    it.dismiss()
                    render(false)
                }
            ),
        )
    }

    private fun tile(text: String, tint: Int) = TextView(this).apply {
        this.text = text
        setTextColor(tint)
        typeface = Type.label
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
        letterSpacing = Type.TRACKING
        setPadding(gap(3), gap(1), gap(3), gap(1))
        background = pill(Ink.fade(tint, 0.16f), gap(2).toFloat())
    }

    private fun noteCard(title: String, body: String, trailing: View? = null): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            setPadding(gap(4), gap(4), gap(4), gap(4))
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(4).toFloat()
                setColor(Ink.fade(palette.accent, 0.07f))
                setStroke(gap(1) / 2, Ink.fade(palette.accent, 0.35f))
            }
            layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                topMargin = gap(3)
                bottomMargin = gap(2)
            }
            addView(
                View(context).apply { foreground = Mark(Mark.Shape.INFO, palette.accent) },
                LinearLayout.LayoutParams(gap(7), gap(7)).apply { marginEnd = gap(3) },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = title
                            setTextColor(palette.accent)
                            typeface = Type.strong
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = body
                            setTextColor(palette.muted)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                            setLineSpacing(gap(1).toFloat(), 1f)
                            setPadding(0, gap(1), 0, 0)
                        }
                    )
                    if (trailing != null) {
                        addView(trailing)
                    }
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
        }

    private fun screenHead(mark: Mark.Shape, title: String, detail: String): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(0, gap(1), 0, gap(1))
            addView(
                View(context).apply { foreground = Mark(mark, palette.foreground) },
                LinearLayout.LayoutParams(gap(11), gap(11)).apply { marginEnd = gap(4) },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = title
                            setTextColor(palette.foreground)
                            typeface = Type.heading
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 18f)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = detail
                            setTextColor(palette.muted)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                            setLineSpacing(gap(1) / 2f, 1f)
                        }
                    )
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
        }

    private fun guideHead(
        mark: Mark.Shape,
        tint: Int,
        title: String,
        detail: String,
        art: Int = R.drawable.omni_art_help_ask,
    ): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            setPadding(0, gap(4), 0, gap(2))
            addView(
                View(context).apply {
                    foreground = Mark(mark, tint, 1.3f, glow = true)
                    importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                },
                LinearLayout.LayoutParams(gap(15), gap(15)).apply { marginEnd = gap(3) },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = title
                            setTextColor(palette.foreground)
                            typeface = Type.heading
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 16f)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = detail
                            setTextColor(Ink.hot(palette.muted, 0.28f))
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
                            setLineSpacing(gap(1) / 2f, 1f)
                            setPadding(0, gap(1), 0, 0)
                        }
                    )
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            addView(
                ImageView(context).apply {
                    setImageResource(art)
                    scaleType = ImageView.ScaleType.FIT_CENTER
                    importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                },
                LinearLayout.LayoutParams(gap(26), gap(21)).apply { marginStart = gap(1) },
            )
        }

    private fun seekField(hint: String, held: String, onTyped: (String) -> Unit): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            minimumHeight = gap(9)
            setPadding(gap(5), 0, gap(5), 0)
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(5).toFloat()
                setColor(palette.raised)
                setStroke(1, Ink.fade(palette.accent, 0.4f))
            }
            addView(
                View(context).apply { foreground = Mark(Mark.Shape.FIND, palette.muted, 1.1f) },
                LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(4) },
            )
            addView(
                EditText(context).apply {
                    setText(held)
                    this.hint = hint
                    setHintTextColor(palette.muted)
                    setTextColor(palette.foreground)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                    setBackgroundColor(Color.TRANSPARENT)
                    setPadding(0, gap(2), 0, gap(2))
                    isSingleLine = true
                    addTextChangedListener(object : TextWatcher {
                        override fun beforeTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
                        override fun onTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
                        override fun afterTextChanged(s: Editable?) {
                            onTyped(s?.toString().orEmpty())
                        }
                    })
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
        }

    private fun bandRow(names: List<String>, picked: Int, onPick: (Int) -> Unit): View =
        HorizontalScrollView(this).apply {
            isHorizontalScrollBarEnabled = false
            setPadding(0, gap(3), 0, gap(2))
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    names.forEachIndexed { at, named ->
                        val on = at == picked
                        addView(
                            TextView(context).apply {
                                text = named
                                setTextColor(if (on) Color.WHITE else Ink.hot(palette.muted, 0.4f))
                                typeface = if (on) Type.strong else Type.body
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                                gravity = Gravity.CENTER
                                setPadding(gap(4), gap(3) / 2, gap(4), gap(3) / 2)
                                isClickable = true
                                isFocusable = true
                                readAs(Button::class.java.name)
                                background = touchable(
                                    GradientDrawable().apply {
                                        shape = GradientDrawable.RECTANGLE
                                        cornerRadius = gap(5).toFloat()
                                        setColor(
                                            if (on) palette.accent
                                            else Ink.fade(palette.accent, 0.1f)
                                        )
                                        setStroke(
                                            1,
                                            if (on) palette.accent
                                            else Ink.fade(palette.accent, 0.24f),
                                        )
                                    },
                                    palette.accent,
                                )
                                setOnClickListener { onPick(at) }
                            },
                            LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT).apply {
                                marginEnd = gap(2)
                            },
                        )
                    }
                },
                MATCH_PARENT,
                WRAP_CONTENT,
            )
        }

    private fun grid(across: Int, cells: List<View>): View {
        val holder = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        var at = 0
        while (at < cells.size) {
            val line = LinearLayout(this).apply { orientation = LinearLayout.HORIZONTAL }
            var column = 0
            while (column < across) {
                line.addView(
                    cells.getOrNull(at + column) ?: View(this),
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply {
                        marginStart = if (column == 0) 0 else gap(1)
                        marginEnd = if (column == across - 1) 0 else gap(1)
                        topMargin = gap(1)
                        bottomMargin = gap(1)
                    },
                )
                column += 1
            }
            holder.addView(line, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            at += across
        }
        return holder
    }

    private fun switchRow(
        title: String,
        detail: String,
        on: Boolean,
        onChange: (Boolean) -> Unit,
    ): View {
        var held = on
        val knob = View(this)
        val track = FrameLayout(this).apply {
            addView(
                knob,
                FrameLayout.LayoutParams(
                    gap(4),
                    gap(4),
                    Gravity.CENTER_VERTICAL or Gravity.START,
                ).apply { marginStart = gap(1) },
            )
        }

        fun repaint(moving: Boolean) {
            track.background = pill(
                if (held) palette.accent else palette.raised,
                gap(3).toFloat(),
            )
            knob.background = pill(
                if (held) palette.background else palette.muted,
                gap(2).toFloat(),
            )
            val slid = if (held) gap(5).toFloat() else 0f
            if (moving && !Motion.still()) {
                knob.animate().translationX(slid).setDuration(Motion.of(Motion.TAP)).start()
            } else {
                knob.translationX = slid
            }
        }

        val holder = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            minimumHeight = gap(TOUCH_UNITS)
            setPadding(0, gap(2), 0, gap(2))
            isClickable = true
            isFocusable = true
            contentDescription = said(title, detail)
            readAs(android.widget.Switch::class.java.name)
            background = touchable(pill(Color.TRANSPARENT, gap(2).toFloat()), palette.accent)
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = title
                            setTextColor(palette.foreground)
                            typeface = Type.strong
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                        }
                    )
                    if (detail.isNotEmpty()) {
                        addView(
                            TextView(context).apply {
                                text = detail
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                            }
                        )
                    }
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            addView(track, LinearLayout.LayoutParams(gap(11), gap(6)))
        }
        holder.setOnClickListener {
            held = !held
            repaint(true)
            onChange(held)
        }
        repaint(false)
        return holder
    }

    private fun counted(
        named: String,
        initial: String,
        limit: Int,
        lines: Int,
        said: Int,
        onChange: (String) -> Unit,
    ): View {
        val counter = TextView(this).apply {
            text = getString(R.string.omni_counted, initial.length, limit)
            setTextColor(palette.muted)
            typeface = Type.data
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
        }
        val writing = EditText(this).apply {
            setText(initial)
            hint = getString(said)
            setHintTextColor(palette.muted)
            setTextColor(palette.foreground)
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
            setBackgroundColor(Color.TRANSPARENT)
            setPadding(0, gap(1), 0, gap(1))
            filters = arrayOf(android.text.InputFilter.LengthFilter(limit))
            if (lines > 1) {
                isSingleLine = false
                minLines = lines
                gravity = Gravity.TOP or Gravity.START
                inputType = InputType.TYPE_CLASS_TEXT or
                    InputType.TYPE_TEXT_FLAG_MULTI_LINE or
                    InputType.TYPE_TEXT_FLAG_CAP_SENTENCES
            } else {
                isSingleLine = true
            }
        }
        writing.addTextChangedListener(object : TextWatcher {
            override fun beforeTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
            override fun onTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
            override fun afterTextChanged(s: Editable?) {
                val written = s?.toString().orEmpty()
                counter.text = getString(R.string.omni_counted, written.length, limit)
                onChange(written)
            }
        })
        writing.setPadding(gap(4), gap(2), gap(4), gap(2))
        writing.background = GradientDrawable().apply {
            shape = GradientDrawable.RECTANGLE
            cornerRadius = gap(3) / 2f
            setColor(palette.raised)
            setStroke(1, palette.divider)
        }
        counter.gravity = Gravity.END
        return LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(0, gap(1), 0, gap(1))
            addView(label(named))
            addView(writing, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            addView(
                counter,
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                    topMargin = gap(1)
                },
            )
        }
    }

    private fun badge(mark: Mark.Shape, tint: Int) = FrameLayout(this).apply {
        background = GradientDrawable().apply {
            shape = GradientDrawable.OVAL
            setColor(Ink.fade(tint, 0.16f))
        }
        addView(
            View(context).apply { foreground = Mark(mark, tint) },
            FrameLayout.LayoutParams(MATCH_PARENT, MATCH_PARENT).apply {
                val inset = gap(2)
                setMargins(inset, inset, inset, inset)
            },
        )
    }

    private fun helpMark(topic: Int): Pair<Mark.Shape, Int> = when (topic) {
        0 -> Mark.Shape.FOLDER to 0xFF2C7BE5.toInt()
        1 -> Mark.Shape.HOME to 0xFF8B5CF6.toInt()
        2 -> Mark.Shape.ADD to 0xFF1FC77A.toInt()
        3 -> Mark.Shape.CODE to 0xFFF08A24.toInt()
        4 -> Mark.Shape.PACKAGE to 0xFFEC4899.toInt()
        5 -> Mark.Shape.SHIELD to 0xFF06B6D4.toInt()
        else -> Mark.Shape.HELP to 0xFF2C7BE5.toInt()
    }

    private fun styledField(initial: String, said: String, secret: Boolean = false): EditText =
        EditText(this).apply {
            setText(initial)
            setSelection(initial.length)
            hint = said
            setHintTextColor(palette.muted)
            isSingleLine = true
            setTextColor(palette.foreground)
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
            setPadding(gap(4), gap(2), gap(4), gap(2))
            if (secret) {
                inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
            }
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(3) / 2f
                setColor(palette.raised)
                setStroke(1, palette.divider)
            }
        }

    private fun dialogWord(named: String, onPress: (AlertDialog) -> Unit, shown: AlertDialog): View =
        TextView(this).apply {
            text = named
            setTextColor(palette.accent)
            typeface = Type.strong
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
            gravity = Gravity.CENTER
            setPadding(gap(4), gap(2), gap(4), gap(2))
            isClickable = true
            isFocusable = true
            readAs(Button::class.java.name)
            background = reacting(Color.TRANSPARENT, gap(2).toFloat(), palette.accent)
            setOnClickListener { onPress(shown) }
        }

    private fun dialogShell(
        titled: String,
        body: View,
        words: List<Pair<String, (AlertDialog) -> Unit>>,
        crown: View? = null,
    ): AlertDialog {
        val panel = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(4).toFloat()
                setColor(palette.surface)
                setStroke(1, palette.divider)
            }
        }
        val shown = AlertDialog.Builder(this).setView(
            FrameLayout(this).apply {
                setPadding(gap(4), gap(4), gap(4), gap(4))
                addView(panel, FrameLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            }
        ).create()
        if (titled.isNotEmpty()) {
            panel.addView(
                LinearLayout(this).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    addView(
                        TextView(context).apply {
                            text = titled
                            setTextColor(palette.foreground)
                            typeface = Type.heading
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 17f)
                            setPadding(gap(5), gap(4), gap(5), gap(3))
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                    if (crown != null) {
                        addView(
                            crown,
                            LinearLayout.LayoutParams(gap(10), gap(10)).apply {
                                marginEnd = gap(3)
                            },
                        )
                    }
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
        }
        panel.addView(
            ScrollView(this).apply {
                isFillViewport = false
                setPadding(gap(5), 0, gap(5), 0)
                addView(body, MATCH_PARENT, WRAP_CONTENT)
            },
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
        )
        panel.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.END
                setPadding(gap(3), gap(3), gap(3), gap(3))
                words.forEach { (named, onPress) ->
                    addView(
                        dialogWord(named, onPress, shown),
                        LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT).apply {
                            marginStart = gap(1)
                        },
                    )
                }
            }
        )
        shown.window?.setBackgroundDrawable(ColorDrawable(Color.TRANSPARENT))
        shown.show()
        return shown
    }

    private fun pickOne(titled: String, choices: List<String>, onPick: (Int) -> Unit) {
        val list = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        lateinit var shown: AlertDialog
        choices.forEachIndexed { at, named ->
            list.addView(
                TextView(this).apply {
                    text = named
                    setTextColor(palette.foreground)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                    minimumHeight = gap(11)
                    gravity = Gravity.CENTER_VERTICAL
                    setPadding(gap(2), gap(2), gap(2), gap(2))
                    isClickable = true
                    isFocusable = true
                    readAs(Button::class.java.name)
                    background = reacting(Color.TRANSPARENT, gap(2).toFloat(), palette.accent)
                    setOnClickListener {
                        shown.dismiss()
                        onPick(at)
                    }
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
        }
        shown = dialogShell(
            titled,
            list,
            listOf(getString(R.string.omni_action_cancel) to { it: AlertDialog -> it.dismiss() }),
        )
    }

    private fun showNote(title: String, said: String) {
        dialogShell(
            title,
            TextView(this).apply {
                text = said
                setTextColor(Ink.hot(palette.muted, 0.3f))
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                setLineSpacing(gap(1) / 2f, 1f)
                setPadding(0, 0, 0, gap(2))
            },
            listOf(getString(R.string.omni_action_confirm) to { it: AlertDialog -> it.dismiss() }),
        )
    }

    private fun themeArt(shown: Palette): Int = when (shown.key) {
        Palette.HACKER.key -> R.drawable.omni_theme_hacker
        Palette.ANIME.key -> R.drawable.omni_theme_anime
        else -> R.drawable.omni_theme_dark
    }

    private fun flagOf(tag: String): Int = when (tag) {
        "tr" -> R.drawable.omni_flag_tr
        "ja" -> R.drawable.omni_flag_ja
        "ko" -> R.drawable.omni_flag_ko
        "hi" -> R.drawable.omni_flag_hi
        "id" -> R.drawable.omni_flag_id
        "pt-BR" -> R.drawable.omni_flag_br
        "de" -> R.drawable.omni_flag_de
        "fr" -> R.drawable.omni_flag_fr
        "es" -> R.drawable.omni_flag_es
        "ru" -> R.drawable.omni_flag_ru
        "ar" -> R.drawable.omni_flag_ar
        "zh" -> R.drawable.omni_flag_zh
        "pt" -> R.drawable.omni_flag_pt
        "it" -> R.drawable.omni_flag_it
        else -> R.drawable.omni_flag_en
    }

    private fun themeNote(shown: Palette): Int = when (shown.key) {
        Palette.HACKER.key -> R.string.omni_theme_hacker_note
        Palette.ANIME.key -> R.string.omni_theme_anime_note
        else -> R.string.omni_theme_dark_note
    }

    private fun themeCard(
        shown: Palette,
        active: Boolean,
        locked: Boolean,
        onPress: () -> Unit,
    ): View {
        val radio = View(this).apply {
            background = radioMark(active)
            importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
        }
        val row = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.TOP
            setPadding(gap(5) / 2, gap(5) / 2, gap(5) / 2, gap(5) / 2)
            background = touchable(
                GradientDrawable().apply {
                    shape = GradientDrawable.RECTANGLE
                    cornerRadius = gap(4).toFloat()
                    setColor(Color.TRANSPARENT)
                    setStroke(
                        if (active) gap(1) / 2 else 1,
                        if (active) palette.accent else palette.divider,
                    )
                },
                palette.accent,
            )
            isClickable = true
            isFocusable = true
            isSelected = active
            contentDescription = said(shown.label, getString(themeNote(shown)))
            readAs(android.widget.RadioButton::class.java.name)
            setOnClickListener { onPress() }
            addView(
                ImageView(context).apply {
                    setImageResource(themeArt(shown))
                    scaleType = ImageView.ScaleType.CENTER_CROP
                    clipToOutline = true
                    background = GradientDrawable().apply {
                        this.shape = GradientDrawable.RECTANGLE
                        cornerRadius = gap(2).toFloat()
                        setColor(palette.raised)
                    }
                },
                LinearLayout.LayoutParams(gap(49), gap(32)).apply { marginEnd = gap(3) },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = shown.label
                            setTextColor(palette.foreground)
                            typeface = Type.heading
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = if (locked) {
                                getString(R.string.omni_theme_soon)
                            } else {
                                getString(themeNote(shown))
                            }
                            setTextColor(if (locked) palette.warning else palette.muted)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                            setLineSpacing(gap(1) / 2f, 1f)
                            setPadding(0, gap(1), 0, 0)
                        }
                    )
                    if (active) {
                        addView(
                            tile(getString(R.string.omni_theme_active), palette.accent),
                            LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT).apply {
                                gravity = Gravity.END
                                topMargin = gap(2)
                            },
                        )
                    }
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
        }
        return FrameLayout(this).apply {
            addView(row, FrameLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
            addView(
                radio,
                FrameLayout.LayoutParams(gap(5), gap(5), Gravity.TOP or Gravity.END).apply {
                    topMargin = gap(4)
                    marginEnd = gap(4)
                },
            )
            layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                topMargin = gap(2)
                bottomMargin = gap(2)
            }
        }
    }

    private fun renderTheme() {
        content.addView(
            screenHead(
                Mark.Shape.BRUSH,
                getString(R.string.omni_settings_theme),
                getString(R.string.omni_theme_note),
            )
        )
        content.addView(
            rule(),
            LinearLayout.LayoutParams(MATCH_PARENT, 1).apply { bottomMargin = gap(2) },
        )
        for (shown in listOf(Palette.DARK, Palette.HACKER, Palette.ANIME)) {
            val locked = shown.key in LOCKED_THEMES
            content.addView(
                themeCard(shown, !locked && shown.key == palette.key, locked) {
                    if (locked) {
                        results.removeAllViews()
                        results.addView(
                            notice(getString(R.string.omni_theme_soon), palette.muted)
                        )
                        return@themeCard
                    }
                    if (shown.key != palette.key) {
                        Preferences.setPalette(this, shown.key)
                        recreate()
                    }
                }
            )
        }
    }

    private fun languageRow(tag: String, named: String, shown: String, current: Boolean) =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            minimumHeight = gap(13)
            setPadding(gap(3), gap(1), gap(3), gap(1))
            isClickable = true
            isFocusable = true
            isSelected = current
            contentDescription = named
            readAs(android.widget.RadioButton::class.java.name)
            background = if (current) {
                GradientDrawable().apply {
                    shape = GradientDrawable.RECTANGLE
                    cornerRadius = gap(3).toFloat()
                    setColor(Color.TRANSPARENT)
                    setStroke(gap(1) / 2, palette.accent)
                }
            } else {
                touchable(pill(Color.TRANSPARENT, gap(3).toFloat()), palette.accent)
            }
            setOnClickListener {
                if (!current) {
                    Preferences.setLanguage(this@BuilderActivity, tag)
                    recreate()
                }
            }
            if (tag.isEmpty()) {
                addView(
                    View(context).apply {
                        foreground = Mark(Mark.Shape.GLOBE, palette.marker)
                        importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                    },
                    LinearLayout.LayoutParams(gap(11), gap(8)).apply { marginEnd = gap(4) },
                )
            }
            addView(
                ImageView(context).apply {
                    setImageResource(flagOf(tag))
                    scaleType = ImageView.ScaleType.FIT_CENTER
                    visibility = if (tag.isEmpty()) View.GONE else View.VISIBLE
                },
                LinearLayout.LayoutParams(gap(11), gap(8)).apply { marginEnd = gap(4) },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = named
                            setTextColor(palette.foreground)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = getString(R.string.omni_language_shown, named, shown)
                            setTextColor(palette.marker)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                        }
                    )
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            addView(
                View(context).apply {
                    background = radioMark(current)
                },
                LinearLayout.LayoutParams(gap(7), gap(7)),
            )
        }

    private fun renderLanguages() {
        content.addView(
            screenHead(
                Mark.Shape.GLOBE,
                getString(R.string.omni_settings_language),
                getString(R.string.omni_language_note),
            )
        )
        val current = Preferences.language(this)
        val list = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                topMargin = gap(2)
            }
        }
        val spoken = deviceLanguage()
        list.addView(
            languageRow(
                "",
                getString(R.string.omni_language_auto),
                Preferences.LANGUAGES.firstOrNull { it.first == spoken }?.second.orEmpty()
                    .ifEmpty { spoken },
                current.isEmpty(),
            )
        )
        Preferences.LANGUAGES.forEach { one ->
            val picked = one.first == current
            if (!picked) {
                list.addView(
                    rule(),
                    LinearLayout.LayoutParams(MATCH_PARENT, 1).apply {
                        marginStart = gap(14)
                        marginEnd = gap(2)
                    },
                )
            }
            list.addView(languageRow(one.first, one.second, one.third, picked))
        }
        content.addView(list)
        content.addView(
            noteCard(getString(R.string.omni_language_told), getString(R.string.omni_language_all))
        )
    }
    private fun storeIntent(): Intent =
        Intent(Intent.ACTION_VIEW, Uri.parse("market://details?id=$packageName"))

    private fun renderRate() {
        content.addView(
            screenHead(
                Mark.Shape.HEART,
                getString(R.string.omni_rate_title),
                getString(R.string.omni_rate_support),
            )
        )
        content.addView(
            ImageView(this).apply {
                setImageResource(R.drawable.omni_art_rate)
                scaleType = ImageView.ScaleType.FIT_CENTER
                layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, gap(46)).apply {
                    topMargin = gap(4)
                }
            }
        )
        content.addView(
            TextView(this).apply {
                text = getString(R.string.omni_rate_ask)
                setTextColor(palette.foreground)
                typeface = Type.heading
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 20f)
                gravity = Gravity.CENTER
                setPadding(0, gap(4), 0, gap(2))
            }
        )
        content.addView(
            TextView(this).apply {
                text = getString(R.string.omni_rate_why)
                setTextColor(palette.muted)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                gravity = Gravity.CENTER
                setLineSpacing(gap(1).toFloat(), 1f)
                setPadding(gap(3), 0, gap(3), gap(4))
            }
        )
        val gains = card()
        listOf(
            Triple(Mark.Shape.STAR, R.string.omni_rate_one, palette.accent),
            Triple(Mark.Shape.PEOPLE, R.string.omni_rate_two, palette.accent),
            Triple(Mark.Shape.CHART, R.string.omni_rate_three, palette.accent),
        ).forEachIndexed { index, gain ->
            if (index > 0) {
                gains.addView(rule(), MATCH_PARENT, 1)
            }
            gains.addView(
                LinearLayout(this).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    setPadding(0, gap(3), 0, gap(3))
                    addView(
                        View(context).apply { foreground = Mark(gain.first, gain.third) },
                        LinearLayout.LayoutParams(gap(7), gap(7)).apply { marginEnd = gap(4) },
                    )
                    addView(
                        TextView(context).apply {
                            text = getString(gain.second)
                            setTextColor(palette.foreground)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                }
            )
        }
        content.addView(gains)
        content.addView(
            markedButton(
                Mark.Shape.PLAY,
                getString(R.string.omni_rate_store),
                Color.WHITE,
                outlined = false,
                filled = palette.accent,
            ) { hand(storeIntent()) }
        )
        content.addView(
            subtle(getString(R.string.omni_rate_later), palette.muted) { stepBack() }
        )
    }
    private fun featureCell(mark: Mark.Shape, named: Int, note: Int, tint: Int): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER_HORIZONTAL
            setPadding(gap(1), gap(2), gap(1), gap(2))
            addView(
                View(context).apply { foreground = Mark(mark, tint, 1.2f) },
                LinearLayout.LayoutParams(gap(6), gap(6)),
            )
            addView(
                TextView(context).apply {
                    text = getString(named)
                    setTextColor(palette.foreground)
                    typeface = Type.strong
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
                    gravity = Gravity.CENTER
                    setPadding(0, gap(2), 0, 0)
                }
            )
            addView(
                TextView(context).apply {
                    text = getString(note)
                    setTextColor(palette.muted)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 9f)
                    gravity = Gravity.CENTER
                    setLineSpacing(gap(1) / 2f, 1f)
                }
            )
        }

    private fun aboutRow(mark: Mark.Shape, tint: Int, title: String, note: String, onPress: () -> Unit) =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            minimumHeight = gap(11)
            setPadding(0, gap(1), 0, gap(1))
            isClickable = true
            isFocusable = true
            contentDescription = said(title, note)
            readAs(Button::class.java.name)
            background = touchable(pill(Color.TRANSPARENT, gap(2).toFloat()), palette.accent)
            setOnClickListener { onPress() }
            addView(
                View(context).apply { foreground = Mark(mark, tint) },
                LinearLayout.LayoutParams(gap(7), gap(7)).apply { marginEnd = gap(4) },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = title
                            setTextColor(palette.foreground)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = note
                            setTextColor(palette.muted)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                        }
                    )
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            addView(
                View(context).apply {
                    foreground = Mark(Mark.Shape.CHEVRON_RIGHT, palette.accent, 1.2f)
                },
                LinearLayout.LayoutParams(gap(4), gap(4)),
            )
        }

    private fun renderAbout() {
        content.addView(
            screenHead(
                Mark.Shape.INFO,
                getString(R.string.omni_settings_about),
                getString(R.string.omni_about_head),
            )
        )
        val face = card()
        face.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                gravity = Gravity.CENTER_HORIZONTAL
                setPadding(0, gap(1), 0, gap(1))
                val held = LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT)
                addView(
                    brandMark(R.drawable.omni_brand_large, gap(3), gap(9) / 4),
                    LinearLayout.LayoutParams(gap(27) / 2, gap(27) / 2),
                )
                addView(
                    TextView(context).apply {
                        text = getString(R.string.omni_app_name)
                        setTextColor(palette.foreground)
                        typeface = Type.heading
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 19f)
                        setPadding(0, gap(2), 0, 0)
                    },
                    held,
                )
                addView(
                    TextView(context).apply {
                        text = getString(R.string.omni_brand_note)
                        setTextColor(palette.muted)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                        letterSpacing = 0.07f
                        setPadding(0, gap(1) / 2, 0, gap(1))
                    },
                    LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT),
                )
                addView(
                    TextView(context).apply {
                        text = getString(R.string.omni_about_version, versionShown())
                        setTextColor(Ink.hot(palette.accent, 0.66f))
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
                        setPadding(gap(4), gap(3) / 2, gap(4), gap(3) / 2)
                        background = pill(Ink.fade(palette.accent, 0.15f), gap(3).toFloat())
                    },
                    LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT),
                )
                addView(
                    TextView(context).apply {
                        text = getString(R.string.omni_about_note)
                        setTextColor(Ink.hot(palette.muted, 0.3f))
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                        gravity = Gravity.CENTER
                        setLineSpacing(gap(1) / 2f, 1f)
                        setPadding(gap(2), gap(1), gap(2), 0)
                    }
                )
            }
        )
        face.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                setPadding(0, gap(1), 0, gap(1))
                val cells = listOf(
                    featureCell(
                        Mark.Shape.CODE,
                        R.string.omni_about_make,
                        R.string.omni_about_make_note,
                        palette.accent,
                    ),
                    featureCell(
                        Mark.Shape.BOLT,
                        R.string.omni_about_try,
                        R.string.omni_about_try_note,
                        0xFFB06BFF.toInt(),
                    ),
                    featureCell(
                        Mark.Shape.BRUSH,
                        R.string.omni_about_own,
                        R.string.omni_about_own_note,
                        0xFFFF5C8A.toInt(),
                    ),
                    featureCell(
                        Mark.Shape.PEOPLE,
                        R.string.omni_about_join,
                        R.string.omni_about_join_note,
                        palette.ok,
                    ),
                )
                cells.forEach { addView(it, LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f)) }
            }
        )
        content.addView(face)

        val rows = card()
        rows.addView(
            aboutRow(
                Mark.Shape.SHIELD,
                palette.accent,
                getString(R.string.omni_about_policy),
                getString(R.string.omni_about_policy_note),
            ) { go(Screen.Legal(false)) }
        )
        rows.addView(rule(), MATCH_PARENT, 1)
        rows.addView(
            aboutRow(
                Mark.Shape.NOTE_FILL,
                0xFFB06BFF.toInt(),
                getString(R.string.omni_about_terms),
                getString(R.string.omni_about_terms_note),
            ) { go(Screen.Legal(true)) }
        )
        rows.addView(rule(), MATCH_PARENT, 1)
        rows.addView(
            aboutRow(
                Mark.Shape.MAIL_FILL,
                palette.ok,
                getString(R.string.omni_about_contact),
                getString(R.string.omni_about_contact_note),
            ) { go(Screen.Feedback) }
        )
        content.addView(rows)

        val who = card()
        who.addView(
            aboutRow(
                Mark.Shape.PERSON_FILL,
                palette.warning,
                getString(R.string.omni_about_developer),
                getString(R.string.omni_about_developer_name),
            ) { go(Screen.Help) }
        )
        who.addView(rule(), MATCH_PARENT, 1)
        who.addView(
            aboutRow(
                Mark.Shape.COPYRIGHT,
                palette.marker,
                getString(R.string.omni_about_reserved),
                getString(
                    R.string.omni_about_rights,
                    Calendar.getInstance().get(Calendar.YEAR).toString(),
                ),
            ) { go(Screen.Legal(true)) }
        )
        content.addView(who)

        val thanks = card()
        thanks.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(gap(2), gap(2), gap(2), 0)
                addView(
                    View(context).apply {
                        foreground = Mark(Mark.Shape.HEART_FILL, palette.accent)
                    },
                    LinearLayout.LayoutParams(gap(9), gap(9)).apply { marginEnd = gap(4) },
                )
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_about_thanks)
                                setTextColor(palette.foreground)
                                typeface = Type.strong
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_about_thanks_note)
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                                setLineSpacing(gap(1).toFloat(), 1f)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            }
        )
        thanks.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(gap(11), gap(4), gap(11), gap(2))
                addView(rule(), LinearLayout.LayoutParams(0, 1, 1f))
                addView(
                    View(context).apply {
                        foreground = Mark(Mark.Shape.DROID, palette.muted)
                        importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                    },
                    LinearLayout.LayoutParams(gap(6), gap(6)).apply {
                        marginStart = gap(3)
                        marginEnd = gap(3)
                    },
                )
                addView(rule(), LinearLayout.LayoutParams(0, 1, 1f))
            }
        )
        content.addView(thanks)
        content.addView(
            keyValue(
                getString(R.string.omni_settings_integrity),
                Sentry.lastChecked(this).takeIf { it > 0L }?.let {
                    SimpleDateFormat("yyyy-MM-dd HH:mm", Locale.US).format(Date(it))
                } ?: "",
                standing,
                when (standing) {
                    "TRUSTED" -> palette.ok
                    "TAMPERED" -> palette.error
                    else -> palette.warning
                },
            )
        )
    }

    private fun feedbackReport(): JSONObject = JSONObject().apply {
        put("kind", resources.getResourceEntryName(FEEDBACK_KINDS[feedbackKind]))
        put("subject", feedbackSubject)
        put("detail", feedbackDetail)
        put("version", versionShown())
        put("device", Build.MANUFACTURER + " " + Build.MODEL)
        put("api", Build.VERSION.SDK_INT)
        put("language", Preferences.language(this@BuilderActivity))
        put("wrappedAt", System.currentTimeMillis())
        put("shots", feedbackShots.size)
    }

    private fun keepFeedback() {
        val held = JSONObject().apply {
            put("kind", feedbackKind)
            put("subject", feedbackSubject)
            put("detail", feedbackDetail)
            put("shots", org.json.JSONArray(feedbackShots))
        }
        Preferences.setFeedbackDraft(this, held.toString())
    }

    private fun takeBackTheFeedback() {
        val held = Preferences.feedbackDraft(this)
        if (held.isEmpty()) {
            return
        }
        val document = runCatching { JSONObject(held) }.getOrNull() ?: return
        feedbackKind = document.optInt("kind").coerceIn(0, FEEDBACK_KINDS.size - 1)
        feedbackSubject = document.optString("subject")
        feedbackDetail = document.optString("detail")
        feedbackShots.clear()
        val listed = document.optJSONArray("shots")
        for (index in 0 until (listed?.length() ?: 0)) {
            val one = listed?.optString(index).orEmpty()
            if (one.isNotEmpty() && File(one).isFile) {
                feedbackShots.add(one)
            }
        }
    }

    private fun shotsFolder(): File = File(cacheDir, SHOTS_FOLDER).apply { mkdirs() }

    private fun outbox(): File = File(filesDir, OUTBOX_FOLDER).apply { mkdirs() }

    private fun holdTheShot(uri: Uri): File? {
        val into = File(shotsFolder(), "Shot-" + System.nanoTime() + ".img")
        val written = runCatching {
            contentResolver.openInputStream(uri).use { from ->
                if (from == null) {
                    return null
                }
                into.outputStream().use { out ->
                    val room = ByteArray(64 * 1024)
                    var carried = 0L
                    while (true) {
                        val read = from.read(room)
                        if (read <= 0) {
                            break
                        }
                        carried += read
                        if (carried > LARGEST_SHOT_BYTES) {
                            return@runCatching -1L
                        }
                        out.write(room, 0, read)
                    }
                    carried
                }
            }
        }.getOrDefault(-1L)
        if (written < 0L) {
            into.delete()
            return null
        }
        return into
    }

    private fun takeTheShots(uris: List<Uri>) {
        results.removeAllViews()
        var refused = false
        for (uri in uris) {
            if (feedbackShots.size >= MOST_SHOTS) {
                results.addView(
                    notice(getString(R.string.omni_feedback_shot_most, MOST_SHOTS), palette.warning)
                )
                break
            }
            val held = holdTheShot(uri)
            if (held == null) {
                refused = true
                continue
            }
            feedbackShots.add(held.absolutePath)
        }
        if (refused) {
            results.addView(
                notice(getString(R.string.omni_feedback_shot_large, SHOT_LARGEST_MB), palette.warning)
            )
        }
        keepFeedback()
        render(false)
    }

    private fun dropTheShot(path: String) {
        feedbackShots.remove(path)
        File(path).delete()
        keepFeedback()
        render(false)
    }

    private fun shotStrip(): View = FlowLayout(this, gap(2), gap(2)).apply {
        feedbackShots.forEach { path ->
            addView(
                FrameLayout(context).apply {
                    addView(
                        thumbnail(path, gap(20)),
                        FrameLayout.LayoutParams(gap(20), gap(20)),
                    )
                    addView(
                        View(context).apply {
                            background = pill(Ink.fade(palette.background, 0.72f), gap(4).toFloat())
                            foreground = Mark(Mark.Shape.CLOSE, palette.error)
                            contentDescription = getString(R.string.omni_action_clear)
                            isClickable = true
                            isFocusable = true
                            readAs(Button::class.java.name)
                            setOnClickListener { dropTheShot(path) }
                        },
                        FrameLayout.LayoutParams(gap(8), gap(8)).apply {
                            gravity = Gravity.END or Gravity.TOP
                        },
                    )
                }
            )
        }
    }

    private fun restingFor(): Long {
        val since = System.currentTimeMillis() - Preferences.feedbackSentAt(this)
        return (FEEDBACK_REST_MILLIS - since).coerceAtLeast(0L)
    }

    private fun sendFeedback() {
        results.removeAllViews()
        if (feedbackSubject.isBlank() || feedbackDetail.isBlank()) {
            results.addView(notice(getString(R.string.omni_feedback_needs), palette.warning))
            return
        }
        val resting = restingFor()
        if (resting > 0L) {
            results.addView(
                notice(
                    getString(
                        R.string.omni_feedback_rest,
                        ((resting + 59_999L) / 60_000L).toInt(),
                    ),
                    palette.warning,
                )
            )
            return
        }
        val stamp = SimpleDateFormat(OUTPUT_STAMP, Locale.US).format(Date())
        val here = File(outbox(), stamp)
        val wrapped = runCatching {
            here.mkdirs()
            File(here, REPORT_FILE).writeText(feedbackReport().toString())
            feedbackShots.forEachIndexed { index, path ->
                File(path).copyTo(File(here, "Shot-" + (index + 1) + ".img"), overwrite = true)
            }
            true
        }.getOrDefault(false)
        if (!wrapped) {
            here.deleteRecursively()
            results.addView(notice(getString(R.string.omni_feedback_kept_not), palette.error))
            return
        }
        Preferences.setFeedbackSentAt(this, System.currentTimeMillis())
        Preferences.forgetFeedbackDraft(this)
        feedbackSubject = ""
        feedbackDetail = ""
        feedbackShots.forEach { File(it).delete() }
        feedbackShots.clear()
        flash(getString(R.string.omni_feedback_kept), palette.ok, Mark.Shape.SPEECH)
        render(false)
    }

    private fun kindCell(at: Int, mark: Mark.Shape, title: String, detail: String): View {
        val on = at == feedbackKind
        return LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER_HORIZONTAL
            setPadding(gap(2), gap(3) / 2, gap(2), gap(3) / 2)
            isClickable = true
            isFocusable = true
            isSelected = on
            contentDescription = said(title, detail)
            readAs(android.widget.RadioButton::class.java.name)
            background = touchable(
                GradientDrawable().apply {
                    this.shape = GradientDrawable.RECTANGLE
                    cornerRadius = gap(3) / 2f
                    setColor(
                        if (on) Ink.fade(palette.accent, 0.14f) else Color.TRANSPARENT
                    )
                    setStroke(
                        if (on) gap(1) / 2 else 1,
                        if (on) palette.accent else Ink.hot(palette.divider, 0.2f),
                    )
                },
                palette.accent,
            )
            setOnClickListener {
                feedbackKind = at
                keepFeedback()
                render(false)
            }
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    addView(View(context), LinearLayout.LayoutParams(0, 1, 1f))
                    addView(
                        View(context).apply { background = radioMark(on) },
                        LinearLayout.LayoutParams(gap(3), gap(3)),
                    )
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
            addView(
                View(context).apply {
                    foreground = Mark(mark, if (on) palette.accent else palette.foreground)
                },
                LinearLayout.LayoutParams(gap(5), gap(5)),
            )
            addView(
                TextView(context).apply {
                    text = title
                    setTextColor(palette.foreground)
                    typeface = Type.strong
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                    gravity = Gravity.CENTER
                    setPadding(0, gap(1), 0, 0)
                }
            )
            addView(
                TextView(context).apply {
                    text = detail
                    setTextColor(palette.muted)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                    gravity = Gravity.CENTER
                }
            )
        }
    }

    private fun renderFeedback() {
        takeBackTheFeedback()
        content.addView(
            screenHead(
                Mark.Shape.SPEECH,
                getString(R.string.omni_feedback_title),
                getString(R.string.omni_feedback_note),
            )
        )
        val kinds = card()
        kinds.addView(label(getString(R.string.omni_feedback_kind)))
        kinds.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                FEEDBACK_KINDS.forEachIndexed { index, named ->
                    addView(
                        kindCell(
                            index,
                            FEEDBACK_MARKS[index],
                            getString(named),
                            getString(FEEDBACK_NOTES[index]),
                        ),
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply {
                            marginStart = if (index == 0) 0 else gap(1)
                            marginEnd = if (index == FEEDBACK_KINDS.size - 1) 0 else gap(1)
                        },
                    )
                }
            }
        )
        content.addView(kinds)

        content.addView(
            counted(
                getString(R.string.omni_feedback_subject),
                feedbackSubject,
                SUBJECT_LIMIT,
                1,
                R.string.omni_feedback_subject_hint,
            ) {
                feedbackSubject = it
                keepFeedback()
            }
        )
        content.addView(
            counted(
                getString(R.string.omni_feedback_detail),
                feedbackDetail,
                DETAIL_LIMIT,
                3,
                R.string.omni_feedback_detail_hint,
            ) {
                feedbackDetail = it
                keepFeedback()
            }
        )

        content.addView(
            label(
                getString(R.string.omni_feedback_shot) + "  " +
                    getString(R.string.omni_counted, feedbackShots.size, MOST_SHOTS)
            )
        )
        if (feedbackShots.isNotEmpty()) {
            content.addView(
                shotStrip(),
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                    topMargin = gap(1)
                    bottomMargin = gap(1)
                },
            )
        }
        if (feedbackShots.size < MOST_SHOTS) {
            content.addView(
                LinearLayout(this).apply {
                    orientation = LinearLayout.VERTICAL
                    gravity = Gravity.CENTER
                    minimumHeight = gap(21)
                    setPadding(gap(4), gap(4), gap(4), gap(4))
                    isClickable = true
                    isFocusable = true
                    contentDescription = getString(R.string.omni_feedback_shot_pick)
                    readAs(Button::class.java.name)
                    background = touchable(
                        GradientDrawable().apply {
                            this.shape = GradientDrawable.RECTANGLE
                            cornerRadius = gap(4).toFloat()
                            setColor(Color.TRANSPARENT)
                            setStroke(
                                gap(1) / 2,
                                palette.divider,
                                gap(2).toFloat(),
                                gap(2).toFloat(),
                            )
                        },
                        palette.accent,
                    )
                    setOnClickListener { chooseShot() }
                    addView(
                        View(context).apply {
                            foreground = Mark(Mark.Shape.IMAGE, palette.muted)
                        },
                        LinearLayout.LayoutParams(gap(7), gap(7)),
                    )
                    addView(
                        TextView(context).apply {
                            text = getString(R.string.omni_feedback_shot_pick)
                            setTextColor(palette.accent)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                            gravity = Gravity.CENTER
                            setPadding(0, gap(2), 0, 0)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = getString(R.string.omni_feedback_shot_note, SHOT_LARGEST_MB)
                            setTextColor(palette.muted)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                            gravity = Gravity.CENTER
                        }
                    )
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply { topMargin = gap(1) },
            )
        }

        content.addView(label(getString(R.string.omni_feedback_reach)))
        val reply = card()
        reply.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                addView(
                    View(context).apply { foreground = Mark(Mark.Shape.CLOCK, palette.muted) },
                    LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(2) },
                )
                addView(
                    TextView(context).apply {
                        text = getString(R.string.omni_feedback_reach_soon)
                        setTextColor(palette.foreground)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            }
        )
        reply.addView(quiet(getString(R.string.omni_feedback_reach_note)))
        content.addView(reply)

        content.addView(primary(getString(R.string.omni_feedback_send)) { sendFeedback() })
        val resting = restingFor()
        if (resting > 0L) {
            content.addView(
                quiet(
                    getString(
                        R.string.omni_feedback_rest,
                        ((resting + 59_999L) / 60_000L).toInt(),
                    )
                )
            )
        }
        content.addView(
            noteCard(
                getString(R.string.omni_language_told),
                getString(R.string.omni_feedback_send_note),
            )
        )
    }

    private fun chooseShot() {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "image/*"
            putExtra(Intent.EXTRA_ALLOW_MULTIPLE, true)
        }
        runCatching { startActivityForResult(intent, SHOT_REQUEST) }
            .onFailure {
                results.removeAllViews()
                results.addView(
                    notice(it.message ?: getString(R.string.omni_form_image_none), palette.error)
                )
            }
    }

    private fun helpChip(named: String, onPress: () -> Unit): View = TextView(this).apply {
        text = named
        setTextColor(Ink.hot(palette.muted, 0.4f))
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
        gravity = Gravity.CENTER
        setPadding(gap(3), gap(3) / 2, gap(3), gap(3) / 2)
        isClickable = true
        isFocusable = true
        readAs(Button::class.java.name)
        background = touchable(
            GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(5).toFloat()
                setColor(Ink.fade(palette.accent, 0.1f))
                setStroke(1, Ink.fade(palette.accent, 0.24f))
            },
            palette.accent,
        )
        setOnClickListener { onPress() }
    }

    private fun helpEntry(
        mark: Mark.Shape,
        tint: Int,
        title: String,
        detail: String,
        onPress: () -> Unit,
    ): View = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(gap(4), gap(4), gap(4), gap(4))
        isClickable = true
        isFocusable = true
        contentDescription = said(title, detail)
        readAs(Button::class.java.name)
        background = touchable(
            GradientDrawable().apply {
                this.shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(4).toFloat()
                setColor(palette.surface)
                setStroke(1, Ink.fade(tint, 0.35f))
            },
            tint,
        )
        setOnClickListener { onPress() }
        addView(
            View(context).apply {
                foreground = Mark(mark, tint, 1.3f, glow = true)
                importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
            },
            LinearLayout.LayoutParams(gap(15), gap(15)),
        )
        addView(
            LinearLayout(context).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, gap(4), 0, gap(1))
                addView(
                    TextView(context).apply {
                        text = title
                        setTextColor(palette.foreground)
                        typeface = Type.heading
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                addView(
                    View(context).apply {
                        foreground = Mark(Mark.Shape.CHEVRON_RIGHT, palette.accent, 1.15f)
                    },
                    LinearLayout.LayoutParams(gap(4), gap(4)),
                )
            }
        )
        addView(
            TextView(context).apply {
                text = detail
                setTextColor(palette.muted)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                setLineSpacing(gap(3) / 4f, 1f)
            }
        )
    }

    private fun helpFooter() {
        content.addView(
            noteCard(
                getString(R.string.omni_help_more),
                getString(R.string.omni_help_contact_note),
                subtle(getString(R.string.omni_feedback_title), palette.accent) {
                    go(Screen.Feedback)
                },
            )
        )
        content.addView(
            TextView(this).apply {
                text = getString(R.string.omni_help_signed)
                setTextColor(palette.muted)
                typeface = Type.label
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                letterSpacing = Type.TRACKING
                gravity = Gravity.END
                setPadding(0, gap(1), 0, gap(3))
            }
        )
    }

    private fun renderHelp() {
        content.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, 0, 0, gap(1))
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_help_hello)
                                setTextColor(palette.foreground)
                                typeface = Type.heading
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 20f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_help_note)
                                setTextColor(Ink.hot(palette.muted, 0.28f))
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                                setLineSpacing(gap(3) / 4f, 1f)
                                setPadding(0, gap(2), 0, 0)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                addView(
                    ImageView(context).apply {
                        setImageResource(R.drawable.omni_art_help_ask)
                        scaleType = ImageView.ScaleType.FIT_CENTER
                        importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                    },
                    LinearLayout.LayoutParams(gap(34), gap(26)).apply { marginStart = gap(2) },
                )
            }
        )

        content.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                minimumHeight = gap(9)
                setPadding(gap(5), 0, gap(5), 0)
                background = GradientDrawable().apply {
                    shape = GradientDrawable.RECTANGLE
                    cornerRadius = gap(5).toFloat()
                    setColor(palette.raised)
                    setStroke(1, Ink.fade(palette.accent, 0.4f))
                }
                addView(
                    View(context).apply { foreground = Mark(Mark.Shape.FIND, palette.muted, 1.1f) },
                    LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(4) },
                )
                addView(
                    EditText(context).apply {
                        setText(helpFor)
                        hint = getString(R.string.omni_help_search)
                        setHintTextColor(palette.muted)
                        setTextColor(palette.foreground)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                        setBackgroundColor(Color.TRANSPARENT)
                        setPadding(0, gap(2), 0, gap(2))
                        isSingleLine = true
                        setOnEditorActionListener { _, _, _ ->
                            go(Screen.HelpFaq)
                            true
                        }
                        addTextChangedListener(object : TextWatcher {
                            override fun beforeTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
                            override fun onTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
                            override fun afterTextChanged(s: Editable?) {
                                helpFor = s?.toString().orEmpty()
                            }
                        })
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            },
            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply { topMargin = gap(3) },
        )

        content.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, gap(4), 0, gap(2))
                addView(
                    TextView(context).apply {
                        text = getString(R.string.omni_help_popular)
                        setTextColor(palette.foreground)
                        typeface = Type.heading
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                    },
                    LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT).apply {
                        marginEnd = gap(3)
                    },
                )
                addView(
                    HorizontalScrollView(context).apply {
                        isHorizontalScrollBarEnabled = false
                        addView(
                            LinearLayout(context).apply {
                                orientation = LinearLayout.HORIZONTAL
                                HELP_POPULAR.forEachIndexed { which, named ->
                                    addView(
                                        helpChip(getString(named)) {
                                            go(
                                                when (which) {
                                                    0 -> Screen.HelpAdb
                                                    1 -> Screen.HelpTalk
                                                    2 -> Screen.HelpDocs
                                                    else -> Screen.HelpFaq
                                                }
                                            )
                                        },
                                        LinearLayout.LayoutParams(
                                            WRAP_CONTENT,
                                            WRAP_CONTENT,
                                        ).apply { marginEnd = gap(2) },
                                    )
                                }
                            },
                            MATCH_PARENT,
                            WRAP_CONTENT,
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
            }
        )

        content.addView(
            grid(
                2,
                listOf(
                    helpEntry(
                        Mark.Shape.LOG,
                        palette.accent,
                        getString(R.string.omni_help_adb),
                        getString(R.string.omni_help_adb_note),
                    ) { go(Screen.HelpAdb) },
                    helpEntry(
                        Mark.Shape.BRAIN,
                        0xFFB06BFF.toInt(),
                        getString(R.string.omni_menu_assistant),
                        getString(R.string.omni_help_talk_note),
                    ) { go(Screen.HelpTalk) },
                    helpEntry(
                        Mark.Shape.CODE,
                        palette.ok,
                        getString(R.string.omni_help_band_terminal),
                        getString(R.string.omni_help_terminal_note),
                    ) { go(Screen.HelpDocs) },
                    helpEntry(
                        Mark.Shape.BOOK,
                        0xFFFF5C8A.toInt(),
                        getString(R.string.omni_help_faq),
                        getString(R.string.omni_help_faq_note),
                    ) { go(Screen.HelpFaq) },
                ),
            )
        )
        helpFooter()
    }

    private fun filledMark(mark: Mark.Shape, tint: Int, round: Int): View =
        FrameLayout(this).apply {
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = round.toFloat()
                setColor(tint)
            }
            importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
            addView(
                View(context).apply { foreground = Mark(mark, Color.WHITE, 1.2f) },
                FrameLayout.LayoutParams(MATCH_PARENT, MATCH_PARENT).apply {
                    val inset = gap(3) / 2
                    setMargins(inset, inset, inset, inset)
                },
            )
        }

    private data class GuideBlock(val code: Boolean, val body: String)

    private data class GuideEntry(val number: Int, val heading: String, val blocks: List<GuideBlock>) {
        val words: String = (heading + " " + blocks.joinToString(" ") { it.body })
            .lowercase(Locale.ROOT)
    }

    private data class GuideSection(val title: String, val lead: String, val entries: List<GuideEntry>)

    private fun terminalGuide(): List<GuideSection> {
        val tag = Preferences.language(this).ifEmpty { deviceLanguage() }
        guideHeld?.let { (held, sections) -> if (held == tag) return sections }
        if (Builder.load() !is Builder.LoadState.Loaded) return emptyList()
        val sections = runCatching {
            val said = JSONObject(Builder.nativeTerminalDocs(tag)).optJSONArray("sections")
                ?: JSONArray()
            (0 until said.length()).map { at ->
                val section = said.getJSONObject(at)
                val entries = section.optJSONArray("entries") ?: JSONArray()
                GuideSection(
                    section.optString("title"),
                    section.optString("lead"),
                    (0 until entries.length()).map { index ->
                        val entry = entries.getJSONObject(index)
                        val blocks = entry.optJSONArray("blocks") ?: JSONArray()
                        GuideEntry(
                            entry.optInt("number"),
                            entry.optString("heading"),
                            (0 until blocks.length()).map { one ->
                                val block = blocks.getJSONObject(one)
                                GuideBlock(block.optString("kind") == "code", block.optString("body"))
                            },
                        )
                    },
                )
            }
        }.getOrElse {
            OmniLog.event(LogLevel.WARN, "native", "The terminal guide could not be read.")
            emptyList()
        }
        if (sections.isNotEmpty()) {
            guideHeld = tag to sections
        }
        return sections
    }

    private fun renderHelpDocs() {
        content.addView(
            guideHead(
                Mark.Shape.CODE,
                0xFF1FC77A.toInt(),
                getString(R.string.omni_help_terminal),
                getString(R.string.omni_help_terminal_body),
                R.drawable.omni_art_docs_read,
            )
        )
        val sections = terminalGuide()
        if (sections.isEmpty()) {
            content.addView(quiet(getString(R.string.omni_help_none)))
            helpFooter()
            return
        }
        val picked = guideSection.coerceIn(0, sections.lastIndex)
        content.addView(
            bandRow(sections.map { it.title }, picked) {
                guideSection = it
                guideOpen = -1
                render(false)
            }
        )
        val section = sections[picked]
        if (section.lead.isNotEmpty()) {
            content.addView(
                TextView(this).apply {
                    text = section.lead
                    setTextColor(palette.foreground)
                    typeface = Type.heading
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                    setPadding(0, gap(1), 0, gap(2))
                }
            )
        }
        val list = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        val hollow = quiet(getString(R.string.omni_help_none))
        fun fill() {
            list.removeAllViews()
            val needle = helpFor.trim().lowercase(Locale.ROOT)
            section.entries
                .filter { needle.isEmpty() || it.words.contains(needle) }
                .forEach { entry ->
                    list.addView(
                        guideRow(entry) {
                            guideOpen = if (guideOpen == entry.number) -1 else entry.number
                            fill()
                        }
                    )
                }
            hollow.visibility = if (list.childCount == 0) View.VISIBLE else View.GONE
        }
        content.addView(
            seekField(getString(R.string.omni_terminal_search), helpFor) {
                helpFor = it
                fill()
            }
        )
        content.addView(list, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
        content.addView(hollow)
        fill()
        helpFooter()
    }

    private fun guideRow(entry: GuideEntry, onToggle: () -> Unit): View {
        val open = entry.number == guideOpen
        val tint = GUIDE_TINTS[(entry.number - 1).mod(GUIDE_TINTS.size)]
        return LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(3), gap(3), gap(3), gap(3))
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(3).toFloat()
                setColor(if (open) palette.surface else Color.TRANSPARENT)
                setStroke(gap(1) / 2, if (open) Ink.fade(tint, 0.5f) else palette.divider)
            }
            layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                topMargin = gap(1)
                bottomMargin = gap(1)
            }
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    isClickable = true
                    isFocusable = true
                    contentDescription = entry.heading
                    readAs(Button::class.java.name)
                    background = reacting(Color.TRANSPARENT, gap(2).toFloat(), tint)
                    setOnClickListener { onToggle() }
                    addView(
                        filledMark(Mark.Shape.CODE, Ink.fade(tint, 0.9f), gap(2)),
                        LinearLayout.LayoutParams(gap(8), gap(8)).apply { marginEnd = gap(3) },
                    )
                    addView(
                        LinearLayout(context).apply {
                            orientation = LinearLayout.VERTICAL
                            addView(
                                TextView(context).apply {
                                    text = entry.heading
                                    setTextColor(palette.foreground)
                                    typeface = Type.heading
                                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                                }
                            )
                            val first = entry.blocks.firstOrNull { !it.code }?.body.orEmpty()
                            if (!open && first.isNotEmpty()) {
                                addView(
                                    TextView(context).apply {
                                        text = first
                                        setTextColor(palette.muted)
                                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                                        maxLines = 2
                                        ellipsize = android.text.TextUtils.TruncateAt.END
                                        setPadding(0, gap(1) / 2, 0, 0)
                                    }
                                )
                            }
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                    addView(
                        View(context).apply {
                            foreground = Mark(
                                if (open) Mark.Shape.CHEVRON_UP else Mark.Shape.CHEVRON_DOWN,
                                tint,
                                1.1f,
                            )
                        },
                        LinearLayout.LayoutParams(gap(5), gap(5)).apply { marginStart = gap(2) },
                    )
                }
            )
            if (open) {
                entry.blocks.forEach { block ->
                    if (block.code) {
                        addView(
                            commandPanel(block.body),
                            LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                                topMargin = gap(2)
                            },
                        )
                    } else {
                        addView(
                            TextView(context).apply {
                                text = block.body
                                setTextColor(Ink.hot(palette.muted, 0.35f))
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                                setLineSpacing(gap(1) / 2f, 1f)
                                setTextIsSelectable(true)
                                setPadding(0, gap(2), 0, 0)
                            }
                        )
                    }
                }
            }
        }
    }

    private fun commandPanel(body: String): View = LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.TOP
        setPadding(gap(3), gap(2), gap(2), gap(2))
        background = GradientDrawable().apply {
            shape = GradientDrawable.RECTANGLE
            cornerRadius = gap(2).toFloat()
            setColor(palette.background)
            setStroke(1, palette.divider)
        }
        addView(
            HorizontalScrollView(context).apply {
                isHorizontalScrollBarEnabled = false
                addView(
                    TextView(context).apply {
                        text = body
                        setTextColor(Ink.hot(palette.foreground, 0.1f))
                        typeface = Type.data
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                        setLineSpacing(gap(1) / 2f, 1f)
                        setTextIsSelectable(true)
                    },
                    WRAP_CONTENT,
                    WRAP_CONTENT,
                )
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
        )
        addView(
            View(context).apply {
                foreground = Mark(Mark.Shape.COPY, palette.muted)
                isClickable = true
                isFocusable = true
                contentDescription = getString(R.string.omni_action_copy)
                readAs(Button::class.java.name)
                background = reacting(Color.TRANSPARENT, gap(5).toFloat(), palette.accent)
                setOnClickListener {
                    val clipboard = getSystemService(ClipboardManager::class.java)
                    if (clipboard != null) {
                        clipboard.setPrimaryClip(
                            ClipData.newPlainText(getString(R.string.omni_help_terminal), body)
                        )
                        flash(
                            getString(R.string.omni_settings_logs_copied),
                            palette.ok,
                            Mark.Shape.COPY,
                        )
                    }
                }
            },
            LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginStart = gap(2) },
        )
    }

    private fun foldRow(at: Int, question: String, answer: String): View {
        val open = at == helpOpen
        val (mark, tint) = helpMark(at)
        return LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(3), gap(3), gap(3), gap(3))
            background = touchable(
                GradientDrawable().apply {
                    this.shape = GradientDrawable.RECTANGLE
                    cornerRadius = gap(3).toFloat()
                    setColor(if (open) palette.surface else Color.TRANSPARENT)
                    setStroke(gap(1) / 2, if (open) Ink.fade(tint, 0.5f) else palette.divider)
                },
                tint,
            )
            layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                topMargin = gap(1)
                bottomMargin = gap(1)
            }
            isClickable = true
            isFocusable = true
            contentDescription = question
            readAs(Button::class.java.name)
            setOnClickListener {
                helpOpen = if (open) -1 else at
                render(false)
            }
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    addView(
                        filledMark(mark, Ink.fade(tint, 0.9f), gap(2)),
                        LinearLayout.LayoutParams(gap(8), gap(8)).apply { marginEnd = gap(3) },
                    )
                    addView(
                        TextView(context).apply {
                            text = getString(R.string.omni_help_numbered, at + 1, question)
                            setTextColor(palette.foreground)
                            typeface = Type.heading
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                    addView(
                        View(context).apply {
                            foreground = Mark(
                                if (open) Mark.Shape.CHEVRON_UP else Mark.Shape.CHEVRON_DOWN,
                                palette.accent,
                                1.15f,
                            )
                        },
                        LinearLayout.LayoutParams(gap(5), gap(5)),
                    )
                }
            )
            if (open) {
                if (at == 0) {
                    addView(
                        codePanel(projectTree()),
                        LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                            topMargin = gap(2)
                        },
                    )
                }
                addView(
                    TextView(context).apply {
                        text = answer
                        setTextColor(Ink.hot(palette.muted, 0.3f))
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                        setLineSpacing(gap(1) / 2f, 1f)
                        setPadding(0, gap(2), 0, gap(1))
                    }
                )
            }
        }
    }

    private fun projectTree(): String {
        val root = openProject ?: "/storage/emulated/0/Projects/My App"
        return buildString {
            append(root).append('/').append('\n')
            append("\u251C\u2500\u2500 AndroidManifest.xml\n")
            append("\u251C\u2500\u2500 ").append(PROJECT_FILE).append('\n')
            append("\u251C\u2500\u2500 Java\n")
            append("\u2502   \u2514\u2500\u2500 App.java\n")
            append("\u2514\u2500\u2500 Res\n")
            append("    \u251C\u2500\u2500 mipmap-xxxhdpi\n")
            append("    \u251C\u2500\u2500 layout\n")
            append("    \u251C\u2500\u2500 values\n")
            append("    \u2514\u2500\u2500 \u2026")
        }
    }

    private fun codePanel(body: String): View = android.widget.HorizontalScrollView(this).apply {
        isHorizontalScrollBarEnabled = false
        setPadding(gap(3), gap(2), gap(3), gap(2))
        background = GradientDrawable().apply {
            shape = GradientDrawable.RECTANGLE
            cornerRadius = gap(2).toFloat()
            setColor(palette.background)
            setStroke(1, palette.divider)
        }
        addView(
            TextView(context).apply {
                val cut = body.indexOf('\n')
                text = SpannableStringBuilder(body).apply {
                    if (cut > 0) {
                        setSpan(
                            ForegroundColorSpan(palette.marker),
                            0,
                            cut,
                            Spannable.SPAN_EXCLUSIVE_EXCLUSIVE,
                        )
                    }
                }
                setTextColor(Ink.hot(palette.muted, 0.35f))
                typeface = Type.data
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
                setLineSpacing(gap(1) / 2f, 1f)
            },
            WRAP_CONTENT,
            WRAP_CONTENT,
        )
    }

    private fun renderHelpFaq() {
        content.addView(
            guideHead(
                Mark.Shape.BOOK,
                0xFFEC4899.toInt(),
                getString(R.string.omni_help_faq),
                getString(R.string.omni_help_faq_note),
            )
        )
        content.addView(
            seekField(getString(R.string.omni_faq_search), helpFor) { helpFor = it }
        )
        content.addView(
            bandRow(HELP_BAND_NAMES.map { getString(it) }, helpBand) {
                helpBand = it
                helpOpen = -1
                render(false)
            }
        )
        val needle = helpFor.trim().lowercase(Locale.getDefault())
        var shown = 0
        HELP_TOPICS.forEachIndexed { index, topic ->
            val band = HELP_BANDS[index]
            val whole = (getString(topic.first) + " " + getString(topic.second))
                .lowercase(Locale.getDefault())
            if ((helpBand == 0 || band == helpBand) &&
                (needle.isEmpty() || whole.contains(needle))
            ) {
                shown += 1
                content.addView(foldRow(index, getString(topic.first), getString(topic.second)))
            }
        }
        if (shown == 0) {
            content.addView(quiet(getString(R.string.omni_help_none)))
        }
        helpFooter()
    }

    private fun renderHelpTalk() {
        content.addView(
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                gravity = Gravity.CENTER_VERTICAL
                setPadding(0, gap(4), 0, gap(2))
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.VERTICAL
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_help_talk_title)
                                setTextColor(palette.foreground)
                                typeface = Type.heading
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 22f)
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(R.string.omni_help_talk_note)
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                                setLineSpacing(gap(1).toFloat(), 1f)
                            }
                        )
                    },
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                )
                addView(
                    ImageView(context).apply {
                        setImageResource(R.drawable.omni_art_help_ask)
                        scaleType = ImageView.ScaleType.FIT_CENTER
                    },
                    LinearLayout.LayoutParams(gap(22), gap(18)).apply { marginStart = gap(2) },
                )
            }
        )
        content.addView(
            grid(
                2,
                TALK_STEPS.mapIndexed { index, step ->
                    LinearLayout(this).apply {
                        orientation = LinearLayout.VERTICAL
                        setPadding(gap(3), gap(3), gap(3), gap(3))
                        background = GradientDrawable().apply {
                            this.shape = GradientDrawable.RECTANGLE
                            cornerRadius = gap(4).toFloat()
                            setColor(palette.surface)
                            setStroke(gap(1) / 2, palette.divider)
                        }
                        addView(
                            LinearLayout(context).apply {
                                orientation = LinearLayout.HORIZONTAL
                                gravity = Gravity.CENTER_VERTICAL
                                addView(
                                    TextView(context).apply {
                                        text = (index + 1).toString()
                                        setTextColor(Color.WHITE)
                                        typeface = Type.heading
                                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                                        gravity = Gravity.CENTER
                                        background = GradientDrawable().apply {
                                            shape = GradientDrawable.OVAL
                                            setColor(palette.accent)
                                        }
                                    },
                                    LinearLayout.LayoutParams(gap(6), gap(6)).apply {
                                        marginEnd = gap(2)
                                    },
                                )
                                addView(
                                    TextView(context).apply {
                                        text = getString(step.first)
                                        setTextColor(palette.foreground)
                                        typeface = Type.heading
                                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                                    },
                                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                                )
                            }
                        )
                        addView(
                            TextView(context).apply {
                                text = getString(step.second)
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
                                setLineSpacing(gap(1) / 2f, 1f)
                                setPadding(0, gap(2), 0, 0)
                            }
                        )
                        talkMock(index)?.let { mock ->
                            addView(
                                mock,
                                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                                    topMargin = gap(2)
                                },
                            )
                        }
                    }
                },
            )
        )
        content.addView(primary(getString(R.string.omni_menu_assistant)) { go(Screen.Assistant) })
        helpFooter()
    }

    private fun linkPanel(title: String, url: String): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(gap(2), gap(2), gap(2), gap(2))
            isClickable = true
            isFocusable = true
            contentDescription = said(title, url)
            readAs(Button::class.java.name)
            background = touchable(
                GradientDrawable().apply {
                    shape = GradientDrawable.RECTANGLE
                    cornerRadius = gap(2).toFloat()
                    setColor(Ink.fade(palette.accent, 0.06f))
                    setStroke(1, Ink.fade(palette.accent, 0.3f))
                },
                palette.accent,
            )
            setOnClickListener { hand(Intent(Intent.ACTION_VIEW, Uri.parse(url))) }
            addView(
                View(context).apply { foreground = Mark(Mark.Shape.LINK, palette.accent, 1.15f) },
                LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(2) },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = title
                            setTextColor(palette.foreground)
                            typeface = Type.strong
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = url
                            setTextColor(Ink.hot(palette.accent, 0.45f))
                            typeface = Type.data
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 8f)
                            maxLines = 1
                            ellipsize = android.text.TextUtils.TruncateAt.END
                        }
                    )
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            addView(
                View(context).apply {
                    foreground = Mark(Mark.Shape.EXPORT, palette.accent, 1.1f)
                    importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                },
                LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginStart = gap(2) },
            )
        }

    private fun noteStrip(body: String): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(gap(3), gap(2), gap(3), gap(2))
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(2).toFloat()
                setColor(Ink.fade(palette.accent, 0.09f))
            }
            addView(
                View(context).apply {
                    foreground = Mark(Mark.Shape.INFO_FILL, palette.accent)
                    importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                },
                LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(3) },
            )
            addView(
                TextView(context).apply {
                    val lead = getString(R.string.omni_note_lead)
                    text = SpannableStringBuilder("$lead $body").apply {
                        setSpan(
                            StyleSpan(Typeface.BOLD),
                            0,
                            lead.length,
                            Spannable.SPAN_EXCLUSIVE_EXCLUSIVE,
                        )
                    }
                    setTextColor(Ink.hot(palette.muted, 0.3f))
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
                    setLineSpacing(gap(1) / 2f, 1f)
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
        }

    private fun guideStep(
        at: Int,
        title: String,
        detail: String,
        art: Int,
        extra: List<View> = emptyList(),
    ): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(3), gap(3), gap(3), gap(3))
            background = GradientDrawable().apply {
                this.shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(4).toFloat()
                setColor(palette.surface)
                setStroke(gap(1) / 2, palette.divider)
            }
            layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                topMargin = gap(2)
                bottomMargin = gap(1)
            }
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    addView(
                        TextView(context).apply {
                            text = at.toString()
                            setTextColor(Color.WHITE)
                            typeface = Type.heading
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                            gravity = Gravity.CENTER
                            background = GradientDrawable().apply {
                                shape = GradientDrawable.OVAL
                                setColor(Ink.fade(palette.accent, 0.9f))
                            }
                        },
                        LinearLayout.LayoutParams(gap(7), gap(7)).apply { marginEnd = gap(3) },
                    )
                    addView(
                        TextView(context).apply {
                            text = title
                            setTextColor(palette.foreground)
                            typeface = Type.heading
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                }
            )
            addView(
                TextView(context).apply {
                    text = detail
                    setTextColor(palette.muted)
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
                    setLineSpacing(gap(1) / 2f, 1f)
                    setPadding(0, gap(2), 0, gap(1))
                }
            )
            extra.forEach { piece ->
                addView(
                    piece,
                    LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                        topMargin = gap(2)
                    },
                )
            }
            if (art != 0) {
                addView(
                    ImageView(context).apply {
                        setImageResource(art)
                        scaleType = ImageView.ScaleType.FIT_CENTER
                        adjustViewBounds = true
                    },
                    LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                        topMargin = gap(2)
                    },
                )
            }
        }

    private fun bulleted(body: String): String {
        if (body.contains('\n')) {
            return body
        }
        return body.split(Regex("(?<=[.!?])\\s+"))
            .filter { it.isNotBlank() }
            .joinToString("\n") { "\u2022 " + it.trim() }
    }

    private fun guideHero(
        mark: Mark.Shape,
        tint: Int,
        title: String,
        note: String,
        art: Int,
    ): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            setPadding(0, gap(1), 0, gap(1))
            addView(
                filledMark(mark, Ink.fade(tint, 0.85f), gap(2)),
                LinearLayout.LayoutParams(gap(11), gap(11)).apply { marginEnd = gap(3) },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = title
                            setTextColor(palette.foreground)
                            typeface = Type.heading
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 16f)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = note
                            setTextColor(Ink.hot(palette.muted, 0.28f))
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
                            setLineSpacing(gap(1) / 2f, 1f)
                            setPadding(0, gap(1), 0, 0)
                        }
                    )
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            addView(
                ImageView(context).apply {
                    setImageResource(art)
                    scaleType = ImageView.ScaleType.FIT_CENTER
                    importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                },
                LinearLayout.LayoutParams(gap(16), gap(14)).apply { marginStart = gap(1) },
            )
            addView(quipBubble(), LinearLayout.LayoutParams(gap(18), WRAP_CONTENT))
        }

    private fun quipBubble(): View = TextView(this).apply {
        text = getString(R.string.omni_guide_quip)
        setTextColor(Ink.hot(palette.muted, 0.35f))
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 9f)
        setLineSpacing(gap(1) / 2f, 1f)
        setPadding(gap(3) / 2, gap(3) / 2, gap(3) / 2, gap(3) / 2)
        importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
        breakStrategy = android.text.Layout.BREAK_STRATEGY_SIMPLE
        hyphenationFrequency = android.text.Layout.HYPHENATION_FREQUENCY_NONE
        background = GradientDrawable().apply {
            shape = GradientDrawable.RECTANGLE
            cornerRadius = gap(2).toFloat()
            setColor(Ink.fade(palette.accent, 0.1f))
            setStroke(1, Ink.fade(palette.accent, 0.35f))
        }
    }

    private fun mockLine(mark: Mark.Shape, tint: Int, named: String): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(gap(2), gap(1), gap(2), gap(1))
            addView(
                View(context).apply { foreground = Mark(mark, tint) },
                LinearLayout.LayoutParams(gap(4), gap(4)).apply { marginEnd = gap(2) },
            )
            addView(
                TextView(context).apply {
                    text = named
                    setTextColor(Ink.hot(palette.muted, 0.3f))
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 9f)
                    maxLines = 1
                    ellipsize = android.text.TextUtils.TruncateAt.END
                }
            )
        }

    private fun mockPanel(rows: List<View>): View = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        setPadding(gap(1), gap(1), gap(1), gap(1))
        importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
        background = GradientDrawable().apply {
            shape = GradientDrawable.RECTANGLE
            cornerRadius = gap(2).toFloat()
            setColor(palette.background)
            setStroke(1, palette.divider)
        }
        rows.forEach { addView(it, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT)) }
    }

    private fun mockAsk(): View = LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER_VERTICAL
        importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
        addView(
            TextView(context).apply {
                text = getString(R.string.omni_talk_hint)
                setTextColor(palette.muted)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 9f)
                maxLines = 1
                ellipsize = android.text.TextUtils.TruncateAt.END
                setPadding(gap(3), gap(2), gap(3), gap(2))
                background = pill(palette.background, gap(4).toFloat())
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply { marginEnd = gap(2) },
        )
        addView(
            View(context).apply {
                background = GradientDrawable().apply {
                    shape = GradientDrawable.OVAL
                    setColor(palette.accent)
                }
                foreground = Mark(Mark.Shape.SEND, Color.WHITE)
            },
            LinearLayout.LayoutParams(gap(7), gap(7)),
        )
    }

    private fun mockJudged(good: Boolean, head: String, said: String): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(2), gap(2), gap(2), gap(2))
            importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
            val tint = if (good) palette.ok else palette.error
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(2).toFloat()
                setColor(Ink.fade(tint, 0.1f))
                setStroke(1, Ink.fade(tint, 0.4f))
            }
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    addView(
                        View(context).apply {
                            foreground = Mark(
                                if (good) Mark.Shape.TICK else Mark.Shape.CLOSE,
                                tint,
                            )
                        },
                        LinearLayout.LayoutParams(gap(4), gap(4)).apply { marginEnd = gap(2) },
                    )
                    addView(
                        TextView(context).apply {
                            text = head
                            setTextColor(tint)
                            typeface = Type.strong
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 9f)
                        }
                    )
                }
            )
            addView(
                TextView(context).apply {
                    text = said
                    setTextColor(Ink.hot(palette.muted, 0.3f))
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 9f)
                    setLineSpacing(gap(1) / 2f, 1f)
                    setPadding(0, gap(1), 0, 0)
                }
            )
        }

    private fun talkMock(at: Int): View? = when (at) {
        0 -> mockAsk()
        1 -> mockPanel(
            listOf(
                mockLine(Mark.Shape.FOLDER, palette.marker, getString(R.string.omni_talk_folder)),
                mockLine(
                    Mark.Shape.DROID,
                    palette.ok,
                    openProject?.let { File(it).name }
                        ?: getString(R.string.omni_projects_none),
                ),
            )
        )
        2 -> mockPanel(
            listOf(
                mockLine(
                    Mark.Shape.NOTE,
                    palette.accent,
                    getString(R.string.omni_talk_add_file),
                ),
                mockLine(
                    Mark.Shape.IMAGE,
                    palette.warning,
                    getString(R.string.omni_talk_add_picture),
                ),
                mockLine(
                    Mark.Shape.LINK,
                    palette.ok,
                    getString(R.string.omni_talk_add_link),
                ),
            )
        )
        3 -> mockPanel(
            listOf(
                mockLine(
                    Mark.Shape.ROBOT,
                    palette.accent,
                    getString(R.string.omni_talk_switch),
                ),
                mockLine(
                    Mark.Shape.HELP,
                    palette.muted,
                    getString(R.string.omni_help_title),
                ),
            )
        )
        4 -> mockPanel(
            listOf(
                mockLine(
                    Mark.Shape.TRASH,
                    palette.error,
                    getString(R.string.omni_talk_clear),
                ),
                mockLine(
                    Mark.Shape.SHARE,
                    palette.accent,
                    getString(R.string.omni_talk_export),
                ),
            )
        )
        5 -> LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            addView(
                mockJudged(
                    false,
                    getString(R.string.omni_talk_vague),
                    getString(R.string.omni_talk_vague_case),
                ),
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
            addView(
                mockJudged(
                    true,
                    getString(R.string.omni_talk_sharp),
                    getString(R.string.omni_talk_sharp_case),
                ),
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                    topMargin = gap(2)
                },
            )
        }
        else -> null
    }

    private fun osRow(): View = LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        OS_TABS.forEachIndexed { at, one ->
            val on = at == osTab
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER
                    minimumHeight = gap(9)
                    setPadding(gap(2), gap(2), gap(2), gap(2))
                    isClickable = true
                    isFocusable = true
                    isSelected = on
                    contentDescription = getString(one.first)
                    readAs(Button::class.java.name)
                    background = touchable(
                        GradientDrawable().apply {
                            shape = GradientDrawable.RECTANGLE
                            cornerRadius = gap(5).toFloat()
                            setColor(if (on) palette.accent else Ink.fade(palette.accent, 0.08f))
                            setStroke(1, if (on) palette.accent else Ink.fade(palette.accent, 0.3f))
                        },
                        palette.accent,
                    )
                    setOnClickListener {
                        osTab = at
                        render(false)
                    }
                    addView(
                        ImageView(context).apply {
                            setImageResource(one.second)
                            scaleType = ImageView.ScaleType.FIT_CENTER
                            importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                        },
                        LinearLayout.LayoutParams(gap(5), gap(5)).apply { marginEnd = gap(2) },
                    )
                    addView(
                        TextView(context).apply {
                            text = getString(one.first)
                            setTextColor(if (on) Color.WHITE else Ink.hot(palette.muted, 0.3f))
                            typeface = if (on) Type.strong else Type.body
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                        }
                    )
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply {
                    marginStart = if (at == 0) 0 else gap(1)
                },
            )
        }
    }

    private fun countedLines(lines: List<String>): View = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        lines.forEachIndexed { at, line ->
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    setPadding(0, gap(1), 0, gap(1))
                    addView(
                        TextView(context).apply {
                            text = (at + 1).toString()
                            setTextColor(Color.WHITE)
                            typeface = Type.heading
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 9f)
                            gravity = Gravity.CENTER
                            background = GradientDrawable().apply {
                                shape = GradientDrawable.OVAL
                                setColor(Ink.fade(palette.accent, 0.9f))
                            }
                        },
                        LinearLayout.LayoutParams(gap(5), gap(5)).apply { marginEnd = gap(2) },
                    )
                    addView(
                        TextView(context).apply {
                            text = line
                            setTextColor(Ink.hot(palette.muted, 0.3f))
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
                            setLineSpacing(gap(1) / 2f, 1f)
                        },
                        LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
                    )
                },
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
        }
    }

    private fun filePanel(named: String, names: List<String>): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(gap(2), gap(2), gap(2), gap(2))
            importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(2).toFloat()
                setColor(palette.background)
                setStroke(1, palette.divider)
            }
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.HORIZONTAL
                    gravity = Gravity.CENTER_VERTICAL
                    setPadding(0, 0, 0, gap(1))
                    addView(
                        View(context).apply {
                            foreground = Mark(Mark.Shape.CHEVRON_LEFT, palette.muted)
                        },
                        LinearLayout.LayoutParams(gap(4), gap(4)).apply { marginEnd = gap(2) },
                    )
                    addView(
                        View(context).apply {
                            foreground = Mark(Mark.Shape.FOLDER, palette.marker)
                        },
                        LinearLayout.LayoutParams(gap(4), gap(4)).apply { marginEnd = gap(2) },
                    )
                    addView(
                        TextView(context).apply {
                            text = named
                            setTextColor(palette.foreground)
                            typeface = Type.strong
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 10f)
                            maxLines = 1
                        }
                    )
                }
            )
            names.forEach { one ->
                addView(
                    LinearLayout(context).apply {
                        orientation = LinearLayout.HORIZONTAL
                        gravity = Gravity.CENTER_VERTICAL
                        setPadding(gap(3), gap(1) / 2, 0, gap(1) / 2)
                        addView(
                            View(context).apply {
                                foreground = Mark(
                                    if (one.endsWith(".exe")) Mark.Shape.BINARY
                                    else Mark.Shape.FILE,
                                    palette.muted,
                                )
                            },
                            LinearLayout.LayoutParams(gap(4), gap(4)).apply {
                                marginEnd = gap(2)
                            },
                        )
                        addView(
                            TextView(context).apply {
                                text = one
                                setTextColor(Ink.hot(palette.muted, 0.2f))
                                typeface = Type.data
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 9f)
                                maxLines = 1
                            }
                        )
                    },
                    LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
                )
            }
        }

    private fun osSteps(): List<View> {
        val lines = OS_STEPS[osTab].map { getString(it) }
        val files = OS_FILES[osTab]
        return listOf(
            osRow(),
            LinearLayout(this).apply {
                orientation = LinearLayout.HORIZONTAL
                addView(
                    countedLines(lines),
                    LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply { marginEnd = gap(2) },
                )
                addView(
                    filePanel("platform-tools", files),
                    LinearLayout.LayoutParams(gap(36), WRAP_CONTENT),
                )
            },
        )
    }

    private fun bulletCard(title: String, body: String): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            setPadding(gap(3), gap(3), gap(3), gap(3))
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(3).toFloat()
                setColor(Ink.fade(palette.accent, 0.07f))
                setStroke(1, Ink.fade(palette.accent, 0.4f))
            }
            layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
                topMargin = gap(2)
                bottomMargin = gap(1)
            }
            addView(
                View(context).apply {
                    foreground = Mark(Mark.Shape.INFO_FILL, palette.accent)
                    importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
                },
                LinearLayout.LayoutParams(gap(8), gap(8)).apply { marginEnd = gap(3) },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = title
                            setTextColor(palette.warning)
                            typeface = Type.heading
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = bulleted(body)
                            setTextColor(Ink.hot(palette.muted, 0.3f))
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                            setLineSpacing(gap(1) / 2f, 1f)
                            setPadding(0, gap(1), 0, 0)
                        }
                    )
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
        }

    private fun renderHelpAdb() {
        content.addView(
            bandRow(
                listOf(
                    getString(R.string.omni_adb_alone),
                    getString(R.string.omni_adb_desk),
                ),
                adbTab,
            ) {
                adbTab = it
                render(false)
            }
        )
        val marked = if (adbTab == 0) Mark.Shape.PHONE else Mark.Shape.SCREEN
        content.addView(
            guideHero(
                marked,
                palette.accent,
                getString(
                    if (adbTab == 0) R.string.omni_adb_alone else R.string.omni_adb_desk
                ),
                getString(
                    if (adbTab == 0) {
                        R.string.omni_adb_alone_note
                    } else {
                        R.string.omni_adb_desk_note
                    }
                ),
                if (adbTab == 1) R.drawable.omni_art_adb_desk
                else R.drawable.omni_art_help_ask,
            )
        )
        content.addView(
            bulletCard(
                getString(R.string.omni_adb_first),
                getString(
                    if (adbTab == 0) {
                        R.string.omni_adb_alone_first
                    } else {
                        R.string.omni_adb_desk_first
                    }
                ),
            )
        )
        val steps = if (adbTab == 0) ADB_ALONE else ADB_DESK
        val art = if (adbTab == 0) ADB_ALONE_ART else ADB_DESK_ART
        steps.forEachIndexed { index, step ->
            val extra = when {
                adbTab == 1 && index == 0 -> listOf(
                    linkPanel(
                        getString(R.string.omni_adb_tools_link),
                        getString(R.string.omni_adb_tools_url),
                    ),
                    noteStrip(getString(R.string.omni_adb_tools_only)),
                )
                adbTab == 1 && index == 1 -> osSteps()
                else -> emptyList()
            }
            content.addView(
                guideStep(
                    index + 1,
                    getString(step.first),
                    getString(step.second),
                    art.getOrElse(index) { 0 },
                    extra,
                )
            )
        }
        helpFooter()
    }

    private fun renderSettings() {
        val look = card()
        look.addView(
            row(
                getString(R.string.omni_settings_theme),
                palette.label,
                "",
                palette.accent,
            ) { go(Screen.Theme) }
        )
        look.addView(rule(), MATCH_PARENT, 1)
        val spoken = Preferences.language(this).ifEmpty { deviceLanguage() }
        look.addView(
            row(
                getString(R.string.omni_settings_language),
                Preferences.LANGUAGES.firstOrNull { it.first == spoken }?.second.orEmpty(),
                "",
                palette.accent,
            ) { go(Screen.Languages) }
        )
        content.addView(look)

        content.addView(heading(getString(R.string.omni_keys_title)))
        val chosen = Preferences.signingKey(this)
        val key = keysHere().firstOrNull { it.path == chosen }
        val signing = card()
        signing.addView(
            keyValue(
                key?.alias ?: getString(R.string.omni_keys_none),
                key?.subject.orEmpty(),
                when {
                    key == null -> ""
                    key.alias == DEFAULT_KEY_ALIAS -> getString(R.string.omni_keys_shared)
                    else -> getString(R.string.omni_keys_yours)
                },
                if (key == null) palette.warning else palette.accent,
            )
        )
        signing.addView(subtle(getString(R.string.omni_keys_manage), palette.accent) {
            go(Screen.Keys)
        })
        content.addView(signing)

        content.addView(heading(getString(R.string.omni_defaults_title)))
        val defaults = card()
        defaults.addView(
            row(
                getString(R.string.omni_form_package),
                Preferences.newPackage(this, DEFAULT_PACKAGE),
                getString(R.string.omni_action_change),
                palette.accent,
            ) {
                askForName(
                    getString(R.string.omni_form_package),
                    Preferences.newPackage(this, DEFAULT_PACKAGE),
                ) { named ->
                    Preferences.setNewPackage(this, named.trim())
                    render(false)
                }
            }
        )
        content.addView(defaults)

        content.addView(label(getString(R.string.omni_form_min_sdk)))
        val chosenMin = Preferences.newMinSdk(this, OLDEST_PLATFORM)
        val chosenTarget = Preferences.newTargetSdk(this, NEWEST_PLATFORM)
        content.addView(
            chips(ANDROID_RELEASES.map { it.second }, { ANDROID_RELEASES[it].first == chosenMin }) {
                val picked = ANDROID_RELEASES[it].first
                Preferences.setNewPlatforms(this, picked, maxOf(picked, chosenTarget))
                render(false)
            }
        )
        content.addView(label(getString(R.string.omni_form_target_sdk)))
        content.addView(
            chips(
                ANDROID_RELEASES.map { it.second },
                { ANDROID_RELEASES[it].first == chosenTarget },
            ) {
                val picked = ANDROID_RELEASES[it].first
                Preferences.setNewPlatforms(this, minOf(picked, chosenMin), picked)
                render(false)
            }
        )
        content.addView(quiet(getString(R.string.omni_defaults_seal)))

        content.addView(
            row(
                getString(R.string.omni_tab_trash),
                getString(R.string.omni_trash_title),
                "",
                palette.muted,
            ) { go(Screen.Trash) }
        )

        content.addView(heading(getString(R.string.omni_talk_key)))
        val brain = card()
        val thinking = Brain.vendorOf(Preferences.assistantVendor(this))
        val keyed = Brain.VENDORS.count { it.keyed(this) }
        brain.addView(
            keyValue(
                thinking.shown + "  \u00b7  " +
                    Brain.modelOf(thinking, Preferences.assistantModel(this)).shown,
                getString(R.string.omni_talk_keyed, keyed, Brain.VENDORS.size),
                if (Vault.inHardware()) {
                    getString(R.string.omni_device_hardware)
                } else {
                    getString(R.string.omni_device_software)
                },
                if (keyed == 0) palette.muted else palette.ok,
            )
        )
        Brain.VENDORS.filter { it.keyed(this) }.forEach { one ->
            brain.addView(rule(), LinearLayout.LayoutParams(MATCH_PARENT, 1))
            brain.addView(
                keyValue(
                    one.shown,
                    one.where,
                    getString(R.string.omni_action_delete),
                    palette.error,
                ).apply {
                    isClickable = true
                    isFocusable = true
                    readAs(Button::class.java.name)
                    pressable()
                    setOnClickListener {
                        Preferences.forgetVendorKey(this@BuilderActivity, one.id)
                        Preferences.setFetchedModels(this@BuilderActivity, one.id, emptyList())
                        results.removeAllViews()
                        results.addView(
                            notice(getString(R.string.omni_talk_key_gone), palette.ok)
                        )
                        render(false)
                    }
                }
            )
        }
        brain.addView(subtle(getString(R.string.omni_talk_providers), palette.accent) {
            chooseTheVendor()
        })
        brain.addView(subtle(getString(R.string.omni_talk_model), palette.accent) {
            chooseTheModel()
        })
        content.addView(brain)
        content.addView(quiet(getString(R.string.omni_talk_key_note)))

        content.addView(heading(getString(R.string.omni_settings_logs)))
        val transcript = OmniLog.transcript().trimEnd()
        val logging = card()
        logging.addView(
            EditText(this).apply {
                setText(
                    transcript.ifEmpty { getString(R.string.omni_settings_logs_empty) }
                )
                setTextColor(palette.foreground)
                setBackgroundColor(Color.TRANSPARENT)
                typeface = Type.data
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                gravity = Gravity.TOP or Gravity.START
                setHorizontallyScrolling(false)
                isFocusable = true
                setTextIsSelectable(true)
                keyListener = null
                minLines = 6
                maxLines = 18
                setPadding(gap(2))
            }
        )
        content.addView(logging)

        val logActions = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
        }
        logActions.addView(
            subtle(getString(R.string.omni_action_copy), palette.accent) {
                val clipboard = getSystemService(ClipboardManager::class.java)
                clipboard?.setPrimaryClip(
                    ClipData.newPlainText(getString(R.string.omni_settings_logs), transcript)
                )
                results.removeAllViews()
                results.addView(notice(getString(R.string.omni_settings_logs_copied), palette.ok))
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply { marginEnd = gap(1) },
        )
        logActions.addView(
            subtle(getString(R.string.omni_action_clear), palette.muted) {
                OmniLog.clearTranscript()
                render(false)
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f).apply { marginStart = gap(1) },
        )
        content.addView(logActions)
        content.addView(quiet(getString(R.string.omni_settings_logs_note)))

        content.addView(
            row(
                getString(R.string.omni_settings_about),
                getString(R.string.omni_about_note),
                "",
                palette.muted,
            ) { go(Screen.About) }
        )
    }

    private fun speakTheDeviceLanguage() {
        if (Builder.load() !is Builder.LoadState.Loaded) return
        val tag = Preferences.language(this).ifEmpty { deviceLanguage() }
        runCatching { Builder.nativeSpeak(tag) }
            .onFailure { OmniLog.event(LogLevel.WARN, "native", "The language was not carried over.") }
    }

    private fun deviceLanguage(): String {
        val spoken = Resources.getSystem().configuration.locales
        for (index in 0 until spoken.size()) {
            val here = spoken.get(index)
            val full = here.toLanguageTag()
            val exact = Preferences.LANGUAGES.firstOrNull { it.first.equals(full, true) }
            if (exact != null) {
                return exact.first
            }
            val plain = Preferences.LANGUAGES.firstOrNull {
                Locale.forLanguageTag(it.first).language == here.language
            }
            if (plain != null) {
                return plain.first
            }
        }
        return BASE_LOCALE
    }

    private fun spec() = ProjectSpec(
        packageName = formPackage.trim(),
        label = formLabel.trim(),
        versionName = formVersionName.trim(),
        versionCode = formVersionCode.trim().toIntOrNull() ?: 0,
        abis = ONLY_ABI,
        minSdk = formMinSdk,
        targetSdk = formTargetSdk,
        languages = formLanguages.toList(),
        locales = (formLocales + BASE_LOCALE).toList(),
    )

    private fun freeFolder(named: String): File {
        val parent = projectsFolder()
        val safe = named.trim()
            .map { if (it.isLetterOrDigit() || it == ' ' || it == '_' || it == '-') it else '_' }
            .joinToString("")
            .trim()
            .ifEmpty { "Project" }
        var candidate = File(parent, safe)
        var next = 2
        while (candidate.exists()) {
            candidate = File(parent, "$safe $next")
            next += 1
        }
        return candidate
    }

    private fun createProject() {
        results.removeAllViews()
        val root = freeFolder(formLabel)
        val image = formImage
        working({ Builder.nativeCreateProject(root.absolutePath, spec().encode()) }) finished@{ answer ->
            val outcome = runCatching { CreateOutcome.parse(answer) }.getOrElse {
                results.addView(notice(it.message ?: it.javaClass.simpleName, palette.error))
                return@finished
            }
            if (!outcome.created) {
                showRefusal(Refusal.parse(JSONObject(answer)), results)
                return@finished
            }
            OmniLog.event(LogLevel.INFO, "project", "Created ${outcome.root}")
            results.addView(notice(getString(R.string.omni_created), palette.ok))
            if (image != null) {
                val stored = runCatching {
                    JSONObject(Builder.nativeSetIcon(root.absolutePath, image))
                }.getOrNull()
                if (stored == null || !stored.optBoolean("stored", false)) {
                    stored?.let { showRefusal(Refusal.parse(it), results) }
                }
            }
            formImage = null
            openHere(root.absolutePath)
        }
    }

    private fun createKey() {
        results.removeAllViews()
        val first = readSecret(keyPasswordView)
        val again = readSecret(keyPasswordAgainView)
        if (!first.contentEquals(again)) {
            first.fill(' ')
            again.fill(' ')
            results.addView(notice(getString(R.string.omni_key_password_mismatch), palette.error))
            return
        }
        again.fill(' ')

        val request = KeySpec(
            alias = keyAlias.trim(),
            commonName = keyCommonName.trim(),
            organisation = keyOrganisation.trim(),
            country = keyCountry.trim().uppercase(Locale.US),
            validityYears = VALIDITY_YEARS[keyYears],
            bits = KEY_SIZES[keyBits],
        )
        val folder = keysFolder().absolutePath

        val forge = KeyForgeView(this, palette).apply {
            caption = getString(R.string.omni_forge_title)
            detail = getString(R.string.omni_forge_bits, request.bits)
        }
        showCeremony(forge)
        forge.begin()

        Thread {
            val answer = runCatching {
                Builder.nativeCreateKey(folder, request.encode(), first)
            }
            runOnUiThread {
                first.fill(' ')
                keyPasswordView?.text?.clear()
                keyPasswordAgainView?.text?.clear()
                if (isFinishing || isDestroyed) {
                    return@runOnUiThread
                }
                answer.fold({ document -> keyForged(forge, document) }) { error ->
                    OmniLog.recordCrash(Thread.currentThread(), error)
                    keyRefused(forge, error.message ?: error.javaClass.simpleName, null)
                }
            }
        }.start()
    }

    private fun keyForged(forge: KeyForgeView, answer: String) {
        val root = runCatching { JSONObject(answer) }.getOrNull()
        if (root == null || !root.optBoolean("created", false)) {
            keyRefused(forge, getString(R.string.omni_forge_refused), root)
            return
        }
        val made = SigningKey.parse(root.getJSONObject("key"))
        OmniLog.event(LogLevel.INFO, "keystore", "Key ${made.alias} created, ${made.fingerprint}")
        Preferences.setSigningKey(this, made.path)
        forge.caption = getString(R.string.omni_forge_sealed)
        forge.seal(made.fingerprint)

        waitFor(forge::sealedThrough) {
            hideCeremony { go(Screen.Keys) }
        }
    }

    private fun keyRefused(forge: KeyForgeView, said: String, root: JSONObject?) {
        forge.caption = said
        forge.detail = ""
        forge.refuse()
        ceremony.postDelayed({
            hideCeremony {
                results.removeAllViews()
                if (root != null) {
                    showRefusal(Refusal.parse(root), results)
                } else {
                    results.addView(notice(said, palette.error))
                }
            }
        }, Motion.of(Motion.REFUSAL))
    }

    private fun waitFor(ready: () -> Boolean, then: () -> Unit) {
        val look = object : Runnable {
            override fun run() {
                if (ready()) then() else ceremony.postDelayed(this, LOOK_MILLIS)
            }
        }
        ceremony.postDelayed(look, LOOK_MILLIS)
    }

    private class Mend(val held: JSONObject) {
        val title: String get() = held.optString("title")
        val path: String get() = held.optString("path")
        val line: Int get() = held.optInt("line")
        val confidence: String get() = held.optString("confidence")
        val after: String get() = held.optString("after")
    }

    private class Want(held: JSONObject) {
        val coordinate: String = held.optString("coordinate")
        val kind: String = held.optString("kind")
        val path: String = held.optString("path")
        val urls: List<String> = buildList {
            held.optJSONArray("urls")?.let { array ->
                for (index in 0 until array.length()) {
                    add(array.getString(index))
                }
            }
        }
    }

    private fun wantedLibraries(answer: String): List<Want> {
        val document = runCatching { JSONObject(answer) }.getOrNull() ?: return emptyList()
        if (!document.optBoolean("planned", false)) {
            return emptyList()
        }
        val held = document.optJSONArray("wanted") ?: return emptyList()
        return buildList {
            for (index in 0 until held.length()) {
                held.optJSONObject(index)?.let { add(Want(it)) }
            }
        }
    }

    private fun downloadOne(root: String, one: Want): String {
        val into = File(root, one.path)
        into.parentFile?.mkdirs()
        var last = ""
        for (where in one.urls) {
            val link = runCatching { URL(where).openConnection() }.getOrNull()
                as? HttpURLConnection ?: continue
            link.connectTimeout = LIBRARY_OPEN_MILLIS
            link.readTimeout = LIBRARY_READ_MILLIS
            link.instanceFollowRedirects = true
            link.setRequestProperty("User-Agent", "omni-builder")
            val code = runCatching { link.responseCode }.getOrElse { -1 }
            if (code != 200) {
                last = one.coordinate + "  " + code
                link.disconnect()
                continue
            }
            val held = runCatching {
                val temporary = File(into.absolutePath + ".part")
                link.inputStream.use { from ->
                    temporary.outputStream().use { out ->
                        val buffer = ByteArray(64 * 1024)
                        var carried = 0L
                        while (true) {
                            val read = from.read(buffer)
                            if (read <= 0) {
                                break
                            }
                            carried += read
                            if (carried > LIBRARY_MOST_BYTES) {
                                throw java.io.IOException(one.coordinate)
                            }
                            out.write(buffer, 0, read)
                        }
                    }
                }
                temporary.renameTo(into)
            }
            link.disconnect()
            if (held.isSuccess && into.isFile) {
                return ""
            }
            File(into.absolutePath + ".part").delete()
            last = one.coordinate
        }
        return last.ifEmpty { one.coordinate }
    }

    private fun fetchTheLibraries(root: String, onDone: () -> Unit) {
        results.removeAllViews()
        results.addView(notice(getString(R.string.omni_libraries_fetching), palette.accent))
        Thread {
            val refused = mutableListOf<String>()
            var carried = 0
            var rounds = 0
            while (rounds < LIBRARY_ROUNDS) {
                rounds += 1
                val wanted = wantedLibraries(
                    runCatching { Builder.nativeMavenPlan(root) }.getOrDefault("")
                ).filter { !refused.contains(it.coordinate + it.kind) }
                if (wanted.isEmpty()) {
                    break
                }
                for (one in wanted) {
                    val why = downloadOne(root, one)
                    if (why.isEmpty()) {
                        carried += 1
                    } else {
                        refused.add(one.coordinate + one.kind)
                        if (!refused.contains(why)) {
                            refused.add(why)
                        }
                    }
                }
            }
            val got = carried
            val left = refused.filter { !it.endsWith("pom") && !it.endsWith("jar") && !it.endsWith("aar") }
            runOnUiThread {
                if (isFinishing || isDestroyed) {
                    return@runOnUiThread
                }
                results.removeAllViews()
                results.addView(
                    notice(
                        getString(R.string.omni_libraries_got, got),
                        if (left.isEmpty()) palette.ok else palette.warning,
                    )
                )
                left.take(LIBRARY_SHOWN).forEach { results.addView(quiet(it)) }
                onDone()
            }
        }.start()
    }

    private fun mendThenBuild(root: String) {
        results.removeAllViews()
        working({ Builder.nativeMavenPlan(root) }) planned@{ answer ->
            val wanted = wantedLibraries(answer)
            if (wanted.isEmpty()) {
                mendThenBuildNow(root)
                return@planned
            }
            dialogShell(
                getString(R.string.omni_libraries_missing, wanted.size),
                LinearLayout(this).apply {
                    orientation = LinearLayout.VERTICAL
                    setPadding(0, gap(1), 0, gap(2))
                    addView(quiet(getString(R.string.omni_libraries_missing_note)))
                    wanted.take(LIBRARY_SHOWN).forEach { one ->
                        addView(
                            TextView(context).apply {
                                text = one.coordinate
                                setTextColor(palette.muted)
                                typeface = Type.data
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                                maxLines = 1
                                ellipsize = android.text.TextUtils.TruncateAt.MIDDLE
                            }
                        )
                    }
                },
                listOf(
                    getString(R.string.omni_libraries_without) to { it: AlertDialog ->
                        it.dismiss()
                        mendThenBuildNow(root)
                    },
                    getString(R.string.omni_libraries_fetch) to { it: AlertDialog ->
                        it.dismiss()
                        fetchTheLibraries(root) { mendThenBuildNow(root) }
                    },
                ),
            )
        }
    }

    private fun mendThenBuildNow(root: String) {
        results.removeAllViews()
        working({ Builder.nativeProjectRepairs(root) }) finished@{ answer ->
            val document = runCatching { JSONObject(answer) }.getOrNull()
            val listed = document?.optJSONArray("repairs")
            val found = mutableListOf<Mend>()
            for (index in 0 until (listed?.length() ?: 0)) {
                listed?.optJSONObject(index)?.let { found.add(Mend(it)) }
            }
            if (found.isEmpty()) {
                buildProject(root)
                return@finished
            }
            offerTheMends(root, found, document?.optString("left").orEmpty())
        }
    }

    private fun mendRow(root: String, one: Mend, onDone: () -> Unit): View =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(gap(2), gap(2), gap(2), gap(2))
            addView(
                View(context).apply {
                    foreground = Mark(
                        Mark.Shape.BROOM,
                        when (one.confidence) {
                            "CERTAIN" -> palette.ok
                            "RISKY" -> palette.error
                            else -> palette.warning
                        },
                    )
                },
                LinearLayout.LayoutParams(gap(6), gap(6)).apply { marginEnd = gap(2) },
            )
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = one.title
                            setTextColor(palette.foreground)
                            typeface = Type.strong
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                        }
                    )
                    addView(
                        TextView(context).apply {
                            text = one.path + ":" + one.line
                            setTextColor(palette.muted)
                            typeface = Type.data
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                            maxLines = 1
                            ellipsize = android.text.TextUtils.TruncateAt.START
                        }
                    )
                    val written = one.after.lines().firstOrNull { it.isNotBlank() }.orEmpty()
                    if (written.isNotEmpty()) {
                        addView(
                            TextView(context).apply {
                                text = written.trim()
                                setTextColor(palette.marker)
                                typeface = Type.data
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                                maxLines = 2
                                ellipsize = android.text.TextUtils.TruncateAt.END
                            }
                        )
                    }
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            addView(
                control().apply {
                    text = getString(R.string.omni_mend_one)
                    setTextColor(palette.background)
                    typeface = Type.strong
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                    background = reacting(palette.accent, gap(4).toFloat(), palette.accent)
                    setPadding(gap(3), gap(1), gap(3), gap(1))
                    setOnClickListener {
                        isEnabled = false
                        alpha = 0.5f
                        applyMends(root, listOf(one)) { onDone() }
                    }
                },
                LinearLayout.LayoutParams(WRAP_CONTENT, WRAP_CONTENT).apply {
                    marginStart = gap(2)
                },
            )
        }

    private fun applyMends(root: String, held: List<Mend>, onDone: () -> Unit) {
        val asked = org.json.JSONArray()
        held.forEach { asked.put(it.held) }
        working({ Builder.nativeApplyRepairs(root, asked.toString()) }) { answer ->
            val document = runCatching { JSONObject(answer) }.getOrNull()
            if (document?.optBoolean("mended", false) != true) {
                document?.let { showRefusal(Refusal.parse(it), results) }
                return@working
            }
            flash(
                getString(R.string.omni_mend_done, document.optInt("repaired")),
                palette.ok,
                Mark.Shape.BROOM,
            )
            onDone()
        }
    }

    private fun offerTheMends(root: String, found: List<Mend>, left: String) {
        var shown: AlertDialog? = null
        val list = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        val again = {
            shown?.dismiss()
            mendThenBuild(root)
        }
        found.forEachIndexed { at, one ->
            if (at > 0) {
                list.addView(rule(), LinearLayout.LayoutParams(MATCH_PARENT, 1))
            }
            list.addView(
                mendRow(root, one, again),
                LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT),
            )
        }
        val body = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            addView(quiet(getString(R.string.omni_mend_note, found.size)))
            if (left.isNotEmpty()) {
                addView(quiet(left))
            }
            addView(list, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
        }
        shown = dialogShell(
            getString(R.string.omni_mend_title),
            body,
            listOf(
                getString(R.string.omni_mend_skip) to { it: AlertDialog ->
                    it.dismiss()
                    buildProject(root)
                },
                getString(R.string.omni_mend_all) to { it: AlertDialog ->
                    it.dismiss()
                    applyMends(root, found) { mendThenBuild(root) }
                },
            ),
        )
    }

    private fun buildProject(root: String) {
        results.removeAllViews()
        val keyPath = Preferences.signingKey(this)
        if (keyPath.isEmpty()) {
            results.addView(notice(getString(R.string.omni_build_no_key), palette.warning))
            return
        }
        if (readSecret(buildPasswordView).isEmpty()) {
            results.addView(notice(getString(R.string.omni_build_needs_password), palette.warning))
            return
        }

        val name = File(root).name
        val here = freshOutput(root)
        val apk = File(here, "$name.apk")
        val aab = File(here, "$name.aab")
        val password = readSecret(buildPasswordView)
        buildSecret = Vault.seal(password)
        val started = System.nanoTime()

        val learned = Preferences.timings(this)
        Builder.nativeBuildExpect(learned.ifEmpty { null })

        val stage = BuildStageView(this, palette).apply {
            title = getString(R.string.omni_stage_title)
            stages = STAGE_NAMES.map { getString(it) }
            note = getString(
                if (learned.isEmpty()) R.string.omni_stage_measuring
                else R.string.omni_stage_learned
            )
        }
        building = true
        showCeremony(stage) { stopTheBuild() }
        stage.begin()
        watchTheBuild(stage)

        Thread {
            val answer = runCatching {
                Builder.nativeBuildAll(
                    root,
                    apk.absolutePath,
                    aab.absolutePath,
                    keyPath,
                    if (password.isEmpty()) null else password,
                )
            }
            runOnUiThread {
                password.fill(' ')
                building = false
                if (isFinishing || isDestroyed) {
                    return@runOnUiThread
                }
                answer.fold({ document ->
                    settle(stage) { buildEnded(document, apk, aab, started) }
                }) { error ->
                    OmniLog.recordCrash(Thread.currentThread(), error)
                    settle(stage) {
                        results.addView(
                            notice(error.message ?: error.javaClass.simpleName, palette.error)
                        )
                    }
                }
            }
        }.start()
    }

    private fun buildWithTheDeviceKey(root: String) {
        results.removeAllViews()
        if (!DeviceKey.present() && !DeviceKey.make()) {
            results.addView(notice(getString(R.string.omni_device_refused), palette.error))
            return
        }
        val certificate = DeviceKey.certificate()
        if (certificate == null) {
            results.addView(notice(getString(R.string.omni_device_refused), palette.error))
            return
        }
        val held = DeviceKey.hex(certificate)

        val name = File(root).name
        val here = freshOutput(root)
        val apk = File(here, "$name.apk")
        val raw = File(here, "$name-unsigned.apk")
        val started = System.nanoTime()

        val learned = Preferences.timings(this)
        Builder.nativeBuildExpect(learned.ifEmpty { null })
        val stage = BuildStageView(this, palette).apply {
            title = getString(R.string.omni_stage_title)
            stages = STAGE_NAMES.map { getString(it) }
            note = getString(R.string.omni_device_note)
        }
        building = true
        showCeremony(stage) { stopTheBuild() }
        stage.begin()
        watchTheBuild(stage)

        Thread {
            val answer = runCatching { sealWithTheDevice(root, held, raw, apk) }
            raw.delete()
            runOnUiThread {
                building = false
                if (isFinishing || isDestroyed) {
                    return@runOnUiThread
                }
                answer.fold({ document ->
                    settle(stage) { buildEnded(document, apk, apk, started) }
                }) { error ->
                    OmniLog.recordCrash(Thread.currentThread(), error)
                    settle(stage) {
                        results.addView(
                            notice(error.message ?: error.javaClass.simpleName, palette.error)
                        )
                    }
                }
            }
        }.start()
    }

    private fun sealWithTheDevice(
        root: String,
        certificate: String,
        raw: File,
        apk: File,
    ): String {
        val prepared = JSONObject(
            Builder.nativeBuildUnsigned(root, raw.absolutePath, certificate)
        )
        if (!prepared.optBoolean("prepared", false)) {
            return prepared.toString()
        }
        val first = DeviceKey.bytes(prepared.optString("toSignV2"))
            ?: throw IOException("the Core asked for a signature over nothing")
        val signedFirst = DeviceKey.sign(first)
            ?: throw IOException("the device would not sign with its own key")

        val signedWith = { named: String ->
            val text = prepared.optString(named)
            if (text.isEmpty()) {
                ""
            } else {
                val held = DeviceKey.bytes(text)
                    ?: throw IOException("the Core asked for a signature over nothing")
                DeviceKey.hex(
                    DeviceKey.sign(held)
                        ?: throw IOException("the device would not sign with its own key")
                )
            }
        }

        return Builder.nativeSealPackage(
            raw.absolutePath,
            apk.absolutePath,
            certificate,
            DeviceKey.hex(signedFirst),
            signedWith("toSignV3"),
            signedWith("toSignV31"),
            prepared.optInt("minSdk"),
            prepared.optInt("maxSdk"),
        )
    }

    private fun stopTheBuild() {
        if (!building) {
            return
        }
        OmniLog.event(LogLevel.INFO, "build", "Stop asked for.")
        Builder.nativeBuildStop()
    }

    private fun watchTheBuild(view: BuildStageView) {
        var lastSaid = ""
        val look = object : Runnable {
            override fun run() {
                if (!ceremonyIsUp()) return
                val said = runCatching { Builder.nativeBuildProgress() }.getOrDefault("")
                if (said != lastSaid) {
                    lastSaid = said
                    val report = runCatching { JSONObject(said) }.getOrNull()
                    if (report != null) {
                        val state = report.optString("state")
                        view.observe(
                            percent = report.optInt("percent"),
                            stage = report.optInt("step", 1) - 1,
                            finished = state == "built",
                            refused = state == "refused" || state == "stopped",
                        )
                        view.remaining = when (state) {
                            "built" -> getString(R.string.omni_stage_built)
                            "stopped" -> getString(R.string.omni_stage_stopped)
                            "refused" -> getString(R.string.omni_refused)
                            else -> left(report.optLong("leftMillis") / 1000L)
                        }
                    }
                }
                view.postDelayed(this, WATCH_MILLIS)
            }
        }
        view.post(look)
    }

    private fun settle(view: BuildStageView, then: () -> Unit) {
        view.postDelayed({
            view.rest()
            hideCeremony {
                results.removeAllViews()
                then()
            }
        }, Motion.of(Motion.SETTLE))
    }

    private fun buildEnded(answer: String, apk: File, aab: File, started: Long) {
        run finished@{
            val elapsed = (System.nanoTime() - started) / 1_000_000
            val outcome = runCatching { BuildOutcome.parse(answer) }.getOrElse {
                results.addView(notice(it.message ?: it.javaClass.simpleName, palette.error))
                return@finished
            }
            if (!outcome.built) {
                OmniLog.event(LogLevel.ERROR, "build", "Refused: ${outcome.error}")
                results.addView(notice(getString(R.string.omni_refused), palette.error))
                if (!showing) {
                    Bell.say(
                        this,
                        Bell.BUILD_NOTE,
                        getString(R.string.omni_refused),
                        outcome.error.orEmpty(),
                    )
                }
                showRefusal(Refusal.parse(JSONObject(answer)), results)
                outcome.findings.forEach { results.addView(quiet(it)) }
                runCatching {
                    val here = apk.parentFile
                    if (here != null && here.isDirectory && here.list()?.isEmpty() == true) {
                        here.delete()
                    }
                }
                return@finished
            }

            outcome.timings?.let { Preferences.setTimings(this, it) }

            OmniLog.event(
                LogLevel.INFO,
                "build",
                "Built ${outcome.bytes} bytes and bundled ${outcome.bundleBytes} in $elapsed ms",
            )
            results.addView(
                notice(getString(R.string.omni_build_done, elapsed), palette.ok)
            )
            if (!showing) {
                Bell.say(
                    this,
                    Bell.BUILD_NOTE,
                    getString(R.string.omni_build_title),
                    apk.name + "  ·  " + size(outcome.bytes),
                )
            }
            val facts = card()
            facts.addView(
                keyValue(
                    getString(R.string.omni_result_package),
                    apk.name,
                    size(outcome.bytes),
                    palette.ok,
                )
            )
            facts.addView(
                keyValue(
                    getString(R.string.omni_result_bundle),
                    aab.name,
                    size(outcome.bundleBytes),
                    palette.ok,
                )
            )
            facts.addView(
                keyValue(
                    getString(R.string.omni_result_contents),
                    if (outcome.carriesCode) "AndroidManifest.xml + classes.dex" else "AndroidManifest.xml",
                    outcome.entries.toString(),
                    palette.muted,
                )
            )
            facts.addView(
                keyValue(
                    getString(R.string.omni_result_languages),
                    "",
                    outcome.locales.toString(),
                    palette.muted,
                )
            )

            val schemes = outcome.signatureSchemes.joinToString(" + ")
            val soundly = outcome.signed &&
                outcome.signatureKeyMatches &&
                outcome.signaturesVerified > 0
            facts.addView(
                keyValue(
                    getString(R.string.omni_result_signature),
                    outcome.signedBy.orEmpty(),
                    schemes.ifEmpty { "?" },
                    if (soundly) palette.ok else palette.error,
                )
            )
            facts.addView(
                keyValue(
                    getString(R.string.omni_result_verified),
                    getString(
                        if (soundly) R.string.omni_result_verified_yes
                        else R.string.omni_result_verified_no
                    ),
                    outcome.signaturesVerified.toString(),
                    if (soundly) palette.ok else palette.error,
                )
            )
            results.addView(facts)

            if (outcome.findings.isNotEmpty()) {
                results.addView(
                    heading(getString(R.string.omni_build_said, outcome.findings.size))
                )
                val said = card()
                outcome.findings.forEachIndexed { index, one ->
                    if (index > 0) {
                        said.addView(rule(), MATCH_PARENT, 1)
                    }
                    said.addView(body(one))
                }
                results.addView(said)
            }

            outcome.path?.let { actionsFor(File(it), results) }
        }
    }

    private fun handleFor(file: File): Pair<Uri, String> {
        val uri = PackageProvider.uriFor(this, file)
        val type = contentResolver.getType(uri) ?: PackageProvider.BUNDLE_TYPE
        return uri to type
    }

    private fun shareOutside(file: File) {
        if (!file.isFile) {
            return
        }
        val (uri, type) = handleFor(file)
        val sending = Intent(Intent.ACTION_SEND)
            .setType(type)
            .putExtra(Intent.EXTRA_STREAM, uri)
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        hand(
            Intent.createChooser(sending, file.name)
                .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        )
    }

    private fun openOutside(file: File) {
        if (!file.isFile) {
            return
        }
        val (uri, type) = handleFor(file)
        hand(
            Intent(Intent.ACTION_VIEW)
                .setDataAndType(uri, type)
                .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        )
    }

    private fun actionsFor(file: File, into: LinearLayout) {
        if (file.name.lowercase(Locale.US).endsWith(".apk")) {
            into.addView(subtle(getString(R.string.omni_install_title), palette.ok) {
                installPackage(file)
            })
        }
        into.addView(subtle(getString(R.string.omni_action_open), palette.ok) {
            openOutside(file)
        })
        into.addView(subtle(getString(R.string.omni_action_share), palette.accent) {
            shareOutside(file)
        })
    }

    private fun hand(intent: Intent) {
        runCatching { startActivity(intent) }.onFailure { why ->
            OmniLog.event(LogLevel.WARN, "handoff", why.message ?: why.javaClass.simpleName)
            results.addView(notice(why.message ?: why.javaClass.simpleName, palette.error))
        }
    }

    private fun chooseImage(forProject: String?) {
        imageForProject = forProject
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "image/*"
            putExtra(
                Intent.EXTRA_MIME_TYPES,
                arrayOf("image/png", "image/jpeg", "image/webp", "image/heif", "image/*"),
            )
        }
        runCatching { startActivityForResult(intent, IMAGE_REQUEST) }
            .onFailure {
                results.removeAllViews()
                results.addView(
                    notice(
                        it.message ?: getString(R.string.omni_form_image_none),
                        palette.error,
                    )
                )
            }
    }

    private fun askForAProjectArchive() {
        val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
            addCategory(Intent.CATEGORY_OPENABLE)
            type = "*/*"
            putExtra(
                Intent.EXTRA_MIME_TYPES,
                arrayOf("application/zip", "application/x-zip-compressed", "application/octet-stream"),
            )
        }
        runCatching { startActivityForResult(intent, ARCHIVE_REQUEST) }
            .onFailure {
                results.removeAllViews()
                results.addView(notice(getString(R.string.omni_import_none), palette.warning))
            }
    }

    private fun namedBy(uri: Uri): String {
        val asked = runCatching {
            contentResolver.query(uri, null, null, null, null)?.use { row ->
                val at = row.getColumnIndex(android.provider.OpenableColumns.DISPLAY_NAME)
                if (at >= 0 && row.moveToFirst()) row.getString(at) else null
            }
        }.getOrNull()
        val named = asked ?: uri.lastPathSegment?.substringAfterLast('/')
        return (named ?: "Project").substringBeforeLast('.')
    }

    private fun stageArchive(uri: Uri): File? {
        val held = File(cacheDir, "Imported.zip")
        return runCatching {
            contentResolver.openInputStream(uri).use { from ->
                if (from == null) {
                    return null
                }
                held.outputStream().use { into -> from.copyTo(into) }
            }
            held
        }.getOrNull()
    }

    private fun openTheArchive(uri: Uri) {
        results.removeAllViews()
        val staged = stageArchive(uri)
        if (staged == null || !staged.isFile) {
            results.addView(notice(getString(R.string.omni_import_none), palette.error))
            return
        }
        openTheStagedArchive(staged, namedBy(uri), "")
    }

    private fun openTheStagedArchive(staged: File, named: String, password: String) {
        val root = freeFolder(named)
        working({ Builder.nativeOpenArchive(staged.absolutePath, root.absolutePath, password) })
        finished@{ answer ->
            val document = runCatching { JSONObject(answer) }.getOrNull()
            if (document == null) {
                staged.delete()
                root.deleteRecursively()
                results.addView(notice(getString(R.string.omni_refused), palette.error))
                return@finished
            }
            if (!document.optBoolean("unpacked", false)) {
                root.deleteRecursively()
                val refusal = Refusal.parse(document)
                if (refusal.code == "EZ023" || refusal.code == "EZ026") {
                    askForTheArchivePassword(staged, named, refusal.code == "EZ026")
                    return@finished
                }
                staged.delete()
                showRefusal(refusal, results)
                return@finished
            }
            staged.delete()
            flash(
                getString(R.string.omni_import_done, document.optInt("files")),
                palette.ok,
                Mark.Shape.ARCHIVE,
            )
            val left = mutableListOf<String>()
            document.optJSONArray("unread")?.let { held ->
                for (index in 0 until held.length()) {
                    left.add(held.getString(index))
                }
            }
            if (left.isEmpty()) {
                openHere(root.absolutePath)
                return@finished
            }
            dialogShell(
                getString(R.string.omni_import_gradle_left),
                LinearLayout(this).apply {
                    orientation = LinearLayout.VERTICAL
                    setPadding(0, gap(1), 0, gap(2))
                    addView(quiet(getString(R.string.omni_import_gradle_left_note)))
                    left.forEach { one ->
                        addView(
                            TextView(context).apply {
                                text = one
                                setTextColor(palette.warning)
                                typeface = Type.data
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                                setPadding(0, gap(1), 0, 0)
                            }
                        )
                    }
                },
                listOf(
                    getString(R.string.omni_action_confirm) to { it: AlertDialog ->
                        it.dismiss()
                        openHere(root.absolutePath)
                    },
                ),
            )
        }
    }

    private fun askForTheArchivePassword(staged: File, named: String, again: Boolean) {
        val editor = styledField("", getString(R.string.omni_pack_password), true).apply {
            typeface = Type.data
        }
        dialogShell(
            getString(R.string.omni_import_locked),
            LinearLayout(this).apply {
                orientation = LinearLayout.VERTICAL
                setPadding(0, gap(1), 0, gap(2))
                addView(editor, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
                addView(
                    quiet(
                        getString(
                            if (again) {
                                R.string.omni_import_locked_again
                            } else {
                                R.string.omni_import_locked_note
                            }
                        )
                    )
                )
            },
            listOf(
                getString(R.string.omni_action_cancel) to { it: AlertDialog ->
                    staged.delete()
                    it.dismiss()
                },
                getString(R.string.omni_action_open) to { it: AlertDialog ->
                    val held = readSecret(editor)
                    val said = String(held)
                    held.fill(' ')
                    editor.text?.clear()
                    it.dismiss()
                    if (said.isEmpty()) {
                        staged.delete()
                    } else {
                        openTheStagedArchive(staged, named, said)
                    }
                },
            ),
        )
        editor.requestFocus()
    }

    private fun stageImage(uri: Uri): File? {
        val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        runCatching {
            contentResolver.openInputStream(uri).use { BitmapFactory.decodeStream(it, null, bounds) }
        }.getOrNull()
        if (bounds.outWidth <= 0 || bounds.outHeight <= 0) {
            return null
        }

        val options = BitmapFactory.Options()
        var step = 1
        while (
            bounds.outWidth / (step * 2) >= LARGEST_ICON_EDGE &&
            bounds.outHeight / (step * 2) >= LARGEST_ICON_EDGE
        ) {
            step *= 2
        }
        options.inSampleSize = step
        options.inPreferredConfig = Bitmap.Config.ARGB_8888

        val decoded = runCatching {
            contentResolver.openInputStream(uri).use { BitmapFactory.decodeStream(it, null, options) }
        }.getOrNull() ?: return null

        val edge = minOf(decoded.width, decoded.height)
        val square = runCatching {
            Bitmap.createBitmap(
                decoded,
                (decoded.width - edge) / 2,
                (decoded.height - edge) / 2,
                edge,
                edge,
            )
        }.getOrDefault(decoded)
        val wanted = minOf(edge, LARGEST_ICON_EDGE)
        val scaled = if (square.width == wanted) {
            square
        } else {
            Bitmap.createScaledBitmap(square, wanted, wanted, true)
        }

        val staged = File(cacheDir, "chosen_image.png")
        val written = runCatching {
            FileOutputStream(staged).use { sink ->
                scaled.compress(Bitmap.CompressFormat.PNG, 100, sink)
            }
        }.getOrDefault(false)

        if (scaled !== square) scaled.recycle()
        if (square !== decoded) square.recycle()
        decoded.recycle()

        return if (written && staged.isFile) staged else null
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (resultCode != RESULT_OK) {
            return
        }
        if (requestCode == SHOT_REQUEST) {
            val picked = mutableListOf<Uri>()
            val many = data?.clipData
            if (many != null) {
                for (index in 0 until many.itemCount) {
                    many.getItemAt(index)?.uri?.let { picked.add(it) }
                }
            } else {
                data?.data?.let { picked.add(it) }
            }
            if (picked.isNotEmpty()) {
                takeTheShots(picked)
            }
            return
        }
        if (requestCode == ARCHIVE_REQUEST) {
            data?.data?.let { openTheArchive(it) }
            return
        }
        if (requestCode != IMAGE_REQUEST) {
            return
        }
        val uri = data?.data ?: return
        val root = imageForProject
        imageForProject = null

        val staged = stageImage(uri)
        if (staged == null) {
            results.removeAllViews()
            results.addView(notice(getString(R.string.omni_form_image_unreadable), palette.error))
            return
        }
        if (root == null) {
            formImage = staged.absolutePath
            render(false)
            return
        }
        applyImage(root, staged.absolutePath)
    }

    private fun applyImage(root: String, source: String) {
        results.removeAllViews()
        working({ Builder.nativeSetIcon(root, source) }) finished@{ answer ->
            val stored = runCatching { JSONObject(answer) }.getOrNull()
            if (stored == null || !stored.optBoolean("stored", false)) {
                stored?.let { showRefusal(Refusal.parse(it), results) }
                return@finished
            }
            OmniLog.event(LogLevel.INFO, "project", "Image set for $root")
            render(false)
            results.addView(notice(getString(R.string.omni_form_image_set), palette.ok))
        }
    }

    private fun working(work: () -> String, finished: (String) -> Unit) {
        results.removeAllViews()
        results.addView(notice(getString(R.string.omni_working), palette.accent))
        Thread {
            val answer = runCatching(work)
            runOnUiThread {
                if (isFinishing || isDestroyed) {
                    return@runOnUiThread
                }
                results.removeAllViews()
                answer.fold(finished) { error ->
                    OmniLog.recordCrash(Thread.currentThread(), error)
                    results.addView(
                        notice(error.message ?: error.javaClass.simpleName, palette.error)
                    )
                }
            }
        }.start()
    }

    private fun act(answer: String, field: String) {
        results.removeAllViews()
        val root = runCatching { JSONObject(answer) }.getOrNull()
        if (root == null) {
            results.addView(notice(getString(R.string.omni_refused), palette.error))
            return
        }
        if (!root.optBoolean(field, false)) {
            showRefusal(Refusal.parse(root), results)
            return
        }
        render(false)
    }

    private fun showRefusal(refusal: Refusal, into: LinearLayout) {
        val heading = listOfNotNull(refusal.code, refusal.message).joinToString("  ")
        into.addView(notice(heading.ifEmpty { getString(R.string.omni_refused) }, palette.error))
        val detail = card()
        refusal.context.forEach { detail.addView(quiet(it)) }
        refusal.suggestion?.let { detail.addView(body(it)) }
        if (detail.childCount > 0) {
            into.addView(detail)
        }
    }

    private fun readSecret(view: EditText?): CharArray {
        val editable = view?.text ?: return CharArray(0)
        val chars = CharArray(editable.length)
        android.text.TextUtils.getChars(editable, 0, editable.length, chars, 0)
        return chars
    }

    private fun View.readAs(className: String) {
        accessibilityDelegate = object : View.AccessibilityDelegate() {
            override fun onInitializeAccessibilityNodeInfo(
                host: View,
                info: AccessibilityNodeInfo,
            ) {
                super.onInitializeAccessibilityNodeInfo(host, info)
                info.className = className
            }
        }
    }

    private fun said(vararg parts: String): String =
        parts.filter { it.isNotBlank() }.joinToString(", ")

    private fun gap(units: Int): Int =
        (units * resources.displayMetrics.density * 4f).toInt().coerceAtLeast(1)

    private fun pill(colour: Int, radius: Float) = GradientDrawable().apply {
        shape = GradientDrawable.RECTANGLE
        cornerRadius = radius
        setColor(colour)
    }

    private fun sheet(fill: Int, stroke: Int) = GradientDrawable().apply {
        shape = GradientDrawable.RECTANGLE
        cornerRadius = gap(4).toFloat()
        setColor(fill)
        setStroke(1, stroke)
    }

    private fun touchable(background: GradientDrawable, ripple: Int): Drawable =
        RippleDrawable(ColorStateList.valueOf(ripple), background, null)

    private fun View.pressable(
        radius: Float = gap(2).toFloat(),
        depth: Float = 0.97f,
        onHeld: (() -> Unit)? = null,
    ) {
        val filling = if (onHeld == null || Motion.still()) {
            null
        } else {
            Filling(Ink.hot(palette.accent, 0.28f), radius)
        }
        background = if (filling == null) {
            touchable(pill(Color.TRANSPARENT, radius), palette.accent)
        } else {
            RippleDrawable(
                ColorStateList.valueOf(palette.accent),
                LayerDrawable(arrayOf(pill(Color.TRANSPARENT, radius), filling)),
                null,
            )
        }
        if (Motion.still()) {
            if (onHeld != null) {
                setOnLongClickListener {
                    onHeld()
                    true
                }
            }
            return
        }
        var held = false
        var backwards = false
        val fills = filling?.let { strip ->
            ValueAnimator.ofFloat(0f, 1f).apply {
                duration = Motion.of(Motion.HOLD)
                interpolator = DecelerateInterpolator()
                addUpdateListener { step -> strip.much = step.animatedValue as Float }
                addListener(object : AnimatorListenerAdapter() {
                    override fun onAnimationEnd(animation: Animator) {
                        if (strip.much < 1f) {
                            return
                        }
                        held = true
                        onHeld?.invoke()
                    }
                })
            }
        }
        setOnTouchListener { view, event ->
            when (event.actionMasked) {
                MotionEvent.ACTION_DOWN -> {
                    held = false
                    backwards = false
                    view.animate()
                        .scaleX(depth).scaleY(depth)
                        .setDuration(Motion.of(Motion.TAP))
                        .setInterpolator(DecelerateInterpolator())
                        .start()
                    fills?.start()
                    false
                }

                MotionEvent.ACTION_MOVE -> {
                    val inside = event.x >= 0f && event.y >= 0f &&
                        event.x <= view.width && event.y <= view.height
                    if (!inside && !backwards && fills != null && fills.isRunning) {
                        backwards = true
                        fills.reverse()
                    }
                    false
                }

                MotionEvent.ACTION_UP,
                MotionEvent.ACTION_CANCEL,
                -> {
                    view.animate()
                        .scaleX(1f).scaleY(1f)
                        .setDuration(Motion.of(Motion.REBOUND))
                        .setInterpolator(OvershootInterpolator(1.4f))
                        .start()
                    if (!backwards) {
                        backwards = true
                        fills?.reverse()
                    }
                    if (held) {
                        view.isPressed = false
                        held = false
                        true
                    } else {
                        false
                    }
                }

                else -> false
            }
        }
    }

    private fun reacting(fill: Int, radius: Float, accent: Int): Drawable {
        val ring = pill(fill, radius).apply {
            setStroke(gap(1) / 2, accent)
        }
        val states = StateListDrawable().apply {
            addState(intArrayOf(android.R.attr.state_focused), ring)
            addState(intArrayOf(), pill(fill, radius))
        }
        return RippleDrawable(ColorStateList.valueOf(accent), states, null)
    }

    private fun stated(colour: Int): ColorStateList = ColorStateList(
        arrayOf(intArrayOf(-android.R.attr.state_enabled), intArrayOf()),
        intArrayOf(Ink.fade(colour, 0.38f), colour),
    )

    private fun card(): LinearLayout = LinearLayout(this).apply {
        orientation = LinearLayout.VERTICAL
        background = sheet(palette.surface, palette.divider)
        setPadding(gap(3), gap(3), gap(3), gap(1))
        layoutTransition = LayoutTransition()
        layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
            topMargin = gap(1)
            bottomMargin = 0
        }
    }

    private fun heading(value: String): View {
        val written = TextView(this).apply {
            text = value.uppercase(Locale.getDefault())
            setTextColor(palette.foreground)
            typeface = Type.heading
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 21f)
            letterSpacing = 0.06f
            setPadding(0, gap(4), 0, gap(1))
        }
        if (palette.key != Palette.ANIME.key) {
            return written
        }
        val row = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.BOTTOM
        }
        row.addView(written, LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f))
        row.addView(
            ImageView(this).apply {
                setImageDrawable(Chibi(companion(value), palette))
                alpha = 0.92f
            },
            LinearLayout.LayoutParams(gap(9), gap(9)).apply {
                marginStart = gap(2)
                bottomMargin = gap(1)
            },
        )
        return row
    }

    private fun companion(of: String): Chibi.Face {
        val faces = Chibi.Face.entries
        var seed = 7
        for (letter in of) {
            seed = seed * 31 + letter.code
        }
        return faces[((seed % faces.size) + faces.size) % faces.size]
    }

    private fun label(value: String) = TextView(this).apply {
        text = value
        setTextColor(palette.accent)
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
        setPadding(0, gap(2), 0, gap(2))
    }

    private fun body(value: String) = TextView(this).apply {
        text = value
        setTextColor(palette.foreground)
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
        setLineSpacing(gap(1).toFloat(), 1f)
        setPadding(0, gap(1), 0, gap(1))
    }

    private fun quiet(value: String) = TextView(this).apply {
        text = value
        setTextColor(palette.muted)
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
        setLineSpacing(gap(1).toFloat(), 1f)
        setPadding(0, gap(1), 0, gap(1))
    }

    private fun warning(value: String) = TextView(this).apply {
        text = value
        setTextColor(palette.warning)
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
        setLineSpacing(gap(1).toFloat(), 1f)
        setPadding(gap(3), gap(2), gap(3), gap(2))
        background = sheet(palette.surface, palette.warning)
        layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
            topMargin = gap(1)
            bottomMargin = gap(1)
        }
    }

    private fun notice(value: String, colour: Int) = TextView(this).apply {
        text = value
        setTextColor(colour)
        typeface = Type.strong
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
        setPadding(gap(3), gap(3), gap(3), gap(3))
        background = sheet(palette.surface, colour)
        layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
            topMargin = gap(2)
            bottomMargin = gap(1)
        }
    }

    private fun rule() = View(this).apply {
        setBackgroundColor(palette.divider)
    }

    private fun control(): Button = Button(this).apply {
        isAllCaps = false
        stateListAnimator = null
        minWidth = 0
        minHeight = gap(TOUCH_UNITS)
        minimumWidth = 0
        minimumHeight = gap(TOUCH_UNITS)
        gravity = Gravity.CENTER
        isFocusable = true
        elevation = 0f
        typeface = Type.strong
    }

    private fun primary(text: String, onPress: () -> Unit) = control().apply {
        this.text = text
        setTextColor(stated(palette.background))
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
        letterSpacing = 0.06f
        setPadding(gap(4), gap(4), gap(4), gap(4))
        background = reacting(palette.accent, gap(3).toFloat(), palette.background)
        setOnClickListener { onPress() }
        layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
            topMargin = gap(2)
            bottomMargin = gap(1)
        }
    }

    private fun subtle(text: String, colour: Int, onPress: () -> Unit) = control().apply {
        this.text = text
        setTextColor(stated(colour))
        setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
        setPadding(gap(4), gap(3), gap(4), gap(3))
        background = reacting(Color.TRANSPARENT, gap(3).toFloat(), colour)
        setOnClickListener { onPress() }
        layoutParams = LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT).apply {
            topMargin = gap(1)
        }
    }

    private fun row(
        title: String,
        detail: String,
        trailing: String,
        trailingColour: Int = palette.muted,
        onPress: () -> Unit,
    ) = LinearLayout(this).apply {
        orientation = LinearLayout.HORIZONTAL
        gravity = Gravity.CENTER_VERTICAL
        setPadding(0, gap(2), 0, gap(2))
        minimumHeight = gap(TOUCH_UNITS)
        background = touchable(pill(Color.TRANSPARENT, gap(2).toFloat()), palette.accent)
        isClickable = true
        isFocusable = true
        contentDescription = said(title, detail, trailing)
        isScreenReaderFocusable = true
        readAs(Button::class.java.name)
        setOnClickListener { onPress() }

        addView(
            LinearLayout(context).apply {
                orientation = LinearLayout.VERTICAL
                addView(
                    TextView(context).apply {
                        text = title
                        setTextColor(palette.foreground)
                        typeface = Type.strong
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
                    }
                )
                if (detail.isNotEmpty()) {
                    addView(
                        TextView(context).apply {
                            text = detail
                            setTextColor(palette.muted)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                            setLineSpacing(gap(1).toFloat(), 1f)
                        }
                    )
                }
            },
            LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
        )

        if (trailing.isNotEmpty()) {
            addView(
                TextView(context).apply {
                    text = trailing
                    setTextColor(trailingColour)
                    typeface = Type.label
                    setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                    letterSpacing = Type.TRACKING
                    setPadding(gap(2), gap(1), gap(2), gap(1))
                    background = pill(palette.raised, gap(2).toFloat())
                }
            )
        }
    }

    private fun keyValue(title: String, detail: String, trailing: String, colour: Int) =
        LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(0, gap(2), 0, gap(2))
            addView(
                LinearLayout(context).apply {
                    orientation = LinearLayout.VERTICAL
                    addView(
                        TextView(context).apply {
                            text = title
                            setTextColor(palette.foreground)
                            setTextSize(TypedValue.COMPLEX_UNIT_SP, 14f)
                        }
                    )
                    if (detail.isNotEmpty()) {
                        addView(
                            TextView(context).apply {
                                text = detail
                                setTextColor(palette.muted)
                                setTextSize(TypedValue.COMPLEX_UNIT_SP, 11f)
                            }
                        )
                    }
                },
                LinearLayout.LayoutParams(0, WRAP_CONTENT, 1f),
            )
            if (trailing.isNotEmpty()) {
                addView(
                    TextView(context).apply {
                        text = trailing
                        setTextColor(colour)
                        setTypeface(Typeface.MONOSPACE, Typeface.BOLD)
                        setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                    }
                )
            }
        }

    private fun input(label: String, initial: String): Pair<View, EditText> {
        val holder = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(0, gap(2), 0, gap(1))
        }
        holder.addView(
            TextView(this).apply {
                text = label
                setTextColor(palette.muted)
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 13f)
                setPadding(0, 0, 0, gap(1))
            }
        )
        val editor = EditText(this).apply {
            setText(initial)
            setTextColor(palette.foreground)
            setTextSize(TypedValue.COMPLEX_UNIT_SP, 15f)
            background = GradientDrawable().apply {
                shape = GradientDrawable.RECTANGLE
                cornerRadius = gap(3).toFloat()
                setColor(palette.surface)
                setStroke(gap(1) / 2, palette.divider)
            }
            setPadding(gap(3), gap(3), gap(3), gap(3))
            isSingleLine = true
        }
        holder.addView(editor, LinearLayout.LayoutParams(MATCH_PARENT, WRAP_CONTENT))
        return holder to editor
    }

    private fun field(label: String, initial: String, onChange: (String) -> Unit): View {
        val (holder, editor) = input(label, initial)
        editor.addTextChangedListener(object : TextWatcher {
            override fun beforeTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
            override fun onTextChanged(s: CharSequence?, a: Int, b: Int, c: Int) = Unit
            override fun afterTextChanged(s: Editable?) {
                onChange(s?.toString().orEmpty())
            }
        })
        return holder
    }

    private fun secret(label: String, initial: String = ""): Pair<View, EditText> {
        val (holder, editor) = input(label, initial)
        editor.inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
        editor.typeface = Typeface.MONOSPACE
        return holder to editor
    }

    private fun chips(
        labels: List<String>,
        selected: (Int) -> Boolean,
        onPick: (Int) -> Unit,
    ): View {
        val holder = FlowLayout(this, gap(2), gap(1)).apply {
            setPadding(0, gap(1), 0, gap(1))
        }
        val views = mutableListOf<Button>()

        fun repaint() {
            views.forEachIndexed { index, view ->
                val on = selected(index)
                view.isSelected = on
                view.setTextColor(stated(if (on) palette.background else palette.foreground))
                view.background = reacting(
                    if (on) palette.accent else palette.raised,
                    gap(4).toFloat(),
                    palette.accent,
                )
            }
        }

        labels.forEachIndexed { index, text ->
            val chip = control().apply {
                this.text = text
                setTextSize(TypedValue.COMPLEX_UNIT_SP, 12f)
                minHeight = 0
                minimumHeight = 0
                setPadding(gap(4), gap(2), gap(4), gap(2))
                maxLines = 1
                readAs(android.widget.ToggleButton::class.java.name)
                setOnClickListener {
                    onPick(index)
                    repaint()
                    press()
                }
            }
            views.add(chip)
            holder.addView(chip)
        }
        repaint()
        return holder
    }

    private fun View.setPadding(all: Int) = setPadding(all, all, all, all)
}

internal object Ink {

    const val FRAME_NANOS_60 = 16_666_667L

    fun ease(value: Float, from: Float, to: Float): Float {
        if (to <= from) return if (value >= to) 1f else 0f
        val held = ((value - from) / (to - from)).coerceIn(0f, 1f)
        return held * held * (3f - 2f * held)
    }

    fun mix(from: Float, to: Float, by: Float): Float = from + (to - from) * by

    fun fade(colour: Int, alpha: Float): Int = Color.argb(
        (Color.alpha(colour) * alpha.coerceIn(0f, 1f)).toInt(),
        Color.red(colour),
        Color.green(colour),
        Color.blue(colour),
    )

    fun dim(colour: Int, by: Float): Int {
        val held = (1f - by.coerceIn(0f, 1f))
        return Color.rgb(
            (Color.red(colour) * held).toInt(),
            (Color.green(colour) * held).toInt(),
            (Color.blue(colour) * held).toInt(),
        )
    }

    fun hot(colour: Int, by: Float): Int {
        val held = by.coerceIn(0f, 1f)
        return Color.rgb(
            mix(Color.red(colour).toFloat(), 255f, held).toInt(),
            mix(Color.green(colour).toFloat(), 255f, held).toInt(),
            mix(Color.blue(colour).toFloat(), 255f, held).toInt(),
        )
    }

    fun vignette(canvas: Canvas, paint: Paint, w: Float, h: Float, strength: Float) {
        val radius = hypot(w, h) * 0.62f
        paint.shader = RadialGradient(
            w * 0.5f,
            h * 0.5f,
            radius,
            intArrayOf(0x00000000, fade(0xFF000000.toInt(), 0.55f * strength)),
            floatArrayOf(0.55f, 1f),
            Shader.TileMode.CLAMP,
        )
        canvas.drawRect(0f, 0f, w, h, paint)
        paint.shader = null
    }

    fun scatter(seed: Int, salt: Int): Float {
        var held = seed * 374_761_393 + salt * 668_265_263
        held = (held xor (held shr 13)) * 1_274_126_177
        return ((held xor (held shr 16)) and 0x00FF_FFFF) / 16_777_216f
    }
}

internal object Tape {

    const val LUMA_LINES = 240f

    const val CHROMA_RED = 0xFFFF2D3A.toInt()
    const val CHROMA_BLUE = 0xFF2BD6FF.toInt()

    const val SWITCH_HEIGHT = 0.024f

    const val DROPOUTS = 7

    fun wobble(life: Float, strength: Float): Float =
        (sin(life * 11.3f) * 0.6f + sin(life * 2.7f) * 0.4f) * strength

    fun trackingAt(life: Float): Float = ((life * 0.13f) % 1.6f) - 0.3f

    fun chroma(life: Float, strength: Float): Float =
        (0.55f + 0.45f * sin(life * 1.9f)) * strength

    fun lines(canvas: Canvas, paint: Paint, w: Float, h: Float, life: Float, strength: Float) {
        val spacing = max(2f, h / LUMA_LINES)
        val drift = (life * 22f) % spacing
        paint.style = Paint.Style.FILL
        paint.color = Ink.fade(0xFF000000.toInt(), 0.26f * strength)
        paint.strokeWidth = spacing * 0.42f
        var y = -spacing + drift
        while (y < h) {
            canvas.drawLine(0f, y, w, y, paint)
            y += spacing
        }
    }

    fun bloom(canvas: Canvas, paint: Paint, w: Float, h: Float, life: Float, strength: Float) {
        val band = ((life * 0.22f) % 1.4f) * h - h * 0.2f
        val tall = h * 0.10f
        paint.style = Paint.Style.FILL
        paint.color = Ink.fade(0xFFFFFFFF.toInt(), 0.035f * strength)
        canvas.drawRect(0f, band, w, band + tall, paint)
    }

    fun headSwitch(canvas: Canvas, paint: Paint, w: Float, h: Float, life: Float, strength: Float) {
        val tall = h * SWITCH_HEIGHT
        val top = h - tall
        paint.style = Paint.Style.FILL
        paint.color = Ink.fade(0xFF000000.toInt(), 0.55f * strength)
        canvas.drawRect(0f, top, w, h, paint)

        var y = top
        var row = 0
        val step = max(1f, tall / 9f)
        while (y < h) {
            val fell = (row + (life * 30f).toInt()) % 97
            val torn = (Ink.scatter(fell, 41) - 0.5f) * w * 1.1f
            paint.color = Ink.fade(0xFFFFFFFF.toInt(), (0.05f + 0.16f * Ink.scatter(fell, 43)) * strength)
            canvas.drawRect(torn, y, torn + w, y + step, paint)
            y += step
            row += 1
        }
        paint.color = Ink.fade(0xFFFFFFFF.toInt(), 0.10f * strength)
        canvas.drawRect(0f, top - h * 0.002f, w, top, paint)
    }

    fun tracking(canvas: Canvas, paint: Paint, w: Float, h: Float, life: Float, strength: Float) {
        val at = trackingAt(life)
        if (at < 0f || at > 1f) return
        val tall = h * 0.055f
        val top = at * h
        val fade = 1f - abs(at - 0.5f) * 1.4f
        if (fade <= 0f) return
        paint.style = Paint.Style.FILL
        var y = top
        var row = 0
        while (y < top + tall) {
            val slip = (Ink.scatter(row, (life * 60f).toInt()) - 0.5f) * w * 0.18f
            paint.color = Ink.fade(0xFFFFFFFF.toInt(), 0.05f * fade * strength)
            canvas.drawRect(slip, y, slip + w, y + 2f, paint)
            paint.color = Ink.fade(0xFF000000.toInt(), 0.16f * fade * strength)
            canvas.drawRect(0f, y + 2f, w, y + 3.4f, paint)
            y += 3.4f
            row += 1
        }
    }

    fun dropouts(canvas: Canvas, paint: Paint, w: Float, h: Float, life: Float, strength: Float) {
        paint.style = Paint.Style.FILL
        val moment = (life * 7f).toInt()
        for (index in 0 until DROPOUTS) {
            val seed = index * 31 + moment
            if (Ink.scatter(seed, 17) > 0.42f) continue
            val y = Ink.scatter(seed, 19) * h
            val from = Ink.scatter(seed, 23) * w * 0.86f
            val wide = w * (0.02f + Ink.scatter(seed, 29) * 0.10f)
            val tall = max(1.4f, h / LUMA_LINES * (1f + Ink.scatter(seed, 31) * 1.6f))
            paint.color = Ink.fade(0xFFFFFFFF.toInt(), 0.45f * strength)
            canvas.drawRect(from, y, from + wide, y + tall, paint)
            paint.color = Ink.fade(0xFF000000.toInt(), 0.35f * strength)
            canvas.drawRect(from, y + tall, from + wide, y + tall * 1.8f, paint)
        }
    }

    fun colourUnder(
        canvas: Canvas,
        paint: Paint,
        w: Float,
        h: Float,
        life: Float,
        strength: Float,
    ) {
        val late = (w * 0.006f) * chroma(life, strength)
        if (late <= 0.2f) return
        paint.style = Paint.Style.FILL
        val rows = 26
        val tall = h / rows
        for (row in 0 until rows) {
            val y = row * tall
            val smear = late * (0.5f + Ink.scatter(row, (life * 4f).toInt()) * 1.2f)
            paint.color = Ink.fade(CHROMA_RED, 0.030f * strength)
            canvas.drawRect(-smear, y, w - smear, y + tall, paint)
            paint.color = Ink.fade(CHROMA_BLUE, 0.030f * strength)
            canvas.drawRect(smear * 1.6f, y, w + smear * 1.6f, y + tall, paint)
        }
    }

    fun over(canvas: Canvas, paint: Paint, w: Float, h: Float, life: Float, strength: Float) {
        if (strength <= 0f) return
        colourUnder(canvas, paint, w, h, life, strength)
        lines(canvas, paint, w, h, life, strength)
        tracking(canvas, paint, w, h, life, strength)
        dropouts(canvas, paint, w, h, life, strength)
        bloom(canvas, paint, w, h, life, strength)
        headSwitch(canvas, paint, w, h, life, strength)
    }
}

class LauncherPreview(context: Context, private val palette: Palette) : View(context) {

    private companion object {
        const val SHAPES = 3
    }

    private val paint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val mask = Path()
    private val box = RectF()
    private var picture: Bitmap? = null
    private val android = ImageView(context).apply {
        setImageResource(R.drawable.omni_art_project)
    }

    fun show(path: String?) {
        picture = path?.let { runCatching { BitmapFactory.decodeFile(it) }.getOrNull() }
        invalidate()
    }

    override fun onDraw(canvas: Canvas) {
        val w = width.toFloat()
        val h = height.toFloat()
        if (w <= 0f || h <= 0f) return

        val edge = min(h * 0.78f, w / SHAPES * 0.72f)
        val gap = (w - edge * SHAPES) / (SHAPES + 1)
        val top = (h - edge) * 0.42f

        for (index in 0 until SHAPES) {
            val left = gap + index * (edge + gap)
            box.set(left, top, left + edge, top + edge)
            mask.rewind()
            when (index) {
                0 -> mask.addRoundRect(box, edge * 0.16f, edge * 0.16f, Path.Direction.CW)
                1 -> mask.addCircle(box.centerX(), box.centerY(), edge * 0.5f, Path.Direction.CW)
                else -> mask.addRoundRect(box, edge * 0.42f, edge * 0.42f, Path.Direction.CW)
            }

            val held = picture
            paint.style = Paint.Style.FILL
            paint.color = palette.raised
            canvas.drawPath(mask, paint)

            if (held != null) {
                canvas.save()
                canvas.clipPath(mask)
                canvas.drawBitmap(held, null, box, paint)
                canvas.restore()
            } else {
                canvas.save()
                canvas.clipPath(mask)
                android.drawable.setBounds(
                    box.left.toInt(),
                    box.top.toInt(),
                    box.right.toInt(),
                    box.bottom.toInt(),
                )
                android.drawable.draw(canvas)
                canvas.restore()
            }

            paint.style = Paint.Style.STROKE
            paint.strokeWidth = max(1f, edge * 0.012f)
            paint.color = Ink.fade(palette.divider, 0.9f)
            canvas.drawPath(mask, paint)
        }
        paint.style = Paint.Style.FILL
    }
}

class BinaryVeil(context: Context) : View(context) {

    private companion object {
        const val WIDEST = 26
        const val TALLEST = 44

        const val CELL_DP = 15f
        const val SWEEP_MILLIS = 420f

        const val LATEST_MILLIS = 400L

        const val EDGE = 0.22f
    }

    private val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        typeface = Typeface.MONOSPACE
        textAlign = Paint.Align.CENTER
    }
    private val line = Paint()
    private var accent = Color.WHITE
    private var began = 0L
    private var running = false
    private var swapped = false
    private var whenDone: (() -> Unit)? = null
    private val glyph = CharArray(1)

    private val step = object : Runnable {
        override fun run() {
            if (!running) return
            invalidate()
            postOnAnimation(this)
        }
    }

    private val deadline = Runnable {
        running = false
        visibility = GONE
        swapNow()
    }

    fun sweep(colour: Int, swap: () -> Unit) {
        swapNow()
        if (Motion.still()) {
            running = false
            visibility = GONE
            removeCallbacks(deadline)
            swap()
            return
        }
        accent = colour
        whenDone = swap
        began = SystemClock.uptimeMillis()
        running = true
        swapped = false
        visibility = VISIBLE
        alpha = 1f
        removeCallbacks(deadline)
        postDelayed(deadline, SWEEP_MILLIS.toLong() + LATEST_MILLIS)
        postOnAnimation(step)
    }

    private fun swapNow() {
        if (swapped) return
        swapped = true
        val work = whenDone
        whenDone = null
        work?.invoke()
    }

    override fun onDetachedFromWindow() {
        running = false
        removeCallbacks(step)
        removeCallbacks(deadline)
        super.onDetachedFromWindow()
    }

    private var columns = WIDEST
    private var rows = TALLEST

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        val cell = CELL_DP * resources.displayMetrics.density
        columns = (w / cell).toInt().coerceIn(8, WIDEST)
        rows = (h / cell).toInt().coerceIn(12, TALLEST)
    }

    override fun onDraw(canvas: Canvas) {
        if (!running) return
        val w = width.toFloat()
        val h = height.toFloat()
        if (w <= 0f || h <= 0f) return

        val t = (SystemClock.uptimeMillis() - began) / SWEEP_MILLIS
        if (t >= 1f) {
            running = false
            visibility = GONE
            removeCallbacks(deadline)
            swapNow()
            return
        }
        if (t >= 0.5f) swapNow()

        val front = t * (1f + EDGE * 2f) - EDGE
        val columnWidth = w / columns
        val rowHeight = h / rows
        paint.textSize = min(columnWidth * 0.72f, rowHeight * 0.86f)

        val tracking = 1f - abs(t - 0.5f) * 2f
        val life = (SystemClock.uptimeMillis() - began) / 1000f

        canvas.save()
        canvas.translate(0f, Tape.wobble(life, h * 0.0035f * (0.4f + tracking)))

        for (row in 0 until rows) {
            val y = (row + 0.5f) / rows
            val distance = abs(y - front)
            if (distance > EDGE) continue
            val near = 1f - distance / EDGE
            val lit = near * near

            val slip = (Ink.scatter(row, (life * 12f).toInt()) - 0.5f) *
                columnWidth * 3.2f * tracking
            val bend = sin((y * 9f + life * 3.4f).toDouble()).toFloat() *
                columnWidth * 0.5f * tracking
            val shift = slip + bend

            val split = columnWidth * (0.10f + 0.30f * tracking) * (0.4f + lit)

            for (column in 0 until columns) {
                val roll = Ink.scatter(row * 131 + column, 7)
                if (roll > 0.55f + near * 0.4f) continue
                glyph[0] = if (Ink.scatter(row * 977 + column, 13) > 0.5f) '1' else '0'
                val x = (column + 0.5f) * columnWidth + shift
                val baseline = (row + 0.72f) * rowHeight

                paint.color = Ink.fade(Tape.CHROMA_RED, 0.20f + lit * 0.26f)
                canvas.drawText(glyph, 0, 1, x - split * 0.55f, baseline, paint)
                paint.color = Ink.fade(Tape.CHROMA_BLUE, 0.22f + lit * 0.30f)
                canvas.drawText(glyph, 0, 1, x + split * 1.45f, baseline, paint)
                paint.color = Ink.fade(Ink.hot(accent, lit * 0.85f), 0.20f + lit * 0.80f)
                canvas.drawText(glyph, 0, 1, x, baseline, paint)
            }
        }

        val seam = h * 0.018f
        val at = (front * h).coerceIn(-seam, h + seam)
        line.strokeWidth = 1f
        var torn = at
        var index = 0
        while (torn < at + seam) {
            val slip = (Ink.scatter(index, (front * 400f).toInt()) - 0.5f) * w * 0.5f
            line.color = Ink.fade(Ink.hot(accent, 0.85f), 0.10f + 0.28f * tracking)
            canvas.drawLine(slip, torn, w + slip, torn, line)
            torn += 1.5f
            index += 1
        }
        line.color = Ink.fade(Ink.hot(accent, 1f), 0.35f * tracking)
        line.strokeWidth = 1.4f
        canvas.drawLine(0f, at, w, at, line)

        canvas.restore()
        Tape.over(canvas, line, w, h, life, 0.65f + 0.35f * tracking)
    }
}

class KeyForgeView(context: Context, private var palette: Palette) : View(context) {

    private companion object {
        const val DOTS = 520
        const val GLYPHS = 130
        const val LOOP_MILLIS = 4_200f
        const val SEAL_MILLIS = 1_150f

        const val GATHERED = 0.44f
    }

    private enum class Phase { FORGING, SEALING, SEALED, REFUSED }

    private val paint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val text = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        typeface = Typeface.MONOSPACE
        textAlign = Paint.Align.CENTER
    }
    private val glass = Paint()
    private val key = Path()
    private val trace = Path()
    private val measure = PathMeasure()
    private val bounds = RectF()
    private val position = FloatArray(2)
    private val glyph = CharArray(1)

    private val dotX = FloatArray(DOTS)
    private val dotY = FloatArray(DOTS)
    private val dotPoints = FloatArray(DOTS * 2)
    private val glyphTargetX = FloatArray(GLYPHS)
    private val glyphTargetY = FloatArray(GLYPHS)

    private var began = SystemClock.uptimeMillis()
    private var sealedAt = 0L
    private var phase = Phase.FORGING
    private var laidOut = 0
    private var running = false

    var caption: String = ""
    var detail: String = ""
    private var fingerprint: String = ""

    private val step = object : Runnable {
        override fun run() {
            if (!running) return
            invalidate()
            postOnAnimation(this)
        }
    }

    fun begin() {
        began = SystemClock.uptimeMillis()
        phase = Phase.FORGING
        running = true
        postOnAnimation(step)
    }

    fun seal(shown: String) {
        if (phase != Phase.FORGING) return
        fingerprint = shown
        sealedAt = SystemClock.uptimeMillis()
        phase = Phase.SEALING
    }

    fun refuse() {
        phase = Phase.REFUSED
        running = false
        removeCallbacks(step)
        invalidate()
    }

    fun sealedThrough(): Boolean =
        phase == Phase.SEALED ||
            (phase == Phase.SEALING &&
                SystemClock.uptimeMillis() - sealedAt >= SEAL_MILLIS)

    fun repaint(next: Palette) {
        palette = next
        invalidate()
    }

    override fun onDetachedFromWindow() {
        running = false
        removeCallbacks(step)
        super.onDetachedFromWindow()
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        layOut(w.toFloat(), h.toFloat())
    }

    private fun layOut(w: Float, h: Float) {
        if (w <= 0f || h <= 0f) return
        buildKey(w, h)
        measure.setPath(key, false)
        var length = measure.length

        val lengths = ArrayList<Float>()
        do {
            lengths.add(measure.length)
        } while (measure.nextContour())
        length = lengths.sum()
        if (length <= 0f) return

        val everything = DOTS + GLYPHS
        for (index in 0 until everything) {

            val onOutline = index % 3 != 2
            var x: Float
            var y: Float
            if (onOutline) {
                val along = (Ink.scatter(index, 3) * length)
                var walked = 0f
                var contour = 0
                measure.setPath(key, false)
                while (contour < lengths.size - 1 && walked + lengths[contour] < along) {
                    walked += lengths[contour]
                    measure.nextContour()
                    contour++
                }
                measure.getPosTan(along - walked, position, null)
                x = position[0]
                y = position[1]
            } else {
                val turn = Ink.scatter(index, 5) * (PI * 2f).toFloat()
                val reach = bounds.width() * 0.5f * 0.62f * Ink.scatter(index, 9)
                x = bounds.centerX() + cos(turn) * reach
                y = bowCentreY + sin(turn) * reach
            }
            if (index < DOTS) {
                dotX[index] = x
                dotY[index] = y
            } else {
                glyphTargetX[index - DOTS] = x
                glyphTargetY[index - DOTS] = y
            }
        }
        laidOut = everything
    }

    private var bowCentreY = 0f
    private var bowRadius = 0f

    private fun buildKey(w: Float, h: Float) {
        key.reset()
        trace.reset()
        val span = min(w, h)
        val r = span * 0.150f
        val cx = w * 0.5f
        val cy = h * 0.33f
        bowCentreY = cy
        bowRadius = r
        bounds.set(cx - r, cy - r, cx + r, cy + r)

        val outer = RectF(cx - r, cy - r, cx + r, cy + r)
        key.addRoundRect(outer, r * 0.42f, r * 0.42f, Path.Direction.CW)
        val ward = RectF(
            cx - r * 0.40f,
            cy - r * 0.46f,
            cx + r * 0.40f,
            cy + r * 0.34f,
        )
        key.addRoundRect(ward, r * 0.18f, r * 0.18f, Path.Direction.CCW)

        val half = span * 0.030f
        val shoulder = cy + r * 0.98f
        val foot = h * 0.79f
        val tooth = span * 0.050f

        key.moveTo(cx - half * 1.9f, shoulder - half * 0.9f)
        key.quadTo(cx - half * 1.5f, shoulder, cx - half, shoulder + half * 0.5f)
        key.lineTo(cx - half, foot - tooth * 0.4f)
        key.quadTo(cx - half, foot, cx - half + tooth * 0.35f, foot)
        key.lineTo(cx + half, foot)
        key.lineTo(cx + half, foot - tooth * 0.60f)
        key.lineTo(cx + half + tooth * 1.05f, foot - tooth * 0.60f)
        key.lineTo(cx + half + tooth * 1.05f, foot - tooth * 1.15f)
        key.lineTo(cx + half, foot - tooth * 1.15f)
        key.lineTo(cx + half, foot - tooth * 1.85f)
        key.lineTo(cx + half + tooth * 0.70f, foot - tooth * 1.85f)
        key.lineTo(cx + half + tooth * 0.70f, foot - tooth * 2.35f)
        key.lineTo(cx + half, foot - tooth * 2.35f)
        key.lineTo(cx + half, shoulder + half * 0.5f)
        key.quadTo(cx + half * 1.5f, shoulder, cx + half * 1.9f, shoulder - half * 0.9f)
        key.close()

        val collar = shoulder + half * 1.6f
        trace.moveTo(cx - half * 2.1f, collar)
        trace.lineTo(cx + half * 2.1f, collar)
        trace.moveTo(cx - half * 1.7f, collar + half * 0.9f)
        trace.lineTo(cx + half * 1.7f, collar + half * 0.9f)

        trace.moveTo(cx, foot - tooth * 3.2f)
        trace.lineTo(cx, collar + half * 2.2f)

        trace.moveTo(cx - r * 0.40f, cy - r * 0.06f)
        trace.lineTo(cx + r * 0.40f, cy - r * 0.06f)
        trace.moveTo(cx - r * 0.22f, cy - r * 0.46f)
        trace.lineTo(cx - r * 0.22f, cy + r * 0.34f)
        trace.moveTo(cx + r * 0.22f, cy - r * 0.46f)
        trace.lineTo(cx + r * 0.22f, cy + r * 0.34f)
    }

    override fun onDraw(canvas: Canvas) {
        val w = width.toFloat()
        val h = height.toFloat()
        if (w <= 0f || h <= 0f) return
        if (laidOut == 0) layOut(w, h)

        val now = SystemClock.uptimeMillis()
        val life = (now - began) / 1000f

        val gather = Ink.ease(life, 0.30f, 2.10f)
        val reveal = Ink.ease(life, 1.05f, 2.35f)
        val sealing = if (phase == Phase.FORGING) 0f else
            ((now - sealedAt) / SEAL_MILLIS).coerceIn(0f, 1f)
        if (phase == Phase.SEALING && sealing >= 1f) phase = Phase.SEALED

        drawGround(canvas, w, h, life)
        drawParticles(canvas, w, h, life, gather, sealing)
        if (reveal > 0f) drawKey(canvas, life, reveal, sealing)
        drawWords(canvas, w, h, life, sealing)
        Tape.over(canvas, glass, w, h, life, 1f)
        Ink.vignette(canvas, glass, w, h, 1f)
    }

    private fun drawGround(canvas: Canvas, w: Float, h: Float, life: Float) {
        canvas.drawColor(palette.background)
        val horizon = h * 0.79f

        paint.shader = RadialGradient(
            w * 0.5f, bowCentreY, max(w, h) * 0.70f,
            Ink.fade(palette.glowFirst, 0.42f),
            Ink.fade(palette.background, 0f),
            Shader.TileMode.CLAMP,
        )
        canvas.drawRect(0f, 0f, w, h, paint)

        val heat = 0.72f + 0.28f * sin(life * 1.7f)
        paint.shader = RadialGradient(
            w * 0.5f, horizon, w * 0.62f,
            Ink.fade(Ink.hot(palette.accent, 0.35f), 0.34f * heat),
            Ink.fade(palette.background, 0f),
            Shader.TileMode.CLAMP,
        )
        canvas.drawRect(0f, horizon - w * 0.62f, w, h, paint)
        paint.shader = null

        paint.style = Paint.Style.STROKE
        paint.strokeWidth = max(1f, h * 0.0012f)
        var step = 1
        while (step <= 7) {
            val fall = step / 7f
            val y = horizon + (h - horizon) * fall * fall
            paint.color = Ink.fade(palette.accent, 0.11f * (1f - fall * 0.7f))
            canvas.drawLine(0f, y, w, y, paint)
            step += 1
        }
        var column = -6
        while (column <= 6) {
            val spread = column / 6f
            paint.color = Ink.fade(palette.accent, 0.09f * (1f - abs(spread) * 0.6f))
            canvas.drawLine(
                w * 0.5f + spread * w * 0.10f,
                horizon,
                w * 0.5f + spread * w * 1.35f,
                h,
                paint,
            )
            column += 1
        }

        paint.strokeWidth = max(1.2f, h * 0.0022f)
        paint.color = Ink.fade(Ink.hot(palette.accent, 0.55f), 0.42f * heat)
        canvas.drawLine(0f, horizon, w, horizon, paint)
        paint.style = Paint.Style.FILL
    }

    private fun drawParticles(
        canvas: Canvas,
        w: Float,
        h: Float,
        life: Float,
        gather: Float,
        sealing: Float,
    ) {
        val cx = w * 0.5f
        val far = max(w, h) * 0.62f

        val thrown = 1f + sealing * sealing * 2.4f
        val faded = 1f - sealing

        var at = 0
        for (index in 0 until DOTS) {
            val turn = Ink.scatter(index, 1) * (PI * 2f).toFloat() +
                life * (0.35f + Ink.scatter(index, 2) * 0.9f)
            val reach = (far * (0.12f + Ink.scatter(index, 4) * 0.88f)) * (1f - gather * 0.78f)
            val fromX = cx + cos(turn) * reach
            val fromY = bowCentreY + sin(turn) * reach * 0.92f
            val toX = dotX[index]
            val toY = dotY[index]
            var x = Ink.mix(fromX, toX, gather)
            var y = Ink.mix(fromY, toY, gather)
            if (sealing > 0f) {
                x = cx + (x - cx) * thrown
                y = bowCentreY + (y - bowCentreY) * thrown
            }
            dotPoints[at++] = x
            dotPoints[at++] = y
        }
        paint.strokeCap = Paint.Cap.ROUND
        paint.strokeWidth = min(w, h) * 0.0055f
        paint.color = Ink.fade(Ink.hot(palette.accent, 0.25f), (0.30f + gather * 0.45f) * faded)
        canvas.drawPoints(dotPoints, 0, at, paint)
        paint.strokeCap = Paint.Cap.BUTT

        text.textSize = min(w, h) * 0.026f
        for (index in 0 until GLYPHS) {
            val seed = DOTS + index
            val turn = Ink.scatter(seed, 1) * (PI * 2f).toFloat() +
                life * (0.30f + Ink.scatter(seed, 2) * 0.8f)
            val reach = (far * (0.14f + Ink.scatter(seed, 4) * 0.86f)) * (1f - gather * 0.80f)
            var x = Ink.mix(cx + cos(turn) * reach, glyphTargetX[index], gather)
            var y = Ink.mix(bowCentreY + sin(turn) * reach * 0.92f, glyphTargetY[index], gather)
            if (sealing > 0f) {
                x = cx + (x - cx) * thrown
                y = bowCentreY + (y - bowCentreY) * thrown
            }
            glyph[0] = if (((life * 3f).toInt() + index) % 2 == 0) '1' else '0'
            val lit = 0.35f + 0.65f * Ink.scatter(seed, 6)
            text.color = Ink.fade(Ink.hot(palette.accent, gather * 0.5f), lit * faded)
            canvas.drawText(glyph, 0, 1, x, y, text)
        }
    }

    private fun split(canvas: Canvas, path: Path, width: Float, alpha: Float, by: Float) {
        paint.style = Paint.Style.STROKE
        paint.strokeWidth = width
        paint.color = Ink.fade(0xFFFF3B30.toInt(), alpha * 0.55f)
        canvas.save()
        canvas.translate(-by, 0f)
        canvas.drawPath(path, paint)
        canvas.restore()
        paint.color = Ink.fade(0xFF2BD6FF.toInt(), alpha * 0.55f)
        canvas.save()
        canvas.translate(by, 0f)
        canvas.drawPath(path, paint)
        canvas.restore()
    }

    private fun drawKey(canvas: Canvas, life: Float, reveal: Float, sealing: Float) {
        val breath = 0.82f + 0.18f * sin(life * 2.1f)
        paint.style = Paint.Style.STROKE
        paint.strokeJoin = Paint.Join.ROUND

        paint.strokeWidth = bowRadius * 0.34f
        paint.color = Ink.fade(palette.accent, 0.13f * reveal * breath)
        canvas.drawPath(key, paint)

        paint.strokeWidth = bowRadius * 0.11f
        paint.color = Ink.fade(Ink.hot(palette.accent, 0.35f), 0.42f * reveal)
        canvas.drawPath(key, paint)

        split(canvas, key, max(1.4f, bowRadius * 0.028f), reveal, bowRadius * 0.030f)
        paint.style = Paint.Style.STROKE
        paint.strokeWidth = max(1.4f, bowRadius * 0.028f)
        paint.color = Ink.fade(Ink.hot(palette.accent, 0.85f + sealing * 0.15f), reveal)
        canvas.drawPath(key, paint)

        val filled = Ink.ease(life, 1.7f, 3.0f)
        if (filled > 0f) {
            paint.strokeWidth = max(1f, bowRadius * 0.020f)
            paint.color = Ink.fade(Ink.hot(palette.accent, 0.55f), 0.75f * filled)
            canvas.drawPath(trace, paint)
        }

        val turning = Ink.ease(life, 1.9f, 2.8f)
        if (turning > 0f) {
            paint.strokeWidth = max(1f, bowRadius * 0.022f)
            paint.color = Ink.fade(palette.accent, 0.55f * turning)
            val inner = RectF(
                bounds.centerX() - bowRadius * 0.66f,
                bowCentreY - bowRadius * 0.66f,
                bounds.centerX() + bowRadius * 0.66f,
                bowCentreY + bowRadius * 0.66f,
            )
            val spin = (life * 62f) % 360f
            canvas.drawArc(inner, spin, 96f, false, paint)
            canvas.drawArc(inner, spin + 180f, 96f, false, paint)
            inner.inset(bowRadius * 0.14f, bowRadius * 0.14f)
            canvas.drawArc(inner, -spin * 1.4f, 62f, false, paint)
        }

        if (sealing > 0f) {
            val shut = Ink.ease(sealing, 0f, 0.55f)
            val body = bowRadius * 0.46f
            val cx = bounds.centerX()
            val cy = bowCentreY + bowRadius * 0.18f
            paint.strokeWidth = max(1.6f, bowRadius * 0.05f)
            paint.color = Ink.fade(Ink.hot(palette.ok, 0.4f), shut)
            val shackle = RectF(
                cx - body * 0.42f,
                cy - body * 1.02f,
                cx + body * 0.42f,
                cy - body * 0.18f,
            )
            canvas.drawArc(shackle, 180f, 180f * shut, false, paint)
            paint.style = Paint.Style.FILL
            canvas.drawRoundRect(
                cx - body * 0.62f, cy - body * 0.30f,
                cx + body * 0.62f, cy + body * 0.58f,
                body * 0.16f, body * 0.16f, paint,
            )
            paint.style = Paint.Style.STROKE

            val ring = Ink.ease(sealing, 0.10f, 1f)
            paint.strokeWidth = max(1f, bowRadius * 0.06f * (1f - ring))
            paint.color = Ink.fade(Ink.hot(palette.ok, 0.6f), (1f - ring) * 0.9f)
            canvas.drawCircle(cx, bowCentreY, bowRadius * (1f + ring * 3.2f), paint)
        }
        paint.style = Paint.Style.FILL
    }

    private fun drawWords(canvas: Canvas, w: Float, h: Float, life: Float, sealing: Float) {
        text.textSize = min(w, h) * 0.038f
        text.color = Ink.fade(palette.foreground, Ink.ease(life, 0.2f, 0.9f))
        canvas.drawText(caption, w * 0.5f, h * 0.90f, text)

        text.textSize = min(w, h) * 0.028f
        val said = if (sealing > 0f && fingerprint.isNotEmpty()) fingerprint else detail
        text.color = Ink.fade(
            if (sealing > 0f) palette.ok else palette.muted,
            Ink.ease(life, 0.5f, 1.2f),
        )
        canvas.drawText(said, w * 0.5f, h * 0.945f, text)
    }
}

class BuildStageView(context: Context, private var palette: Palette) : View(context) {

    private companion object {
        const val CATCH_UP = 3.4f
    }

    private val paint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val text = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        typeface = Typeface.MONOSPACE
    }
    private val glass = Paint()
    private val ring = RectF()

    private var running = false
    private var lastFrame = 0L
    private var began = SystemClock.uptimeMillis()

    private var reported = 0f
    private var drawn = 0f
    private var stageAt = 0
    private var refusedAt = -1
    private var done = false
    private var stamps = arrayOfNulls<String>(0)
    private val clock = SimpleDateFormat("HH:mm:ss", Locale.US)

    var title: String = ""
    var stages: List<String> = emptyList()
    var remaining: String = ""
    var note: String = ""

    private val step = object : Runnable {
        override fun run() {
            if (!running) return
            invalidate()
            postOnAnimation(this)
        }
    }

    fun begin() {
        began = SystemClock.uptimeMillis()
        lastFrame = began
        reported = 0f
        drawn = 0f
        stageAt = 0
        refusedAt = -1
        done = false
        running = true
        stamps = arrayOfNulls(stages.size)
        stamp(0)
        postOnAnimation(step)
    }

    private fun stamp(at: Int) {
        if (at in stamps.indices && stamps[at] == null) {
            stamps[at] = clock.format(Date())
        }
    }

    fun observe(percent: Int, stage: Int, finished: Boolean, refused: Boolean) {
        reported = percent.coerceIn(0, 100).toFloat()
        stageAt = stage.coerceAtLeast(0)
        done = finished
        for (at in 0..stageAt) {
            stamp(at)
        }
        if (finished) {
            stages.indices.forEach { stamp(it) }
        }
        if (refused) refusedAt = stageAt
    }

    fun rest() {
        running = false
        removeCallbacks(step)
    }

    fun repaint(next: Palette) {
        palette = next
        invalidate()
    }

    override fun onDetachedFromWindow() {
        rest()
        super.onDetachedFromWindow()
    }

    override fun onDraw(canvas: Canvas) {
        val w = width.toFloat()
        val h = height.toFloat()
        if (w <= 0f || h <= 0f) return

        val now = SystemClock.uptimeMillis()
        val since = ((now - lastFrame).coerceIn(0L, 100L)) / 1000f
        lastFrame = now
        drawn += (reported - drawn) * (since * CATCH_UP).coerceIn(0f, 1f)
        if (done && reported >= 100f && drawn > 99.4f) drawn = 100f

        canvas.drawColor(palette.background)

        val pad = min(w, h) * 0.05f
        val card = RectF(pad, pad, w - pad, h - pad)
        paint.style = Paint.Style.FILL
        paint.color = palette.surface
        canvas.drawRoundRect(card, pad * 0.6f, pad * 0.6f, paint)
        paint.style = Paint.Style.STROKE
        paint.strokeWidth = 1f
        paint.color = palette.divider
        canvas.drawRoundRect(card, pad * 0.6f, pad * 0.6f, paint)
        paint.style = Paint.Style.FILL

        val left = card.left + pad
        val right = card.right - pad
        val mark = min(w, h) * 0.075f
        val hammer = Mark(Mark.Shape.PACKAGE, palette.accent)
        hammer.setBounds(
            left.toInt(),
            (card.top + pad * 0.7f).toInt(),
            (left + mark).toInt(),
            (card.top + pad * 0.7f + mark).toInt(),
        )
        hammer.draw(canvas)

        text.textAlign = Paint.Align.LEFT
        text.textSize = min(w, h) * 0.036f
        text.color = palette.foreground
        canvas.drawText(title, left + mark + pad * 0.6f, card.top + pad * 1.5f, text)
        text.textSize = min(w, h) * 0.026f
        text.color = palette.muted
        canvas.drawText(note, left + mark + pad * 0.6f, card.top + pad * 2.3f, text)

        val barTop = card.top + pad * 3.2f
        val barTall = min(w, h) * 0.018f
        val shown = "%" + drawn.toInt().coerceIn(0, 100)
        text.textSize = min(w, h) * 0.028f
        val room = text.measureText(shown) + pad * 0.5f
        ring.set(left, barTop, right - room, barTop + barTall)
        paint.color = palette.raised
        canvas.drawRoundRect(ring, barTall / 2f, barTall / 2f, paint)
        ring.right = left + (right - room - left) * (drawn / 100f).coerceIn(0f, 1f)
        paint.color = if (refusedAt >= 0) palette.error else palette.accent
        canvas.drawRoundRect(ring, barTall / 2f, barTall / 2f, paint)
        text.color = palette.foreground
        text.textAlign = Paint.Align.RIGHT
        canvas.drawText(shown, right, barTop + barTall, text)
        text.textAlign = Paint.Align.LEFT

        val listTop = barTop + barTall + pad * 1.2f
        val room2 = (card.bottom - pad * 0.6f - listTop).coerceAtLeast(1f)
        val rows = stages.size.coerceAtLeast(1)
        val step = (room2 / rows).coerceAtMost(min(w, h) * 0.062f)
        text.textSize = (step * 0.44f).coerceAtMost(min(w, h) * 0.026f)
        stages.forEachIndexed { index, named ->
            val y = listTop + step * (index + 0.7f)
            val past = index < stageAt || (done && reported >= 100f)
            val here = index == stageAt && !done
            val stamp = stamps.getOrNull(index).orEmpty()
            text.color = palette.muted
            canvas.drawText(stamp, left, y, text)
            val dot = left + step * 1.7f
            when {
                refusedAt == index -> {
                    paint.color = palette.error
                    canvas.drawCircle(dot, y - step * 0.16f, step * 0.16f, paint)
                }
                past -> {
                    paint.style = Paint.Style.STROKE
                    paint.strokeWidth = step * 0.08f
                    paint.color = palette.ok
                    canvas.drawLine(
                        dot - step * 0.16f,
                        y - step * 0.16f,
                        dot - step * 0.03f,
                        y - step * 0.03f,
                        paint,
                    )
                    canvas.drawLine(
                        dot - step * 0.03f,
                        y - step * 0.03f,
                        dot + step * 0.18f,
                        y - step * 0.32f,
                        paint,
                    )
                    paint.style = Paint.Style.FILL
                }
                else -> {
                    paint.style = Paint.Style.STROKE
                    paint.strokeWidth = step * 0.06f
                    paint.color = if (here) palette.accent else palette.divider
                    canvas.drawCircle(dot, y - step * 0.16f, step * 0.16f, paint)
                    paint.style = Paint.Style.FILL
                }
            }
            text.color = when {
                refusedAt == index -> palette.error
                here -> palette.accent
                past -> palette.foreground
                else -> palette.muted
            }
            canvas.drawText(named, dot + step * 0.42f, y, text)
        }

        text.textAlign = Paint.Align.RIGHT
        text.color = palette.muted
        text.textSize = min(w, h) * 0.026f
        canvas.drawText(remaining, right, card.bottom - pad * 0.4f, text)
        text.textAlign = Paint.Align.LEFT
    }
}

internal enum class Kind {
    COMMENT,
    TEXT,
    NUMBER,
    KEYWORD,
    TYPE,
    ANNOTATION,
    TAG,
    ATTRIBUTE,
}

internal class Token(val from: Int, val to: Int, val kind: Kind)

internal class Grammar(
    val keywords: Set<String>,
    val lineComment: String?,
    val blockComment: Pair<String, String>?,

    val characters: Boolean,

    val annotations: Boolean,

    val markup: Boolean,

    val folded: Boolean = false,

    val keys: Boolean = false,

    val fields: Boolean = false,

    val tripled: Boolean = false,

    val ticks: Boolean = false,

    val embedded: Boolean = false,

    val notes: Boolean = false,
) {
    companion object {

        private val JAVA = setOf(
            "abstract", "assert", "boolean", "break", "byte", "case", "catch", "char", "class",
            "const", "continue", "default", "do", "double", "else", "enum", "extends", "final",
            "finally", "float", "for", "goto", "if", "implements", "import", "instanceof", "int",
            "interface", "long", "native", "new", "package", "private", "protected", "public",
            "return", "short", "static", "strictfp", "super", "switch", "synchronized", "this",
            "throw", "throws", "transient", "try", "void", "volatile", "while",

            "exports", "module", "non-sealed", "open", "opens", "permits", "provides", "record",
            "requires", "sealed", "to", "transitive", "uses", "var", "when", "with", "yield",
            "true", "false", "null",
        )

        private val KOTLIN = setOf(
            "as", "break", "class", "continue", "do", "else", "false", "for", "fun", "if", "in",
            "interface", "is", "null", "object", "package", "return", "super", "this", "throw",
            "true", "try", "typealias", "typeof", "val", "var", "when", "while",
            "by", "catch", "constructor", "delegate", "dynamic", "field", "file", "finally",
            "get", "import", "init", "param", "property", "receiver", "set", "setparam", "value",
            "where", "actual", "abstract", "annotation", "companion", "const", "crossinline",
            "data", "enum", "expect", "external", "final", "infix", "inline", "inner", "internal",
            "lateinit", "noinline", "open", "operator", "out", "override", "private", "protected",
            "public", "reified", "sealed", "suspend", "tailrec", "vararg",
        )

        private val RUST = setOf(
            "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
            "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod",
            "move", "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super",
            "trait", "true", "type", "union", "unsafe", "use", "where", "while",
        )

        private val CPP = setOf(
            "alignas", "alignof", "and", "asm", "auto", "bool", "break", "case", "catch", "char",
            "class", "const", "consteval", "constexpr", "constinit", "const_cast", "continue",
            "co_await", "co_return", "co_yield", "decltype", "default", "delete", "do", "double",
            "dynamic_cast", "else", "enum", "explicit", "export", "extern", "false", "float",
            "for", "friend", "goto", "if", "inline", "int", "long", "mutable", "namespace", "new",
            "noexcept", "not", "nullptr", "operator", "or", "private", "protected", "public",
            "register", "reinterpret_cast", "requires", "return", "short", "signed", "sizeof",
            "static", "static_assert", "static_cast", "struct", "switch", "template", "this",
            "thread_local", "throw", "true", "try", "typedef", "typeid", "typename", "union",
            "unsigned", "using", "virtual", "void", "volatile", "wchar_t", "while", "xor",
        )

        private val SCRIPT_WORDS = setOf(
            "arguments", "as", "async", "await", "break", "case", "catch", "class", "const",
            "continue", "debugger", "default", "delete", "do", "else", "export", "extends",
            "false", "finally", "for", "from", "function", "get", "globalThis", "if", "import",
            "in", "instanceof", "let", "new", "null", "of", "return", "set", "static", "super",
            "switch", "this", "throw", "true", "try", "typeof", "undefined", "var", "void",
            "while", "with", "yield",
        )

        private val TYPED_WORDS = SCRIPT_WORDS + setOf(
            "abstract", "any", "asserts", "bigint", "boolean", "declare", "enum", "implements",
            "infer", "interface", "is", "keyof", "module", "namespace", "never", "number",
            "object", "override", "private", "protected", "public", "readonly", "require",
            "satisfies", "string", "symbol", "type", "unique", "unknown",
        )

        private val PYTHON = setOf(
            "and", "as", "assert", "async", "await", "break", "case", "class", "continue", "def",
            "del", "elif", "else", "except", "False", "finally", "for", "from", "global", "if",
            "import", "in", "is", "lambda", "match", "None", "nonlocal", "not", "or", "pass",
            "raise", "return", "self", "True", "try", "while", "with", "yield",
        )

        private val SHELL = setOf(
            "alias", "break", "case", "cd", "continue", "declare", "do", "done", "echo", "elif",
            "else", "esac", "eval", "exec", "exit", "export", "false", "fi", "for", "function",
            "if", "in", "local", "read", "readonly", "return", "select", "set", "shift", "source",
            "test", "then", "time", "trap", "true", "type", "umask", "unalias", "unset", "until",
            "wait", "while",
        )

        private val SQL = setOf(
            "add", "all", "alter", "and", "as", "asc", "begin", "between", "by", "case", "cast",
            "check", "column", "commit", "constraint", "create", "cross", "default", "delete",
            "desc", "distinct", "drop", "else", "end", "exists", "foreign", "from", "full",
            "group", "having", "if", "in", "index", "inner", "insert", "into", "is", "join",
            "key", "left", "like", "limit", "not", "null", "offset", "on", "or", "order",
            "outer", "primary", "references", "replace", "right", "rollback", "select", "set",
            "table", "then", "transaction", "trigger", "union", "unique", "update", "using",
            "values", "view", "when", "where", "with",
        )

        private val SHEET_WORDS = setOf(
            "and", "auto", "from", "important", "inherit", "initial", "none", "not", "only",
            "revert", "to", "unset",
        )

        private val NOTED = setOf(
            "false", "no", "null", "off", "on", "true", "yes",
        )

        val PLAIN = Grammar(emptySet(), null, null, false, false, false)

        val SCRIPT = Grammar(
            SCRIPT_WORDS, "//", "/*" to "*/", true, true, false,
            ticks = true,
        )

        val SHEET = Grammar(
            SHEET_WORDS, null, "/*" to "*/", true, true, false,
            fields = true,
        )

        fun of(name: String): Grammar {
            val lower = name.lowercase()
            return when {
                lower.endsWith(".java") ->
                    Grammar(JAVA, "//", "/*" to "*/", true, true, false)
                lower.endsWith(".kt") || lower.endsWith(".kts") ->
                    Grammar(KOTLIN, "//", "/*" to "*/", true, true, false, tripled = true)
                lower.endsWith(".rs") ->
                    Grammar(RUST, "//", "/*" to "*/", true, true, false)
                lower.endsWith(".c") || lower.endsWith(".h") ||
                    lower.endsWith(".cc") || lower.endsWith(".cpp") ||
                    lower.endsWith(".hpp") || lower.endsWith(".cxx") ->
                    Grammar(CPP, "//", "/*" to "*/", true, false, false)
                lower.endsWith(".html") || lower.endsWith(".htm") ->
                    Grammar(
                        emptySet(), null, "<!--" to "-->", false, false, true,
                        embedded = true,
                    )
                lower.endsWith(".xml") || lower.endsWith(".svg") ->
                    Grammar(emptySet(), null, "<!--" to "-->", false, false, true)
                lower.endsWith(".json") ->
                    Grammar(
                        setOf("true", "false", "null"), null, null, false, false, false,
                        keys = true,
                    )
                lower.endsWith(".js") || lower.endsWith(".mjs") || lower.endsWith(".cjs") ||
                    lower.endsWith(".jsx") ->
                    SCRIPT
                lower.endsWith(".ts") || lower.endsWith(".tsx") ->
                    Grammar(
                        TYPED_WORDS, "//", "/*" to "*/", true, true, false,
                        ticks = true,
                    )
                lower.endsWith(".css") || lower.endsWith(".scss") || lower.endsWith(".less") ->
                    SHEET
                lower.endsWith(".py") || lower.endsWith(".pyi") ->
                    Grammar(
                        PYTHON, "#", null, true, true, false,
                        tripled = true,
                    )
                lower.endsWith(".sh") || lower.endsWith(".bash") || lower.endsWith(".zsh") ->
                    Grammar(
                        SHELL, "#", null, true, false, false,
                        ticks = true,
                    )
                lower.endsWith(".sql") ->
                    Grammar(SQL, "--", "/*" to "*/", true, false, false, folded = true)
                lower.endsWith(".md") || lower.endsWith(".markdown") ->
                    Grammar(emptySet(), null, null, false, false, false, notes = true)
                lower.endsWith(".yaml") || lower.endsWith(".yml") ->
                    Grammar(
                        NOTED, "#", null, true, false, false,
                        keys = true, fields = true,
                    )
                lower.endsWith(".omni") || lower.endsWith(".toml") ||
                    lower.endsWith(".properties") || lower.endsWith(".pro") ||
                    lower.endsWith(".gradle") || lower.endsWith(".cfg") ||
                    lower.endsWith(".ini") ->
                    Grammar(emptySet(), "#", null, false, false, false, fields = true)
                else -> PLAIN
            }
        }
    }
}

internal object Reader {

    const val LIMIT = 2_000_000

    fun read(text: CharSequence, grammar: Grammar): List<Token> {
        val out = ArrayList<Token>(min(text.length / 6 + 16, 200_000))
        when {
            grammar.notes -> readNotes(text, out)
            grammar.markup -> readMarkup(text, grammar, out)
            else -> readCode(text, grammar, out)
        }
        return out
    }

    private fun readCode(text: CharSequence, grammar: Grammar, out: MutableList<Token>) {
        val end = min(text.length, LIMIT)
        val line = grammar.lineComment
        val block = grammar.blockComment
        var at = 0
        while (at < end) {
            val here = text[at]

            if (line != null && starts(text, at, line)) {
                var to = at
                while (to < end && text[to] != '\n') to++
                out.add(Token(at, to, Kind.COMMENT))
                at = to
                continue
            }

            if (block != null && starts(text, at, block.first)) {
                var to = at + block.first.length
                while (to < end && !starts(text, to, block.second)) to++
                to = min(end, to + block.second.length)
                out.add(Token(at, to, Kind.COMMENT))
                at = to
                continue
            }

            if (here == '"') {
                if (grammar.tripled && starts(text, at, "\"\"\"")) {
                    at = fenced(text, at, end, "\"\"\"", false, out)
                    continue
                }
                val to = quoted(text, at, end, '"', out)
                if (grammar.keys && labelled(text, to, end)) {
                    out[out.size - 1] = Token(at, to, Kind.ATTRIBUTE)
                }
                at = to
                continue
            }
            if (grammar.characters && here == '\'') {
                if (grammar.tripled && starts(text, at, "'''")) {
                    at = fenced(text, at, end, "'''", false, out)
                    continue
                }
                at = quoted(text, at, end, '\'', out)
                continue
            }
            if (grammar.ticks && here == '`') {
                at = fenced(text, at, end, "`", false, out)
                continue
            }

            if (grammar.annotations && here == '@' && at + 1 < end && isWordStart(text[at + 1])) {
                var to = at + 1
                while (to < end && isWord(text[to])) to++
                out.add(Token(at, to, Kind.ANNOTATION))
                at = to
                continue
            }

            if (here.isDigit()) {
                var to = at
                while (to < end && (text[to].isLetterOrDigit() || text[to] == '.' ||
                        text[to] == '_' || text[to] == 'x' || text[to] == 'X')
                ) {
                    to++
                }
                out.add(Token(at, to, Kind.NUMBER))
                at = to
                continue
            }

            if (isWordStart(here)) {
                var to = at
                while (to < end && isWord(text[to])) to++
                val word = text.subSequence(at, to).toString()
                val looked = if (grammar.folded) word.lowercase() else word
                when {
                    grammar.keywords.contains(looked) -> out.add(Token(at, to, Kind.KEYWORD))

                    grammar.fields && labelled(text, to, end) ->
                        out.add(Token(at, to, Kind.ATTRIBUTE))

                    here.isUpperCase() -> out.add(Token(at, to, Kind.TYPE))
                }
                at = to
                continue
            }

            at++
        }
    }

    private fun labelled(text: CharSequence, from: Int, end: Int): Boolean {
        var at = from
        while (at < end && (text[at] == ' ' || text[at] == '\t')) at++
        return at < end && text[at] == ':'
    }

    private fun fenced(
        text: CharSequence,
        from: Int,
        end: Int,
        fence: String,
        withinLine: Boolean,
        out: MutableList<Token>,
    ): Int {
        var to = from + fence.length
        while (to < end && !starts(text, to, fence)) {
            if (withinLine && text[to] == '\n') break
            to++
        }
        to = min(end, to + fence.length)
        out.add(Token(from, to, Kind.TEXT))
        return max(to, from + 1)
    }

    private fun quoted(
        text: CharSequence,
        from: Int,
        end: Int,
        quote: Char,
        out: MutableList<Token>,
    ): Int {
        var to = from + 1
        while (to < end) {
            val c = text[to]
            if (c == '\\') {
                to += 2
                continue
            }

            if (c == '\n') break
            to++
            if (c == quote) break
        }
        to = min(to, end)
        out.add(Token(from, to, Kind.TEXT))
        return max(to, from + 1)
    }

    private fun readMarkup(text: CharSequence, grammar: Grammar, out: MutableList<Token>) {
        val end = min(text.length, LIMIT)
        val block = grammar.blockComment
        var at = 0
        while (at < end) {
            if (block != null && starts(text, at, block.first)) {
                var to = at + block.first.length
                while (to < end && !starts(text, to, block.second)) to++
                to = min(end, to + block.second.length)
                out.add(Token(at, to, Kind.COMMENT))
                at = to
                continue
            }
            if (text[at] != '<') {
                at++
                continue
            }

            var to = at + 1
            var shutting = false
            if (to < end && (text[to] == '/' || text[to] == '?' || text[to] == '!')) {
                shutting = text[to] == '/'
                to++
            }
            val nameFrom = to
            while (to < end && (isWord(text[to]) || text[to] == ':' || text[to] == '-')) to++
            val named = text.subSequence(nameFrom, to).toString().lowercase()
            if (to > nameFrom) out.add(Token(at, to, Kind.TAG))

            while (to < end && text[to] != '>') {
                when {
                    text[to] == '"' || text[to] == '\'' -> {
                        to = quoted(text, to, end, text[to], out)
                    }
                    isWordStart(text[to]) -> {
                        val from = to
                        while (to < end && (isWord(text[to]) || text[to] == ':' ||
                                text[to] == '-' || text[to] == '.')
                        ) {
                            to++
                        }
                        out.add(Token(from, to, Kind.ATTRIBUTE))
                    }
                    else -> to++
                }
            }
            if (to < end) {
                out.add(Token(to, to + 1, Kind.TAG))
                to++
            }
            val inner = when {
                !grammar.embedded || shutting -> null
                named == "script" -> Grammar.SCRIPT
                named == "style" -> Grammar.SHEET
                else -> null
            }
            if (inner != null) {
                val shut = "</$named"
                var body = to
                while (body < end && !startsLoosely(text, body, shut)) body++
                if (body > to) {
                    val held = ArrayList<Token>()
                    readCode(text.subSequence(to, body), inner, held)
                    held.forEach { out.add(Token(it.from + to, it.to + to, it.kind)) }
                }
                to = body
            }
            at = max(to, at + 1)
        }
    }

    private fun readNotes(text: CharSequence, out: MutableList<Token>) {
        val end = min(text.length, LIMIT)
        var at = 0
        var fresh = true
        while (at < end) {
            val here = text[at]
            if (here == '\n') {
                fresh = true
                at++
                continue
            }
            if (fresh) {
                fresh = false
                var probe = at
                while (probe < end && (text[probe] == ' ' || text[probe] == '\t')) probe++
                if (probe < end && (text[probe] == '#' || text[probe] == '>')) {
                    val kind = if (text[probe] == '#') Kind.KEYWORD else Kind.COMMENT
                    var to = probe
                    while (to < end && text[to] != '\n') to++
                    out.add(Token(probe, to, kind))
                    at = to
                    continue
                }
                if (probe + 1 < end && text[probe + 1] == ' ' &&
                    (text[probe] == '-' || text[probe] == '*' || text[probe] == '+')
                ) {
                    out.add(Token(probe, probe + 1, Kind.TAG))
                    at = probe + 1
                    continue
                }
            }
            if (here == '`') {
                val fence = if (starts(text, at, "```")) "```" else "`"
                at = fenced(text, at, end, fence, fence == "`", out)
                continue
            }
            if (here == '*' && at + 1 < end && text[at + 1] == '*') {
                var to = at + 2
                while (to < end && text[to] != '\n' && !starts(text, to, "**")) to++
                to = min(end, to + 2)
                out.add(Token(at, to, Kind.TYPE))
                at = max(to, at + 1)
                continue
            }
            if (here == '[') {
                var to = at + 1
                while (to < end && text[to] != '\n' && text[to] != ']') to++
                if (to < end && text[to] == ']') to++
                out.add(Token(at, to, Kind.ATTRIBUTE))
                at = max(to, at + 1)
                continue
            }
            if (here == '(') {
                var to = at + 1
                while (to < end && text[to] != '\n' && text[to] != ')') to++
                if (to < end && text[to] == ')') to++
                out.add(Token(at, to, Kind.TEXT))
                at = max(to, at + 1)
                continue
            }
            at++
        }
    }

    private fun starts(text: CharSequence, at: Int, what: String): Boolean {
        if (at + what.length > text.length) return false
        for (index in what.indices) {
            if (text[at + index] != what[index]) return false
        }
        return true
    }

    private fun startsLoosely(text: CharSequence, at: Int, what: String): Boolean {
        if (at + what.length > text.length) return false
        for (index in what.indices) {
            if (text[at + index].lowercaseChar() != what[index].lowercaseChar()) return false
        }
        return true
    }

    private fun isWordStart(c: Char): Boolean = c.isLetter() || c == '_' || c == '$'

    private fun isWord(c: Char): Boolean = c.isLetterOrDigit() || c == '_' || c == '$'
}

internal class History(private val limit: Int = 400) {

    class Change(
        val at: Int,
        val removed: CharSequence,
        val added: CharSequence,
        val whenMade: Long,
    )

    private val done = ArrayList<Change>()
    private val undone = ArrayList<Change>()

    companion object {

        const val BREAK_MILLIS = 700L
    }

    fun canUndo(): Boolean = done.isNotEmpty()

    fun canRedo(): Boolean = undone.isNotEmpty()

    fun forget() {
        done.clear()
        undone.clear()
    }

    fun record(change: Change) {
        undone.clear()
        val last = done.lastOrNull()
        val merges = last != null &&
            last.removed.isEmpty() &&
            change.removed.isEmpty() &&
            change.added.length == 1 &&
            change.added[0] != '\n' &&
            change.at == last.at + last.added.length &&
            change.whenMade - last.whenMade < BREAK_MILLIS
        if (merges) {
            done[done.size - 1] = Change(
                last.at,
                last.removed,
                SpannableStringBuilder(last.added).append(change.added),
                change.whenMade,
            )
            return
        }
        done.add(change)
        while (done.size > limit) done.removeAt(0)
    }

    fun undo(into: Editable): Int? {
        val change = done.removeLastOrNull() ?: return null
        undone.add(change)
        into.replace(change.at, change.at + change.added.length, change.removed)
        return change.at + change.removed.length
    }

    fun redo(into: Editable): Int? {
        val change = undone.removeLastOrNull() ?: return null
        done.add(change)
        into.replace(change.at, change.at + change.removed.length, change.added)
        return change.at + change.added.length
    }
}

class CodeEditor(context: Context, private var palette: Palette) : EditText(context) {

    private companion object {

        const val MARGIN_LINES = 40

        const val REST_MILLIS = 140L

        const val STEP = "    "

        const val SMALLEST_SP = 9f

        const val LARGEST_SP = 28f

        const val REACH = 40_000

        val OPENERS = "{(["
        val CLOSERS = "})]"
    }

    private val gutter = Paint(Paint.ANTI_ALIAS_FLAG).apply {
        typeface = Typeface.MONOSPACE
        textAlign = Paint.Align.RIGHT
    }
    private val rule = Paint()

    private var grammar: Grammar = Grammar.PLAIN
    private var tokens: List<Token> = emptyList()
    private var painted = -1 to -1
    private var reading = false
    private var generation = 0

    private var replaying = false
    private var helping = false
    private var typedAt = -1
    private var typedOne = '\u0000'
    private var matchHere = -1
    private var matchThere = -1
    private var faultLine = -1
    private var said = ""
    private val squiggle = Path()
    private val guide = Paint()
    private var readingSize = 13f

    private val pinch = android.view.ScaleGestureDetector(
        context,
        object : android.view.ScaleGestureDetector.SimpleOnScaleGestureListener() {
            override fun onScale(detector: android.view.ScaleGestureDetector): Boolean {
                readAt(readingSize * detector.scaleFactor)
                return true
            }
        },
    )

    private val history = History()

    fun canUndo(): Boolean = history.canUndo()

    fun canRedo(): Boolean = history.canRedo()

    var onChanged: (() -> Unit)? = null

    var onCaret: ((Int, Int) -> Unit)? = null

    fun caretAt(): Pair<Int, Int> {
        val at = selectionStart.coerceIn(0, text?.length ?: 0)
        val said = text?.toString().orEmpty()
        var line = 1
        var opened = 0
        for (index in 0 until at) {
            if (said[index] == '\n') {
                line += 1
                opened = index + 1
            }
        }
        return line to (at - opened + 1)
    }

    private val readAgain = Runnable { read() }

    init {
        setTextColor(palette.foreground)
        setBackgroundColor(Color.TRANSPARENT)
        typeface = Typeface.MONOSPACE
        setTextSize(TypedValue.COMPLEX_UNIT_SP, readingSize)
        gravity = Gravity.TOP or Gravity.START
        setHorizontallyScrolling(true)
        isHorizontalScrollBarEnabled = true
        includeFontPadding = false
        setLineSpacing(0f, 1.15f)
        rule.strokeWidth = 1f

        addTextChangedListener(object : TextWatcher {
            private var removed: CharSequence = ""

            override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) {
                removed = s?.subSequence(start, start + count)?.toString().orEmpty()
            }

            override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {
                if (replaying || s == null) return
                typedAt = start
                typedOne = if (before == 0 && count == 1) s[start] else '\u0000'
                history.record(
                    History.Change(
                        start,
                        removed,
                        s.subSequence(start, start + count).toString(),
                        android.os.SystemClock.uptimeMillis(),
                    )
                )
            }

            override fun afterTextChanged(s: Editable?) {
                if (!helping) {
                    faultLine = -1
                }
                if (s != null && !helping && !replaying && typedOne != '\u0000') {
                    val typed = typedOne
                    typedOne = '\u0000'
                    helpTyping(s, typedAt, typed)
                }
                onChanged?.invoke()
                removeCallbacks(readAgain)
                postDelayed(readAgain, REST_MILLIS)
            }
        })
    }

    fun open(name: String, text: String) {
        grammar = Grammar.of(name)
        replaying = true
        setText(text)
        replaying = false
        history.forget()
        setSelection(0)
        read()
    }

    fun repaint(next: Palette) {
        palette = next
        setTextColor(palette.foreground)
        painted = -1 to -1
        colour()
        invalidate()
    }

    fun showAt(line: Int, column: Int, length: Int) {
        post {
            val held = text ?: return@post
            var at = 0
            var seen = 1
            while (seen < line && at < held.length) {
                if (held[at] == '\n') {
                    seen += 1
                }
                at += 1
            }
            val from = (at + (column - 1).coerceAtLeast(0)).coerceIn(0, held.length)
            val to = (from + length).coerceIn(from, held.length)
            show(from, to)
            requestFocus()
        }
    }

    fun showLine(line: Int) {
        post {
            val held = text ?: return@post
            var at = 0
            var seen = 1
            while (seen < line && at < held.length) {
                if (held[at] == '\n') seen += 1
                at += 1
            }
            setSelection(at.coerceIn(0, held.length))
            val layout = layout ?: return@post
            val row = layout.getLineForOffset(at)

            scrollTo(0, (layout.getLineTop(row) - height / 3).coerceAtLeast(0))
            invalidate()
        }
    }

    private fun helpTyping(s: Editable, at: Int, typed: Char) {
        if (at < 0 || at >= s.length) return
        val closer = OPENERS.indexOf(typed).takeIf { it >= 0 }?.let { CLOSERS[it] }
        if (closer != null) {
            val after = if (at + 1 < s.length) s[at + 1] else '\n'
            if (after != '\n' && after != ' ' && CLOSERS.indexOf(after) < 0) return
            helping = true
            s.insert(at + 1, closer.toString())
            helping = false
            setSelection((at + 1).coerceIn(0, s.length))
            return
        }
        if (typed != '\n') return

        var lineStart = at
        while (lineStart > 0 && s[lineStart - 1] != '\n') {
            lineStart -= 1
        }
        val indent = StringBuilder()
        var walk = lineStart
        while (walk < at && (s[walk] == ' ' || s[walk] == '\t')) {
            indent.append(s[walk])
            walk += 1
        }
        val above = s.subSequence(lineStart, at).toString().trimEnd()
        val deeper = above.isNotEmpty() && OPENERS.indexOf(above.last()) >= 0
        val here = indent.toString() + if (deeper) STEP else ""
        val closes = at + 1 < s.length && CLOSERS.indexOf(s[at + 1]) >= 0
        val put = StringBuilder(here)
        if (deeper && closes) {
            put.append('\n').append(indent)
        }
        if (put.isEmpty()) return
        helping = true
        s.insert(at + 1, put.toString())
        helping = false
        setSelection((at + 1 + here.length).coerceIn(0, s.length))
    }

    private fun matchAround(caret: Int) {
        matchHere = -1
        matchThere = -1
        val held = text ?: return
        if (held.length > REACH) return

        var at = -1
        var forward = true
        if (caret < held.length && OPENERS.indexOf(held[caret]) >= 0) {
            at = caret
        } else if (caret > 0 && CLOSERS.indexOf(held[caret - 1]) >= 0) {
            at = caret - 1
            forward = false
        } else if (caret < held.length && CLOSERS.indexOf(held[caret]) >= 0) {
            at = caret
            forward = false
        } else if (caret > 0 && OPENERS.indexOf(held[caret - 1]) >= 0) {
            at = caret - 1
        }
        if (at < 0) return

        val mine = held[at]
        val other = if (forward) {
            CLOSERS[OPENERS.indexOf(mine)]
        } else {
            OPENERS[CLOSERS.indexOf(mine)]
        }
        var depth = 0
        var walk = at
        while (walk >= 0 && walk < held.length) {
            val one = held[walk]
            if (one == mine) {
                depth += 1
            } else if (one == other) {
                depth -= 1
                if (depth == 0) {
                    matchHere = at
                    matchThere = walk
                    return
                }
            }
            walk += if (forward) 1 else -1
        }
    }

    fun fault(line: Int) {
        faultLine = line
        if (line > 0) {
            showLine(line)
        }
        invalidate()
    }

    fun say(what: String) {
        said = what
        invalidate()
    }

    fun readAt(size: Float) {
        val wanted = size.coerceIn(SMALLEST_SP, LARGEST_SP)
        if (kotlin.math.abs(wanted - readingSize) < 0.05f) {
            return
        }
        readingSize = wanted
        setTextSize(TypedValue.COMPLEX_UNIT_SP, readingSize)
        painted = -1 to -1
        colour()
        invalidate()
        onResized?.invoke(readingSize)
    }

    fun readingAt(): Float = readingSize

    private var wrapping = true

    fun wrapping(): Boolean = wrapping

    fun wrapAt(on: Boolean) {
        wrapping = on
        setHorizontallyScrolling(!on)
        isSingleLine = false
        painted = -1 to -1
        colour()
        requestLayout()
        invalidate()
    }

    fun stripComments(): Int {
        val held = text ?: return 0
        if (grammar === Grammar.PLAIN) {
            return 0
        }
        val marks = Reader.read(held, grammar).filter { it.kind == Kind.COMMENT }
        if (marks.isEmpty()) {
            return 0
        }
        val whole = held.toString()
        val out = StringBuilder(whole.length)
        var at = 0
        for (one in marks.sortedBy { it.from }) {
            if (one.from < at) {
                continue
            }
            out.append(whole, at, one.from)
            at = one.to.coerceAtMost(whole.length)
        }
        out.append(whole, at.coerceAtMost(whole.length), whole.length)
        val kept = out.lines()
        val tidied = StringBuilder()
        for ((index, line) in kept.withIndex()) {
            val trimmed = line.trimEnd()
            if (trimmed.isBlank() && index < kept.size - 1 && kept[index + 1].isBlank()) {
                continue
            }
            tidied.append(trimmed)
            if (index < kept.size - 1) {
                tidied.append('\n')
            }
        }
        val removed = whole.length - tidied.length
        if (removed <= 0) {
            return 0
        }
        val caret = selectionStart.coerceIn(0, tidied.length)
        replaying = true
        setText(tidied.toString())
        replaying = false
        history.forget()
        setSelection(caret.coerceIn(0, text?.length ?: 0))
        onChanged?.invoke()
        read()
        return removed
    }

    var onResized: ((Float) -> Unit)? = null

    @Suppress("ClickableViewAccessibility")
    override fun onTouchEvent(event: android.view.MotionEvent): Boolean {
        pinch.onTouchEvent(event)
        if (pinch.isInProgress) {
            return true
        }
        return super.onTouchEvent(event)
    }

    fun replaceEvery(needle: String, with: String): Int {
        if (needle.isEmpty()) return 0
        val held = text ?: return 0
        val whole = held.toString()
        if (!whole.contains(needle)) return 0
        val count = whole.split(needle).size - 1
        val caret = selectionStart.coerceIn(0, whole.length)
        replaying = true
        setText(whole.replace(needle, with))
        replaying = false
        history.forget()
        setSelection(caret.coerceIn(0, text?.length ?: 0))
        onChanged?.invoke()
        read()
        return count
    }

    fun findFrom(needle: String, from: Int): Int {
        if (needle.isEmpty()) return -1
        val held = text?.toString() ?: return -1
        val ahead = held.indexOf(needle, from.coerceIn(0, held.length))
        return if (ahead >= 0) ahead else held.indexOf(needle)
    }

    fun findBefore(needle: String, from: Int): Int {
        if (needle.isEmpty()) return -1
        val held = text?.toString() ?: return -1
        val back = held.lastIndexOf(needle, (from - 1).coerceIn(-1, held.length))
        return if (back >= 0) back else held.lastIndexOf(needle)
    }

    fun show(from: Int, to: Int) {
        val held = text ?: return
        setSelection(from.coerceIn(0, held.length), to.coerceIn(0, held.length))
        val layout = layout ?: return
        val row = layout.getLineForOffset(from.coerceIn(0, held.length))
        scrollTo(0, (layout.getLineTop(row) - height / 3).coerceAtLeast(0))
        invalidate()
    }

    fun undo() = move { history.undo(it) }

    fun redo() = move { history.redo(it) }

    private fun move(what: (Editable) -> Int?) {
        val editable = text ?: return
        replaying = true
        val caret = what(editable)
        replaying = false
        if (caret != null) {
            setSelection(caret.coerceIn(0, editable.length))
            onChanged?.invoke()
            read()
        }
    }

    private fun read() {
        if (grammar === Grammar.PLAIN) {
            tokens = emptyList()
            return
        }
        if (reading) return
        reading = true
        generation += 1
        val mine = generation
        val snapshot = text?.toString().orEmpty()
        Thread {
            val found = runCatching { Reader.read(snapshot, grammar) }.getOrDefault(emptyList())
            post {
                reading = false
                if (mine == generation) {
                    tokens = found
                    painted = -1 to -1
                    colour()
                    invalidate()
                }
            }
        }.start()
    }

    override fun onScrollChanged(l: Int, t: Int, oldl: Int, oldt: Int) {
        super.onScrollChanged(l, t, oldl, oldt)
        colour()
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        colour()
    }

    private fun colour() {
        val layout = layout ?: return
        val editable = text ?: return
        if (tokens.isEmpty()) return

        val first = layout.getLineForVertical(scrollY)
        val last = layout.getLineForVertical(scrollY + height)
        val from = layout.getLineStart(max(0, first - MARGIN_LINES))
        val to = layout.getLineEnd(min(layout.lineCount - 1, last + MARGIN_LINES))
        if (from >= painted.first && to <= painted.second) return
        painted = from to to

        for (span in editable.getSpans(0, editable.length, ForegroundColorSpan::class.java)) {
            editable.removeSpan(span)
        }
        for (span in editable.getSpans(0, editable.length, StyleSpan::class.java)) {
            editable.removeSpan(span)
        }

        for (token in tokens) {
            if (token.to <= from) continue
            if (token.from >= to) break
            val start = max(token.from, 0)
            val end = min(token.to, editable.length)
            if (start >= end) continue
            editable.setSpan(
                ForegroundColorSpan(colourOf(token.kind)),
                start,
                end,
                Spannable.SPAN_EXCLUSIVE_EXCLUSIVE,
            )
            if (token.kind == Kind.KEYWORD || token.kind == Kind.TAG) {
                editable.setSpan(
                    StyleSpan(Typeface.BOLD),
                    start,
                    end,
                    Spannable.SPAN_EXCLUSIVE_EXCLUSIVE,
                )
            }
        }
    }

    private fun colourOf(kind: Kind): Int = when (kind) {
        Kind.COMMENT -> Ink.fade(palette.muted, 0.85f)
        Kind.TEXT -> palette.ok
        Kind.NUMBER -> palette.warning
        Kind.KEYWORD -> palette.accent
        Kind.TYPE -> Ink.hot(palette.accent, 0.45f)
        Kind.ANNOTATION -> palette.glowThird.let { Ink.hot(it, 0.55f) }
        Kind.TAG -> palette.accent
        Kind.ATTRIBUTE -> Ink.hot(palette.warning, 0.25f)
    }

    override fun onDraw(canvas: Canvas) {
        val layout = layout
        if (layout != null && paddingLeft > 0) {
            gutter.textSize = textSize * 0.78f
            val liftX = totalPaddingLeft.toFloat()
            val liftY = totalPaddingTop.toFloat()
            val caretLine = layout.getLineForOffset(selectionStart.coerceIn(0, text?.length ?: 0))
            val first = layout.getLineForVertical((scrollY - liftY).toInt())
            val last = layout.getLineForVertical((scrollY + height - liftY).toInt())
            val lowest = min(last, layout.lineCount - 1)
            val edge = (paddingLeft - gutter.textSize * 0.9f) + scrollX

            rule.color = Ink.fade(palette.accent, 0.06f)
            canvas.drawRect(
                scrollX.toFloat(),
                liftY + layout.getLineTop(caretLine),
                (scrollX + width).toFloat(),
                liftY + layout.getLineBottom(caretLine),
                rule,
            )

            canvas.save()
            canvas.translate(0f, liftY)
            drawGuides(canvas, layout, first, lowest)
            canvas.restore()

            val faulty = faultLine - 1
            if (faulty in first..lowest) {
                canvas.save()
                canvas.translate(liftX, liftY)
                drawFault(canvas, layout, faulty)
                canvas.restore()
            }

            for (line in first..lowest) {
                gutter.color = when (line) {
                    faulty -> palette.error
                    caretLine -> palette.accent
                    else -> Ink.fade(palette.muted, 0.55f)
                }
                canvas.drawText(
                    (line + 1).toString(),
                    edge,
                    liftY + layout.getLineBaseline(line),
                    gutter,
                )
            }

            if (matchHere >= 0 && matchThere >= 0) {
                canvas.save()
                canvas.translate(liftX, liftY)
                drawMatch(canvas, layout, matchHere)
                drawMatch(canvas, layout, matchThere)
                canvas.restore()
            }

            if (said.isNotEmpty()) {
                gutter.textAlign = Paint.Align.LEFT
                gutter.color = if (faultLine > 0) palette.error else palette.muted
                val tall = gutter.textSize * 1.9f
                rule.color = Ink.fade(palette.background, 0.92f)
                canvas.drawRect(
                    scrollX.toFloat(),
                    (scrollY + height).toFloat() - tall,
                    (scrollX + width).toFloat(),
                    (scrollY + height).toFloat(),
                    rule,
                )
                canvas.drawText(
                    said,
                    scrollX + gutter.textSize,
                    (scrollY + height).toFloat() - tall * 0.42f,
                    gutter,
                )
                gutter.textAlign = Paint.Align.RIGHT
            }

            rule.color = Ink.fade(palette.divider, 0.9f)
            val at = (paddingLeft - gutter.textSize * 0.35f) + scrollX
            canvas.drawLine(
                at,
                scrollY.toFloat(),
                at,
                (scrollY + height).toFloat(),
                rule,
            )
        }
        super.onDraw(canvas)
    }

    override fun onSelectionChanged(start: Int, end: Int) {
        super.onSelectionChanged(start, end)
        if (start == end) {
            matchAround(start)
        } else {
            matchHere = -1
            matchThere = -1
        }
        val (line, column) = caretAt()
        onCaret?.invoke(line, column)
        invalidate()
    }

    private fun drawGuides(
        canvas: Canvas,
        layout: android.text.Layout,
        first: Int,
        last: Int,
    ) {
        val held = text ?: return
        val step = paint.measureText("    ")
        if (step <= 0f) return
        guide.color = Ink.fade(palette.divider, 0.55f)
        guide.strokeWidth = 1f
        for (line in first..last) {
            val from = layout.getLineStart(line)
            val to = layout.getLineEnd(line)
            var spaces = 0
            var at = from
            while (at < to && held[at] == ' ') {
                spaces += 1
                at += 1
            }
            if (at >= to || held[at] == '\n') continue
            var depth = 1
            while (depth * 4 <= spaces) {
                val x = paddingLeft + step * depth
                canvas.drawLine(
                    x,
                    layout.getLineTop(line).toFloat(),
                    x,
                    layout.getLineBottom(line).toFloat(),
                    guide,
                )
                depth += 1
            }
        }
    }

    private fun drawFault(canvas: Canvas, layout: android.text.Layout, row: Int) {
        val from = layout.getLineLeft(row)
        val to = max(layout.getLineRight(row), from + textSize)
        val base = layout.getLineBaseline(row) + textSize * 0.18f
        val wave = textSize * 0.16f
        squiggle.rewind()
        squiggle.moveTo(from, base)
        var x = from
        var up = true
        while (x < to) {
            val next = min(x + wave * 2f, to)
            squiggle.quadTo(
                (x + next) / 2f,
                if (up) base - wave else base + wave,
                next,
                base,
            )
            up = !up
            x = next
        }
        rule.style = Paint.Style.STROKE
        rule.strokeWidth = max(1f, textSize * 0.05f)
        rule.color = palette.error
        canvas.drawPath(squiggle, rule)
        rule.style = Paint.Style.FILL
        rule.strokeWidth = 1f
    }

    private fun drawMatch(canvas: Canvas, layout: android.text.Layout, at: Int) {
        if (at < 0 || at >= (text?.length ?: 0)) return
        val row = layout.getLineForOffset(at)
        val from = layout.getPrimaryHorizontal(at)
        val to = layout.getPrimaryHorizontal(at + 1)
        rule.color = Ink.fade(palette.accent, 0.28f)
        canvas.drawRect(
            min(from, to),
            layout.getLineTop(row).toFloat(),
            max(from, to),
            layout.getLineBottom(row).toFloat(),
            rule,
        )
    }

    override fun onDetachedFromWindow() {
        removeCallbacks(readAgain)
        super.onDetachedFromWindow()
    }
}

internal object DeviceKey {

    const val ALIAS = "omni_builder_signing"

    const val STORE = "AndroidKeyStore"

    const val YEARS = 30

    private fun store(): KeyStore = KeyStore.getInstance(STORE).apply { load(null) }

    fun present(): Boolean = runCatching { store().containsAlias(ALIAS) }.getOrDefault(false)

    fun make(): Boolean = runCatching {
        val generator = KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_RSA, STORE)
        val from = Calendar.getInstance()
        val until = Calendar.getInstance().apply { add(Calendar.YEAR, YEARS) }
        val spec = KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_SIGN)
            .setDigests(KeyProperties.DIGEST_SHA256)
            .setSignaturePaddings(KeyProperties.SIGNATURE_PADDING_RSA_PKCS1)
            .setKeySize(4096)
            .setCertificateSubject(javax.security.auth.x500.X500Principal("CN=Omni Builder"))
            .setCertificateNotBefore(from.time)
            .setCertificateNotAfter(until.time)
            .setUserAuthenticationRequired(false)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            runCatching { spec.setIsStrongBoxBacked(true) }
        }
        generator.initialize(spec.build())
        generator.generateKeyPair()
        true
    }.getOrElse {
        runCatching {
            val generator = KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_RSA, STORE)
            val from = Calendar.getInstance()
            val until = Calendar.getInstance().apply { add(Calendar.YEAR, YEARS) }
            generator.initialize(
                KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_SIGN)
                    .setDigests(KeyProperties.DIGEST_SHA256)
                    .setSignaturePaddings(KeyProperties.SIGNATURE_PADDING_RSA_PKCS1)
                    .setKeySize(2048)
                    .setCertificateSubject(
                        javax.security.auth.x500.X500Principal("CN=Omni Builder")
                    )
                    .setCertificateNotBefore(from.time)
                    .setCertificateNotAfter(until.time)
                    .setUserAuthenticationRequired(false)
                    .build()
            )
            generator.generateKeyPair()
            true
        }.getOrDefault(false)
    }

    fun certificate(): ByteArray? = runCatching {
        store().getCertificate(ALIAS)?.encoded
    }.getOrNull()

    fun inHardware(): Boolean = runCatching {
        val entry = store().getEntry(ALIAS, null) as? KeyStore.PrivateKeyEntry ?: return false
        val factory = KeyFactory.getInstance(entry.privateKey.algorithm, STORE)
        val info = factory.getKeySpec(entry.privateKey, KeyInfo::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            info.securityLevel != KeyProperties.SECURITY_LEVEL_SOFTWARE
        } else {
            @Suppress("DEPRECATION")
            info.isInsideSecureHardware
        }
    }.getOrDefault(false)

    fun sign(what: ByteArray): ByteArray? = runCatching {
        val entry = store().getEntry(ALIAS, null) as? KeyStore.PrivateKeyEntry ?: return null
        Signature.getInstance("SHA256withRSA").run {
            initSign(entry.privateKey)
            update(what)
            sign()
        }
    }.getOrNull()

    fun forget(): Boolean = runCatching {
        store().deleteEntry(ALIAS)
        true
    }.getOrDefault(false)

    fun hex(bytes: ByteArray): String {
        val out = StringBuilder(bytes.size * 2)
        for (byte in bytes) {
            out.append("0123456789ABCDEF"[(byte.toInt() shr 4) and 0x0f])
            out.append("0123456789ABCDEF"[byte.toInt() and 0x0f])
        }
        return out.toString()
    }

    fun bytes(text: String): ByteArray? {
        if (text.length % 2 != 0) return null
        val out = ByteArray(text.length / 2)
        for (index in out.indices) {
            val held = text.substring(index * 2, index * 2 + 2).toIntOrNull(16) ?: return null
            out[index] = held.toByte()
        }
        return out
    }
}

internal object Brain {

    private const val ANTHROPIC_DATED = "2023-06-01"

    private const val OPEN_MILLIS = 20_000

    private const val READ_MILLIS = 180_000

    private const val REPLY_BYTES = 8 * 1024 * 1024

    const val MOST_TOKENS = 16_000

    const val MOST_STREAMED = 64_000

    const val REST_MILLIS = 120_000L

    const val FAULTS_BEFORE_REST = 3

    class Model(
        val id: String,
        val shown: String,
        val window: Long,
    )

    class Vendor(
        val id: String,
        val shown: String,
        val home: String,
        val anthropic: Boolean,
        val free: Boolean,
        val where: String,
        val models: List<Model>,
        val keyless: Boolean = false,
        val copilot: Boolean = false,
    ) {
        fun keyed(context: Context): Boolean =
            keyless || Preferences.vendorKey(context, id).isNotEmpty()
    }

    private val CLAUDE = listOf(
        Model("claude-opus-5", "Opus 5", 1_000_000L),
        Model("claude-sonnet-5", "Sonnet 5", 1_000_000L),
        Model("claude-fable-5-1", "Fable 5.1", 1_000_000L),
        Model("claude-haiku-4-5", "Haiku 4.5", 200_000L),
    )

    val VENDORS: List<Vendor> = listOf(
        Vendor(
            "pollinations", "Pollinations", "https://text.pollinations.ai/openai",
            false, true, "pollinations.ai", listOf(
                Model("openai", "GPT-OSS 20B", 128_000L),
                Model("openai-fast", "GPT-OSS 20B fast", 128_000L),
            ),
            keyless = true,
        ),
        Vendor(
            "anthropic", "Anthropic", "https://api.anthropic.com", true, false,
            "console.anthropic.com", CLAUDE,
        ),
        Vendor(
            "openai", "OpenAI", "https://api.openai.com/v1", false, false,
            "platform.openai.com", listOf(
                Model("gpt-5.2", "GPT-5.2", 400_000L),
                Model("gpt-5.2-mini", "GPT-5.2 mini", 400_000L),
                Model("gpt-5.1", "GPT-5.1", 400_000L),
                Model("o4-mini", "o4 mini", 200_000L),
            ),
        ),
        Vendor(
            "google", "Google AI Studio",
            "https://generativelanguage.googleapis.com/v1beta/openai", false, true,
            "aistudio.google.com", listOf(
                Model("gemini-3-pro", "Gemini 3 Pro", 1_000_000L),
                Model("gemini-3-flash", "Gemini 3 Flash", 1_000_000L),
                Model("gemini-2.5-flash", "Gemini 2.5 Flash", 1_000_000L),
            ),
        ),
        Vendor(
            "groq", "Groq", "https://api.groq.com/openai/v1", false, true,
            "console.groq.com", listOf(
                Model("llama-3.3-70b-versatile", "Llama 3.3 70B", 128_000L),
                Model("qwen3-32b", "Qwen3 32B", 128_000L),
                Model("openai/gpt-oss-120b", "GPT-OSS 120B", 128_000L),
            ),
        ),
        Vendor(
            "cerebras", "Cerebras", "https://api.cerebras.ai/v1", false, true,
            "cloud.cerebras.ai", listOf(
                Model("llama-3.3-70b", "Llama 3.3 70B", 128_000L),
                Model("qwen-3-235b-a22b-instruct", "Qwen3 235B", 128_000L),
            ),
        ),
        Vendor(
            "openrouter", "OpenRouter", "https://openrouter.ai/api/v1", false, true,
            "openrouter.ai", listOf(
                Model("deepseek/deepseek-r1:free", "DeepSeek R1 free", 164_000L),
                Model("qwen/qwen3-coder:free", "Qwen3 Coder free", 262_000L),
                Model("google/gemini-2.0-flash-exp:free", "Gemini 2.0 Flash free", 1_000_000L),
            ),
        ),
        Vendor(
            "opencode", "OpenCode Zen", "https://opencode.ai/zen/v1", false, true,
            "opencode.ai/zen", listOf(
                Model("big-pickle", "Big Pickle", 256_000L),
                Model("nemotron-3-ultra-free", "Nemotron 3 Ultra", 128_000L),
                Model("muse-spark-1.3-contributor-free", "Muse Spark 1.3", 128_000L),
                Model("mimo-v2.5-free", "MiMo v2.5", 128_000L),
            ),
        ),
        Vendor(
            "github", "GitHub Models", "https://models.github.ai/inference", false, true,
            "github.com/marketplace/models", listOf(
                Model("openai/gpt-4.1", "GPT-4.1", 1_000_000L),
                Model("meta/Llama-4-Scout-17B-16E-Instruct", "Llama 4 Scout", 128_000L),
            ),
        ),
        Vendor(
            "copilot", "GitHub Copilot", "https://api.githubcopilot.com", false, false,
            "github.com", listOf(
                Model("gpt-4.1", "GPT-4.1", 128_000L),
                Model("gpt-5-mini", "GPT-5 mini", 128_000L),
                Model("claude-sonnet-4.5", "Claude Sonnet 4.5", 200_000L),
                Model("gemini-2.5-pro", "Gemini 2.5 Pro", 1_000_000L),
            ),
            copilot = true,
        ),
        Vendor(
            "nvidia", "NVIDIA NIM", "https://integrate.api.nvidia.com/v1", false, true,
            "build.nvidia.com", listOf(
                Model("nvidia/llama-3.3-nemotron-super-49b-v1", "Nemotron Super 49B", 128_000L),
                Model("deepseek-ai/deepseek-r1", "DeepSeek R1", 164_000L),
            ),
        ),
        Vendor(
            "mistral", "Mistral", "https://api.mistral.ai/v1", false, true,
            "console.mistral.ai", listOf(
                Model("mistral-large-latest", "Mistral Large", 128_000L),
                Model("codestral-latest", "Codestral", 256_000L),
            ),
        ),
        Vendor(
            "deepseek", "DeepSeek", "https://api.deepseek.com/v1", false, false,
            "platform.deepseek.com", listOf(
                Model("deepseek-chat", "DeepSeek Chat", 128_000L),
                Model("deepseek-reasoner", "DeepSeek Reasoner", 128_000L),
            ),
        ),
        Vendor(
            "xai", "xAI", "https://api.x.ai/v1", false, false,
            "console.x.ai", listOf(
                Model("grok-4", "Grok 4", 256_000L),
                Model("grok-code-fast-1", "Grok Code Fast", 256_000L),
            ),
        ),
        Vendor(
            "together", "Together", "https://api.together.xyz/v1", false, false,
            "api.together.ai", listOf(
                Model("Qwen/Qwen3-235B-A22B-Instruct-2507-tput", "Qwen3 235B", 256_000L),
                Model("deepseek-ai/DeepSeek-V3", "DeepSeek V3", 128_000L),
            ),
        ),
        Vendor(
            "fireworks", "Fireworks", "https://api.fireworks.ai/inference/v1", false, false,
            "fireworks.ai", listOf(
                Model("accounts/fireworks/models/qwen3-coder-480b-a35b-instruct",
                    "Qwen3 Coder 480B", 256_000L),
                Model("accounts/fireworks/models/deepseek-v3", "DeepSeek V3", 128_000L),
            ),
        ),
    )

    fun vendorOf(id: String): Vendor = VENDORS.firstOrNull { it.id == id } ?: VENDORS.first()

    fun modelOf(vendor: Vendor, id: String): Model =
        vendor.models.firstOrNull { it.id == id }
            ?: Model(id.ifEmpty { vendor.models.first().id }, id.ifEmpty {
                vendor.models.first().shown
            }, vendor.models.first().window)

    class Turn(val mine: Boolean, val said: String)

    class Limits(
        val requestsLeft: Long,
        val tokensLeft: Long,
        val resetsAt: String,
        val retryAfter: Long,
    ) {
        fun known(): Boolean = requestsLeft >= 0 || tokensLeft >= 0
    }

    class Spend(
        val input: Long,
        val output: Long,
        val cacheWritten: Long,
        val cacheRead: Long,
    ) {
        fun window(): Long = input + cacheWritten + cacheRead
    }

    class Answer(
        val said: String,
        val stopped: String,
        val spend: Spend,
        val limits: Limits,
    )

    class Refused(val said: Int, val detail: String = "") : Exception()

    private fun messagesOf(turns: List<Turn>): JSONArray {
        val messages = JSONArray()
        turns.forEach { one ->
            messages.put(
                JSONObject()
                    .put("role", if (one.mine) "user" else "assistant")
                    .put("content", one.said)
            )
        }
        return messages
    }

    private fun anthropicBody(
        turns: List<Turn>,
        model: String,
        effort: String,
        system: String,
    ): JSONObject {
        val asked = JSONObject()
            .put("model", model)
            .put("max_tokens", MOST_TOKENS)
            .put("messages", messagesOf(turns))
            .put("thinking", JSONObject().put("type", "adaptive"))
        if (effort.isNotEmpty()) {
            asked.put("output_config", JSONObject().put("effort", effort))
        }
        if (system.isNotEmpty()) {
            asked.put(
                "system",
                JSONArray().put(
                    JSONObject()
                        .put("type", "text")
                        .put("text", system)
                        .put(
                            "cache_control",
                            JSONObject().put("type", "ephemeral"),
                        )
                )
            )
        }
        return asked
    }

    private fun openBody(
        turns: List<Turn>,
        model: String,
        effort: String,
        system: String,
    ): JSONObject {
        val messages = JSONArray()
        if (system.isNotEmpty()) {
            messages.put(JSONObject().put("role", "system").put("content", system))
        }
        val held = messagesOf(turns)
        for (at in 0 until held.length()) {
            messages.put(held.get(at))
        }
        val asked = JSONObject()
            .put("model", model)
            .put("max_tokens", MOST_TOKENS)
            .put("messages", messages)
        if (effort.isNotEmpty()) {
            asked.put("reasoning_effort", effort)
        }
        return asked
    }

    private fun bodyFor(
        vendor: Vendor,
        turns: List<Turn>,
        model: String,
        effort: String,
        system: String,
    ): JSONObject =
        if (vendor.anthropic) {
            anthropicBody(turns, model, effort, system)
        } else {
            openBody(turns, model, effort, system)
        }

    private fun refusalFor(code: Int): Int = when (code) {
        401, 403 -> R.string.omni_talk_bad_key
        404 -> R.string.omni_talk_no_model
        429 -> R.string.omni_talk_spent
        529 -> R.string.omni_talk_busy
        in 500..599 -> R.string.omni_talk_server
        else -> R.string.omni_talk_refused
    }

    private fun detailFor(code: Int, limits: Limits, detail: String): String =
        if (code == 429 && limits.retryAfter > 0) limits.retryAfter.toString() else detail

    private fun numbered(link: HttpURLConnection, named: String): Long =
        link.getHeaderField(named)?.trim()?.toLongOrNull() ?: -1L

    private fun limitsOf(link: HttpURLConnection): Limits {
        var requests = -1L
        var tokens = -1L
        var resets = ""
        link.headerFields?.forEach { (named, values) ->
            val key = named?.lowercase(Locale.US) ?: return@forEach
            if (!key.contains("ratelimit")) {
                return@forEach
            }
            val held = values.firstOrNull()?.trim().orEmpty()
            when {
                key.endsWith("requests-remaining") -> requests = held.toLongOrNull() ?: requests
                key.endsWith("tokens-remaining") -> tokens = held.toLongOrNull() ?: tokens
                key.endsWith("tokens-reset") && resets.isEmpty() -> resets = held
                key.endsWith("requests-reset") && resets.isEmpty() -> resets = held
            }
        }
        return Limits(requests, tokens, resets, numbered(link, "retry-after"))
    }

    private fun read(from: java.io.InputStream?): String {
        if (from == null) {
            return ""
        }
        return runCatching {
            from.use { held ->
                val room = ByteArrayOutputStream()
                val slab = ByteArray(32 * 1024)
                while (room.size() < REPLY_BYTES) {
                    val got = held.read(slab)
                    if (got < 0) {
                        break
                    }
                    room.write(slab, 0, got)
                }
                room.toString(Charsets.UTF_8.name())
            }
        }.getOrDefault("")
    }

    private fun spokenFrom(reply: JSONObject): String {
        val parts = reply.optJSONArray("content") ?: return ""
        val out = StringBuilder()
        for (at in 0 until parts.length()) {
            val one = parts.optJSONObject(at) ?: continue
            if (one.optString("type") != "text") {
                continue
            }
            if (out.isNotEmpty()) {
                out.append("\n\n")
            }
            out.append(one.optString("text"))
        }
        return out.toString()
    }

    private fun openSpokenFrom(reply: JSONObject): String {
        val first = reply.optJSONArray("choices")?.optJSONObject(0) ?: return ""
        val message = first.optJSONObject("message") ?: return ""
        val held = message.opt("content")
        if (held is JSONArray) {
            val out = StringBuilder()
            for (at in 0 until held.length()) {
                val one = held.optJSONObject(at) ?: continue
                val part = one.optString("text")
                if (part.isNotEmpty()) {
                    out.append(part)
                }
            }
            return out.toString()
        }
        return held?.toString().orEmpty()
    }

    private fun spendFrom(reply: JSONObject): Spend {
        val used = reply.optJSONObject("usage") ?: JSONObject()
        return Spend(
            used.optLong("input_tokens"),
            used.optLong("output_tokens"),
            used.optLong("cache_creation_input_tokens"),
            used.optLong("cache_read_input_tokens"),
        )
    }

    private fun openSpendFrom(reply: JSONObject): Spend {
        val used = reply.optJSONObject("usage") ?: JSONObject()
        val cached = used.optJSONObject("prompt_tokens_details")?.optLong("cached_tokens") ?: 0L
        return Spend(
            (used.optLong("prompt_tokens") - cached).coerceAtLeast(0L),
            used.optLong("completion_tokens"),
            0L,
            cached,
        )
    }

    private fun opened(vendor: Vendor, key: CharArray, path: String, listening: Boolean):
        HttpURLConnection {
        val link = runCatching { URL(vendor.home + path).openConnection() }
            .getOrNull() as? HttpURLConnection
            ?: throw Refused(R.string.omni_talk_unreachable)
        link.connectTimeout = OPEN_MILLIS
        link.readTimeout = READ_MILLIS
        link.setRequestProperty("content-type", "application/json")
        link.setRequestProperty(
            "accept",
            if (listening) "text/event-stream" else "application/json",
        )
        link.setRequestProperty("User-Agent", "omni-builder")
        when {
            vendor.anthropic -> {
                link.setRequestProperty("x-api-key", String(key))
                link.setRequestProperty("anthropic-version", ANTHROPIC_DATED)
            }
            vendor.copilot -> {
                link.setRequestProperty(
                    "authorization",
                    "Bearer " + Copilot.ticketFor(key),
                )
                link.setRequestProperty("editor-version", Copilot.EDITOR)
                link.setRequestProperty("editor-plugin-version", Copilot.PLUGIN)
                link.setRequestProperty("copilot-integration-id", Copilot.INTEGRATION)
            }
            key.isNotEmpty() -> {
                link.setRequestProperty("authorization", "Bearer " + String(key))
            }
        }
        return link
    }

    private fun sent(link: HttpURLConnection, body: String) {
        runCatching {
            link.outputStream.use { out -> out.write(body.toByteArray(Charsets.UTF_8)) }
        }.getOrElse {
            link.disconnect()
            throw Refused(R.string.omni_talk_unreachable)
        }
    }

    private fun answered(link: HttpURLConnection): Limits {
        val code = runCatching { link.responseCode }.getOrElse {
            link.disconnect()
            throw Refused(R.string.omni_talk_unreachable)
        }
        val limits = limitsOf(link)
        if (code != 200) {
            val trouble = read(link.errorStream)
            link.disconnect()
            val detail = runCatching {
                val held = JSONObject(trouble)
                held.optJSONObject("error")?.optString("message").orEmpty()
                    .ifEmpty { held.optString("message") }
            }.getOrDefault("")
            throw Refused(refusalFor(code), detailFor(code, limits, detail))
        }
        return limits
    }

    fun ask(
        vendor: Vendor,
        key: CharArray,
        turns: List<Turn>,
        model: String,
        effort: String,
        system: String,
    ): Answer {
        if (key.isEmpty() && !vendor.keyless) {
            throw Refused(R.string.omni_talk_no_key)
        }
        val link = opened(vendor, key, if (vendor.anthropic) "/v1/messages" else "/chat/completions", false)
        link.requestMethod = "POST"
        link.doOutput = true
        sent(link, bodyFor(vendor, turns, model, effort, system).toString())
        val limits = answered(link)
        val said = read(link.inputStream)
        link.disconnect()
        val reply = runCatching { JSONObject(said) }
            .getOrElse { throw Refused(R.string.omni_talk_refused) }
        if (vendor.anthropic) {
            val stopped = reply.optString("stop_reason")
            if (stopped == "refusal") {
                throw Refused(R.string.omni_talk_declined)
            }
            return Answer(spokenFrom(reply), stopped, spendFrom(reply), limits)
        }
        val stopped = reply.optJSONArray("choices")
            ?.optJSONObject(0)
            ?.optString("finish_reason")
            .orEmpty()
        return Answer(
            openSpokenFrom(reply),
            if (stopped == "length") "max_tokens" else stopped,
            openSpendFrom(reply),
            limits,
        )
    }

    fun listed(vendor: Vendor, key: CharArray): List<String> {
        if ((key.isEmpty() && !vendor.keyless) || vendor.anthropic) {
            return emptyList()
        }
        val link = runCatching { opened(vendor, key, "/models", false) }.getOrNull()
            ?: return emptyList()
        link.requestMethod = "GET"
        return runCatching {
            if (link.responseCode != 200) {
                emptyList()
            } else {
                val held = JSONObject(read(link.inputStream)).optJSONArray("data")
                val out = mutableListOf<String>()
                for (at in 0 until (held?.length() ?: 0)) {
                    val named = held?.optJSONObject(at)?.optString("id").orEmpty()
                    if (named.isNotEmpty()) {
                        out.add(named)
                    }
                }
                out.sorted()
            }
        }.getOrDefault(emptyList()).also { link.disconnect() }
    }

    private fun anthropicStream(
        event: JSONObject,
        whole: StringBuilder,
        onText: (String) -> Unit,
        held: LongArray,
    ): String {
        when (event.optString("type")) {
            "message_start" -> {
                val used = event.optJSONObject("message")?.optJSONObject("usage")
                held[0] = used?.optLong("input_tokens") ?: 0L
                held[2] = used?.optLong("cache_creation_input_tokens") ?: 0L
                held[3] = used?.optLong("cache_read_input_tokens") ?: 0L
            }
            "content_block_delta" -> {
                val delta = event.optJSONObject("delta") ?: return ""
                if (delta.optString("type") != "text_delta") {
                    return ""
                }
                val part = delta.optString("text")
                if (part.isNotEmpty()) {
                    whole.append(part)
                    onText(part)
                }
            }
            "message_delta" -> {
                held[1] = event.optJSONObject("usage")?.optLong("output_tokens") ?: held[1]
                return event.optJSONObject("delta")?.optString("stop_reason").orEmpty()
            }
            "error" -> {
                val why = event.optJSONObject("error")?.optString("message").orEmpty()
                throw Refused(R.string.omni_talk_server, why)
            }
        }
        return ""
    }

    private fun openStream(
        event: JSONObject,
        whole: StringBuilder,
        onText: (String) -> Unit,
        held: LongArray,
    ): String {
        event.optJSONObject("error")?.let { why ->
            throw Refused(R.string.omni_talk_server, why.optString("message"))
        }
        event.optJSONObject("usage")?.let { used ->
            val cached = used.optJSONObject("prompt_tokens_details")?.optLong("cached_tokens") ?: 0L
            held[0] = (used.optLong("prompt_tokens") - cached).coerceAtLeast(0L)
            held[1] = used.optLong("completion_tokens")
            held[3] = cached
        }
        val first = event.optJSONArray("choices")?.optJSONObject(0) ?: return ""
        val part = first.optJSONObject("delta")?.optString("content").orEmpty()
        if (part.isNotEmpty()) {
            whole.append(part)
            onText(part)
        }
        val stopped = first.optString("finish_reason")
        return if (stopped == "length") "max_tokens" else stopped
    }

    fun stream(
        vendor: Vendor,
        key: CharArray,
        turns: List<Turn>,
        model: String,
        effort: String,
        system: String,
        stopping: java.util.concurrent.atomic.AtomicBoolean,
        onText: (String) -> Unit,
    ): Answer {
        if (key.isEmpty() && !vendor.keyless) {
            throw Refused(R.string.omni_talk_no_key)
        }
        val link = opened(vendor, key, if (vendor.anthropic) "/v1/messages" else "/chat/completions", true)
        link.requestMethod = "POST"
        link.doOutput = true
        val asked = bodyFor(vendor, turns, model, effort, system)
        asked.put("max_tokens", MOST_STREAMED)
        asked.put("stream", true)
        if (!vendor.anthropic) {
            asked.put("stream_options", JSONObject().put("include_usage", true))
        }
        sent(link, asked.toString())
        val limits = answered(link)

        val whole = StringBuilder()
        val held = longArrayOf(0L, 0L, 0L, 0L)
        var stopped = ""
        runCatching {
            link.inputStream.bufferedReader(Charsets.UTF_8).use { from ->
                while (true) {
                    if (stopping.get()) {
                        stopped = "stopped"
                        break
                    }
                    val line = from.readLine() ?: break
                    if (!line.startsWith("data:")) {
                        continue
                    }
                    val part = line.removePrefix("data:").trim()
                    if (part.isEmpty() || part == "[DONE]") {
                        continue
                    }
                    val event = runCatching { JSONObject(part) }.getOrNull() ?: continue
                    val said = if (vendor.anthropic) {
                        anthropicStream(event, whole, onText, held)
                    } else {
                        openStream(event, whole, onText, held)
                    }
                    if (said.isNotEmpty()) {
                        stopped = said
                    }
                }
            }
        }.getOrElse { why ->
            link.disconnect()
            throw (why as? Refused ?: Refused(R.string.omni_talk_unreachable))
        }
        link.disconnect()
        if (stopped == "refusal") {
            throw Refused(R.string.omni_talk_declined)
        }
        return Answer(
            whole.toString(),
            stopped,
            Spend(held[0], held[1], held[2], held[3]),
            limits,
        )
    }

    fun roughly(turns: List<Turn>, system: String): Long {
        val letters = turns.sumOf { it.said.length.toLong() } + system.length
        return letters / 4L
    }
}

internal object Bell {

    const val CHANNEL = "omni_builder"

    const val BUILD_NOTE = 1

    const val PAIRING_NOTE = 2

    fun channelHere(context: Context) {
        val manager = context.getSystemService(NotificationManager::class.java) ?: return
        if (manager.getNotificationChannel(CHANNEL) != null) {
            return
        }
        runCatching {
            manager.createNotificationChannel(
                NotificationChannel(
                    CHANNEL,
                    context.getString(R.string.omni_app_name),
                    NotificationManager.IMPORTANCE_DEFAULT,
                )
            )
        }
    }

    fun allowed(context: Context): Boolean =
        Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU ||
            context.checkSelfPermission(android.Manifest.permission.POST_NOTIFICATIONS) ==
            PackageManager.PERMISSION_GRANTED

    fun say(context: Context, id: Int, title: String, body: String) {
        if (!allowed(context)) {
            return
        }
        channelHere(context)
        val opening = PendingIntent.getActivity(
            context,
            id,
            Intent(context, BuilderActivity::class.java)
                .addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        val note = Notification.Builder(context, CHANNEL)
            .setSmallIcon(R.mipmap.ic_launcher)
            .setContentTitle(title)
            .setContentText(body)
            .setStyle(Notification.BigTextStyle().bigText(body))
            .setAutoCancel(true)
            .setContentIntent(opening)
            .build()
        runCatching {
            context.getSystemService(NotificationManager::class.java)?.notify(id, note)
        }
    }

    fun hush(context: Context, id: Int) {
        runCatching {
            context.getSystemService(NotificationManager::class.java)?.cancel(id)
        }
    }
}

internal object Vault {

    private const val ALIAS = "omni_builder_secrets"

    private const val STORE = "AndroidKeyStore"

    private const val SHAPE = "AES/GCM/NoPadding"

    private const val TAG_BITS = 128

    private const val NONCE_BYTES = 12

    private fun store(): KeyStore = KeyStore.getInstance(STORE).apply { load(null) }

    private fun key(): SecretKey? = runCatching {
        val held = store()
        (held.getEntry(ALIAS, null) as? KeyStore.SecretKeyEntry)?.secretKey ?: make()
    }.getOrNull()

    private fun make(): SecretKey? = runCatching {
        val maker = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, STORE)
        val spec = KeyGenParameterSpec.Builder(
            ALIAS,
            KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
        )
            .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
            .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
            .setKeySize(256)
            .setUserAuthenticationRequired(false)
        maker.init(spec.build())
        maker.generateKey()
    }.getOrNull()

    fun inHardware(): Boolean = runCatching {
        val held = key() ?: return false
        val facts = KeyFactory.getInstance(held.algorithm, STORE)
        val about = facts.getKeySpec(held, KeyInfo::class.java) as KeyInfo
        about.securityLevel != KeyProperties.SECURITY_LEVEL_SOFTWARE
    }.getOrDefault(false)

    fun seal(said: CharArray): String? = runCatching {
        val held = key() ?: return null
        val cipher = Cipher.getInstance(SHAPE)
        cipher.init(Cipher.ENCRYPT_MODE, held)
        val plain = ByteArray(said.size) { said[it].code.toByte() }
        val sealed = cipher.doFinal(plain)
        plain.fill(0)
        val out = ByteArray(cipher.iv.size + sealed.size)
        cipher.iv.copyInto(out)
        sealed.copyInto(out, cipher.iv.size)
        Base64.encodeToString(out, Base64.NO_WRAP)
    }.getOrNull()

    fun open(sealed: String): CharArray? = runCatching {
        if (sealed.isEmpty()) {
            return null
        }
        val held = key() ?: return null
        val raw = Base64.decode(sealed, Base64.NO_WRAP)
        if (raw.size <= NONCE_BYTES) {
            return null
        }
        val cipher = Cipher.getInstance(SHAPE)
        cipher.init(
            Cipher.DECRYPT_MODE,
            held,
            GCMParameterSpec(TAG_BITS, raw, 0, NONCE_BYTES),
        )
        val plain = cipher.doFinal(raw, NONCE_BYTES, raw.size - NONCE_BYTES)
        val out = CharArray(plain.size) { plain[it].toInt().toChar() }
        plain.fill(0)
        out
    }.getOrNull()

    fun forget() {
        runCatching { store().deleteEntry(ALIAS) }
    }
}

internal object Remote {

    private const val SERVICE = "git-upload-pack"

    private const val AGENT = "omni-builder"

    private const val OPEN_MILLIS = 20_000

    private const val READ_MILLIS = 60_000

    private const val ROOM = 64 * 1024

    private const val NOTHING = '\u0000'

    const val MOST_BYTES = 512L * 1024L * 1024L

    class Refused(val said: Int, val detail: String = "") : Exception()

    class Found(
        val head: String,
        val branch: String,
        val shallow: Boolean,
    )

    fun address(said: String): String {
        var text = said.trim()
        if (text.isEmpty()) {
            throw Refused(R.string.omni_clone_address_bad)
        }
        if (text.startsWith("git@")) {
            text = "https://" + text.removePrefix("git@").replaceFirst(':', '/')
        }
        if (text.startsWith("http://")) {
            throw Refused(R.string.omni_clone_needs_https)
        }
        if (!text.startsWith("https://")) {
            text = "https://$text"
        }
        text = text.trimEnd('/')
        if (!text.endsWith(".git")) {
            text = "$text.git"
        }
        val opened = runCatching { URL(text) }.getOrNull()
            ?: throw Refused(R.string.omni_clone_address_bad)
        if (opened.host.isNullOrEmpty() || opened.path.trim('/').count { it == '/' } < 1) {
            throw Refused(R.string.omni_clone_address_bad)
        }
        return text
    }

    fun named(address: String): String =
        address.trimEnd('/').substringAfterLast('/').removeSuffix(".git")

    private fun reach(
        where: String,
        token: String,
        sending: String?,
    ): HttpURLConnection {
        val link = runCatching { URL(where).openConnection() }.getOrNull() as? HttpURLConnection
            ?: throw Refused(R.string.omni_clone_address_bad)
        link.connectTimeout = OPEN_MILLIS
        link.readTimeout = READ_MILLIS
        link.instanceFollowRedirects = true
        link.setRequestProperty("User-Agent", AGENT)
        link.setRequestProperty("Accept-Encoding", "identity")
        if (token.isNotEmpty()) {
            val pair = "x-access-token:$token".toByteArray(Charsets.UTF_8)
            val coded = Base64.encodeToString(pair, Base64.NO_WRAP)
            link.setRequestProperty("Authorization", "Basic $coded")
        }
        if (sending != null) {
            link.requestMethod = "POST"
            link.doOutput = true
            link.setRequestProperty("Content-Type", sending)
        }
        return link
    }

    private fun answered(link: HttpURLConnection) {
        val code = runCatching { link.responseCode }.getOrElse {
            throw Refused(R.string.omni_clone_unreachable)
        }
        when (code) {
            200 -> return
            401, 403 -> throw Refused(R.string.omni_clone_needs_token)
            404 -> throw Refused(R.string.omni_clone_not_found)
            else -> throw Refused(R.string.omni_clone_server_refused, code.toString())
        }
    }

    private fun packet(said: String): ByteArray {
        val body = said.toByteArray(Charsets.UTF_8)
        val size = body.size + 4
        return String.format(Locale.US, "%04x", size).toByteArray(Charsets.US_ASCII) + body
    }

    private fun fill(from: InputStream, into: ByteArray): Boolean {
        var at = 0
        while (at < into.size) {
            val got = from.read(into, at, into.size - at)
            if (got < 0) {
                return false
            }
            at += got
        }
        return true
    }

    private fun packetLength(from: InputStream): Int {
        val head = ByteArray(4)
        if (!fill(from, head)) {
            return -1
        }
        val said = String(head, Charsets.US_ASCII)
        return said.toIntOrNull(16) ?: throw Refused(R.string.omni_clone_not_git)
    }

    private fun packetBody(from: InputStream, size: Int): ByteArray? {
        if (size < 4) {
            throw Refused(R.string.omni_clone_not_git)
        }
        val room = ByteArray(size - 4)
        return if (fill(from, room)) room else null
    }

    fun advertise(address: String, token: String): Found {
        val link = reach("$address/info/refs?service=$SERVICE", token, null)
        answered(link)
        val body = runCatching { link.inputStream.use { it.readBytes() } }.getOrElse {
            throw Refused(R.string.omni_clone_unreachable)
        }
        link.disconnect()

        val from = ByteArrayInputStream(body)
        val names = ArrayList<Pair<String, String>>()
        var capable = ""
        while (true) {
            val size = packetLength(from)
            if (size < 0) {
                break
            }
            if (size == 0) {
                continue
            }
            val room = packetBody(from, size) ?: break
            val said = String(room, Charsets.UTF_8)
            if (said.startsWith("#")) {
                continue
            }
            val split = said.indexOf(NOTHING)
            val plain = if (split < 0) said else said.substring(0, split)
            if (split >= 0 && capable.isEmpty()) {
                capable = said.substring(split + 1)
            }
            val parts = plain.trim().split(' ', limit = 2)
            if (parts.size == 2 && parts[0].length == 40) {
                names.add(parts[0] to parts[1])
            }
        }
        if (names.isEmpty()) {
            throw Refused(R.string.omni_clone_empty)
        }

        val pointed = capable.split(' ')
            .firstOrNull { it.startsWith("symref=HEAD:") }
            ?.removePrefix("symref=HEAD:")
            .orEmpty()
        val head = names.firstOrNull { it.second == "HEAD" }?.first
            ?: names.firstOrNull { it.second == pointed }?.first
            ?: names.firstOrNull { it.second.startsWith("refs/heads/") }?.first
            ?: throw Refused(R.string.omni_clone_empty)
        val branch = pointed.ifEmpty {
            names.firstOrNull { it.first == head && it.second.startsWith("refs/heads/") }
                ?.second.orEmpty()
        }
        return Found(head, branch.removePrefix("refs/heads/"), capable.contains("shallow"))
    }

    fun fetch(
        address: String,
        token: String,
        want: String,
        shallow: Boolean,
        into: File,
        watching: (Long) -> Boolean,
    ): Long {
        val asked = ByteArrayOutputStream()
        val first = StringBuilder("want ").append(want).append(" ofs-delta")
        if (shallow) {
            first.append(" shallow")
        }
        first.append('\n')
        asked.write(packet(first.toString()))
        if (shallow) {
            asked.write(packet("deepen 1\n"))
        }
        asked.write("0000".toByteArray(Charsets.US_ASCII))
        asked.write(packet("done\n"))

        val link = reach("$address/$SERVICE", token, "application/x-git-upload-pack-request")
        runCatching { link.outputStream.use { it.write(asked.toByteArray()) } }.getOrElse {
            throw Refused(R.string.omni_clone_unreachable)
        }
        answered(link)

        val from = runCatching { BufferedInputStream(link.inputStream, ROOM) }.getOrElse {
            throw Refused(R.string.omni_clone_unreachable)
        }
        var carried = 0L
        try {
            while (true) {
                val size = packetLength(from)
                if (size < 0) {
                    throw Refused(R.string.omni_clone_empty)
                }
                if (size == 0) {
                    continue
                }
                val room = packetBody(from, size) ?: throw Refused(R.string.omni_clone_empty)
                val said = String(room, Charsets.UTF_8)
                if (said.startsWith("NAK") || said.startsWith("ACK")) {
                    break
                }
                if (said.startsWith("ERR")) {
                    throw Refused(
                        R.string.omni_clone_server_refused,
                        said.removePrefix("ERR").trim(),
                    )
                }
            }
            FileOutputStream(into).use { out ->
                val room = ByteArray(ROOM)
                while (true) {
                    val got = from.read(room)
                    if (got < 0) {
                        break
                    }
                    carried += got
                    if (carried > MOST_BYTES) {
                        throw Refused(R.string.omni_clone_too_large)
                    }
                    out.write(room, 0, got)
                    if (!watching(carried)) {
                        throw Refused(R.string.omni_clone_stopped)
                    }
                }
            }
        } finally {
            runCatching { from.close() }
            link.disconnect()
        }
        if (carried < 32) {
            throw Refused(R.string.omni_clone_empty)
        }
        return carried
    }
}

internal object Forge {

    private const val HOST = "https://api.github.com"

    private const val KIND = "application/vnd.github+json"

    private const val DATED = "2022-11-28"

    private const val MOST = 40

    private const val REPLY_BYTES = 4 * 1024 * 1024

    class Repository(
        val full: String,
        val about: String,
        val stars: Int,
        val language: String,
        val address: String,
        val shut: Boolean,
    )

    private fun ask(where: String, token: String): String {
        val link = runCatching { URL(where).openConnection() }.getOrNull() as? HttpURLConnection
            ?: throw Remote.Refused(R.string.omni_repos_refused)
        link.connectTimeout = 20_000
        link.readTimeout = 30_000
        link.setRequestProperty("Accept", KIND)
        link.setRequestProperty("X-GitHub-Api-Version", DATED)
        link.setRequestProperty("User-Agent", "omni-builder")
        if (token.isNotEmpty()) {
            link.setRequestProperty("Authorization", "Bearer $token")
        }
        val code = runCatching { link.responseCode }.getOrElse {
            throw Remote.Refused(R.string.omni_clone_unreachable)
        }
        if (code == 401 || code == 403) {
            link.disconnect()
            throw Remote.Refused(R.string.omni_repos_needs_token)
        }
        if (code != 200) {
            link.disconnect()
            throw Remote.Refused(R.string.omni_repos_refused, code.toString())
        }
        val said = runCatching {
            link.inputStream.use { from ->
                val room = ByteArrayOutputStream()
                val slab = ByteArray(32 * 1024)
                while (room.size() < REPLY_BYTES) {
                    val got = from.read(slab)
                    if (got < 0) {
                        break
                    }
                    room.write(slab, 0, got)
                }
                room.toString(Charsets.UTF_8.name())
            }
        }.getOrElse { throw Remote.Refused(R.string.omni_clone_unreachable) }
        link.disconnect()
        return said
    }

    private fun said(from: JSONObject, key: String): String =
        if (from.isNull(key)) "" else from.optString(key)

    private fun readOne(from: JSONObject): Repository? {
        val full = said(from, "full_name")
        val address = said(from, "clone_url")
        if (full.isEmpty() || !address.startsWith("https://")) {
            return null
        }
        return Repository(
            full = full,
            about = said(from, "description"),
            stars = from.optInt("stargazers_count"),
            language = said(from, "language"),
            address = address,
            shut = from.optBoolean("private"),
        )
    }

    private fun readMany(said: JSONArray): List<Repository> {
        val found = ArrayList<Repository>(said.length())
        for (index in 0 until said.length()) {
            val one = said.optJSONObject(index) ?: continue
            readOne(one)?.let { found.add(it) }
        }
        return found
    }

    fun search(said: String, token: String): List<Repository> {
        val asked = said.trim()
        if (asked.isEmpty()) {
            return emptyList()
        }
        val quoted = URLEncoder.encode(asked, Charsets.UTF_8.name())
        val body = ask("$HOST/search/repositories?q=$quoted&per_page=$MOST", token)
        val held = runCatching { JSONObject(body) }.getOrElse {
            throw Remote.Refused(R.string.omni_repos_refused)
        }
        return readMany(held.optJSONArray("items") ?: JSONArray())
    }

    fun mine(token: String): List<Repository> {
        if (token.isEmpty()) {
            throw Remote.Refused(R.string.omni_repos_needs_token)
        }
        val body = ask("$HOST/user/repos?per_page=$MOST&sort=updated&affiliation=owner", token)
        val held = runCatching { JSONArray(body) }.getOrElse {
            throw Remote.Refused(R.string.omni_repos_refused)
        }
        return readMany(held)
    }
}

internal object Copilot {

    private const val CLIENT = "Iv1.b507a08c87ecfe98"

    private const val SCOPE = "read:user"

    private const val ASKING = "https://github.com/login/device/code"

    private const val GRANTING = "https://github.com/login/oauth/access_token"

    private const val TICKETING = "https://api.github.com/copilot_internal/v2/token"

    private const val GRANT = "urn:ietf:params:oauth:grant-type:device_code"

    const val EDITOR = "vscode/1.99.0"

    const val PLUGIN = "copilot-chat/0.26.0"

    const val INTEGRATION = "vscode-chat"

    private const val OPEN_MILLIS = 20_000

    private const val READ_MILLIS = 30_000

    private const val REPLY_BYTES = 256 * 1024

    private const val SOONER = 60L

    class Coming(
        val code: String,
        val shown: String,
        val where: String,
        val every: Long,
        val until: Long,
    )

    class Waiting : Exception()

    private var ticket = ""

    private var until = 0L

    private fun read(link: HttpURLConnection): String {
        val code = runCatching { link.responseCode }.getOrElse {
            link.disconnect()
            throw Brain.Refused(R.string.omni_talk_unreachable)
        }
        val stream = if (code in 200..299) link.inputStream else link.errorStream
        val said = runCatching {
            stream?.bufferedReader(Charsets.UTF_8)?.use { it.readText().take(REPLY_BYTES) }
                .orEmpty()
        }.getOrDefault("")
        link.disconnect()
        if (said.isEmpty()) {
            throw Brain.Refused(R.string.omni_copilot_refused)
        }
        return said
    }

    private fun posted(where: String, form: String): JSONObject {
        val link = runCatching { URL(where).openConnection() }.getOrNull() as? HttpURLConnection
            ?: throw Brain.Refused(R.string.omni_talk_unreachable)
        link.connectTimeout = OPEN_MILLIS
        link.readTimeout = READ_MILLIS
        link.requestMethod = "POST"
        link.doOutput = true
        link.setRequestProperty("accept", "application/json")
        link.setRequestProperty("content-type", "application/x-www-form-urlencoded")
        link.setRequestProperty("user-agent", "omni-builder")
        runCatching {
            link.outputStream.use { out -> out.write(form.toByteArray(Charsets.UTF_8)) }
        }.getOrElse {
            link.disconnect()
            throw Brain.Refused(R.string.omni_talk_unreachable)
        }
        return runCatching { JSONObject(read(link)) }.getOrElse {
            throw Brain.Refused(R.string.omni_copilot_refused)
        }
    }

    fun begin(): Coming {
        val said = posted(
            ASKING,
            "client_id=" + CLIENT + "&scope=" + URLEncoder.encode(SCOPE, "UTF-8"),
        )
        val code = said.optString("device_code")
        val shown = said.optString("user_code")
        if (code.isEmpty() || shown.isEmpty()) {
            throw Brain.Refused(R.string.omni_copilot_refused, said.optString("error"))
        }
        val every = said.optLong("interval", 5L).coerceIn(1L, 30L)
        val lasts = said.optLong("expires_in", 900L).coerceIn(60L, 3600L)
        return Coming(
            code,
            shown,
            said.optString("verification_uri").ifEmpty { "https://github.com/login/device" },
            every,
            System.currentTimeMillis() + lasts * 1000L,
        )
    }

    fun granted(coming: Coming): String {
        val said = posted(
            GRANTING,
            "client_id=" + CLIENT +
                "&device_code=" + URLEncoder.encode(coming.code, "UTF-8") +
                "&grant_type=" + URLEncoder.encode(GRANT, "UTF-8"),
        )
        val token = said.optString("access_token")
        if (token.isNotEmpty()) {
            return token
        }
        when (said.optString("error")) {
            "authorization_pending", "slow_down" -> throw Waiting()
            "access_denied" -> throw Brain.Refused(R.string.omni_copilot_denied)
            else -> throw Brain.Refused(
                R.string.omni_copilot_refused,
                said.optString("error_description").ifEmpty { said.optString("error") },
            )
        }
    }

    fun forget() {
        synchronized(this) {
            ticket = ""
            until = 0L
        }
    }

    fun ticketFor(key: CharArray): String {
        val now = System.currentTimeMillis() / 1000L
        synchronized(this) {
            if (ticket.isNotEmpty() && until - SOONER > now) {
                return ticket
            }
        }
        if (key.isEmpty()) {
            throw Brain.Refused(R.string.omni_talk_no_key)
        }
        val link = runCatching { URL(TICKETING).openConnection() }.getOrNull()
            as? HttpURLConnection
            ?: throw Brain.Refused(R.string.omni_talk_unreachable)
        link.connectTimeout = OPEN_MILLIS
        link.readTimeout = READ_MILLIS
        link.setRequestProperty("accept", "application/json")
        link.setRequestProperty("authorization", "token " + String(key))
        link.setRequestProperty("editor-version", EDITOR)
        link.setRequestProperty("editor-plugin-version", PLUGIN)
        link.setRequestProperty("user-agent", "omni-builder")
        val said = runCatching { JSONObject(read(link)) }.getOrElse {
            throw Brain.Refused(R.string.omni_copilot_no_seat)
        }
        val given = said.optString("token")
        if (given.isEmpty()) {
            throw Brain.Refused(R.string.omni_copilot_no_seat, said.optString("message"))
        }
        synchronized(this) {
            ticket = given
            until = said.optLong("expires_at", now + 600L)
        }
        return given
    }
}

internal object Drafts {

    private const val FOLDER = "Drafts"

    const val REST_MILLIS = 1_500L

    private fun folder(context: Context): File =
        File(context.filesDir, FOLDER).also { it.mkdirs() }

    private fun named(root: String, path: String): String {
        var held = 0xcbf2_9ce4_8422_2325uL
        for (character in "$root $path") {
            held = held xor character.code.toULong()
            held *= 0x100_0000_01b3uL
        }
        val stem = path.substringAfterLast('/').filter { it.isLetterOrDigit() || it == '.' }
        return "${stem.takeLast(40)}.${held.toString(16)}.draft"
    }

    fun read(context: Context, root: String, path: String): String? {
        val file = File(folder(context), named(root, path))
        if (!file.isFile) return null
        return runCatching { file.readText() }.getOrNull()
    }

    fun write(context: Context, root: String, path: String, text: String) {
        val here = folder(context)
        val target = File(here, named(root, path))
        val beside = File(here, "${target.name}.part")
        runCatching {
            FileOutputStream(beside).use { out ->
                out.write(text.toByteArray(Charsets.UTF_8))
                out.flush()
                out.fd.sync()
            }
            if (!beside.renameTo(target)) {
                target.delete()
                if (!beside.renameTo(target)) {
                    throw IOException("the draft could not be put in place")
                }
            }
        }.onFailure { beside.delete() }
    }

    fun forget(context: Context, root: String, path: String) {
        runCatching { File(folder(context), named(root, path)).delete() }
    }
}
