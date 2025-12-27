/// Project Documents Screen
/// 
/// Shows all documents assigned to a specific project.
/// Uses the same grid layout as the inbox for consistency.

import 'package:flutter/material.dart';
import 'package:cached_network_image/cached_network_image.dart';
import '../models/document.dart';
import '../models/project.dart';
import '../services/api_service.dart';
import '../theme/app_theme.dart';

class ProjectDocumentsScreen extends StatefulWidget {
  final Project project;

  const ProjectDocumentsScreen({
    super.key,
    required this.project,
  });

  @override
  State<ProjectDocumentsScreen> createState() => _ProjectDocumentsScreenState();
}

class _ProjectDocumentsScreenState extends State<ProjectDocumentsScreen> {
  final ApiService _api = ApiService();
  
  List<Document> _documents = [];
  bool _isLoading = true;
  String? _error;
  late Project _project;

  @override
  void initState() {
    super.initState();
    _project = widget.project;
    _loadDocuments();
  }

  Future<void> _loadDocuments() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    final result = await _api.getProjectDocuments(widget.project.id);

    setState(() {
      _isLoading = false;
      if (result.isSuccess) {
        _documents = result.data!;
      } else {
        _error = result.error;
      }
    });
  }

  void _openImagePreview(Document document) {
    if (!document.isImage) return;
    
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => _ImagePreview(document: document),
      ),
    );
  }

  /// Show options menu for the project
  void _showOptionsMenu() {
    showModalBottomSheet(
      context: context,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (context) => SafeArea(
        child: Padding(
          padding: const EdgeInsets.symmetric(vertical: 16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Handle bar
              Container(
                margin: const EdgeInsets.only(bottom: 16),
                width: 40,
                height: 4,
                decoration: BoxDecoration(
                  color: Colors.grey[300],
                  borderRadius: BorderRadius.circular(2),
                ),
              ),
              // Project name
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 8),
                child: Text(
                  _project.name,
                  style: Theme.of(context).textTheme.titleLarge,
                  textAlign: TextAlign.center,
                ),
              ),
              const Divider(),
              // Toggle status option
              ListTile(
                leading: Icon(
                  _project.isActive ? Icons.archive : Icons.unarchive,
                  color: _project.isActive ? Colors.orange : AppTheme.accentColor,
                ),
                title: Text(
                  _project.isActive ? 'Marcar como Concluída' : 'Reativar Obra',
                  style: const TextStyle(fontSize: 18),
                ),
                subtitle: Text(
                  _project.isActive 
                      ? 'Move para obras arquivadas'
                      : 'Volta para obras ativas',
                ),
                onTap: () {
                  Navigator.pop(context);
                  _toggleProjectStatus();
                },
              ),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _toggleProjectStatus() async {
    final newStatus = _project.isActive ? 'ARCHIVED' : 'ACTIVE';
    final actionText = _project.isActive ? 'arquivar' : 'reativar';
    
    // Confirm action
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(_project.isActive ? 'Arquivar Obra?' : 'Reativar Obra?'),
        content: Text(
          _project.isActive
              ? 'A obra será marcada como concluída e movida para arquivadas.'
              : 'A obra voltará para a lista de obras ativas.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancelar'),
          ),
          ElevatedButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(_project.isActive ? 'Arquivar' : 'Reativar'),
          ),
        ],
      ),
    );

    if (confirmed != true) return;

    // Show loading
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => const Center(child: CircularProgressIndicator()),
    );

    final result = await _api.updateProjectStatus(_project.id, newStatus);

    if (mounted) Navigator.pop(context); // Close loading

    if (result.isSuccess) {
      setState(() {
        _project = result.data!;
      });
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Row(
              children: [
                const Icon(Icons.check_circle, color: Colors.white),
                const SizedBox(width: 12),
                Expanded(
                  child: Text(
                    _project.isActive 
                        ? 'Obra reativada!'
                        : 'Obra arquivada!',
                  ),
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
            content: Text('Erro ao $actionText: ${result.error}'),
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
        title: Text(
          _project.name,
          style: const TextStyle(fontSize: 18),
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _loadDocuments,
            tooltip: 'Atualizar',
          ),
          // Settings/options button
          IconButton(
            icon: const Icon(Icons.more_vert),
            onPressed: _showOptionsMenu,
            tooltip: 'Opções',
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
              Icon(Icons.error_outline, size: 64, color: Colors.grey[400]),
              const SizedBox(height: 16),
              Text(
                _error!,
                style: const TextStyle(fontSize: 18),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 24),
              ElevatedButton.icon(
                onPressed: _loadDocuments,
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
              Icon(Icons.image_not_supported, size: 80, color: Colors.grey[300]),
              const SizedBox(height: 24),
              Text(
                'Sem Documentos',
                style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                  color: Colors.grey[600],
                ),
              ),
              const SizedBox(height: 12),
              Text(
                'Esta obra ainda não tem\ndocumentos atribuídos.',
                style: TextStyle(fontSize: 16, color: Colors.grey[500]),
                textAlign: TextAlign.center,
              ),
            ],
          ),
        ),
      );
    }

    // Responsive grid
    return RefreshIndicator(
      onRefresh: _loadDocuments,
      child: LayoutBuilder(
        builder: (context, constraints) {
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
        onTap: () => _openImagePreview(document),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
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
                    style: TextStyle(fontSize: 12, color: Colors.grey[600]),
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

/// Simple full-screen image preview with zoom
class _ImagePreview extends StatelessWidget {
  final Document document;

  const _ImagePreview({required this.document});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.black,
        foregroundColor: Colors.white,
        title: Text(
          document.originalName,
          style: const TextStyle(fontSize: 16),
        ),
      ),
      body: InteractiveViewer(
        minScale: 0.5,
        maxScale: 4.0,
        child: Center(
          child: CachedNetworkImage(
            imageUrl: document.fileUrl,
            fit: BoxFit.contain,
            placeholder: (context, url) => const Center(
              child: CircularProgressIndicator(color: Colors.white),
            ),
            errorWidget: (context, url, error) => const Icon(
              Icons.broken_image,
              size: 64,
              color: Colors.grey,
            ),
          ),
        ),
      ),
    );
  }
}
