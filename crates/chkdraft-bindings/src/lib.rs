//! Rust bindings for Chkdraft map rendering library.
//!
//! This crate provides safe Rust wrappers around the Chkdraft C++ library
//! for rendering StarCraft maps to WebP images.
//!
//! # Example
//!
//! ```no_run
//! use chkdraft_bindings::{GfxUtil, RenderSkin, RenderOptions};
//!
//! fn main() -> Result<(), chkdraft_bindings::Error> {
//!     let mut gfx = GfxUtil::new()?;
//!     gfx.load_sc_data("/path/to/starcraft")?;
//!
//!     let renderer = gfx.create_renderer(RenderSkin::Classic)?;
//!     let mut map = gfx.load_map("/path/to/map.scx")?;
//!
//!     // Simulate animations (extends siege tanks, etc.)
//!     map.simulate_anim(52);
//!
//!     let options = RenderOptions::default();
//!     renderer.save_webp(&map, &options, "/path/to/output.webp")?;
//!
//!     Ok(())
//! }
//! ```

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

mod ffi {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::ffi::{CStr, CString};
use std::fmt;
use std::ptr;

/// Error type for Chkdraft operations.
#[derive(Debug, Clone)]
pub struct Error {
    pub code: i32,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chkdraft error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

impl From<ffi::ChkError> for Error {
    fn from(err: ffi::ChkError) -> Self {
        let message = unsafe {
            CStr::from_ptr(err.message.as_ptr())
                .to_string_lossy()
                .into_owned()
        };
        Error {
            code: err.code,
            message,
        }
    }
}

/// Render skin options for map rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderSkin {
    /// Original StarCraft graphics
    #[default]
    Classic,
    /// StarCraft Remastered SD graphics
    RemasteredSd,
    /// StarCraft Remastered HD2 graphics (2x)
    RemasteredHd2,
    /// StarCraft Remastered HD graphics (4x)
    RemasteredHd,
    /// Carbot HD2 graphics (2x)
    CarbotHd2,
    /// Carbot HD graphics (4x)
    CarbotHd,
}

impl RenderSkin {
    fn to_ffi(self) -> ffi::ChkRenderSkin {
        match self {
            RenderSkin::Classic => ffi::ChkRenderSkin_CHK_SKIN_CLASSIC,
            RenderSkin::RemasteredSd => ffi::ChkRenderSkin_CHK_SKIN_REMASTERED_SD,
            RenderSkin::RemasteredHd2 => ffi::ChkRenderSkin_CHK_SKIN_REMASTERED_HD2,
            RenderSkin::RemasteredHd => ffi::ChkRenderSkin_CHK_SKIN_REMASTERED_HD,
            RenderSkin::CarbotHd2 => ffi::ChkRenderSkin_CHK_SKIN_CARBOT_HD2,
            RenderSkin::CarbotHd => ffi::ChkRenderSkin_CHK_SKIN_CARBOT_HD,
        }
    }
}

/// Options for map rendering.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Draw background stars (for space tilesets)
    pub draw_stars: bool,
    /// Draw terrain tiles
    pub draw_terrain: bool,
    /// Draw units and sprites
    pub draw_actors: bool,
    /// Draw fog of war for a specific player (0-11), or None for no fog
    pub draw_fog_player: Option<u8>,
    /// Draw location rectangles
    pub draw_locations: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            draw_stars: true,
            draw_terrain: true,
            draw_actors: true,
            draw_fog_player: None,
            draw_locations: false,
        }
    }
}

impl RenderOptions {
    fn to_ffi(&self) -> ffi::ChkRenderOptions {
        ffi::ChkRenderOptions {
            draw_stars: self.draw_stars as i32,
            draw_terrain: self.draw_terrain as i32,
            draw_actors: self.draw_actors as i32,
            draw_fog_player: self.draw_fog_player.map_or(-1, |p| p as i32),
            draw_locations: self.draw_locations as i32,
        }
    }
}

/// Result of a WebP save operation.
#[derive(Debug, Clone)]
pub struct SaveWebpResult {
    /// Time spent loading skin and tileset (ms)
    pub load_skin_tileset_ms: i32,
    /// Time spent rendering (ms)
    pub render_ms: i32,
    /// Time spent encoding WebP (ms)
    pub encode_ms: i32,
    /// Time spent writing to disk (ms)
    pub out_file_ms: i32,
}

/// Raw rendered image (RGB pixels, not yet encoded to any format).
#[derive(Debug, Clone)]
pub struct RawImage {
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// RGB pixel data (length = width * height * 3)
    pub rgb_data: Vec<u8>,
}

/// Result of animation simulation.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// Number of ticks simulated
    pub ticks: i32,
    /// Game time simulated (ms)
    pub game_time_ms: i32,
    /// Real time spent simulating (ms)
    pub real_time_ms: i32,
}

/// Initialize the C++ logging system and verify it's working.
///
/// This should be called early in the application to ensure logging is properly
/// set up. It prints a test message to stdout.
///
/// Returns `true` if logging is working.
pub fn init_logging() -> bool {
    unsafe { ffi::chk_init_logging() != 0 }
}

