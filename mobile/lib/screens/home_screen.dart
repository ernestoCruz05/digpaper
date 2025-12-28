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
  
  // Keys to trigger refresh on tab change
  final GlobalKey<InboxScreenState> _inboxKey = GlobalKey<InboxScreenState>();
  final GlobalKey<ProjectsScreenState> _projectsKey = GlobalKey<ProjectsScreenState>();

  // Screens for each tab - built dynamically to use keys
  late final List<Widget> _screens;
  
  @override
  void initState() {
    super.initState();
    _screens = [
      const UploadScreen(),
      InboxScreen(key: _inboxKey),
      ProjectsScreen(key: _projectsKey),
    ];
  }

  void _onTabSelected(int index) {
    setState(() {
      _currentIndex = index;
    });
    
    // Auto-refresh when switching to inbox or projects
    if (index == 1) {
      _inboxKey.currentState?.refresh();
    } else if (index == 2) {
      _projectsKey.currentState?.refresh();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: IndexedStack(
        index: _currentIndex,
        children: _screens,
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: _currentIndex,
        onDestinationSelected: _onTabSelected,
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
