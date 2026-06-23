package se.reflective.quorum.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

// Light "paper" scheme matching the Quorum Sense web app. No dark variant yet —
// the brand is light-first; add a darkColorScheme here when needed.
private val QuorumLightColors = lightColorScheme(
    primary = Accent,
    onPrimary = Color.White,
    primaryContainer = SurfaceMuted,
    onPrimaryContainer = AccentDark,
    secondary = Blue,
    onSecondary = Color.White,
    tertiary = Gold,
    background = Paper,
    onBackground = Ink,
    surface = Surface,
    onSurface = Ink,
    surfaceVariant = SurfaceMuted,
    onSurfaceVariant = InkSoft,
    outline = InkMuted,
    outlineVariant = AccentSoft,
    error = Danger,
    onError = Color.White,
)

@Composable
fun QuorumTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = QuorumLightColors,
        typography = QuorumTypography,
        content = content,
    )
}
