#include "chkdraft_wrapper.h"
#include <map_gfx_utils/gfx_util.h>
#include <map_gfx_utils/renderer.h>
#include <map_gfx_utils/sc_map.h>
#include <cross_cut/logger.h>

#include <cstring>
#include <iostream>
#include <memory>
#include <string>

// Access the global logger defined in gfx_util.cpp
extern Logger logger;

// Internal wrapper structures that hold the C++ objects
struct ChkGfxUtil {
    GfxUtil impl;
};

struct ChkRenderer {
    std::unique_ptr<Renderer> impl;
    ChkGfxUtil* gfx; // Keep reference to parent for scData lifetime
};

struct ChkScMap {
    std::unique_ptr<ScMap> impl;
    ChkGfxUtil* gfx; // Keep reference to parent for scData lifetime
};

// Helper to set error message
static void set_error(ChkError* error, int code, const char* message) {
    if (error) {
        error->code = code;
        strncpy(error->message, message, sizeof(error->message) - 1);
        error->message[sizeof(error->message) - 1] = '\0';
    }
}

static void clear_error(ChkError* error) {
    if (error) {
        error->code = 0;
        error->message[0] = '\0';
    }
}

// Convert C RenderSkin enum to C++ RenderSkin enum
static RenderSkin to_cpp_skin(ChkRenderSkin skin) {
    switch (skin) {
        case CHK_SKIN_CLASSIC: return RenderSkin::Classic;
        case CHK_SKIN_REMASTERED_SD: return RenderSkin::RemasteredSD;
        case CHK_SKIN_REMASTERED_HD2: return RenderSkin::RemasteredHD2;
        case CHK_SKIN_REMASTERED_HD: return RenderSkin::RemasteredHD;
        case CHK_SKIN_CARBOT_HD2: return RenderSkin::CarbotHD2;
        case CHK_SKIN_CARBOT_HD: return RenderSkin::CarbotHD;
        default: return RenderSkin::Classic;
    }
}

// Convert C RenderOptions to C++ Renderer::Options
static Renderer::Options to_cpp_options(const ChkRenderOptions* opts) {
    Renderer::Options result;
    if (opts) {
        result.drawStars = opts->draw_stars != 0;
        result.drawTerrain = opts->draw_terrain != 0;
        result.drawActors = opts->draw_actors != 0;
        if (opts->draw_fog_player >= 0 && opts->draw_fog_player <= 11) {
            result.drawFogPlayer = static_cast<u8>(opts->draw_fog_player);
        } else {
            result.drawFogPlayer = std::nullopt;
        }
        result.drawLocations = opts->draw_locations != 0;
        result.displayFps = false; // Not exposed in C API
        result.webpQuality = opts->webp_quality;
    }
    return result;
}

// ============================================================================
// Logging functions
// ============================================================================

int chk_init_logging(void) {
    // Test that the logger is working by printing a message
    // Use std::endl to ensure the output is flushed immediately
    logger.info() << "chkdraft-bindings: logger initialized and working" << std::endl;

    // Also print directly to cout as a fallback diagnostic
    std::cout << "chkdraft-bindings: direct cout test" << std::endl;

    return 1;
}

// ============================================================================
// GfxUtil functions
// ============================================================================

ChkGfxUtil* chk_gfxutil_create(void) {
    try {
        return new ChkGfxUtil();
    } catch (...) {
        return nullptr;
    }
}

void chk_gfxutil_destroy(ChkGfxUtil* gfx) {
    delete gfx;
}

int chk_gfxutil_load_sc_data(ChkGfxUtil* gfx, const char* sc_path, ChkError* error) {
    clear_error(error);
    if (!gfx) {
        set_error(error, 1, "GfxUtil is null");
        return 1;
    }
    try {
        std::string path = sc_path ? sc_path : "";
        if (path.empty()) {
            gfx->impl.loadScData();
        } else {
            gfx->impl.loadScData(path);
        }
        return 0;
    } catch (const std::exception& e) {
        set_error(error, 2, e.what());
        return 2;
    } catch (...) {
        set_error(error, 3, "Unknown error loading SC data");
        return 3;
    }
}

