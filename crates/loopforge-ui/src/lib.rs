use floem::View;
use floem::views::{container, h_stack, label, v_stack};

pub fn monitor_shell() -> impl View {
    h_stack((
        container(label(|| "Sidebar")),
        v_stack((container(label(|| "Output")), container(label(|| "Diff")))),
    ))
}
