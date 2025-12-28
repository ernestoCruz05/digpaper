/// Document model
/// 
/// Represents an uploaded file (photo, PDF, etc.) in the system.
/// Documents can be in the inbox (no project) or assigned to a project.

import '../services/api_service.dart';

class Document {
  final String id;
  final String? projectId;
  final String filePath;
  final String fileType;
  final String originalName;
  final String? uploadedAt;
  final String fileUrl;

  Document({
    required this.id,
    this.projectId,
    required this.filePath,
    required this.fileType,
    required this.originalName,
    this.uploadedAt,
    required this.fileUrl,
  });

  /// Convert relative URL to absolute URL using the API base URL
  static String _resolveFileUrl(String url) {
    // If already absolute, return as-is
    if (url.startsWith('http://') || url.startsWith('https://')) {
      return url;
    }
    // Prepend the base URL for relative paths
    return '${ApiConfig.baseUrl}$url';
  }

  /// Parse from full Document response (inbox, project documents)
  factory Document.fromJson(Map<String, dynamic> json) {
    return Document(
      id: json['id'] as String,
      projectId: json['project_id'] as String?,
      filePath: json['file_path'] as String,
      fileType: json['file_type'] as String,
      originalName: json['original_name'] as String,
      uploadedAt: json['uploaded_at'] as String?,
      fileUrl: _resolveFileUrl(json['file_url'] as String),
    );
  }

  /// Parse from UploadResponse (doesn't have uploaded_at)
  factory Document.fromUploadJson(Map<String, dynamic> json) {
    return Document(
      id: json['id'] as String,
      projectId: null,
      filePath: json['file_path'] as String,
      fileType: json['file_type'] as String,
      originalName: json['original_name'] as String,
      uploadedAt: null,
      fileUrl: _resolveFileUrl(json['file_url'] as String),
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'project_id': projectId,
      'file_path': filePath,
      'file_type': fileType,
      'original_name': originalName,
      'uploaded_at': uploadedAt,
      'file_url': fileUrl,
    };
  }

  /// Check if document is in inbox (not assigned to any project)
  bool get isInInbox => projectId == null;

  /// Check if document is an image (by type or file extension)
  bool get isImage {
    if (fileType == 'image') return true;
    // Fallback: check file extension for legacy uploads
    final ext = originalName.split('.').last.toLowerCase();
    return ['jpg', 'jpeg', 'png', 'gif', 'webp', 'heic', 'heif'].contains(ext);
  }

  /// Check if document is a PDF
  bool get isPdf => fileType == 'pdf' || originalName.toLowerCase().endsWith('.pdf');

  /// Format the upload date for display
  String get formattedDate {
    if (uploadedAt == null) return 'Agora';
    try {
      final date = DateTime.parse(uploadedAt!);
      return '${date.day}/${date.month}/${date.year} ${date.hour}:${date.minute.toString().padLeft(2, '0')}';
    } catch (_) {
      return uploadedAt!;
    }
  }
}
