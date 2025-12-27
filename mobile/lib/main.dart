/// DigPaper Mobile App
/// 
/// Main entry point for the mobile application.
/// Designed for workshop employees to quickly upload and manage documents.

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'theme/app_theme.dart';
import 'screens/home_screen.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  
  // Lock orientation to portrait for simpler UX on phones
  // Tablets can use landscape via adaptive layouts
  SystemChrome.setPreferredOrientations([
    DeviceOrientation.portraitUp,
    DeviceOrientation.portraitDown,
  ]);
  
  // Set status bar style
  SystemChrome.setSystemUIOverlayStyle(const SystemUiOverlayStyle(
    statusBarColor: Colors.transparent,
    statusBarIconBrightness: Brightness.light,
  ));
  
  runApp(const DigPaperApp());
}

class DigPaperApp extends StatelessWidget {
  const DigPaperApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'DigPaper',
      debugShowCheckedModeBanner: false,
      theme: AppTheme.lightTheme,
      home: const HomeScreen(),
    );
  }
}
