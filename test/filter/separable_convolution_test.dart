import 'dart:io';
import 'dart:math';

import 'package:image/image.dart';
import 'package:image/src/native/transform_backend.dart';
import 'package:test/test.dart';

import '../_test_util.dart';

void main() {
  group('Filter', () {
    test('separableConvolution', () {
      final bytes = File('test/_data/png/buck_24.png').readAsBytesSync();
      final i0 = decodePng(bytes)!;

      const radius = 5;
      final kernel = SeparableKernel(radius);
      // Compute coefficients
      const num sigma = radius * (2.0 / 3.0);
      const num s = 2.0 * sigma * sigma;

      num sum = 0.0;
      for (var x = -radius; x <= radius; ++x) {
        final num c = exp(-(x * x) / s);
        sum += c;
        kernel[x + radius] = c;
      }
      // Normalize the coefficients
      kernel.scaleCoefficients(1.0 / sum);

      separableConvolution(i0, kernel: kernel);

      File('$testOutputPath/filter/separableConvolution.png')
        ..createSync(recursive: true)
        ..writeAsBytesSync(encodePng(i0));
    });

    test('separableConvolution with mask matches Dart fallback', () {
      if (!nativeImageBackendAvailable) {
        return;
      }

      final src = Image(width: 5, height: 4, numChannels: 4);
      for (final p in src) {
        p
          ..r = p.x * 20 + p.y * 11
          ..g = p.x * 17 + p.y * 30
          ..b = 220 - p.x * 15 - p.y * 7
          ..a = 255;
      }

      final mask = Image(width: 6, height: 5, numChannels: 4);
      for (final p in mask) {
        p
          ..r = p.x * 25
          ..g = p.y * 45
          ..b = 200 - p.x * 10
          ..a = 255;
      }

      final kernel = SeparableKernel(1)
        ..[0] = 0.25
        ..[1] = 0.5
        ..[2] = 0.25;

      final expected = Image.from(src);
      final actual = Image.from(src);
      final previousMode = imageBackendMode;

      try {
        imageBackendMode = ImageBackendMode.dartOnly;
        separableConvolution(
          expected,
          kernel: kernel,
          mask: mask,
          maskChannel: Channel.luminance,
        );

        imageBackendMode = ImageBackendMode.nativeOnly;
        separableConvolution(
          actual,
          kernel: kernel,
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
