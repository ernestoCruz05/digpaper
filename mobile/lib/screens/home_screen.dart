/// Home Screen with Bottom Navigation
/// 
/// Main navigation hub for the app. Uses bottom navigation
/// for easy one-handed access on phones.
/// 
/// Designed with large touch targets and clear labels
/// for users of all ages.

import 'package:flutter/material.dart';
import 'upload_screen.dart';
import 'inbox_screen.dart';
import 'projects_screen.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  int _currentIndex = 0;

  // Screens for each tab
  final List<Widget> _screens = const [
    UploadScreen(),
    InboxScreen(),
    ProjectsScreen(),
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: IndexedStack(
        index: _currentIndex,
        children: _screens,
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _currentIndex,
        onDestinationSelected: (index) {
          setState(() {
            _currentIndex = index;
          });
        },
        destinations: const [
          // Upload - primary action, prominent icon
          NavigationDestination(
            icon: Icon(Icons.add_a_photo_outlined, size: 28),
            selectedIcon: Icon(Icons.add_a_photo, size: 28),
            label: 'Fotografar',
          ),
          // Inbox - documents waiting for organization
          NavigationDestination(
            icon: Icon(Icons.inbox_outlined, size: 28),
            selectedIcon: Icon(Icons.inbox, size: 28),
            label: 'Caixa de Entrada',
          ),
          // Projects - view organized work
          NavigationDestination(
            icon: Icon(Icons.folder_outlined, size: 28),
            selectedIcon: Icon(Icons.folder, size: 28),
            label: 'Obras',
          ),
        ],
      ),
    );
  }
}
