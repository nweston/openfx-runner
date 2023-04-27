# OpenFX Image Effect Plugin Runner

A scriptable host for OpenFX Image Effect Plugins, suitable for
automated testing.

## Goals
The primary goal is to support loading and rendering of well-behaved
plugins, with minimal overhead, while maintaining memory- and
thread-safety. In particular it should be possible to test plugins
using dynamic analysis tools such as valgrind, with reasonable
performance and without false positives in the host code.

Conformance checking of plugins is not a goal. For example, if plugins
attempt to set read-only properties or perform other illegal options,
this may or may not be detected or handled gracefully.

Support for overlays and other UI-specific aspects of the OFX API is
not planned.

## Current Status and Limitations
Basic rendering in the Filter context works and has been tested with
several different plugins. The implementation properties and suite
functions has been driven by testing and is not exhaustive, so you may
encounter unimplemented functions or missing properties.

Major limitations:
 - Single-threaded
 - Only supports still images
 - Only supports the Filter context
 - No time/timeline support. All clips/projects are one frame long and
   the current frame is always zero.

## Usage
```
cargo run <command-file>
```

`command-file` is a JSON file containing an array of commands to
execute. The following commands are supported:

### CreatePlugin
Load a bundle, find and describe a plugin. This must be done first as
all other commands will reference the loaded plugin.

```
{"type":"CreatePlugin",
 "bundle_name":"<name>",
 "plugin_name":"<name>"}
```

### CreateFilter
Create an instance of a loaded plugin with the Filter context. Assigns
a name to the instance which can be referenced by other commands.

```
{"type":"CreateFilter",
 "plugin_name":"<name>",
 "instance_name":"<name>"}
```

### RenderFilter
Render a single frame with a filter instance. Reads the input from an
EXR file, and writes the result to another EXR.

```
{"type":"RenderFilter",
 "instance_name":"<name>",
 "input_file":"<in>",
 "output_file":"<out>"}
```
    
### SetParams
Set parameter values on an instance. If `call_instance_changed` is
true, call the BeginInstanceChanged, InstanceChanged, and
EndInstanceChanged actions on the plugin. This should only be
necessary if the plugin is expected to respond to those actions. When
setting up param values for render, `call_instance_changed` can be
false -- this will change the stored parameter values without
interacting with the plugin directly.

```
{"type":"SetParams",
 "instance_name":"<name>",
 "values":[
    ["<param-name>",{"type":"<type>", "v": <value>}]]
 "call_instance_changed":<value>},
```

The "type" field is the parameter type without a prefix: e.g. for
`OfxParamTypeBoolean`, use "Boolean". The "value" field is either a
single value of the appropriate type (boolean, integer, or double), or
an array for multiple-valued types such as Double2D, RGB, etc.

Note that Custom and String params store CStrings, so their values are
represented in JSON as arrays of byte values.

### DestroyInstance
Destroy an effect instance.

```
{"type":"DestroyInstance",
 "instance_name":"<name>"}
```

### UnloadPlugin
Unload a plugin and its bundle.

```
{"type":"UnloadPlugin",
 "plugin_name":"<name>"}
```


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

