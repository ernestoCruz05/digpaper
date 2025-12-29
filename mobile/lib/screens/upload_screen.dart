/// Upload Screen
/// 
/// Primary screen for workshop employees to capture and upload photos.
/// Optimized for quick one-tap workflow:
/// 1. Tap camera button → Take photo
/// 2. Review and confirm → Upload
/// 
/// Large buttons and clear feedback for all age groups.

import 'dart:typed_data';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:file_picker/file_picker.dart';
import '../models/document.dart';
import '../services/api_service.dart';
import '../theme/app_theme.dart';

class UploadScreen extends StatefulWidget {
  const UploadScreen({super.key});

  @override
  State<UploadScreen> createState() => _UploadScreenState();
}

class _UploadScreenState extends State<UploadScreen> {
  final ApiService _api = ApiService();
  final ImagePicker _picker = ImagePicker();
  
  XFile? _selectedImage;
  Uint8List? _imageBytes; // For displaying on web
  PlatformFile? _selectedPdf; // For PDF files
  Uint8List? _pdfBytes;
  bool _isUploading = false;
  String? _lastUploadedName;

  /// Open camera to capture photo
  Future<void> _takePhoto() async {
    try {
      final XFile? photo = await _picker.pickImage(
        source: ImageSource.camera,
        imageQuality: 85, // Good quality, reasonable file size
        maxWidth: 2048,   // Limit resolution for faster uploads
        maxHeight: 2048,
      );
      
      if (photo != null) {
        final bytes = await photo.readAsBytes();
        setState(() {
          _selectedImage = photo;
          _imageBytes = bytes;
          _selectedPdf = null;
          _pdfBytes = null;
        });
      }
    } catch (e) {
      _showError('Não foi possível aceder à câmara.');
    }
  }

  /// Pick image from gallery
  Future<void> _pickFromGallery() async {
    try {
      final XFile? image = await _picker.pickImage(
        source: ImageSource.gallery,
        imageQuality: 85,
        maxWidth: 2048,
        maxHeight: 2048,
      );
      
      if (image != null) {
        final bytes = await image.readAsBytes();
        setState(() {
          _selectedImage = image;
          _imageBytes = bytes;
          _selectedPdf = null;
          _pdfBytes = null;
        });
      }
    } catch (e) {
      _showError('Não foi possível aceder à galeria.');
    }
  }

  /// Pick PDF file
  Future<void> _pickPdf() async {
    try {
      final result = await FilePicker.platform.pickFiles(
        type: FileType.custom,
        allowedExtensions: ['pdf'],
        withData: true,
      );
      
      if (result != null && result.files.isNotEmpty) {
        final file = result.files.first;
        if (file.bytes != null) {
          setState(() {
            _selectedPdf = file;
            _pdfBytes = file.bytes;
            _selectedImage = null;
            _imageBytes = null;
          });
        }
      }
    } catch (e) {
      _showError('Não foi possível selecionar o PDF.');
    }
  }

  /// Upload the selected image or PDF
  Future<void> _uploadImage() async {
    if (_selectedImage == null && _selectedPdf == null) return;
    
    setState(() {
      _isUploading = true;
    });

    ApiResult<Document> result;
    
    if (_selectedPdf != null && _pdfBytes != null) {
      // Upload PDF
      final xFile = XFile.fromData(
        _pdfBytes!,
        name: _selectedPdf!.name,
        mimeType: 'application/pdf',
      );
      result = await _api.uploadXFile(xFile, _pdfBytes!);
    } else {
      // Upload image
      result = await _api.uploadXFile(_selectedImage!, _imageBytes!);
    }
    
    setState(() {
      _isUploading = false;
    });

    if (result.isSuccess) {
      setState(() {
        _lastUploadedName = result.data!.originalName;
        _selectedImage = null;
        _imageBytes = null;
        _selectedPdf = null;
        _pdfBytes = null;
      });
      _showSuccess(_selectedPdf != null ? 'PDF enviado com sucesso!' : 'Foto enviada com sucesso!');
    } else {
      _showError(result.error!);
    }
  }

  /// Clear selected image or PDF
  void _clearImage() {
    setState(() {
      _selectedImage = null;
      _imageBytes = null;
      _selectedPdf = null;
      _pdfBytes = null;
    });
  }

