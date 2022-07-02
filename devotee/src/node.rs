/// `Node` is basic building block of `devotee`.
/// Every app has one `Node` as root and may contain other inner nodes.
pub trait Node {
    /// The update context provided to this `Node` during update call.
    type Update;
    /// The render context provided to this `Node` during render call.
    type Render;

    /// Update this node mutably.
    fn update(&mut self, update: &mut Self::Update);

    /// Perform render with this node.
    fn render(&self, render: &mut Self::Render);
}
