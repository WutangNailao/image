import 'dart:io';
import 'package:image/image.dart';
import 'package:image/src/native/transform_backend.dart';
import 'package:test/test.dart';

import '../_test_util.dart';

void main() {
  group('Filter', () {
    test('convolution', () {
      final bytes = File('test/_data/png/buck_24.png').readAsBytesSync();
      final i0 = decodePng(bytes)!;
      // sharpening kernel
      /*const filter = [ 0, -1,  0,
                      -1,  5, -1,
                       0, -1,  0 ];*/
      // laplacian kernel
      const filter = [0, 1, 0, 1, -4, 1, 0, 1, 0];
      convolution(i0, filter: filter, div: 1, offset: 0);
      File('$testOutputPath/filter/convolution.png')
        ..createSync(recursive: true)
        ..writeAsBytesSync(encodePng(i0));
    });

    test('convolution with mask matches Dart fallback', () {
      if (!nativeImageBackendAvailable) {
        return;
      }

      final src = Image(width: 5, height: 4, numChannels: 4);
      for (final p in src) {
        p
          ..r = p.x * 30 + p.y * 7
          ..g = p.x * 11 + p.y * 40
          ..b = 200 - p.x * 13 - p.y * 9
          ..a = 255;
      }

      final mask = Image(width: 6, height: 5, numChannels: 4);
      for (final p in mask) {
        p
          ..r = p.x * 20
          ..g = p.y * 40
          ..b = 255 - p.x * 10
          ..a = 255;
      }

      const filter = [0, -1, 0, -1, 5, -1, 0, -1, 0];
      final expected = Image.from(src);
      final actual = Image.from(src);
      final previousMode = imageBackendMode;

      try {
        imageBackendMode = ImageBackendMode.dartOnly;
        convolution(
          expected,
          filter: filter,
          div: 1,
          offset: 3,
          amount: 0.65,
          mask: mask,
          maskChannel: Channel.luminance,
        );

        imageBackendMode = ImageBackendMode.nativeOnly;
        convolution(
          actual,
          filter: filter,
          div: 1,
          offset: 3,
          amount: 0.65,
          mask: mask,
          maskChannel: Channel.luminance,
        );
      } finally {
        imageBackendMode = previousMode;
      }

      testImageEquals(expected, actual);
    });
  });
}
