import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';

import '../color/format.dart';
import '../image/image.dart';
import '../image/interpolation.dart';
import 'bindings.dart';
import 'transform_backend.dart' show createNativeImageFromRgba;

const _imageOk = 0;

bool? _nativeBackendAvailable;
final Expando<Uint8List> _retainedNativeImageBytes = Expando<Uint8List>(
  'nativeImageBytes',
);
final Pointer<NativeFunction<Void Function(Pointer<Void>)>>
_imageFreeBufferPointer = Native.addressOf(image_free_buffer);

bool get nativeImageBackendAvailable {
  if (!_supportsNativePlatform) {
    return false;
  }
  try {
    return _nativeBackendAvailable ??= _probeNativeBackend();
  } catch (_) {
    return false;
  }
}

Image? tryNativeCopyResize(
  Image src, {
  required int width,
  required int height,
  required Interpolation interpolation,
}) {
  if (!nativeImageBackendAvailable ||
      width <= 0 ||
      height <= 0 ||
      src.hasPalette ||
      src.hasAnimation ||
      (src.hasExif && src.exif.imageIfd.hasOrientation) ||
      interpolation == Interpolation.average ||
      interpolation == Interpolation.cubic) {
    return null;
  }

  final prepared = _prepareImage(src);
  if (prepared == null) {
    return null;
  }

  final input = malloc<Uint8>(prepared.length);
  final sourceBytes = prepared.toUint8List();
  input.asTypedList(sourceBytes.length).setAll(0, sourceBytes);
  try {
    final result = image_resize_rgba8(
      input,
      prepared.width,
      prepared.height,
      prepared.numChannels,
      width,
      height,
      interpolation.index,
    );
    return _materializeImageResult(result, original: src);
  } finally {
    malloc.free(input);
  }
}

Image? tryNativeCopyCrop(
  Image src, {
  required int x,
  required int y,
  required int width,
  required int height,
}) {
  if (!nativeImageBackendAvailable ||
      width <= 0 ||
      height <= 0 ||
      src.hasPalette ||
      src.hasAnimation) {
    return null;
  }

  final prepared = _prepareImage(src);
  if (prepared == null) {
    return null;
  }

  final clampedX = x.clamp(0, prepared.width - 1).toInt();
  final clampedY = y.clamp(0, prepared.height - 1).toInt();
  var clampedWidth = width;
  var clampedHeight = height;
  if (clampedX + clampedWidth > prepared.width) {
    clampedWidth = prepared.width - clampedX;
  }
  if (clampedY + clampedHeight > prepared.height) {
    clampedHeight = prepared.height - clampedY;
  }
  if (clampedWidth <= 0 || clampedHeight <= 0) {
    return null;
  }

  final input = malloc<Uint8>(prepared.length);
  final sourceBytes = prepared.toUint8List();
  input.asTypedList(sourceBytes.length).setAll(0, sourceBytes);
  try {
    final result = image_crop_rgba8(
      input,
      prepared.width,
      prepared.height,
      prepared.numChannels,
      clampedX,
      clampedY,
      clampedWidth,
      clampedHeight,
    );
    return _materializeImageResult(result, original: src);
  } finally {
    malloc.free(input);
  }
}

bool tryNativeGaussianBlur(Image src, {required int radius}) {
  if (!nativeImageBackendAvailable ||
      radius <= 0 ||
      src.hasPalette ||
      src.hasAnimation) {
    return false;
  }

  final prepared = _prepareImage(src);
  if (prepared == null || !_canWriteBackToSource(src, prepared)) {
    return false;
  }

  final input = malloc<Uint8>(prepared.length);
  final sourceBytes = prepared.toUint8List();
  input.asTypedList(sourceBytes.length).setAll(0, sourceBytes);
  try {
    final result = image_gaussian_blur_rgba8(
      input,
      prepared.width,
      prepared.height,
      prepared.numChannels,
      radius,
    );
    final blurred = _materializeImageResult(result, original: src);
    if (blurred == null ||
        blurred.width != src.width ||
        blurred.height != src.height ||
        src.lengthInBytes != blurred.lengthInBytes) {
      return false;
    }
    src.toUint8List().setAll(0, blurred.toUint8List());
    return true;
  } finally {
    malloc.free(input);
  }
}

