/// Document model
/// 
/// Represents an uploaded file (photo, PDF, etc.) in the system.
/// Documents can be in the inbox (no project) or assigned to a project.

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

  /// Parse from full Document response (inbox, project documents)
  factory Document.fromJson(Map<String, dynamic> json) {
    return Document(
      id: json['id'] as String,
      projectId: json['project_id'] as String?,
      filePath: json['file_path'] as String,
      fileType: json['file_type'] as String,
      originalName: json['original_name'] as String,
      uploadedAt: json['uploaded_at'] as String?,
      fileUrl: json['file_url'] as String,
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
      fileUrl: json['file_url'] as String,
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

  /// Check if document is an image
  bool get isImage => fileType == 'image';

  /// Check if document is a PDF
  bool get isPdf => fileType == 'pdf';

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