ChkRenderer* chk_gfxutil_create_renderer(ChkGfxUtil* gfx, ChkRenderSkin skin, ChkError* error) {
    clear_error(error);
    if (!gfx) {
        set_error(error, 1, "GfxUtil is null");
        return nullptr;
    }
    if (!gfx->impl.scData) {
        set_error(error, 2, "SC data not loaded");
        return nullptr;
    }
    try {
        auto renderer = new ChkRenderer();
        renderer->gfx = gfx;
        renderer->impl = gfx->impl.createRenderer(to_cpp_skin(skin));
        if (!renderer->impl) {
            delete renderer;
            set_error(error, 3, "Failed to create renderer");
            return nullptr;
        }
        return renderer;
    } catch (const std::exception& e) {
        set_error(error, 4, e.what());
        return nullptr;
    } catch (...) {
        set_error(error, 5, "Unknown error creating renderer");
        return nullptr;
    }
}

ChkScMap* chk_gfxutil_load_map(ChkGfxUtil* gfx, const char* map_path, ChkError* error) {
    clear_error(error);
    if (!gfx) {
        set_error(error, 1, "GfxUtil is null");
        return nullptr;
    }
    if (!gfx->impl.scData) {
        set_error(error, 2, "SC data not loaded");
        return nullptr;
    }
    if (!map_path || map_path[0] == '\0') {
        set_error(error, 3, "Map path is empty");
        return nullptr;
    }
    try {
        auto map = new ChkScMap();
        map->gfx = gfx;
        map->impl = gfx->impl.loadMap(std::string(map_path));
        if (!map->impl) {
            delete map;
            set_error(error, 4, "Failed to load map");
            return nullptr;
        }
        return map;
    } catch (const std::exception& e) {
        set_error(error, 5, e.what());
        return nullptr;
    } catch (...) {
        set_error(error, 6, "Unknown error loading map");
        return nullptr;
    }
}

// ============================================================================
// Renderer functions
// ============================================================================

void chk_renderer_destroy(ChkRenderer* renderer) {
    delete renderer;
}

ChkSaveWebpResult chk_renderer_save_webp(
    ChkRenderer* renderer,
    ChkScMap* map,
    const ChkRenderOptions* options,
    const char* output_path,
    ChkError* error
) {
    ChkSaveWebpResult result = {0, 0, 0, 0, 0};
    clear_error(error);

    if (!renderer || !renderer->impl) {
        set_error(error, 1, "Renderer is null");
        return result;
    }
    if (!map || !map->impl) {
        set_error(error, 2, "Map is null");
        return result;
    }
    if (!output_path || output_path[0] == '\0') {
        set_error(error, 3, "Output path is empty");
        return result;
    }

    try {
        Renderer::Options cpp_opts = to_cpp_options(options);
        auto save_result = renderer->impl->saveMapImageAsWebP(*map->impl, cpp_opts, std::string(output_path));
        if (save_result) {
            result.success = 1;
            result.load_skin_tileset_ms = save_result->loadSkinAndTilesetTimeMs;
            result.render_ms = save_result->renderTimeMs;
            result.encode_ms = save_result->encodeTimeMs;
            result.out_file_ms = save_result->outFileTimeMs;
        } else {
            set_error(error, 4, "saveMapImageAsWebP returned empty result");
        }
    } catch (const std::exception& e) {
        set_error(error, 5, e.what());
    } catch (...) {
        set_error(error, 6, "Unknown error saving WebP");
    }

    return result;
}

