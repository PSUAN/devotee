/// `Node` is basic building block of `devotee`.
/// Every app has one `Node` as root and may contain other inner nodes.
pub trait Node<U, R> {
    /// Update this node mutably.
    fn update(&mut self, update: U);

    /// Perform render with this node.
    fn render(&self, render: R);
}
