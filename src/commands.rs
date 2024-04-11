use crate::types::{OfxRectD, OfxRectI};
use crate::{FrameNumber, ParamValue};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RenderLayout {
    pub project_dims: (f64, f64),
    pub input_origin: (i32, i32),
    pub render_window: OfxRectI,
}

fn default_frame_range() -> (FrameNumber, FrameNumber) {
    (FrameNumber(0), FrameNumber(1))
}

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
        output_directory: String,
        layout: Option<RenderLayout>,
        #[serde(default = "default_frame_range")]
        frame_range: (FrameNumber, FrameNumber),
        #[serde(default)]
        thread_count: u32,
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
    /// Describe plugin and print results
    Describe {
        bundle_name: String,
        plugin_name: String,
    },
    /// Describe plugin in filter context and print results
    DescribeFilter {
        bundle_name: String,
        plugin_name: String,
    },
    PrintRoIs {
        instance_name: String,
        region_of_interest: OfxRectD,
        project_extent: (f64, f64),
    },
    PrintRoD {
        instance_name: String,
        input_rod: OfxRectD,
        project_extent: (f64, f64),
    },
}
