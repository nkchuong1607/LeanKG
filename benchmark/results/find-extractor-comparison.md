# Benchmark Comparison: find-extractor

## With LeanKG
- Total Tokens: 30890
- Input: 13735
- Cached: 16800
- Files Referenced: ["src/indexer/extractor.rs", "src/pkg/f.go", "src/tests/whatever_test.rs", "tests/integration_test.rs", "tests/whatever_test.rs", "pkg/math.go", "pkg/person.go", "pkg/io.go", "pkg/f.go", "pkg/math_test.go", "pkg/math_wrong.go", "pkg/math_test.rs", "pkg/math.rs"]

## Without LeanKG
- Total Tokens: 30821
- Input: 13825
- Cached: 16512
- Files Referenced: ["src/indexer/extractor.rs", "src/pkg/f.go", "src/tests/whatever_test.rs", "tests/integration_test.rs", "tests/whatever_test.rs", "pkg/math.go", "pkg/person.go", "pkg/io.go", "pkg/f.go", "pkg/math_test.go", "pkg/math_wrong.go", "pkg/math_test.rs", "pkg/math.rs"]

## Overhead
- Token Delta: 69
