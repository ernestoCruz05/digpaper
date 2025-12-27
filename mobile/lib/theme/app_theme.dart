/// App Theme Configuration
/// 
/// Defines colors, fonts, and component styles optimized for:
/// - Readability (larger fonts, good contrast)
/// - Touch targets (minimum 48dp for accessibility)
/// - All age groups (clear visual hierarchy)

import 'package:flutter/material.dart';

class AppTheme {
  // Brand colors - warm, professional palette for carpentry business
  static const Color primaryColor = Color(0xFF8B4513);      // Saddle brown (wood-like)
  static const Color primaryDark = Color(0xFF5D2E0C);       // Dark brown
  static const Color primaryLight = Color(0xFFD2691E);      // Chocolate
  static const Color accentColor = Color(0xFF2E7D32);       // Forest green (success)
  static const Color backgroundColor = Color(0xFFFAF8F5);   // Warm white
  static const Color surfaceColor = Colors.white;
  static const Color errorColor = Color(0xFFD32F2F);
  static const Color textPrimary = Color(0xFF1A1A1A);
  static const Color textSecondary = Color(0xFF666666);
  static const Color dividerColor = Color(0xFFE0E0E0);

  /// Main theme data
  static ThemeData get lightTheme {
    return ThemeData(
      useMaterial3: true,
      colorScheme: ColorScheme.light(
        primary: primaryColor,
        secondary: accentColor,
        surface: surfaceColor,
        error: errorColor,
      ),
      scaffoldBackgroundColor: backgroundColor,
      
      // AppBar - clean and prominent
      appBarTheme: const AppBarTheme(
        backgroundColor: primaryColor,
        foregroundColor: Colors.white,
        elevation: 0,
        centerTitle: true,
        titleTextStyle: TextStyle(
          fontSize: 20,
          fontWeight: FontWeight.w600,
          color: Colors.white,
        ),
      ),

      // Cards - subtle elevation, rounded corners
      cardTheme: CardThemeData(
        color: surfaceColor,
        elevation: 2,
        shadowColor: Colors.black.withValues(alpha: 0.1),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
        ),
        margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      ),

      // Buttons - large touch targets (48dp minimum)
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: primaryColor,
          foregroundColor: Colors.white,
          minimumSize: const Size(double.infinity, 56), // Large touch target
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
          textStyle: const TextStyle(
            fontSize: 18,
            fontWeight: FontWeight.w600,
          ),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
          ),
        ),
      ),

      // Outlined buttons
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: primaryColor,
          minimumSize: const Size(double.infinity, 56),
          padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
          textStyle: const TextStyle(
            fontSize: 18,
            fontWeight: FontWeight.w600,
          ),
          side: const BorderSide(color: primaryColor, width: 2),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
          ),
        ),
      ),

      // Text buttons
      textButtonTheme: TextButtonThemeData(
        style: TextButton.styleFrom(
          foregroundColor: primaryColor,
          minimumSize: const Size(48, 48),
          textStyle: const TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.w600,
          ),
        ),
      ),

      // Floating action button - prominent and accessible
      floatingActionButtonTheme: const FloatingActionButtonThemeData(
        backgroundColor: primaryColor,
        foregroundColor: Colors.white,
        elevation: 4,
        extendedPadding: EdgeInsets.symmetric(horizontal: 24, vertical: 16),
        extendedTextStyle: TextStyle(
          fontSize: 16,
          fontWeight: FontWeight.w600,
        ),
      ),

      // Input fields - clear and accessible
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: Colors.white,
        contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 16),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: const BorderSide(color: dividerColor),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: const BorderSide(color: dividerColor),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: const BorderSide(color: primaryColor, width: 2),
        ),
        errorBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: const BorderSide(color: errorColor),
        ),
        labelStyle: const TextStyle(fontSize: 16),
        hintStyle: TextStyle(fontSize: 16, color: textSecondary.withValues(alpha: 0.7)),
      ),

      // Bottom navigation - clear icons and labels
      bottomNavigationBarTheme: const BottomNavigationBarThemeData(
        backgroundColor: Colors.white,
        selectedItemColor: primaryColor,
        unselectedItemColor: textSecondary,
        selectedLabelStyle: TextStyle(fontSize: 14, fontWeight: FontWeight.w600),
        unselectedLabelStyle: TextStyle(fontSize: 14),
        type: BottomNavigationBarType.fixed,
        elevation: 8,
      ),

      // Dialogs
      dialogTheme: DialogThemeData(
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(16),
        ),
        titleTextStyle: const TextStyle(
          fontSize: 20,
          fontWeight: FontWeight.w600,
          color: textPrimary,
        ),
      ),

      // Snackbars - prominent feedback
      snackBarTheme: SnackBarThemeData(
        backgroundColor: textPrimary,
        contentTextStyle: const TextStyle(fontSize: 16, color: Colors.white),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(8),
        ),
        behavior: SnackBarBehavior.floating,
      ),

      // Text theme - readable sizes
      textTheme: const TextTheme(
        headlineLarge: TextStyle(fontSize: 28, fontWeight: FontWeight.bold, color: textPrimary),
        headlineMedium: TextStyle(fontSize: 24, fontWeight: FontWeight.bold, color: textPrimary),
        headlineSmall: TextStyle(fontSize: 20, fontWeight: FontWeight.w600, color: textPrimary),
        titleLarge: TextStyle(fontSize: 18, fontWeight: FontWeight.w600, color: textPrimary),
        titleMedium: TextStyle(fontSize: 16, fontWeight: FontWeight.w600, color: textPrimary),
        titleSmall: TextStyle(fontSize: 14, fontWeight: FontWeight.w600, color: textPrimary),
        bodyLarge: TextStyle(fontSize: 18, color: textPrimary),       // Larger for readability
        bodyMedium: TextStyle(fontSize: 16, color: textPrimary),
        bodySmall: TextStyle(fontSize: 14, color: textSecondary),
        labelLarge: TextStyle(fontSize: 16, fontWeight: FontWeight.w600, color: textPrimary),
      ),

      // Dividers
      dividerTheme: const DividerThemeData(
        color: dividerColor,
        thickness: 1,
        space: 1,
      ),

      // List tiles - accessible touch targets
      listTileTheme: const ListTileThemeData(
        minVerticalPadding: 12,
        contentPadding: EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      ),
    );
  }
}
