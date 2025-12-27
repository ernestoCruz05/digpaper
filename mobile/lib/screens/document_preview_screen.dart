/// Document Preview Screen
/// 
/// Full-screen view of a document with pinch-to-zoom capability.
/// Allows assigning document to a project from this view.

import 'package:flutter/material.dart';
import 'package:cached_network_image/cached_network_image.dart';
import '../models/document.dart';
import '../models/project.dart';
import '../services/api_service.dart';
import '../theme/app_theme.dart';

class DocumentPreviewScreen extends StatefulWidget {
  final Document document;
  final List<Project> projects;
  final VoidCallback? onAssigned;

  const DocumentPreviewScreen({
    super.key,
    required this.document,
    required this.projects,
    this.onAssigned,
  });

  @override
  State<DocumentPreviewScreen> createState() => _DocumentPreviewScreenState();
}

class _DocumentPreviewScreenState extends State<DocumentPreviewScreen> {
  final ApiService _api = ApiService();
  final TransformationController _transformController = TransformationController();

  @override
  void dispose() {
    _transformController.dispose();
    super.dispose();
  }

  /// Show project selection and assign document
  Future<void> _assignToProject() async {
    final selectedProject = await showModalBottomSheet<Project>(
      context: context,
      isScrollControlled: true,
      shape: const RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(20)),
      ),
      builder: (context) => _buildProjectSelector(),
    );

    if (selectedProject != null) {
      await _doAssign(selectedProject);
    }
  }

  Widget _buildProjectSelector() {
    return DraggableScrollableSheet(
      initialChildSize: 0.6,
      minChildSize: 0.3,
      maxChildSize: 0.9,
      expand: false,
      builder: (context, scrollController) {
        return Column(
          children: [
            Container(
              margin: const EdgeInsets.only(top: 12, bottom: 8),
              width: 40,
              height: 4,
              decoration: BoxDecoration(
                color: Colors.grey[300],
                borderRadius: BorderRadius.circular(2),
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(16),
              child: Text(
                'Mover para Obra',
                style: Theme.of(context).textTheme.headlineSmall,
              ),
            ),
            const Divider(height: 1),
            Expanded(
              child: widget.projects.isEmpty
                  ? Center(
                      child: Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(Icons.folder_off, size: 64, color: Colors.grey[400]),
                          const SizedBox(height: 16),
                          Text(
                            'Nenhuma obra ativa',
                            style: TextStyle(fontSize: 18, color: Colors.grey[600]),
                          ),
                        ],
                      ),
                    )
                  : ListView.builder(
                      controller: scrollController,
                      itemCount: widget.projects.length,
                      itemBuilder: (context, index) {
                        final project = widget.projects[index];
                        return ListTile(
                          leading: const CircleAvatar(
                            backgroundColor: AppTheme.primaryLight,
                            child: Icon(Icons.folder, color: Colors.white),
                          ),
                          title: Text(
                            project.name,
                            style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w500),
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

  Future<void> _doAssign(Project project) async {
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => const Center(child: CircularProgressIndicator()),
    );

    final result = await _api.assignDocument(widget.document.id, project.id);

    if (mounted) Navigator.pop(context); // Close loading

    if (result.isSuccess) {
      widget.onAssigned?.call();
      if (mounted) {
        Navigator.pop(context); // Close preview
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Row(
              children: [
                const Icon(Icons.check_circle, color: Colors.white),
                const SizedBox(width: 12),
                Expanded(child: Text('Movido para "${project.name}"')),
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

  /// Reset zoom to default
  void _resetZoom() {
    _transformController.value = Matrix4.identity();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      appBar: AppBar(
        backgroundColor: Colors.black,
        foregroundColor: Colors.white,
        title: Text(
          widget.document.originalName,
          style: const TextStyle(fontSize: 16),
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.zoom_out_map),
            onPressed: _resetZoom,
            tooltip: 'Repor zoom',
          ),
        ],
      ),
      body: Column(
        children: [
          // Image with pinch-to-zoom
          Expanded(
            child: InteractiveViewer(
              transformationController: _transformController,
              minScale: 0.5,
              maxScale: 4.0,
              child: Center(
                child: widget.document.isImage
                    ? CachedNetworkImage(
                        imageUrl: widget.document.fileUrl,
                        fit: BoxFit.contain,
                        placeholder: (context, url) => const Center(
                          child: CircularProgressIndicator(color: Colors.white),
                        ),
                        errorWidget: (context, url, error) => const Column(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            Icon(Icons.broken_image, size: 64, color: Colors.grey),
                            SizedBox(height: 16),
                            Text(
                              'Não foi possível carregar a imagem',
                              style: TextStyle(color: Colors.grey),
                            ),
                          ],
                        ),
                      )
                    : Column(
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          Icon(
                            widget.document.isPdf 
                                ? Icons.picture_as_pdf 
                                : Icons.insert_drive_file,
                            size: 100,
                            color: widget.document.isPdf ? Colors.red : Colors.grey,
                          ),
                          const SizedBox(height: 24),
                          Text(
                            widget.document.fileType.toUpperCase(),
                            style: const TextStyle(
                              color: Colors.white,
                              fontSize: 24,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ],
                      ),
              ),
            ),
          ),
          
          // Info bar and action button
          Container(
            color: Colors.grey[900],
            padding: const EdgeInsets.all(16),
            child: SafeArea(
              top: false,
              child: Column(
                children: [
                  // Document info
                  Row(
                    children: [
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              widget.document.originalName,
                              style: const TextStyle(
                                color: Colors.white,
                                fontSize: 16,
                                fontWeight: FontWeight.w600,
                              ),
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                            ),
                            const SizedBox(height: 4),
                            Text(
                              'Enviado: ${widget.document.formattedDate}',
                              style: TextStyle(
                                color: Colors.grey[400],
                                fontSize: 14,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ],
                  ),
                  const SizedBox(height: 16),
                  
                  // Assign button
                  SizedBox(
                    width: double.infinity,
                    height: 56,
                    child: ElevatedButton.icon(
                      onPressed: _assignToProject,
                      icon: const Icon(Icons.drive_file_move),
                      label: const Text('Mover para Obra'),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
