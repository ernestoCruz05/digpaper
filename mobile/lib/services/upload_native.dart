/// Native (mobile) upload implementation - stub for conditional import
import 'dart:async';
import 'dart:typed_data';
import 'package:http/http.dart' as http;
import 'dart:convert';
import 'package:image_picker/image_picker.dart';
import '../models/document.dart';
import 'api_service.dart';

/// Upload file using http package (works on native platforms)
Future<ApiResult<Document>> uploadFileWeb(XFile file, Uint8List bytes) async {
  try {
    final uri = Uri.parse('${ApiConfig.apiUrl}/upload');
    final boundary = '----FlutterFormBoundary${DateTime.now().millisecondsSinceEpoch}';
    
    // Build multipart body manually
    final List<int> body = [];
    body.addAll('--$boundary\r\n'.codeUnits);
    body.addAll('Content-Disposition: form-data; name="file"; filename="${file.name}"\r\n'.codeUnits);
    body.addAll('Content-Type: application/octet-stream\r\n\r\n'.codeUnits);
    body.addAll(bytes);
    body.addAll('\r\n--$boundary--\r\n'.codeUnits);
    
    final response = await http.post(
      uri,
      headers: {
        'Content-Type': 'multipart/form-data; boundary=$boundary',
      },
      body: body,
    ).timeout(ApiConfig.timeout);

    if (response.statusCode == 201) {
      final doc = Document.fromUploadJson(json.decode(response.body));
      return ApiResult.success(doc);
    } else {
      return ApiResult.failure('Upload failed: ${response.statusCode}');
    }
  } catch (e) {
    return ApiResult.failure('Upload error: $e');
  }
}
