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
    
    final formData = html.FormData();
    final blob = html.Blob([bytes]);
    formData.appendBlob('file', blob, file.name);
    
    final xhr = html.HttpRequest();
    xhr.open('POST', '${ApiConfig.apiUrl}/upload');
    
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
