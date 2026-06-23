package se.reflective.quorum.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import se.reflective.quorum.R

// Bundled brand families (res/font). Mirrors the website's DM Serif Display
// (display), DM Sans (UI/body), IBM Plex Mono (data/code).
val DMSerifDisplay = FontFamily(
    Font(R.font.dm_serif_display_regular, FontWeight.Normal),
    Font(R.font.dm_serif_display_italic, FontWeight.Normal, FontStyle.Italic),
)

val DMSans = FontFamily(
    Font(R.font.dm_sans_regular, FontWeight.Normal),
    Font(R.font.dm_sans_medium, FontWeight.Medium),
    Font(R.font.dm_sans_bold, FontWeight.Bold),
)

val IBMPlexMono = FontFamily(
    Font(R.font.ibm_plex_mono_regular, FontWeight.Normal),
    Font(R.font.ibm_plex_mono_medium, FontWeight.Medium),
    Font(R.font.ibm_plex_mono_semibold, FontWeight.SemiBold),
)

val QuorumTypography = Typography(
    // Display / headings → DM Serif Display
    displayLarge = TextStyle(fontFamily = DMSerifDisplay, fontWeight = FontWeight.Normal, fontSize = 40.sp),
    headlineMedium = TextStyle(fontFamily = DMSerifDisplay, fontWeight = FontWeight.Normal, fontSize = 28.sp),
    headlineSmall = TextStyle(fontFamily = DMSerifDisplay, fontWeight = FontWeight.Normal, fontSize = 24.sp),
    // Titles / UI → DM Sans
    titleLarge = TextStyle(fontFamily = DMSans, fontWeight = FontWeight.Medium, fontSize = 22.sp),
    titleMedium = TextStyle(fontFamily = DMSans, fontWeight = FontWeight.Medium, fontSize = 16.sp),
    // Body → DM Sans
    bodyLarge = TextStyle(fontFamily = DMSans, fontWeight = FontWeight.Normal, fontSize = 16.sp),
    bodyMedium = TextStyle(fontFamily = DMSans, fontWeight = FontWeight.Normal, fontSize = 14.sp),
    // Small / data → IBM Plex Mono
    bodySmall = TextStyle(fontFamily = IBMPlexMono, fontWeight = FontWeight.Normal, fontSize = 12.sp),
    // Buttons / labels → DM Sans
    labelLarge = TextStyle(fontFamily = DMSans, fontWeight = FontWeight.Medium, fontSize = 14.sp),
)
