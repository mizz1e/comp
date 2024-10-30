use {
    smithay::backend::{drm::DrmNode, udev},
    std::path::PathBuf,
};

/// Open a DRM render node with render nodes.
fn open_drm_node(path: PathBuf) -> Option<DrmNode> {
    let drm_node = DrmNode::from_path(&path).ok()?;

    drm_node.has_render().then_some(drm_node)
}

/// Find the primary DRM node with render nodes.
fn find_primary_gpu(seat_name: &str) -> Option<DrmNode> {
    udev::primary_gpu(seat_name)
        .into_iter()
        .flatten()
        .find_map(open_drm_node)
}

/// Find a fallback DRM node with render nodes.
fn find_fallback_gpu(seat_name: &str) -> Option<DrmNode> {
    udev::all_gpus(seat_name)
        .into_iter()
        .flatten()
        .find_map(open_drm_node)
}

/// Find the best DRM node with render nodes.
pub fn find_best_gpu(seat_name: &str) -> Option<DrmNode> {
    find_primary_gpu(seat_name).or_else(|| find_fallback_gpu(seat_name))
}