size_t chk_renderer_get_webp(
    ChkRenderer* renderer,
    ChkScMap* map,
    const ChkRenderOptions* options,
    uint8_t** out_data,
    ChkError* error
) {
    clear_error(error);
    if (out_data) *out_data = nullptr;

    if (!renderer || !renderer->impl) {
        set_error(error, 1, "Renderer is null");
        return 0;
    }
    if (!map || !map->impl) {
        set_error(error, 2, "Map is null");
        return 0;
    }
    if (!out_data) {
        set_error(error, 3, "out_data pointer is null");
        return 0;
    }

    try {
        // Load skin and tileset for this specific map (different maps may have different tilesets)
        // This is necessary because getMapImageAsWebP doesn't call loadSkinAndTileSet internally
        // (unlike saveMapImageAsWebP which does)
        auto tileWidth = map->impl->getTileWidth();
        auto tileHeight = map->impl->getTileHeight();
        logger.info() << "chk_renderer_get_webp: map size " << tileWidth << "x" << tileHeight
                      << " tiles, loading skin and tileset" << std::endl;

        renderer->impl->loadSkinAndTileSet(renderer->impl->renderSkin, *map->impl);
        logger.info() << "chk_renderer_get_webp: skin and tileset loaded, starting render" << std::endl;

        Renderer::Options cpp_opts = to_cpp_options(options);
        EncodedWebP encoded;
        auto renderTimeMs = renderer->impl->getMapImageAsWebP(*map->impl, cpp_opts, encoded);

        logger.info() << "chk_renderer_get_webp: render complete in " << renderTimeMs << "ms, "
                      << "encoded.size=" << encoded.size << ", encoded.data=" << (encoded.data ? "valid" : "null") << std::endl;

        if (encoded.size > 0 && encoded.data) {
            // Copy data to a new buffer that can be freed by the caller
            *out_data = static_cast<uint8_t*>(malloc(encoded.size));
            if (*out_data) {
                memcpy(*out_data, encoded.data, encoded.size);
                logger.info() << "chk_renderer_get_webp: success, returning " << encoded.size << " bytes" << std::endl;
                return encoded.size;
            } else {
                logger.error() << "chk_renderer_get_webp: failed to allocate " << encoded.size << " bytes for output" << std::endl;
                set_error(error, 4, "Failed to allocate memory for WebP data");
                return 0;
            }
        } else {
            logger.error() << "chk_renderer_get_webp: WebP encoding failed or returned empty data" << std::endl;
            set_error(error, 5, "Failed to encode WebP");
            return 0;
        }
    } catch (const std::exception& e) {
        logger.error() << "chk_renderer_get_webp: exception: " << e.what() << std::endl;
        set_error(error, 6, e.what());
        return 0;
    } catch (...) {
        logger.error() << "chk_renderer_get_webp: unknown exception" << std::endl;
        set_error(error, 7, "Unknown error getting WebP data");
        return 0;
    }
}

void chk_free_webp_data(uint8_t* data) {
    free(data);
}

// ============================================================================
// ScMap functions
// ============================================================================

void chk_scmap_destroy(ChkScMap* map) {
    delete map;
}

ChkSimulationResult chk_scmap_simulate_anim(ChkScMap* map, uint64_t ticks) {
    ChkSimulationResult result = {0, 0, 0};
    if (!map || !map->impl) {
        return result;
    }
    try {
        auto sim = map->impl->simulateAnim(ticks);
        result.ticks = sim.ticks;
        result.game_time_ms = sim.gameTimeSimulatedMs;
        result.real_time_ms = sim.realTimeSpentMs;
    } catch (...) {
        // Ignore errors in simulation
    }
    return result;
}

uint16_t chk_scmap_get_tile_width(ChkScMap* map) {
    if (!map || !map->impl) return 0;
    try {
        return map->impl->getTileWidth();
    } catch (...) {
        return 0;
    }
}

uint16_t chk_scmap_get_tile_height(ChkScMap* map) {
    if (!map || !map->impl) return 0;
    try {
        return map->impl->getTileHeight();
    } catch (...) {
        return 0;
    }
}