/// Set the C++ logger log level.
///
/// Levels: 0=Off, 100=Fatal, 200=Error, 300=Warn, 400=Info, 500=Debug, 600=Trace
pub fn set_log_level(level: u32) {
    unsafe { ffi::chk_set_log_level(level) }
}

/// Graphics utility for loading StarCraft data and creating renderers/maps.
pub struct GfxUtil {
    ptr: *mut ffi::ChkGfxUtil,
}

// SAFETY: GfxUtil is only accessed through &self or &mut self
unsafe impl Send for GfxUtil {}

impl GfxUtil {
    /// Create a new GfxUtil instance.
    pub fn new() -> Result<Self, Error> {
        let ptr = unsafe { ffi::chk_gfxutil_create() };
        if ptr.is_null() {
            return Err(Error {
                code: -1,
                message: "Failed to create GfxUtil".to_string(),
            });
        }
        Ok(GfxUtil { ptr })
    }

    /// Load StarCraft data files from the given path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to StarCraft installation directory
    pub fn load_sc_data(&mut self, path: &str) -> Result<(), Error> {
        let c_path = CString::new(path).map_err(|_| Error {
            code: -1,
            message: "Invalid path string".to_string(),
        })?;

        let mut error = ffi::ChkError {
            code: 0,
            message: [0; 256],
        };

        let result =
            unsafe { ffi::chk_gfxutil_load_sc_data(self.ptr, c_path.as_ptr(), &raw mut error) };

        if result != 0 {
            return Err(error.into());
        }
        Ok(())
    }

    /// Create a renderer with the specified skin.
    ///
    /// # Arguments
    ///
    /// * `skin` - The render skin to use
    pub fn create_renderer(&self, skin: RenderSkin) -> Result<Renderer, Error> {
        let mut error = ffi::ChkError {
            code: 0,
            message: [0; 256],
        };

        let ptr =
            unsafe { ffi::chk_gfxutil_create_renderer(self.ptr, skin.to_ffi(), &raw mut error) };

        if ptr.is_null() {
            return Err(error.into());
        }
        Ok(Renderer { ptr })
    }

    /// Load a map from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the map file (.scx or .scm)
    pub fn load_map(&self, path: &str) -> Result<ScMap, Error> {
        let c_path = CString::new(path).map_err(|_| Error {
            code: -1,
            message: "Invalid path string".to_string(),
        })?;

        let mut error = ffi::ChkError {
            code: 0,
            message: [0; 256],
        };

        let ptr = unsafe { ffi::chk_gfxutil_load_map(self.ptr, c_path.as_ptr(), &raw mut error) };

        if ptr.is_null() {
            return Err(error.into());
        }
        Ok(ScMap { ptr })
    }
}

impl Drop for GfxUtil {
    fn drop(&mut self) {
        unsafe {
            ffi::chk_gfxutil_destroy(self.ptr);
        }
    }
}

/// Renderer for creating map images.
pub struct Renderer {
    ptr: *mut ffi::ChkRenderer,
}

// SAFETY: Renderer is only accessed through &self
unsafe impl Send for Renderer {}

impl Renderer {
    /// Change the renderer's active skin.
    ///
    /// The skin will be loaded on the next render call.
    pub fn set_skin(&self, skin: RenderSkin) {
        unsafe { ffi::chk_renderer_set_skin(self.ptr, skin.to_ffi()) }
    }

    /// Save a map as a WebP image.
    ///
    /// # Arguments
    ///
    /// * `map` - The map to render
    /// * `options` - Rendering options
    /// * `output_path` - Path to save the WebP image
    pub fn save_webp(
        &self,
        map: &ScMap,
        options: &RenderOptions,
        output_path: &str,
    ) -> Result<SaveWebpResult, Error> {
        let c_path = CString::new(output_path).map_err(|_| Error {
            code: -1,
            message: "Invalid output path string".to_string(),
        })?;

        let mut error = ffi::ChkError {
            code: 0,
            message: [0; 256],
        };

        let ffi_options = options.to_ffi();

        let result = unsafe {
            ffi::chk_renderer_save_webp(
                self.ptr,
                map.ptr,
                &raw const ffi_options,
                c_path.as_ptr(),
                &raw mut error,
            )
        };

        if result.success == 0 {
            return Err(error.into());
        }

        Ok(SaveWebpResult {
            load_skin_tileset_ms: result.load_skin_tileset_ms,
            render_ms: result.render_ms,
            encode_ms: result.encode_ms,
            out_file_ms: result.out_file_ms,
        })
    }

