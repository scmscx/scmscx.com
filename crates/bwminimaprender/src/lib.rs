mod minimap;
#[cfg(test)]
mod test;

pub use minimap::calculate_perceptual_hash;
pub use minimap::render_minimap;
