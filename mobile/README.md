# DigPaper Mobile App

Flutter mobile app for the DigPaper document management system.

## Features

- 📷 **Quick Photo Capture** - One-tap camera access for workshop employees
- 📥 **Inbox Management** - View and organize uploaded documents
- 📁 **Project Browsing** - View documents organized by project
- 🔍 **Pinch-to-Zoom** - Detailed document viewing
- 📱 **Responsive Layout** - Optimized for phones and tablets

## Requirements

- Flutter 3.0+
- iOS 12.0+ / Android API 21+

## Setup

### 1. Install Flutter

Follow the official guide: https://docs.flutter.dev/get-started/install

### 2. Configure Server URL

Edit `lib/services/api_service.dart` and update `ApiConfig.baseUrl`:

```dart
class ApiConfig {
  // For real device testing, use your server's IP
  static const String baseUrl = 'http://192.168.1.100:3000';
}
```

### 3. Run the App

```bash
cd mobile
flutter pub get
flutter run
```

## Project Structure

```
lib/
├── main.dart                 # App entry point
├── models/
│   ├── document.dart         # Document entity
│   └── project.dart          # Project entity
├── services/
│   └── api_service.dart      # Backend API client
├── screens/
│   ├── home_screen.dart      # Bottom navigation
│   ├── upload_screen.dart    # Camera/upload flow
│   ├── inbox_screen.dart     # Unassigned documents
│   ├── projects_screen.dart  # Project list
│   ├── project_documents_screen.dart
│   └── document_preview_screen.dart
└── theme/
    └── app_theme.dart        # Colors, fonts, styles
```

## Building for Production

### iOS

```bash
flutter build ios --release
```

Then open `ios/Runner.xcworkspace` in Xcode and archive.

### Android

```bash
flutter build apk --release
# or for app bundle
flutter build appbundle --release
```

## Accessibility Features

- Large touch targets (56dp minimum)
- High contrast colors
- Clear typography (16-20px body text)
- Portuguese labels for local users
