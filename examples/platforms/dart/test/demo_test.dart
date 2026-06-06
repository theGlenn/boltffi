// Smoke tests for the generated Dart bindings.
//
// These exercise the public surface that codegen produces today: C-style
// enums. Free functions still live in a private FFI class (Phase D) and
// data/error enums are placeholders (Phase C.2), so coverage there comes
// later. Importing `package:demo/demo.dart` at all is itself a regression
// check — it only compiles once enum codegen emits valid Dart.

import 'package:test/test.dart';
import 'package:demo/demo.dart';

void main() {
  group('C-style enums', () {
    test('expose declared discriminants as values', () {
      expect(Priority.low.value, 0);
      expect(Priority.medium.value, 1);
      expect(Priority.high.value, 2);
      expect(Priority.critical.value, 3);
    });

    test('fromValue maps a discriminant back to its variant', () {
      expect(Priority.fromValue(0), Priority.low);
      expect(Priority.fromValue(3), Priority.critical);
      expect(LogLevel.fromValue(4), LogLevel.error);
    });

    test('fromValue throws on an unknown discriminant', () {
      expect(() => Priority.fromValue(99), throwsA(isA<Object>()));
    });

    test('preserve non-ordinal discriminants', () {
      // HttpCode and Sign use discriminants that differ from their ordinals,
      // proving codegen keys on the declared value rather than position.
      expect(HttpCode.ok.value, 200);
      expect(HttpCode.notFound.value, 404);
      expect(HttpCode.serverError.value, 500);
      expect(Sign.negative.value, -1);
      expect(Sign.zero.value, 0);
      expect(Sign.positive.value, 1);
    });

    test('fromValue round-trips through value for every variant', () {
      for (final variant in Priority.values) {
        expect(Priority.fromValue(variant.value), variant);
      }
      for (final variant in HttpCode.values) {
        expect(HttpCode.fromValue(variant.value), variant);
      }
    });
  });

  group('scalar function wrappers (native FFI)', () {
    test('primitive in / primitive out', () {
      expect(add(2, 3), 5);
      expect(addI32(-4, 10), 6);
      expect(multiply(1.5, 4.0), 6.0);
      expect(echoBool(true), true);
      expect(negateBool(true), false);
      expect(echoI32(42), 42);
      expect(echoF64(2.71828), 2.71828);
    });

    test('void function executes without throwing', () {
      expect(noop, returnsNormally);
    });

    test('C-style enum in / enum out round-trips through native', () {
      expect(echoPriority(Priority.high), Priority.high);
      expect(echoLogLevel(LogLevel.warn), LogLevel.warn);
      expect(echoHttpCode(HttpCode.notFound), HttpCode.notFound);
      expect(echoSign(Sign.negative), Sign.negative);
    });

    test('enum in / primitive out reflects native logic', () {
      expect(isHighPriority(Priority.critical), true);
      expect(isHighPriority(Priority.low), false);
      expect(shouldLog(LogLevel.error, LogLevel.info), true);
      expect(shouldLog(LogLevel.trace, LogLevel.warn), false);
    });

    test('enum-returning constructors', () {
      expect(httpCodeNotFound(), HttpCode.notFound);
      expect(signNegative(), Sign.negative);
    });
  });
}