bool tryNativeConvolution(
  Image src, {
  required List<num> filter,
  required num div,
  required num offset,
  required num amount,
  Image? mask,
  int? maskChannel,
}) {
  if (!nativeImageBackendAvailable ||
      filter.length != 9 ||
      src.hasPalette ||
      src.hasAnimation ||
      div.isNaN ||
      div.isInfinite ||
      offset.isNaN ||
      offset.isInfinite ||
      amount.isNaN ||
      amount.isInfinite ||
      amount < 0 ||
      amount > 1) {
    return false;
  }

  for (final coefficient in filter) {
    if (coefficient.isNaN || coefficient.isInfinite) {
      return false;
    }
  }

  final prepared = _prepareImage(src);
  if (prepared == null || !_canWriteBackToSource(src, prepared)) {
    return false;
  }

  _PreparedImage? preparedMask;
  if (mask != null) {
    if (maskChannel == null) {
      return false;
    }
    if (maskChannel < 0 || maskChannel > 4) {
      return false;
    }
    preparedMask = _prepareImage(mask);
    if (preparedMask == null ||
        preparedMask.width < prepared.width ||
        preparedMask.height < prepared.height) {
      return false;
    }
  }

  final input = malloc<Uint8>(prepared.length);
  final filterPtr = malloc<Double>(9);
  final sourceBytes = prepared.toUint8List();
  Pointer<Uint8>? maskInput;
  input.asTypedList(sourceBytes.length).setAll(0, sourceBytes);
  for (var i = 0; i < 9; i++) {
    filterPtr[i] = filter[i].toDouble();
  }
  if (preparedMask != null) {
    final maskBytes = preparedMask.toUint8List();
    maskInput = malloc<Uint8>(maskBytes.length);
    maskInput.asTypedList(maskBytes.length).setAll(0, maskBytes);
  }

  try {
    final result = image_convolution_rgba8(
      input,
      prepared.width,
      prepared.height,
      prepared.numChannels,
      maskInput ?? nullptr,
      preparedMask?.width ?? 0,
      preparedMask?.height ?? 0,
      preparedMask?.numChannels ?? 0,
      maskChannel ?? -1,
      filterPtr,
      div.toDouble(),
      offset.toDouble(),
      amount.toDouble(),
    );
    final convolved = _materializeImageResult(result, original: src);
    if (convolved == null ||
        convolved.width != src.width ||
        convolved.height != src.height ||
        src.lengthInBytes != convolved.lengthInBytes) {
      return false;
    }
    src.toUint8List().setAll(0, convolved.toUint8List());
    return true;
  } finally {
    if (maskInput != null) {
      malloc.free(maskInput);
    }
    malloc
      ..free(filterPtr)
      ..free(input);
  }
}

