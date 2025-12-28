/// Projects Screen
/// 
/// Lists all projects (Obras) with their document counts.
/// Allows viewing documents within each project.
/// 
/// Designed with clear visual hierarchy for easy scanning.

import 'package:flutter/material.dart';
import '../models/project.dart';
import '../services/api_service.dart';
import '../theme/app_theme.dart';
import 'project_documents_screen.dart';

class ProjectsScreen extends StatefulWidget {
  const ProjectsScreen({super.key});

  @override
  State<ProjectsScreen> createState() => ProjectsScreenState();
}

class ProjectsScreenState extends State<ProjectsScreen> {
  final ApiService _api = ApiService();
  
  List<Project> _projects = [];
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadProjects();
  }

  /// Public refresh method for external trigger
  void refresh() {
    _loadProjects();
  }

  Future<void> _loadProjects() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    final result = await _api.getAllProjects();

    setState(() {
      _isLoading = false;
      if (result.isSuccess) {
        _projects = result.data!;
      } else {
        _error = result.error;
      }
    });
  }

  void _openProject(Project project) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => ProjectDocumentsScreen(project: project),
      ),
    );
  }

  /// Show dialog to create a new project
  Future<void> _showCreateProjectDialog() async {
    final nameController = TextEditingController();
    
    final result = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Nova Obra'),
        content: TextField(
          controller: nameController,
          autofocus: true,
          textCapitalization: TextCapitalization.words,
          decoration: const InputDecoration(
            labelText: 'Nome da Obra',
            hintText: 'Ex: Obra Porto Seg Social',
          ),
          onSubmitted: (value) => Navigator.pop(context, value),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancelar'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.pop(context, nameController.text),
            child: const Text('Criar'),
          ),
        ],
      ),
    );

    if (result != null && result.trim().isNotEmpty) {
      await _createProject(result.trim());
    }
  }

  Future<void> _createProject(String name) async {
    // Show loading
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => const Center(child: CircularProgressIndicator()),
    );

    final result = await _api.createProject(name);

    if (mounted) Navigator.pop(context); // Close loading

    if (result.isSuccess) {
      _loadProjects(); // Refresh list
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Row(
              children: [
                const Icon(Icons.check_circle, color: Colors.white),
                const SizedBox(width: 12),
                Expanded(child: Text('Obra "$name" criada!')),
              ],
            ),
            backgroundColor: AppTheme.accentColor,
          ),
        );
      }
    } else {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(result.error!),
            backgroundColor: AppTheme.errorColor,
          ),
        );
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Obras'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadProjects,
            tooltip: 'Atualizar',
          ),
        ],
      ),
      body: _buildBody(),
      // FAB for creating new projects - large and accessible
      floatingActionButton: FloatingActionButton.extended(
        onPressed: _showCreateProjectDialog,
        icon: const Icon(Icons.add),
        label: const Text('Nova Obra'),
      ),
    );
  }

  Widget _buildBody() {
    if (_isLoading) {
      return const Center(
        child: CircularProgressIndicator(),
      );
    }

    if (_error != null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(
                Icons.error_outline,
                size: 64,
                color: Colors.grey[400],
              ),
              const SizedBox(height: 16),
              Text(
                _error!,
                style: const TextStyle(fontSize: 18),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 24),
              ElevatedButton.icon(
                onPressed: _loadProjects,
                icon: const Icon(Icons.refresh),
                label: const Text('Tentar novamente'),
              ),
            ],
          ),
        ),
      );
    }

    if (_projects.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(
                Icons.folder_open,
                size: 80,
                color: Colors.grey[300],
              ),
              const SizedBox(height: 24),
              Text(
                'Sem Obras',
                style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                  color: Colors.grey[600],
                ),
              ),
              const SizedBox(height: 12),
              Text(
                'As obras criadas no escritório\naparecerão aqui.',
                style: TextStyle(
                  fontSize: 16,
                  color: Colors.grey[500],
                ),
                textAlign: TextAlign.center,
              ),
            ],
          ),
        ),
      );
    }

    return RefreshIndicator(
      onRefresh: _loadProjects,
      child: ListView.builder(
        padding: const EdgeInsets.symmetric(vertical: 8),
        itemCount: _projects.length,
        itemBuilder: (context, index) {
          return _buildProjectCard(_projects[index]);
        },
      ),
    );
  }

  Widget _buildProjectCard(Project project) {
    return Card(
      margin: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
      child: InkWell(
        onTap: () => _openProject(project),
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            children: [
              // Folder icon with status indicator
              Container(
                width: 56,
                height: 56,
                decoration: BoxDecoration(
                  color: project.isActive 
                      ? AppTheme.primaryLight.withValues(alpha: 0.15)
                      : Colors.grey.withValues(alpha: 0.15),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Icon(
                  project.isActive ? Icons.folder : Icons.folder_outlined,
                  size: 32,
                  color: project.isActive ? AppTheme.primaryColor : Colors.grey,
                ),
              ),
              const SizedBox(width: 16),
              
              // Project info
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      project.name,
                      style: const TextStyle(
                        fontSize: 18,
                        fontWeight: FontWeight.w600,
                      ),
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                    const SizedBox(height: 4),
                    Row(
                      children: [
                        // Status badge
                        Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 8,
                            vertical: 2,
                          ),
                          decoration: BoxDecoration(
                            color: project.isActive
                                ? AppTheme.accentColor.withValues(alpha: 0.15)
                                : Colors.grey.withValues(alpha: 0.15),
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            project.isActive ? 'ATIVA' : 'ARQUIVADA',
                            style: TextStyle(
                              fontSize: 11,
                              fontWeight: FontWeight.w600,
                              color: project.isActive 
                                  ? AppTheme.accentColor 
                                  : Colors.grey[600],
                            ),
                          ),
                        ),
                        const SizedBox(width: 8),
                        // Date
                        Text(
                          project.formattedDate,
                          style: TextStyle(
                            fontSize: 14,
                            color: Colors.grey[600],
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
              
              // Arrow
              Icon(
                Icons.chevron_right,
                color: Colors.grey[400],
                size: 28,
              ),
            ],
          ),
        ),
      ),
    );
  }
}