  void _showSuccess(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Row(
          children: [
            const Icon(Icons.check_circle, color: Colors.white),
            const SizedBox(width: 12),
            Expanded(child: Text(message)),
          ],
        ),
        backgroundColor: AppTheme.accentColor,
        duration: const Duration(seconds: 3),
      ),
    );
  }

  void _showError(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Row(
          children: [
            const Icon(Icons.error, color: Colors.white),
            const SizedBox(width: 12),
            Expanded(child: Text(message)),
          ],
        ),
        backgroundColor: AppTheme.errorColor,
        duration: const Duration(seconds: 4),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final hasSelection = _imageBytes != null || _pdfBytes != null;
    return Scaffold(
      appBar: AppBar(
        title: const Text('Enviar Documento'),
      ),
      body: SafeArea(
        child: hasSelection 
            ? _buildPreviewView()
            : _buildCaptureView(),
      ),
    );
  }

  /// Initial view with camera/gallery buttons
  Widget _buildCaptureView() {
    return Padding(
      padding: const EdgeInsets.all(24),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Instructions
          const Icon(
            Icons.document_scanner,
            size: 80,
            color: AppTheme.primaryLight,
          ),
          const SizedBox(height: 24),
          Text(
            'Enviar Documento',
            style: Theme.of(context).textTheme.headlineSmall,
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 12),
          Text(
            'Tire uma foto ou escolha um PDF.\nO documento será enviado para a Caixa de Entrada.',
            style: Theme.of(context).textTheme.bodyLarge?.copyWith(
              color: AppTheme.textSecondary,
            ),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 48),
          
          // Camera button - primary action, extra large
          SizedBox(
            height: 72,
            child: ElevatedButton.icon(
              onPressed: _takePhoto,
              icon: const Icon(Icons.camera_alt, size: 32),
              label: const Text('Tirar Foto', style: TextStyle(fontSize: 20)),
            ),
          ),
          const SizedBox(height: 16),
          
          // Gallery button - secondary action
          OutlinedButton.icon(
            onPressed: _pickFromGallery,
            icon: const Icon(Icons.photo_library, size: 24),
            label: const Text('Escolher da Galeria'),
          ),
          const SizedBox(height: 12),
          
          // PDF button
          OutlinedButton.icon(
            onPressed: _pickPdf,
            style: OutlinedButton.styleFrom(
              foregroundColor: Colors.red[700],
              side: BorderSide(color: Colors.red[300]!),
            ),
            icon: const Icon(Icons.picture_as_pdf, size: 24),
            label: const Text('Escolher PDF'),
          ),
          
          // Show last upload confirmation
          if (_lastUploadedName != null) ...[
            const SizedBox(height: 32),
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: AppTheme.accentColor.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(12),
                border: Border.all(color: AppTheme.accentColor.withValues(alpha: 0.3)),
              ),
              child: Row(
                children: [
                  const Icon(Icons.check_circle, color: AppTheme.accentColor),
                  const SizedBox(width: 12),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        const Text(
                          'Última foto enviada:',
                          style: TextStyle(
                            fontSize: 14,
                            color: AppTheme.textSecondary,
                          ),
                        ),
                        Text(
                          _lastUploadedName!,
                          style: const TextStyle(
                            fontSize: 16,
                            fontWeight: FontWeight.w600,
                          ),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  /// Preview view with image/PDF and upload/cancel buttons
  Widget _buildPreviewView() {
    final isPdf = _pdfBytes != null;
    
    return Column(
      children: [
        // Preview - takes most of the screen
        Expanded(
          child: Container(
            margin: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(12),
              boxShadow: [
                BoxShadow(
                  color: Colors.black.withValues(alpha: 0.1),
                  blurRadius: 10,
                  offset: const Offset(0, 4),
                ),
              ],
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(12),
              child: isPdf 
                  ? Container(
                      color: Colors.grey[100],
                      child: Center(
                        child: Column(
                          mainAxisAlignment: MainAxisAlignment.center,
                          children: [
                            Icon(
                              Icons.picture_as_pdf,
                              size: 100,
                              color: Colors.red[600],
                            ),
                            const SizedBox(height: 24),
                            Text(
                              _selectedPdf!.name,
                              style: const TextStyle(
                                fontSize: 18,
                                fontWeight: FontWeight.w600,
                              ),
                              textAlign: TextAlign.center,
                            ),
                            const SizedBox(height: 8),
                            Text(
                              '${(_selectedPdf!.size / 1024).toStringAsFixed(1)} KB',
                              style: TextStyle(
                                fontSize: 14,
                                color: Colors.grey[600],
                              ),
                            ),
                          ],
                        ),
                      ),
                    )
                  : Image.memory(
                      _imageBytes!,
                      fit: BoxFit.contain,
                    ),
            ),
          ),
        ),
        
        // Action buttons
        Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Upload button - primary action
              SizedBox(
                height: 64,
                child: ElevatedButton.icon(
                  onPressed: _isUploading ? null : _uploadImage,
                  icon: _isUploading
                      ? const SizedBox(
                          width: 24,
                          height: 24,
                          child: CircularProgressIndicator(
                            color: Colors.white,
                            strokeWidth: 3,
                          ),
                        )
                      : const Icon(Icons.cloud_upload, size: 28),
                  label: Text(
                    _isUploading ? 'A enviar...' : (isPdf ? 'Enviar PDF' : 'Enviar Foto'),
                    style: const TextStyle(fontSize: 20),
                  ),
                ),
              ),
              const SizedBox(height: 12),
              
              // Cancel button
              TextButton.icon(
                onPressed: _isUploading ? null : _clearImage,
                icon: const Icon(Icons.close),
                label: const Text('Cancelar'),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
