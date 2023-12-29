use crate::ParamValue;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
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
    /// Print params of an effect instance.
    PrintParams { instance_name: String },
    /// Destroy an effect instance.
    DestroyInstance { instance_name: String },
    /// Unload a plugin/bundle.
    UnloadPlugin { plugin_name: String },
    /// Set parameter values on an instance. Optionally call
    /// BeginInstanceChanged, InstanceChanged, and EndInstanceChanged.
    SetParams {
        instance_name: String,
        values: Vec<(String, ParamValue)>,
        call_instance_changed: bool,
    },
    /// List all plugins in a bundle
    ListPlugins { bundle_name: String },
}
