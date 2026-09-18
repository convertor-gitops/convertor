use crate::error::RenderError;
pub(crate) mod clash_renderer;
pub(crate) mod surge_renderer;
pub const INDENT: usize = 4;
pub trait Render<F> {
    fn render(&self, content: &mut String) -> Result<(), RenderError>;
}
