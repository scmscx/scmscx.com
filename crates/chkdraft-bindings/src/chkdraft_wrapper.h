#ifndef CHKDRAFT_WRAPPER_H
#define CHKDRAFT_WRAPPER_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Opaque handle types
typedef struct ChkGfxUtil ChkGfxUtil;
typedef struct ChkRenderer ChkRenderer;
typedef struct ChkScMap ChkScMap;

// Render skin enum (matches C++ RenderSkin)
typedef enum ChkRenderSkin {
    CHK_SKIN_CLASSIC = 0,
    CHK_SKIN_REMASTERED_SD = 1,
    CHK_SKIN_REMASTERED_HD2 = 2,
    CHK_SKIN_REMASTERED_HD = 3,
    CHK_SKIN_CARBOT_HD2 = 4,
    CHK_SKIN_CARBOT_HD = 5
} ChkRenderSkin;

// Render options (C-compatible version of Renderer::Options)
typedef struct ChkRenderOptions {
    int draw_stars;       // bool
    int draw_terrain;     // bool
    int draw_actors;      // bool (units & sprites)
    int draw_fog_player;  // -1 for none, 0-11 for player index
    int draw_locations;   // bool
    float webp_quality;   // 0-100, where 100 is highest quality. Values <= 0 use lossless encoding.
} ChkRenderOptions;

// Result struct for save_webp operation
typedef struct ChkSaveWebpResult {
    int success;
    int load_skin_tileset_ms;
    int render_ms;
    int encode_ms;
    int out_file_ms;
} ChkSaveWebpResult;

// Error info
typedef struct ChkError {
    int code;
    char message[256];
} ChkError;

// ============================================================================
// Logging functions
// ============================================================================

// Initialize and test logging - call this early to verify logging is working
// Prints a test message to stdout and returns 1 if successful
int chk_init_logging(void);

// ============================================================================
// GfxUtil functions
// ============================================================================

// Create a new GfxUtil instance
ChkGfxUtil* chk_gfxutil_create(void);

// Destroy a GfxUtil instance
void chk_gfxutil_destroy(ChkGfxUtil* gfx);

// Load StarCraft data files from the given path
// Returns 0 on success, non-zero on failure
int chk_gfxutil_load_sc_data(ChkGfxUtil* gfx, const char* sc_path, ChkError* error);

// Create a renderer with the given skin
// Returns NULL on failure
ChkRenderer* chk_gfxutil_create_renderer(ChkGfxUtil* gfx, ChkRenderSkin skin, ChkError* error);

// Load a map from a file path
// Returns NULL on failure
ChkScMap* chk_gfxutil_load_map(ChkGfxUtil* gfx, const char* map_path, ChkError* error);

// ============================================================================
// Renderer functions
// ============================================================================

// Destroy a renderer
void chk_renderer_destroy(ChkRenderer* renderer);

// Save map as WebP image
// Returns result with success=1 on success, success=0 on failure
ChkSaveWebpResult chk_renderer_save_webp(
    ChkRenderer* renderer,
    ChkScMap* map,
    const ChkRenderOptions* options,
    const char* output_path,
    ChkError* error
);

// Get map image as WebP in memory
// Returns the size of the encoded data, or 0 on failure
// The caller must free the returned data with chk_free_webp_data()
size_t chk_renderer_get_webp(
    ChkRenderer* renderer,
    ChkScMap* map,
    const ChkRenderOptions* options,
    uint8_t** out_data,
    ChkError* error
);

// Free WebP data allocated by chk_renderer_get_webp
void chk_free_webp_data(uint8_t* data);

// ============================================================================
// ScMap functions
// ============================================================================

// Destroy a map
void chk_scmap_destroy(ChkScMap* map);

// Simulate animation ticks
// Returns info about the simulation
typedef struct ChkSimulationResult {
    int ticks;
    int game_time_ms;
    int real_time_ms;
} ChkSimulationResult;

ChkSimulationResult chk_scmap_simulate_anim(ChkScMap* map, uint64_t ticks);

// Get map dimensions
uint16_t chk_scmap_get_tile_width(ChkScMap* map);
uint16_t chk_scmap_get_tile_height(ChkScMap* map);

#ifdef __cplusplus
}
#endif

#endif // CHKDRAFT_WRAPPER_H
