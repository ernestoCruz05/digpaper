/// API Service for DigPaper backend communication
/// 
/// This service handles all HTTP communication with the Rust backend.
/// It uses a singleton pattern for easy access throughout the app.

import 'dart:convert';
import 'dart:io';
import 'package:http/http.dart' as http;
import '../models/project.dart';
import '../models/document.dart';

/// API configuration
class ApiConfig {
  // ============================================
  // CHANGE THIS TO YOUR SERVER URL FOR PRODUCTION
  // ============================================
  static const String baseUrl = 'https://digpaper.faky.dev';
  
  // For local development, use one of these:
  // static const String baseUrl = 'http://10.0.2.2:3000';  // Android emulator
  // static const String baseUrl = 'http://localhost:3000'; // iOS simulator
  // static const String baseUrl = 'http://192.168.1.x:3000'; // Real device on LAN
  
  // Timeout for API calls - generous for slow networks
  static const Duration timeout = Duration(seconds: 30);
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
          .get(Uri.parse('${ApiConfig.baseUrl}/projects?status=active'))
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
          .get(Uri.parse('${ApiConfig.baseUrl}/projects'))
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

  /// Upload a file to the server
  /// 
  /// Streams the file to avoid loading it entirely in memory.
  /// Returns the created document record.
  Future<ApiResult<Document>> uploadFile(File file) async {
    try {
      final request = http.MultipartRequest(
        'POST',
        Uri.parse('${ApiConfig.baseUrl}/upload'),
      );

      // Add the file - Flutter's http package handles streaming
      request.files.add(await http.MultipartFile.fromPath(
        'file',
        file.path,
      ));

      final streamedResponse = await request.send().timeout(ApiConfig.timeout);
      final response = await http.Response.fromStream(streamedResponse);

      if (response.statusCode == 201) {
        final doc = Document.fromUploadJson(json.decode(response.body));
        return ApiResult.success(doc);
      } else {
        return ApiResult.failure('Upload failed: ${response.statusCode}');
      }
    } catch (e) {
      return ApiResult.failure(_handleError(e));
    }
  }

  /// Fetch all documents in the inbox (unassigned)
  Future<ApiResult<List<Document>>> getInboxDocuments() async {
    try {
      final response = await _client
          .get(Uri.parse('${ApiConfig.baseUrl}/documents/inbox'))
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
          .get(Uri.parse('${ApiConfig.baseUrl}/projects/$projectId/documents'))
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
            Uri.parse('${ApiConfig.baseUrl}/documents/$documentId/assign'),
            headers: {'Content-Type': 'application/json'},
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
            Uri.parse('${ApiConfig.baseUrl}/projects'),
            headers: {'Content-Type': 'application/json'},
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
            Uri.parse('${ApiConfig.baseUrl}/projects/$projectId/status'),
            headers: {'Content-Type': 'application/json'},
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
    if (error is SocketException) {
      return 'Sem ligação ao servidor. Verifique a sua internet.';
    } else if (error is HttpException) {
      return 'Erro de comunicação com o servidor.';
    } else if (error is FormatException) {
      return 'Resposta inválida do servidor.';
    } else {
      return 'Erro: ${error.toString()}';
    }
  }
}
