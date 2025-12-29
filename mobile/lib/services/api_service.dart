/// API Service for DigPaper backend communication
/// 
/// This service handles all HTTP communication with the Rust backend.
/// It uses a singleton pattern for easy access throughout the app.

import 'dart:convert';
import 'dart:typed_data';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:http/http.dart' as http;
import 'package:image_picker/image_picker.dart';
import '../models/project.dart';
import '../models/document.dart';

// Conditional import for web vs native upload
import 'upload_native.dart' if (dart.library.html) 'upload_web.dart' as uploader;

/// API configuration
class ApiConfig {
  /// Get the base URL based on the platform
  /// - Web: Uses relative URL (same origin as the web app)
  /// - Native: Uses the production server URL
  static String get baseUrl {
    if (kIsWeb) {
      // Web app is served from the same server, use relative URLs
      return '';
    }
    
    // For native apps (Android/iOS), always use the production URL
    return 'https://digpaper.faky.dev';
  }
  
  /// API prefix - all API routes are under /api
  static const String apiPrefix = '/api';
  
  /// Full API URL
  static String get apiUrl => '$baseUrl$apiPrefix';
  
  // Timeout for API calls - generous for slow networks
  static const Duration timeout = Duration(seconds: 30);
  
  /// API key for authentication - hardcoded for native apps
  /// In production, this should be securely stored
  static const String apiKey = 'Carpintaria1.';
  
  /// Get headers with authentication
  static Map<String, String> get headers => {
    'X-API-Key': apiKey,
  };
  
  /// Get headers with authentication and JSON content type
  static Map<String, String> get jsonHeaders => {
    'X-API-Key': apiKey,
    'Content-Type': 'application/json',
  };
}

/// Result wrapper for API calls
/// Makes error handling explicit and type-safe
class ApiResult<T> {
  final T? data;
  final String? error;
  
  ApiResult.success(this.data) : error = null;
  ApiResult.failure(this.error) : data = null;
  
  bool get isSuccess => error == null;
  bool get isFailure => error != null;
}

/// Main API service class
class ApiService {
  static final ApiService _instance = ApiService._internal();
  factory ApiService() => _instance;
  ApiService._internal();

  final http.Client _client = http.Client();

  /// Fetch all active projects (for dropdown selection)
  /// 
  /// Used in the upload flow when user selects which project
  /// the photo belongs to.
  Future<ApiResult<List<Project>>> getActiveProjects() async {
    try {
      final response = await _client
          .get(
            Uri.parse('${ApiConfig.apiUrl}/projects?status=active'),
            headers: ApiConfig.headers,
          )
          .timeout(ApiConfig.timeout);

      if (response.statusCode == 200) {
        final List<dynamic> jsonList = json.decode(response.body);
        final projects = jsonList.map((j) => Project.fromJson(j)).toList();
        return ApiResult.success(projects);
      } else {
        return ApiResult.failure('Failed to load projects: ${response.statusCode}');
      }
    } catch (e) {
      return ApiResult.failure(_handleError(e));
    }
  }

  /// Fetch all projects (for projects list screen)
  Future<ApiResult<List<Project>>> getAllProjects() async {
    try {
      final response = await _client
          .get(
            Uri.parse('${ApiConfig.apiUrl}/projects'),
            headers: ApiConfig.headers,
          )
          .timeout(ApiConfig.timeout);

      if (response.statusCode == 200) {
        final List<dynamic> jsonList = json.decode(response.body);
        final projects = jsonList.map((j) => Project.fromJson(j)).toList();
        return ApiResult.success(projects);
      } else {
        return ApiResult.failure('Failed to load projects: ${response.statusCode}');
      }
    } catch (e) {
      return ApiResult.failure(_handleError(e));
    }
  }

  /// Upload a file to the server (cross-platform)
  /// 
  /// Works on both web and native platforms using bytes.
  /// Returns the created document record.
  Future<ApiResult<Document>> uploadXFile(XFile file, Uint8List bytes) async {
    // Use the platform-specific uploader
    return uploader.uploadFileWeb(file, bytes);
  }