    /// Get a map image as WebP data in memory.
    ///
    /// # Arguments
    ///
    /// * `map` - The map to render
    /// * `options` - Rendering options
    ///
    /// # Returns
    ///
    /// The WebP-encoded image data
    pub fn get_webp(&self, map: &ScMap, options: &RenderOptions) -> Result<Vec<u8>, Error> {
        let mut error = ffi::ChkError {
            code: 0,
            message: [0; 256],
        };

        let ffi_options = options.to_ffi();
        let mut data_ptr: *mut u8 = ptr::null_mut();

        let size = unsafe {
            ffi::chk_renderer_get_webp(
                self.ptr,
                map.ptr,
                &raw const ffi_options,
                &raw mut data_ptr,
                &raw mut error,
            )
        };

        if size == 0 || data_ptr.is_null() {
            return Err(error.into());
        }

        // Copy data to a Rust Vec and free the C allocation
        let data = unsafe {
            let slice = std::slice::from_raw_parts(data_ptr, size);
            let vec = slice.to_vec();
            ffi::chk_free_webp_data(data_ptr);
            vec
        };

        Ok(data)
    }

    /// Render a map to raw RGB pixels without WebP encoding.
    ///
    /// Returns a `RawImage` containing the raw RGB pixel data, width, and height.
    /// The pixel data is in row-major order, 3 bytes per pixel (R, G, B).
    pub fn get_raw_image(&self, map: &ScMap, options: &RenderOptions) -> Result<RawImage, Error> {
        let mut error = ffi::ChkError {
            code: 0,
            message: [0; 256],
        };

        let ffi_options = options.to_ffi();
        let mut raw_image = ffi::ChkRawImage {
            data: ptr::null_mut(),
            width: 0,
            height: 0,
            data_size: 0,
        };

        let success = unsafe {
            ffi::chk_renderer_get_raw_image(
                self.ptr,
                map.ptr,
                &raw const ffi_options,
                &raw mut raw_image,
                &raw mut error,
            )
        };

        if success == 0 || raw_image.data.is_null() {
            return Err(error.into());
        }

        let data = unsafe {
            let slice = std::slice::from_raw_parts(raw_image.data, raw_image.data_size);
            let vec = slice.to_vec();
            ffi::chk_free_raw_image_data(raw_image.data);
            vec
        };

        Ok(RawImage {
            width: raw_image.width as u32,
            height: raw_image.height as u32,
            rgb_data: data,
        })
    }

    /// Render a minimap to raw RGB pixels.
    ///
    /// Returns a `RawImage` containing the 128x128 minimap as raw RGB pixel data.
    pub fn get_raw_minimap(&self, map: &ScMap) -> Result<RawImage, Error> {
        let mut error = ffi::ChkError {
            code: 0,
            message: [0; 256],
        };

        let mut raw_image = ffi::ChkRawImage {
            data: ptr::null_mut(),
            width: 0,
            height: 0,
            data_size: 0,
        };

        let success = unsafe {
            ffi::chk_renderer_get_raw_minimap(self.ptr, map.ptr, &raw mut raw_image, &raw mut error)
        };

        if success == 0 || raw_image.data.is_null() {
            return Err(error.into());
        }

        let data = unsafe {
            let slice = std::slice::from_raw_parts(raw_image.data, raw_image.data_size);
            let vec = slice.to_vec();
            ffi::chk_free_raw_image_data(raw_image.data);
            vec
        };

        Ok(RawImage {
            width: raw_image.width as u32,
            height: raw_image.height as u32,
            rgb_data: data,
        })
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            ffi::chk_renderer_destroy(self.ptr);
        }
    }
}

/// A StarCraft map with animation state.
pub struct ScMap {
    ptr: *mut ffi::ChkScMap,
}

// SAFETY: ScMap is only accessed through &self or &mut self
unsafe impl Send for ScMap {}

impl ScMap {
    /// Simulate animation ticks.
    ///
    /// This advances the animation state of units on the map.
    /// For example, 52 ticks is enough to fully extend siege tanks.
    ///
    /// # Arguments
    ///
    /// * `ticks` - Number of game ticks to simulate
    pub fn simulate_anim(&mut self, ticks: u64) -> SimulationResult {
        let result = unsafe { ffi::chk_scmap_simulate_anim(self.ptr, ticks) };
        SimulationResult {
            ticks: result.ticks,
            game_time_ms: result.game_time_ms,
            real_time_ms: result.real_time_ms,
        }
    }

    /// Get the map width in tiles.
    pub fn tile_width(&self) -> u16 {
        unsafe { ffi::chk_scmap_get_tile_width(self.ptr) }
    }

    /// Get the map height in tiles.
    pub fn tile_height(&self) -> u16 {
        unsafe { ffi::chk_scmap_get_tile_height(self.ptr) }
    }
}

impl Drop for ScMap {
    fn drop(&mut self) {
        unsafe {
            ffi::chk_scmap_destroy(self.ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_options_default() {
        let opts = RenderOptions::default();
        assert!(opts.draw_terrain);
        assert!(opts.draw_actors);
        assert!(opts.draw_stars);
        assert!(!opts.draw_locations);
        assert!(opts.draw_fog_player.is_none());
    }

    #[test]
    fn test_render_skin_to_ffi() {
        assert_eq!(
            RenderSkin::Classic.to_ffi(),
            ffi::ChkRenderSkin_CHK_SKIN_CLASSIC
        );
        assert_eq!(
            RenderSkin::RemasteredHd.to_ffi(),
            ffi::ChkRenderSkin_CHK_SKIN_REMASTERED_HD
        );
    }
}
