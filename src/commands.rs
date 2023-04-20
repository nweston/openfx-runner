pub enum Command {
    /// Load a bundle, find and describe a plugin.
    /// Calls Load and Describe actions
    CreatePlugin {
        bundle_name: String,
        plugin_name: String,
    },
    /// Create an instance of a plugin with the Filter context.
    /// Calls DescribeInContext and CreateInstance actions.
    CreateFilter {
        plugin_name: String,
        instance_name: String,
    },
    /// Render a single frame with a filter instance.
    RenderFilter {
        instance_name: String,
        input_file: String,
        output_file: String,
    },
    /// Destroy an effect instance.
    DestroyInstance { instance_name: String },
    /// Unload a plugin/bundle.
    UnloadPlugin { plugin_name: String },
}