  /// Fetch all documents in the inbox (unassigned)
  Future<ApiResult<List<Document>>> getInboxDocuments() async {
    try {
      final response = await _client
          .get(
            Uri.parse('${ApiConfig.apiUrl}/documents/inbox'),
            headers: ApiConfig.headers,
          )
          .timeout(ApiConfig.timeout);

      if (response.statusCode == 200) {
        final List<dynamic> jsonList = json.decode(response.body);
        final docs = jsonList.map((j) => Document.fromJson(j)).toList();
        return ApiResult.success(docs);
      } else {
        return ApiResult.failure('Failed to load inbox: ${response.statusCode}');
      }
    } catch (e) {
      return ApiResult.failure(_handleError(e));
    }
  }

  /// Fetch documents for a specific project
  Future<ApiResult<List<Document>>> getProjectDocuments(String projectId) async {
    try {
      final response = await _client
          .get(
            Uri.parse('${ApiConfig.apiUrl}/projects/$projectId/documents'),
            headers: ApiConfig.headers,
          )
          .timeout(ApiConfig.timeout);

      if (response.statusCode == 200) {
        final List<dynamic> jsonList = json.decode(response.body);
        final docs = jsonList.map((j) => Document.fromJson(j)).toList();
        return ApiResult.success(docs);
      } else {
        return ApiResult.failure('Failed to load documents: ${response.statusCode}');
      }
    } catch (e) {
      return ApiResult.failure(_handleError(e));
    }
  }

  /// Assign a document to a project
  /// 
  /// Pass null for projectId to move back to inbox
  Future<ApiResult<Document>> assignDocument(String documentId, String? projectId) async {
    try {
      final response = await _client
          .patch(
            Uri.parse('${ApiConfig.apiUrl}/documents/$documentId/assign'),
            headers: ApiConfig.jsonHeaders,
            body: json.encode({'project_id': projectId}),
          )
          .timeout(ApiConfig.timeout);

      if (response.statusCode == 200) {
        final doc = Document.fromJson(json.decode(response.body));
        return ApiResult.success(doc);
      } else {
        return ApiResult.failure('Failed to assign document: ${response.statusCode}');
      }
    } catch (e) {
      return ApiResult.failure(_handleError(e));
    }
  }

  /// Create a new project
  Future<ApiResult<Project>> createProject(String name) async {
    try {
      final response = await _client
          .post(
            Uri.parse('${ApiConfig.apiUrl}/projects'),
            headers: ApiConfig.jsonHeaders,
            body: json.encode({'name': name}),
          )
          .timeout(ApiConfig.timeout);

      if (response.statusCode == 201) {
        final project = Project.fromJson(json.decode(response.body));
        return ApiResult.success(project);
      } else {
        return ApiResult.failure('Failed to create project: ${response.statusCode}');
      }
    } catch (e) {
      return ApiResult.failure(_handleError(e));
    }
  }

  /// Update project status (ACTIVE/ARCHIVED)
  /// 
  /// Use this to mark a project as complete/done or reactivate it
  Future<ApiResult<Project>> updateProjectStatus(String projectId, String status) async {
    try {
      final response = await _client
          .patch(
            Uri.parse('${ApiConfig.apiUrl}/projects/$projectId/status'),
            headers: ApiConfig.jsonHeaders,
            body: json.encode({'status': status}),
          )
          .timeout(ApiConfig.timeout);

      if (response.statusCode == 200) {
        final project = Project.fromJson(json.decode(response.body));
        return ApiResult.success(project);
      } else {
        return ApiResult.failure('Failed to update status: ${response.statusCode}');
      }
    } catch (e) {
      return ApiResult.failure(_handleError(e));
    }
  }

  /// Convert exceptions to user-friendly messages
  String _handleError(dynamic error) {
    final errorStr = error.toString().toLowerCase();
    
    // Network-related errors
    if (errorStr.contains('socket') || 
        errorStr.contains('connection') ||
        errorStr.contains('network') ||
        errorStr.contains('failed host lookup')) {
      return 'Sem ligação ao servidor. Verifique a sua internet.';
    } else if (errorStr.contains('timeout')) {
      return 'O servidor demorou muito a responder.';
    } else if (errorStr.contains('format')) {
      return 'Resposta inválida do servidor.';
    } else {
      return 'Erro: ${error.toString()}';
    }
  }
}
