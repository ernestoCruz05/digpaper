/// Inbox Screen
/// 
/// Displays all documents that haven't been assigned to a project yet.
/// Office staff use this to organize uploaded photos into projects.
/// 
/// Features:
/// - Pull-to-refresh for updating list
/// - Thumbnail grid for easy visual identification
/// - Tap to preview, long-press to assign to project

import 'package:flutter/material.dart';
import 'package:cached_network_image/cached_network_image.dart';
import '../models/document.dart';
import '../models/project.dart';
import '../services/api_service.dart';
import '../theme/app_theme.dart';
import 'document_preview_screen.dart';

class InboxScreen extends StatefulWidget {
  const InboxScreen({super.key});

  @override
  State<InboxScreen> createState() => _InboxScreenState();
}

class _InboxScreenState extends State<InboxScreen> {
  final ApiService _api = ApiService();
  
  List<Document> _documents = [];
  List<Project> _projects = [];
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadData();
  }

  Future<void> _loadData() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    // Load inbox documents and projects in parallel
    final results = await Future.wait([
      _api.getInboxDocuments(),
      _api.getActiveProjects(),
    ]);

    final docsResult = results[0] as ApiResult<List<Document>>;
    final projectsResult = results[1] as ApiResult<List<Project>>;

    setState(() {
      _isLoading = false;
      
      if (docsResult.isSuccess) {
        _documents = docsResult.data!;
      } else {
        _error = docsResult.error;
      }
      
      if (projectsResult.isSuccess) {
        _projects = projectsResult.data!;
      }
    });
  }

  /// Show dialog to assign document to a project
  Future<void> _showAssignDialog(Document document) async {
    final selectedProject = await showModalBottomSheet<Project>(
      context: context,
      isScrollControlled: true,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (context) => _buildProjectSelector(document),
    );

    if (selectedProject != null) {
      await _assignToProject(document, selectedProject);
    }
  }

  Widget _buildProjectSelector(Document document) {
    return DraggableScrollableSheet(
      initialChildSize: 0.6,
      minChildSize: 0.3,
      maxChildSize: 0.9,
      expand: false,
      builder: (context, scrollController) {
        return Column(
          children: [
            // Handle bar
            Container(
              margin: const EdgeInsets.only(top: 12, bottom: 8),
              width: 40,
              height: 4,
              decoration: BoxDecoration(
                color: Colors.grey[300],
                borderRadius: BorderRadius.circular(2),
              ),
            ),
            // Title
            Padding(
              padding: const EdgeInsets.all(16),
              child: Text(
                'Mover para Obra',
                style: Theme.of(context).textTheme.headlineSmall,
              ),
            ),
            const Divider(height: 1),
            // Project list
            Expanded(
              child: _projects.isEmpty
                  ? Center(
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(
                            Icons.folder_off,
                            size: 64,
                            color: Colors.grey[400],
                          ),
                          const SizedBox(height: 16),
                          Text(
                            'Nenhuma obra ativa',
                            style: TextStyle(
                              fontSize: 18,
                              color: Colors.grey[600],
                            ),
                          ),
                        ],
                      ),
                    )
                  : ListView.builder(
                      controller: scrollController,
                      itemCount: _projects.length,
                      itemBuilder: (context, index) {
                        final project = _projects[index];
                        return ListTile(
                          leading: const CircleAvatar(
                            backgroundColor: AppTheme.primaryLight,
                            child: Icon(Icons.folder, color: Colors.white),
                          ),
                          title: Text(
                            project.name,
                            style: const TextStyle(
                              fontSize: 18,
                              fontWeight: FontWeight.w500,
                            ),
                          ),
                          subtitle: Text('Criado: ${project.formattedDate}'),
                          trailing: const Icon(Icons.chevron_right),
                          onTap: () => Navigator.pop(context, project),
                        );
                      },
                    ),
            ),
          ],
        );
      },
    );
  }

  Future<void> _assignToProject(Document document, Project project) async {
    // Show loading indicator
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => const Center(
        child: CircularProgressIndicator(),
      ),
    );

    final result = await _api.assignDocument(document.id, project.id);

    // Close loading dialog
    if (mounted) Navigator.pop(context);

    if (result.isSuccess) {
      // Remove from inbox list
      setState(() {
        _documents.removeWhere((d) => d.id == document.id);
      });
      
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Row(
              children: [
                const Icon(Icons.check_circle, color: Colors.white),
                const SizedBox(width: 12),
                Expanded(
                  child: Text('Movido para "${project.name}"'),
                ),
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

  void _openPreview(Document document) {
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => DocumentPreviewScreen(
          document: document,
          projects: _projects,
          onAssigned: () {
            // Refresh inbox when document is assigned from preview
            _loadData();
          },
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Caixa de Entrada'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadData,
            tooltip: 'Atualizar',
          ),
        ],
      ),
      body: _buildBody(),
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
                onPressed: _loadData,
                icon: const Icon(Icons.refresh),
                label: const Text('Tentar novamente'),
              ),
            ],
          ),
        ),
      );
    }

    if (_documents.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(
                Icons.inbox,
                size: 80,
                color: Colors.grey[300],
              ),
              const SizedBox(height: 24),
              Text(
                'Caixa de Entrada Vazia',
                style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                  color: Colors.grey[600],
                ),
              ),
              const SizedBox(height: 12),
              Text(
                'Os documentos fotografados\naparecerão aqui.',
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

    // Responsive grid - more columns on tablets
    return RefreshIndicator(
      onRefresh: _loadData,
      child: LayoutBuilder(
        builder: (context, constraints) {
          // Calculate columns based on screen width
          // Phones: 2 columns, Tablets: 3-4 columns
          final crossAxisCount = constraints.maxWidth > 600 
              ? (constraints.maxWidth > 900 ? 4 : 3)
              : 2;
          
          return GridView.builder(
            padding: const EdgeInsets.all(12),
            gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
              crossAxisCount: crossAxisCount,
              crossAxisSpacing: 12,
              mainAxisSpacing: 12,
              childAspectRatio: 0.85,
            ),
            itemCount: _documents.length,
            itemBuilder: (context, index) {
              return _buildDocumentCard(_documents[index]);
            },
          );
        },
      ),
    );
  }

  Widget _buildDocumentCard(Document document) {
    return Card(
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: () => _openPreview(document),
        onLongPress: () => _showAssignDialog(document),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Image thumbnail
            Expanded(
              child: document.isImage
                  ? CachedNetworkImage(
                      imageUrl: document.fileUrl,
                      fit: BoxFit.cover,
                      placeholder: (context, url) => Container(
                        color: Colors.grey[200],
                        child: const Center(
                          child: CircularProgressIndicator(strokeWidth: 2),
                        ),
                      ),
                      errorWidget: (context, url, error) => Container(
                        color: Colors.grey[200],
                        child: const Icon(Icons.broken_image, size: 48),
                      ),
                    )
                  : Container(
                      color: Colors.grey[200],
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(
                            document.isPdf ? Icons.picture_as_pdf : Icons.insert_drive_file,
                            size: 48,
                            color: document.isPdf ? Colors.red : Colors.grey,
                          ),
                          const SizedBox(height: 8),
                          Text(
                            document.fileType.toUpperCase(),
                            style: const TextStyle(
                              fontSize: 12,
                              fontWeight: FontWeight.w600,
                            ),
                          ),
                        ],
                      ),
                    ),
            ),
            // Info footer
            Padding(
              padding: const EdgeInsets.all(12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    document.originalName,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      fontSize: 14,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    document.formattedDate,
                    style: TextStyle(
                      fontSize: 12,
                      color: Colors.grey[600],
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