bool tryNativeSeparableConvolution(
  Image src, {
  required List<num> coefficients,
  Image? mask,
  int? maskChannel,
}) {
  if (!nativeImageBackendAvailable ||
      coefficients.isEmpty ||
      coefficients.length.isEven ||
      src.hasPalette ||
      src.hasAnimation) {
    return false;
  }

  for (final coefficient in coefficients) {
    if (coefficient.isNaN || coefficient.isInfinite) {
      return false;
    }
  }

  final prepared = _prepareImage(src);
  if (prepared == null || !_canWriteBackToSource(src, prepared)) {
    return false;
  }

  _PreparedImage? preparedMask;
  if (mask != null) {
    if (maskChannel == null) {
      return false;
    }
    if (maskChannel < 0 || maskChannel > 4) {
      return false;
    }
    preparedMask = _prepareImage(mask);
    if (preparedMask == null ||
        preparedMask.width < prepared.width ||
        preparedMask.height < prepared.height) {
      return false;
    }
  }

  final input = malloc<Uint8>(prepared.length);
  final coefficientPtr = malloc<Double>(coefficients.length);
  final sourceBytes = prepared.toUint8List();
  Pointer<Uint8>? maskInput;
  input.asTypedList(sourceBytes.length).setAll(0, sourceBytes);
  for (var i = 0; i < coefficients.length; i++) {
    coefficientPtr[i] = coefficients[i].toDouble();
  }
  if (preparedMask != null) {
    final maskBytes = preparedMask.toUint8List();
    maskInput = malloc<Uint8>(maskBytes.length);
    maskInput.asTypedList(maskBytes.length).setAll(0, maskBytes);
  }

  try {
    final result = image_separable_convolution_rgba8(
      input,
      prepared.width,
      prepared.height,
      prepared.numChannels,
      maskInput ?? nullptr,
      preparedMask?.width ?? 0,
      preparedMask?.height ?? 0,
      preparedMask?.numChannels ?? 0,
      maskChannel ?? -1,
      coefficientPtr,
      coefficients.length,
    );
    final convolved = _materializeImageResult(result, original: src);
    if (convolved == null ||
        convolved.width != src.width ||
        convolved.height != src.height ||
        src.lengthInBytes != convolved.lengthInBytes) {
      return false;
    }
    src.toUint8List().setAll(0, convolved.toUint8List());
    return true;
  } finally {
    if (maskInput != null) {
      malloc.free(maskInput);
    }
    malloc.free(coefficientPtr);
    malloc.free(input);
  }
}

_PreparedImage? _prepareImage(Image src) {
  if (src.numFrames != 1) {
    return null;
  }
  var prepared = src;
  if (prepared.format != Format.uint8 || prepared.numChannels != 4) {
    prepared = prepared.convert(format: Format.uint8, numChannels: 4);
  }
  if (prepared.hasPalette || prepared.numChannels != 4) {
    return null;
  }
  return _PreparedImage(prepared);
}

Image? _materializeImageResult(ImageResult result, {required Image original}) {
  if (result.code != _imageOk ||
      result.buffer.data == nullptr ||
      result.buffer.release_handle == nullptr) {
    return null;
  }

  final length = result.buffer.stride * result.buffer.height;
  if (length <= 0) {
    image_free_buffer(result.buffer.release_handle);
    return null;
  }

  if (original.format == Format.uint8 && original.numChannels == 4) {
    final bytes = result.buffer.data.asTypedList(
      length,
      finalizer: _imageFreeBufferPointer,
      token: result.buffer.release_handle,
    );
    final image = createNativeImageFromRgba(
      template: original,
      bytes: bytes,
      width: result.buffer.width,
      height: result.buffer.height,
    );
    _retainedNativeImageBytes[image] = bytes;
    return image;
  }

  try {
    final bytes = result.buffer.data.asTypedList(length);
    final image = createNativeImageFromRgba(
      template: original,
      bytes: bytes,
      width: result.buffer.width,
      height: result.buffer.height,
    );
    return image.convert(
      format: original.format,
      numChannels: original.numChannels,
      noAnimation: true,
    );
  } finally {
    image_free_buffer(result.buffer.release_handle);
  }
}

bool get _supportsNativePlatform =>
    Platform.isAndroid ||
    Platform.isIOS ||
    Platform.isMacOS ||
    Platform.isLinux ||
    Platform.isWindows;

bool _probeNativeBackend() {
  image_last_error_message();
  return true;
}

bool _canWriteBackToSource(Image src, _PreparedImage prepared) =>
    identical(src, prepared.image) &&
    src.format == Format.uint8 &&
    src.numChannels == 4 &&
    !src.hasPalette &&
    !src.hasAnimation;

final class _PreparedImage {
  _PreparedImage(this.image);

  final Image image;

  int get width => image.width;
  int get height => image.height;
  int get numChannels => image.numChannels;
  int get length => image.lengthInBytes;

  Uint8List toUint8List() => image.toUint8List();
}
