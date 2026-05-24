// Placeholder smoke test for the Dart example workspace.
//
// We can't yet import `package:demo/demo.dart` because the generated
// bindings still depend on unimplemented enum codegen. This test exists
// to prove the sandboxed Dart SDK + `dart test` harness is wired up;
// once enum support lands (Phase C), real coverage can replace this.

import 'package:test/test.dart';

void main() {
  test('toolchain wired up', () {
    expect(1 + 1, 2);
  });
}
