import '../image/image.dart';
import '../image/interpolation.dart';

bool get nativeImageBackendAvailable => false;

Image? tryNativeCopyResize(
  Image src, {
  required int width,
  required int height,
  required Interpolation interpolation,
}) => null;

Image? tryNativeCopyCrop(
  Image src, {
  required int x,
  required int y,
  required int width,
  required int height,
}) => null;

bool tryNativeGaussianBlur(Image src, {required int radius}) => false;

bool tryNativeConvolution(
  Image src, {
  required List<num> filter,
  required num div,
  required num offset,
  required num amount,
  Image? mask,
  int? maskChannel,
}) => false;

bool tryNativeSeparableConvolution(
  Image src, {
  required List<num> coefficients,
  Image? mask,
  int? maskChannel,
}) => false;
