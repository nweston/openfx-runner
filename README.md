# OpenFX Image Effect Plugin Test Harness

A minimal OpenFX host, suitable for automated testing of plugins.

## Goals
1. Memory safe
2. Lightweight, adding minimal overhead to the plugin code itself
3. Scriptable

Taken together, these qualities should make it feasible to test
plugins with dynamic analysis tools such as valgrind, with reasonable
performance and without false positives in the host code.

## Style Guidelines
Names should match the OpenFX API to avoid confusion and allow for
easy searching. This includes:
 - Names of types defined by the API
 - Names of fields
 - Parameter names in suite functions
 
The OFX naming conventions are not idiomatic for Rust, so warnings are
disabled where appropriate.

Exceptions may be made to this rule in a couple of cases:
- The leading "k" should be omitted from constants
- To avoid repetition due to namespacing (i.e. module or enum names),
  especially for frequently-used identifiers

For example, OfxStatus::Ok instead of OfxStatus::kOfxStatOK.
