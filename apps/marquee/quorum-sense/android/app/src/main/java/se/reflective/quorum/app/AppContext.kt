package se.reflective.quorum.app

import android.content.Context

/** Package version label for queue persistence metadata. */
internal fun Context.clientVersion(): String? =
    runCatching {
        @Suppress("DEPRECATION")
        packageManager.getPackageInfo(packageName, 0).versionName
    }.getOrNull()
