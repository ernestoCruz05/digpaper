/// PDF Viewer Screen
/// 
/// Full-screen PDF viewer with proper navigation and controls.
/// Uses Syncfusion PDF Viewer for native rendering.

import 'package:flutter/material.dart';
import 'package:syncfusion_flutter_pdfviewer/pdfviewer.dart';
import '../models/document.dart';
import '../models/project.dart';
import '../services/api_service.dart';
import '../theme/app_theme.dart';

class PdfViewerScreen extends StatefulWidget {
  final Document document;
  final List<Project> projects;
  final VoidCallback? onAssigned;

  const PdfViewerScreen({
    super.key,
    required this.document,
    required this.projects,
    this.onAssigned,
  });

  @override
  State<PdfViewerScreen> createState() => _PdfViewerScreenState();
}

class _PdfViewerScreenState extends State<PdfViewerScreen> {
  final ApiService _api = ApiService();
  final GlobalKey<SfPdfViewerState> _pdfViewerKey = GlobalKey();
  late PdfViewerController _pdfController;
  int _currentPage = 1;
  int _totalPages = 0;
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _pdfController = PdfViewerController();
  }

  @override
  void dispose() {
    _pdfController.dispose();
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
        Navigator.pop(context); // Close PDF viewer
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

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(
          widget.document.originalName,
          style: const TextStyle(fontSize: 16),
        ),
        actions: [
          if (_totalPages > 0)
            Center(
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                child: Text(
                  '$_currentPage / $_totalPages',
                  style: const TextStyle(fontSize: 14),
                ),
              ),
            ),
        ],
      ),
      body: Column(
        children: [
          // PDF Viewer
          Expanded(
            child: _error != null
                ? Center(
                    child: Column(
                      mainAxisAlignment: MainAxisAlignment.center,
                      children: [
                        Icon(Icons.error_outline, size: 64, color: Colors.grey[400]),
                        const SizedBox(height: 16),
                        Padding(
                          padding: const EdgeInsets.symmetric(horizontal: 32),
                          child: Text(
                            _error!,
                            style: TextStyle(fontSize: 16, color: Colors.grey[600]),
                            textAlign: TextAlign.center,
                          ),
                        ),
                      ],
                    ),
                  )
                : Stack(
                    children: [
                      SfPdfViewer.network(
                        widget.document.fileUrl,
                        key: _pdfViewerKey,
                        controller: _pdfController,
                        canShowScrollHead: true,
                        canShowScrollStatus: true,
                        enableDoubleTapZooming: true,
                        onDocumentLoaded: (details) {
                          setState(() {
                            _isLoading = false;
                            _totalPages = details.document.pages.count;
                          });
                        },
                        onDocumentLoadFailed: (details) {
                          setState(() {
                            _isLoading = false;
                            _error = 'Não foi possível carregar o PDF: ${details.description}';
                          });
                        },
                        onPageChanged: (details) {
                          setState(() {
                            _currentPage = details.newPageNumber;
                          });
                        },
                      ),
                      if (_isLoading)
                        const Center(
                          child: CircularProgressIndicator(),
                        ),
                    ],
                  ),
          ),
          
          // Action bar (only for inbox documents)
          if (widget.document.isInInbox)
            Container(
              color: Colors.grey[100],
              padding: const EdgeInsets.all(16),
              child: SafeArea(
                top: false,
                child: SizedBox(
                  width: double.infinity,
                  height: 56,
                  child: ElevatedButton.icon(
                    onPressed: _assignToProject,
                    icon: const Icon(Icons.drive_file_move),
                    label: const Text('Mover para Obra'),
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}
