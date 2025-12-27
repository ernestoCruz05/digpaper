/// Project model
/// 
/// Represents a work order (Obra) in the system.

class Project {
  final String id;
  final String name;
  final String status;
  final String createdAt;

  Project({
    required this.id,
    required this.name,
    required this.status,
    required this.createdAt,
  });

  factory Project.fromJson(Map<String, dynamic> json) {
    return Project(
      id: json['id'] as String,
      name: json['name'] as String,
      status: json['status'] as String,
      createdAt: json['created_at'] as String,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'status': status,
      'created_at': createdAt,
    };
  }

  /// Check if project is active
  bool get isActive => status == 'ACTIVE';

  /// Format the creation date for display
  String get formattedDate {
    try {
      final date = DateTime.parse(createdAt);
      return '${date.day}/${date.month}/${date.year}';
    } catch (_) {
      return createdAt;
    }
  }
}
