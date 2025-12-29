/// Web-specific upload implementation using XMLHttpRequest
import 'dart:async';
import 'dart:convert';
// ignore: avoid_web_libraries_in_flutter
import 'dart:html' as html;
import 'dart:typed_data';
import 'package:image_picker/image_picker.dart';
import '../models/document.dart';
import 'api_service.dart';

/// Upload file using browser's XMLHttpRequest (works on web)
Future<ApiResult<Document>> uploadFileWeb(XFile file, Uint8List bytes) async {
  try {
    final completer = Completer<ApiResult<Document>>();
    
    // Get MIME type from file, default to octet-stream
    final mimeType = file.mimeType ?? _getMimeTypeFromExtension(file.name);
    
    final formData = html.FormData();
    // Create blob with explicit MIME type so server detects it correctly
    final blob = html.Blob([bytes], mimeType);
    formData.appendBlob('file', blob, file.name);
    
    final xhr = html.HttpRequest();
    xhr.open('POST', '${ApiConfig.apiUrl}/upload');
    xhr.setRequestHeader('X-API-Key', ApiConfig.apiKey);
    
    xhr.onLoad.listen((event) {
      if (xhr.status == 201) {
        final doc = Document.fromUploadJson(json.decode(xhr.responseText!));
        completer.complete(ApiResult.success(doc));
      } else {
        completer.complete(ApiResult.failure('Upload failed: ${xhr.status}'));
      }
    });
    
    xhr.onError.listen((event) {
      completer.complete(ApiResult.failure('Network error during upload'));
    });
    
    xhr.send(formData);
    
    return completer.future;
  } catch (e) {
    return ApiResult.failure('Upload error: $e');
  }
}

/// Get MIME type from file extension
String _getMimeTypeFromExtension(String filename) {
  final ext = filename.split('.').last.toLowerCase();
  switch (ext) {
    case 'jpg':
    case 'jpeg':
      return 'image/jpeg';
    case 'png':
      return 'image/png';
    case 'gif':
      return 'image/gif';
    case 'webp':
      return 'image/webp';
    case 'heic':
    case 'heif':
      return 'image/heic';
    case 'pdf':
      return 'application/pdf';
    case 'mp4':
      return 'video/mp4';
    case 'mov':
      return 'video/quicktime';
    default:
      return 'application/octet-stream';
  }
}
