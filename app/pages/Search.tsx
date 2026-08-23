import { For, createEffect, onCleanup, onMount } from "solid-js";
import {
  A,
  useBeforeLeave,
  useLocation,
  useNavigate,
  useParams,
  useSearchParams,
} from "@solidjs/router";

import { autofocus } from "@solid-primitives/autofocus";

import { createSignal, Switch, Match, Show } from "solid-js";

import style from "./Search.module.scss";

import MinimapHover from "../modules/MinimapHover";
import {
  ColoredTextMenu,
  ColoredTextMenuNoNewlines,
} from "../modules/ColoredText";
import { I18nSpan, i18n_internal } from "../modules/language";
import { useLang, useSession } from "../modules/context";
import MinimapImg from "../modules/MinimapImg";
import { unix_time_to_timestamp, useApi } from "../util/util";

const convertDateToLocalDateTime = (d: Date): string => {
  return new Date(d.getTime() - d.getTimezoneOffset() * 60000)
    .toISOString()
    .slice(0, -1);
};

const convertDateToLocalDate = (d: Date): string => {
  return new Date(d.getTime() - d.getTimezoneOffset() * 60000) // TODO: this is definitely incorrect.
    .toISOString()
    .split("T")[0];
};

const mapValue = (x: string | undefined): string =>
  x === "true" || x === undefined ? "true" : "false";

const mapSort = (x: string | undefined): string => {
  if (x === "scenario") {
    return "scenario";
  }

  if (x === "scenariodesc") {
    return "scenariodesc";
  }

  if (x === "filename") {
    return "filename";
  }

  if (x === "filenamedesc") {
    return "filenamedesc";
  }

  if (x === "lastmodifiedold") {
    return "lastmodifiedold";
  }

  if (x === "lastmodifiednew") {
    return "lastmodifiednew";
  }

  if (x === "timeuploadedold") {
    return "timeuploadedold";
  }

  if (x === "timeuploadednew") {
    return "timeuploadednew";
  }

  return "relevancy";
};

const mapValueInt = (x: string | undefined, d: number): string => {
  if (x === undefined) {
    return `${d}`;
  } else {
    const parsed = parseInt(x);
    return `${isNaN(parsed) ? d : parsed}`;
  }
};

const parseIntWithDefault = (x: string | undefined, d: number): number => {
  if (x === undefined) {
    return d;
  } else {
    const parsed = parseInt(x);
    return isNaN(parsed) ? d : parsed;
  }
};

const getSearchUrl = (
  prefix: string,
  query: string,
  queryParams: URLSearchParams
) => {
  if (queryParams.get("sort") === "relevancy") queryParams.delete("sort");

  if (queryParams.get("unit_names") === "true")
    queryParams.delete("unit_names");
  if (queryParams.get("force_names") === "true")
    queryParams.delete("force_names");
  if (queryParams.get("file_names") === "true")
    queryParams.delete("file_names");
  if (queryParams.get("scenario_names") === "true")
    queryParams.delete("scenario_names");
  if (queryParams.get("scenario_descriptions") === "true")
    queryParams.delete("scenario_descriptions");

  if (queryParams.get("tileset_badlands") === "true")
    queryParams.delete("tileset_badlands");

  if (queryParams.get("tileset_space_platform") === "true")
    queryParams.delete("tileset_space_platform");

  if (queryParams.get("tileset_installation") === "true")
    queryParams.delete("tileset_installation");

  if (queryParams.get("tileset_ashworld") === "true")
    queryParams.delete("tileset_ashworld");

  if (queryParams.get("tileset_jungle") === "true")
    queryParams.delete("tileset_jungle");

  if (queryParams.get("tileset_desert") === "true")
    queryParams.delete("tileset_desert");

  if (queryParams.get("tileset_ice") === "true")
    queryParams.delete("tileset_ice");

  if (queryParams.get("tileset_twilight") === "true")
    queryParams.delete("tileset_twilight");

  if (queryParams.get("minimum_map_width") === "0")
    queryParams.delete("minimum_map_width");

  if (queryParams.get("maximum_map_width") === "256")
    queryParams.delete("maximum_map_width");

  if (queryParams.get("minimum_map_height") === "0")
    queryParams.delete("minimum_map_height");

  if (queryParams.get("maximum_map_height") === "256")
    queryParams.delete("maximum_map_height");

  if (queryParams.get("minimum_human_players") === "0")
    queryParams.delete("minimum_human_players");

  if (queryParams.get("maximum_human_players") === "12")
    queryParams.delete("maximum_human_players");

  if (queryParams.get("minimum_computer_players") === "0")
    queryParams.delete("minimum_computer_players");

  if (queryParams.get("maximum_computer_players") === "12")
    queryParams.delete("maximum_computer_players");

  if (queryParams.get("last_modified_after") === "0")
    queryParams.delete("last_modified_after");

  if (
    queryParams.get("last_modified_before") ===
    `${new Date("2050-01-01").getTime()}`
  )
    queryParams.delete("last_modified_before");

  if (queryParams.get("time_uploaded_after") === "0")
    queryParams.delete("time_uploaded_after");

  if (
    queryParams.get("time_uploaded_before") ===
    `${new Date("2050-01-01").getTime()}`
  )
    queryParams.delete("time_uploaded_before");

  if (queryParams.get("uploaded_by") === "")
    queryParams.delete("uploaded_by");
  if (queryParams.get("include_broken") === "false")
    queryParams.delete("include_broken");
  if (queryParams.get("include_outdated") === "false")
    queryParams.delete("include_outdated");
  if (queryParams.get("include_unfinished") === "false")
    queryParams.delete("include_unfinished");
  if (queryParams.get("include_nsfw") === "false")
    queryParams.delete("include_nsfw");

  let apiUrl;
  if (query.length == 0) {
    if (queryParams.size == 0) {
      apiUrl = `${prefix}/search`;
    } else {
      apiUrl = `${prefix}/search?${queryParams}`;
    }
  } else {
    const encodedQuery = encodeURIComponent(query);

    if (queryParams.size == 0) {
      apiUrl = `${prefix}/search/${encodedQuery}`;
    } else {
      apiUrl = `${prefix}/search/${encodedQuery}?${queryParams}`;
    }
  }

  return apiUrl;
};

/** A search page as the user last saw it, including how far down it they were. */
type CachedSearch = {
  results: any[];
  numResults: number;
  dontRequestMore: boolean;
  scrollY: number;
};

// Clicking a result unmounts this component and disposes every signal in it, so
// coming back would otherwise refetch offset 0: whatever infinite scroll had
// loaded is dropped, and the document is left too short to restore a scroll
// position into even if one had been saved. Parking the results here instead
// lets the remount render the same page the user left.
//
// Keyed by the full search URL so an unrelated search never reads these
// results, and deliberately memory-only — this is for the back button, not for
// reloads or new tabs.
const searchCache = new Map<string, CachedSearch>();

// Deep enough to cover backing out of a few searches in a row without letting a
// long browsing session accumulate result sets forever.
const SEARCH_CACHE_ENTRIES = 10;

const rememberSearch = (key: string, entry: CachedSearch) => {
  // Re-insert rather than overwrite, so Map iteration order stays
  // least-recently-used first and the eviction below drops the right entry.
  searchCache.delete(key);
  searchCache.set(key, entry);

  while (searchCache.size > SEARCH_CACHE_ENTRIES) {
    const oldest = searchCache.keys().next();
    if (oldest.done) break;
    searchCache.delete(oldest.value);
  }
};

export default function (prop: any) {
  const [lang, _] = useLang();
  const [session] = useSession();
  const location = useLocation();
  const params = useParams();
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  // Read live rather than captured once: `doNavigate` changes the URL without
  // remounting, so a key captured at setup would file the next save under the
  // previous search.
  const cacheKey = () => location.pathname + location.search;
  const restored = searchCache.get(cacheKey());

  const [currentQuery, setCurrentQuery] = createSignal("");
  const [currentPendingQuery, setCurrentPendingQuery] = createSignal("");
  const [numSearchResults, setNumSearchResults] = createSignal(
    restored?.numResults ?? 0
  );
  const [dontRequestMore, setDontRequestMore] = createSignal(
    restored?.dontRequestMore ?? false
  );
  const [searchResults, setSearchResults] = createSignal<any[]>(
    // Seeded here rather than in `onMount` so the rows exist in the DOM by the
    // first paint — otherwise the scroll restore below has nothing to land on.
    restored?.results ?? []
  );

  const [targetsShown, setTargetsShown] = createSignal(
    searchParams.unit_names != undefined ||
      searchParams.force_names != undefined ||
      searchParams.file_names != undefined ||
      searchParams.scenario_names != undefined ||
      searchParams.scenario_descriptions != undefined
  );

  const [filtersShown, setFiltersShown] = createSignal(
    searchParams.tileset_badlands != undefined ||
      searchParams.tileset_space_platform != undefined ||
      searchParams.tileset_installation != undefined ||
      searchParams.tileset_ashworld != undefined ||
      searchParams.tileset_jungle != undefined ||
      searchParams.tileset_desert != undefined ||
      searchParams.tileset_ice != undefined ||
      searchParams.tileset_twilight != undefined ||
      searchParams.minimum_map_width != undefined ||
      searchParams.maximum_map_width != undefined ||
      searchParams.minimum_map_height != undefined ||
      searchParams.maximum_map_height != undefined ||
      searchParams.minimum_human_players != undefined ||
      searchParams.maximum_human_players != undefined ||
      searchParams.minimum_computer_players != undefined ||
      searchParams.maximum_computer_players != undefined ||
      searchParams.last_modified_after != undefined ||
      searchParams.last_modified_before != undefined ||
      searchParams.uploaded_after != undefined ||
      searchParams.uploaded_before != undefined ||
      searchParams.uploaded_by != undefined ||
      searchParams.include_broken != undefined ||
      searchParams.include_outdated != undefined ||
      searchParams.include_unfinished != undefined ||
      searchParams.include_nsfw != undefined
  );

  const [sortShown, setSortShown] = createSignal(
    searchParams.sort != undefined
  );

  const [formData, setFormData] = createSignal({
    sort: mapSort(searchParams.sort),

    unit_names: mapValue(searchParams.unit_names),
    force_names: mapValue(searchParams.force_names),
    file_names: mapValue(searchParams.file_names),
    scenario_names: mapValue(searchParams.scenario_names),
    scenario_descriptions: mapValue(searchParams.scenario_descriptions),

    tileset_badlands: mapValue(searchParams.tileset_badlands),
    tileset_space_platform: mapValue(searchParams.tileset_space_platform),
    tileset_installation: mapValue(searchParams.tileset_installation),
    tileset_ashworld: mapValue(searchParams.tileset_ashworld),
    tileset_jungle: mapValue(searchParams.tileset_jungle),
    tileset_desert: mapValue(searchParams.tileset_desert),
    tileset_ice: mapValue(searchParams.tileset_ice),
    tileset_twilight: mapValue(searchParams.tileset_twilight),
    minimum_map_width: mapValueInt(searchParams.minimum_map_width, 0),
    maximum_map_width: mapValueInt(searchParams.maximum_map_width, 256),

    minimum_map_height: mapValueInt(searchParams.minimum_map_height, 0),
    maximum_map_height: mapValueInt(searchParams.maximum_map_height, 256),

    minimum_human_players: mapValueInt(searchParams.minimum_human_players, 0),
    maximum_human_players: mapValueInt(searchParams.maximum_human_players, 12),

    minimum_computer_players: mapValueInt(
      searchParams.minimum_computer_players,
      0
    ),
    maximum_computer_players: mapValueInt(
      searchParams.maximum_computer_players,
      12
    ),

    last_modified_after: mapValueInt(searchParams.last_modified_after, 0),
    last_modified_before: mapValueInt(
      searchParams.last_modified_before,
      new Date("2050-01-01").getTime()
    ),
    time_uploaded_after: mapValueInt(searchParams.uploaded_after, 0),
    time_uploaded_before: mapValueInt(
      searchParams.uploaded_before,
      new Date("2050-01-01").getTime()
    ),

    uploaded_by: searchParams.uploaded_by ?? "",
    include_broken: searchParams.include_broken === "true" ? "true" : "false",
    include_outdated: searchParams.include_outdated === "true" ? "true" : "false",
    include_unfinished: searchParams.include_unfinished === "true" ? "true" : "false",
    include_nsfw: searchParams.include_nsfw === "true" ? "true" : "false",
  });

  if (params.query == undefined) {
    setCurrentQuery("");
  } else {
    setCurrentQuery(decodeURIComponent(params.query));
  }

  const doNavigate = async (isRandom: boolean) => {
    if (isRandom) {
      const queryParams2 = {
        ...formData(),
        query: currentQuery(),
      };

      const queryObject = new URLSearchParams(queryParams2);

      const map_id = await (
        await fetch(
          `/api/uiv2/random/${queryObject.get("query")}?${queryObject}`
        )
      ).json();

      navigate(`/map/${map_id}#${queryObject}`);
    } else {
      const url = getSearchUrl(
        "",
        currentQuery(),
        new URLSearchParams(formData())
      );

      navigate(url, {
        scroll: false,
      });

      fetchData();
    }
  };

  const [isLoading, setIsLoading] = createSignal(false);

  const fetchData = async () => {
    setSearchResults([]);
    setIsLoading(true);

    const query_params = new URLSearchParams(formData());
    const url = getSearchUrl("/api/uiv2", currentQuery(), query_params);

    try {
      setCurrentPendingQuery(url);
      const response = await fetch(url);
      const json = await response.json();

      if (currentPendingQuery() != url) {
        // the user updated the query while the request was in flight, ignore the results.
        return;
      }

      setSearchResults(json.maps);
      setNumSearchResults(json.total_results);
      setDontRequestMore(json.total_results <= json.maps.length);
    } catch (e) {
      console.log("error: ", e);
    } finally {
      setIsLoading(false);
    }
  };

  const extendData = async () => {
    if (dontRequestMore()) return;

    setIsLoading(true);

    const query_params = new URLSearchParams(formData());
    query_params.append("offset", `${searchResults().length}`);
    const url = getSearchUrl("/api/uiv2", currentQuery(), query_params);

    try {
      const response = await fetch(url);
      const json = await response.json();
      if (json.maps.length == 0) {
        setDontRequestMore(true);
      } else {
        setSearchResults([...searchResults(), ...json.maps]);
      }
    } catch (e) {
      console.log("error: ", e);
    } finally {
      setIsLoading(false);
    }
  };

  onMount(() => {
    if (restored) {
      window.scrollTo(0, restored.scrollY);
      return;
    }

    fetchData();
  });

  // Fires on the way out of any navigation, including the click on a result
  // that unmounts this component and the programmatic one in `doNavigate`. In
  // both cases the router has not moved yet, so `cacheKey()` still names the
  // page these results belong to.
  useBeforeLeave(() => {
    // Leaving before the first page lands would otherwise cache an empty list,
    // and the remount would restore that and skip the fetch — a search stuck
    // blank forever. An empty result set is cheap to fetch again, so just don't
    // record one. A page-2 fetch in flight is fine: page 1 is already here.
    if (searchResults().length === 0) {
      return;
    }

    rememberSearch(cacheKey(), {
      results: searchResults(),
      numResults: numSearchResults(),
      dontRequestMore: dontRequestMore(),
      scrollY: window.scrollY,
    });
  });

  createEffect(() => {
    const handleScroll = (e: any) => {
      if (!isLoading()) {
        if (
          document.documentElement.offsetHeight <
          window.innerHeight + document.documentElement.scrollTop + 8000
        ) {
          extendData();
        }
      }
    };

    const controller = new AbortController();

    window.addEventListener("scroll", handleScroll, {
      signal: controller.signal,
    });

    onCleanup(() => controller.abort());
  });

  // createEffect(async () => {
  //   const resp = await fetch(
  //     `/api/uiv2/search/${
  //       params.query === undefined ? "" : decodeURIComponent(params.query)
  //     }${location.search}`
  //   );

  //   const json = await resp.json();

  // });

  // const handleScroll = () => {
  //   if (
  //     window.innerHeight + document.documentElement.scrollTop !==
  //       document.documentElement.offsetHeight ||
  //     isLoading
  //   ) {
  //     return;
  //   }
  //   fetchData();
  // };

  return (
    <div class={style["vertical-container"]}>
      <h1 class={style.h1}>
        <I18nSpan text="common.search_2" />
      </h1>

      <form
        onSubmit={(e) => {
          e.preventDefault();

          if (e.submitter.getAttribute("name") == "random") {
            doNavigate(true);
          } else {
            doNavigate(false);
          }
        }}
      >
        {/* Focusing the box scrolls it into view, and it sits at the top of the
            page — on a restored search that would undo the scroll restore, so
            the box is left unfocused there. */}
        <input
          use:autofocus
          autofocus={!restored && window.screen.height < window.screen.width}
          class={style.search}
          value={currentQuery()}
          placeholder={i18n_internal(lang(), "search.query")}
          onInput={(evt) => setCurrentQuery(evt.target.value)}
        />
        <button class={style["search-button"]} name="search" type="submit">
          <I18nSpan text="search.submit" />
        </button>
        <button class={style["search-button"]} name="random" type="submit">
          <I18nSpan text="search.random" />
        </button>

        <h4
          class={style.dropdown}
          onClick={() => {
            setTargetsShown(!targetsShown());
          }}
        >
          <I18nSpan text="search.targets" />
          <span
            classList={{
              [style["dropdown-arrow"]]: !targetsShown(),
              [style["dropdown-arrow-highlighted"]]: targetsShown(),
            }}
          >
            ▼
          </span>
        </h4>

        <Show when={targetsShown()}>
          <div class={style["search-flags"]}>
            <div class={style.flexbox}>
              <div class={style["search-flag"]}>
                <label for="unit_names">
                  <input
                    type="checkbox"
                    id="unit_names"
                    checked={formData().unit_names === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        unit_names:
                          formData().unit_names === "true" ? "false" : "true",
                      });
                    }}
                  />
                  <I18nSpan text="common.units" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="forces">
                  <input
                    type="checkbox"
                    id="forces"
                    checked={formData().force_names === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        force_names:
                          formData().force_names === "true" ? "false" : "true",
                      });
                    }}
                  />
                  <I18nSpan text="common.forces" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="filenames">
                  <input
                    type="checkbox"
                    id="filenames"
                    checked={formData().file_names === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        file_names:
                          formData().file_names === "true" ? "false" : "true",
                      });
                    }}
                  />
                  <I18nSpan text="search.filenames" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="scenarios">
                  <input
                    type="checkbox"
                    id="scenarios"
                    checked={formData().scenario_names === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        scenario_names:
                          formData().scenario_names === "true"
                            ? "false"
                            : "true",
                      });
                    }}
                  />
                  <I18nSpan text="search.scenarios" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="scenarioDescriptions">
                  <input
                    type="checkbox"
                    id="scenarioDescriptions"
                    checked={formData().scenario_descriptions === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        scenario_descriptions:
                          formData().scenario_descriptions === "true"
                            ? "false"
                            : "true",
                      });
                    }}
                  />
                  <I18nSpan text="search.scenario_descriptions" />
                </label>
              </div>
            </div>
          </div>
        </Show>

        <h4
          class={style.dropdown}
          onClick={() => {
            setFiltersShown(!filtersShown());
          }}
        >
          <I18nSpan text="search.filters" />
          <span
            classList={{
              [style["dropdown-arrow"]]: !filtersShown(),
              [style["dropdown-arrow-highlighted"]]: filtersShown(),
            }}
          >
            ▼
          </span>
        </h4>

        <Show when={filtersShown()}>
          <div class={style["search-flags"]}>
            <div class={style.flexbox}>
              <div class={style["search-flag"]}>
                <label for="badlands">
                  <input
                    type="checkbox"
                    id="badlands"
                    checked={formData().tileset_badlands === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        tileset_badlands:
                          formData().tileset_badlands === "true"
                            ? "false"
                            : "true",
                      });
                    }}
                  />
                  <I18nSpan text="common.tileset_badlands" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="space">
                  <input
                    type="checkbox"
                    id="space"
                    checked={formData().tileset_space_platform === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        tileset_space_platform:
                          formData().tileset_space_platform === "true"
                            ? "false"
                            : "true",
                      });
                    }}
                  />
                  <I18nSpan text="common.tileset_space" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="installation">
                  <input
                    type="checkbox"
                    id="installation"
                    checked={formData().tileset_installation === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        tileset_installation:
                          formData().tileset_installation === "true"
                            ? "false"
                            : "true",
                      });
                    }}
                  />
                  <I18nSpan text="common.tileset_installation" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="ashworld">
                  <input
                    type="checkbox"
                    id="ashworld"
                    checked={formData().tileset_ashworld === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        tileset_ashworld:
                          formData().tileset_ashworld === "true"
                            ? "false"
                            : "true",
                      });
                    }}
                  />
                  <I18nSpan text="common.tileset_ashworld" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="jungle">
                  <input
                    type="checkbox"
                    id="jungle"
                    checked={formData().tileset_jungle === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        tileset_jungle:
                          formData().tileset_jungle === "true"
                            ? "false"
                            : "true",
                      });
                    }}
                  />
                  <I18nSpan text="common.tileset_jungle" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="desert">
                  <input
                    type="checkbox"
                    id="desert"
                    checked={formData().tileset_desert === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        tileset_desert:
                          formData().tileset_desert === "true"
                            ? "false"
                            : "true",
                      });
                    }}
                  />
                  <I18nSpan text="common.tileset_desert" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="ice">
                  <input
                    type="checkbox"
                    id="ice"
                    checked={formData().tileset_ice === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        tileset_ice:
                          formData().tileset_ice === "true" ? "false" : "true",
                      });
                    }}
                  />
                  <I18nSpan text="common.tileset_ice" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="twilight">
                  <input
                    type="checkbox"
                    id="twilight"
                    checked={formData().tileset_twilight === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        tileset_twilight:
                          formData().tileset_twilight === "true"
                            ? "false"
                            : "true",
                      });
                    }}
                  />
                  <I18nSpan text="common.tileset_twilight" />
                </label>
              </div>
            </div>

            <div class={style.flexbox}>
              <div class={style["search-filter-textbox"]}>
                <label for="minimumMapWidth">
                  <I18nSpan text="search.minimum_map_width" />
                  <br />
                  <input
                    type="number"
                    min="0"
                    max="256"
                    id="minimumMapWidth"
                    value={formData().minimum_map_width}
                    onChange={(evt) => {
                      setFormData({
                        ...formData(),
                        minimum_map_width: mapValueInt(evt.target.value, 0),
                      });
                    }}
                  />
                </label>
              </div>

              <div class={style["search-filter-textbox"]}>
                <label for="maximumMapWidth">
                  <I18nSpan text="search.maximum_map_width" />
                  <br />
                  <input
                    type="number"
                    min="0"
                    max="256"
                    id="maximumMapWidth"
                    value={formData().maximum_map_width}
                    onChange={(evt) => {
                      setFormData({
                        ...formData(),
                        maximum_map_width: mapValueInt(evt.target.value, 256),
                      });
                    }}
                  />
                </label>
              </div>

              <div class={style["search-filter-textbox"]}>
                <label for="minimumMapHeight">
                  <I18nSpan text="search.minimum_map_height" />
                  <br />
                  <input
                    type="number"
                    min="0"
                    max="256"
                    id="minimumMapHeight"
                    value={formData().minimum_map_height}
                    onChange={(evt) => {
                      setFormData({
                        ...formData(),
                        minimum_map_height: mapValueInt(evt.target.value, 0),
                      });
                    }}
                  />
                </label>
              </div>

              <div class={style["search-filter-textbox"]}>
                <label for="maximumMapHeight">
                  <I18nSpan text="search.maximum_map_height" />
                  <br />
                  <input
                    type="number"
                    min="0"
                    max="256"
                    id="maximumMapHeight"
                    value={formData().maximum_map_height}
                    onChange={(evt) => {
                      setFormData({
                        ...formData(),
                        maximum_map_height: mapValueInt(evt.target.value, 256),
                      });
                    }}
                  />
                </label>
              </div>
            </div>

            <div class={style.flexbox}>
              <div class={style["search-filter-textbox"]}>
                <label for="minimumHumanPlayers">
                  <I18nSpan text="search.minimum_human_players" />
                  <br />
                  <input
                    type="number"
                    min="0"
                    max="12"
                    id="minimumHumanPlayers"
                    value={formData().minimum_human_players}
                    onChange={(evt) => {
                      setFormData({
                        ...formData(),
                        minimum_human_players: mapValueInt(evt.target.value, 0),
                      });
                    }}
                  />
                </label>
              </div>

              <div class={style["search-filter-textbox"]}>
                <label for="maximumHumanPlayers">
                  <I18nSpan text="search.maximum_human_players" />
                  <br />
                  <input
                    type="number"
                    min="0"
                    max="12"
                    id="maximumHumanPlayers"
                    value={formData().maximum_human_players}
                    onChange={(evt) => {
                      setFormData({
                        ...formData(),
                        maximum_human_players: mapValueInt(
                          evt.target.value,
                          12
                        ),
                      });
                    }}
                  />
                </label>
              </div>

              <div class={style["search-filter-textbox"]}>
                <label for="minimumComputerPlayers">
                  <I18nSpan text="search.minimum_computer_players" />
                  <br />
                  <input
                    type="number"
                    min="0"
                    max="12"
                    id="minimumComputerPlayers"
                    value={formData().minimum_computer_players}
                    onChange={(evt) => {
                      setFormData({
                        ...formData(),
                        minimum_computer_players: mapValueInt(
                          evt.target.value,
                          0
                        ),
                      });
                    }}
                  />
                </label>
              </div>

              <div class={style["search-filter-textbox"]}>
                <label for="maximumComputerPlayers">
                  <I18nSpan text="search.maximum_computer_players" />
                  <br />
                  <input
                    type="number"
                    min="0"
                    max="12"
                    id="maximumComputerPlayers"
                    value={formData().maximum_computer_players}
                    onChange={(evt) => {
                      setFormData({
                        ...formData(),
                        maximum_computer_players: mapValueInt(
                          evt.target.value,
                          12
                        ),
                      });
                    }}
                  />
                </label>
              </div>
            </div>

            <div class={style.flexbox}>
              <div class={style["search-filter-textbox"]}>
                <label for="lastModifiedAfter">
                  <I18nSpan text="search.last_modified_after" />
                  <br />
                  <input
                    type="date"
                    id="lastModifiedAfter"
                    value={convertDateToLocalDate(
                      new Date(parseInt(formData().last_modified_after))
                    )}
                    onChange={(evt) => {
                      setFormData({
                        ...formData(),
                        last_modified_after: `${new Date(
                          evt.target.value + "T00:00"
                        ).getTime()}`,
                      });
                    }}
                  />
                </label>
              </div>

              <div class={style["search-filter-textbox"]}>
                <label for="lastModifiedBefore">
                  <I18nSpan text="search.last_modified_before" />
                  <br />
                  <input
                    type="date"
                    id="lastModifiedBefore"
                    value={convertDateToLocalDate(
                      new Date(parseInt(formData().last_modified_before))
                    )}
                    onChange={(evt) => {
                      setFormData({
                        ...formData(),
                        last_modified_before: `${new Date(
                          evt.target.value + "T00:00"
                        ).getTime()}`,
                      });
                    }}
                  />
                </label>
              </div>

              <div class={style["search-filter-textbox"]}>
                <label for="uploadedAfter">
                  <I18nSpan text="search.time_uploaded_after" />
                  <br />
                  <input
                    type="date"
                    id="uploadedAfter"
                    value={convertDateToLocalDate(
                      new Date(parseInt(formData().time_uploaded_after))
                    )}
                    onChange={(evt) => {
                      setFormData({
                        ...formData(),
                        time_uploaded_after: `${new Date(
                          evt.target.value + "T00:00"
                        ).getTime()}`,
                      });
                    }}
                  />
                </label>
              </div>

              <div class={style["search-filter-textbox"]}>
                <label for="uploadedBefore">
                  <I18nSpan text="search.time_uploaded_before" />
                  <br />
                  <input
                    type="date"
                    id="uploadedBefore"
                    value={convertDateToLocalDate(
                      new Date(parseInt(formData().time_uploaded_before))
                    )}
                    onChange={(evt) => {
                      setFormData({
                        ...formData(),
                        time_uploaded_before: `${new Date(
                          evt.target.value + "T00:00"
                        ).getTime()}`,
                      });
                    }}
                  />
                </label>
              </div>
            </div>

            <div class={style.flexbox}>
              <div class={style["search-filter-textbox"]}>
                <label for="uploadedBy">
                  <I18nSpan text="search.uploaded_by" />
                  <br />
                  <input
                    type="text"
                    id="uploadedBy"
                    placeholder={i18n_internal(lang(), "common.username")}
                    value={formData().uploaded_by}
                    onInput={(evt) => {
                      setFormData({
                        ...formData(),
                        uploaded_by: evt.target.value,
                      });
                    }}
                  />
                </label>
              </div>
            </div>

            <div class={style.flexbox}>
              <div class={style["search-flag"]}>
                <label for="include_broken">
                  <input
                    type="checkbox"
                    id="include_broken"
                    checked={formData().include_broken === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        include_broken:
                          formData().include_broken === "true" ? "false" : "true",
                      });
                    }}
                  />
                  <I18nSpan text="search.include_broken" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="include_outdated">
                  <input
                    type="checkbox"
                    id="include_outdated"
                    checked={formData().include_outdated === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        include_outdated:
                          formData().include_outdated === "true" ? "false" : "true",
                      });
                    }}
                  />
                  <I18nSpan text="search.include_outdated" />
                </label>
              </div>
              <div class={style["search-flag"]}>
                <label for="include_unfinished">
                  <input
                    type="checkbox"
                    id="include_unfinished"
                    checked={formData().include_unfinished === "true"}
                    onChange={() => {
                      setFormData({
                        ...formData(),
                        include_unfinished:
                          formData().include_unfinished === "true" ? "false" : "true",
                      });
                    }}
                  />
                  <I18nSpan text="search.include_unfinished" />
                </label>
              </div>
              <Show when={session()}>
                <div class={style["search-flag"]}>
                  <label for="include_nsfw">
                    <input
                      type="checkbox"
                      id="include_nsfw"
                      checked={formData().include_nsfw === "true"}
                      onChange={() => {
                        setFormData({
                          ...formData(),
                          include_nsfw:
                            formData().include_nsfw === "true" ? "false" : "true",
                        });
                      }}
                    />
                    <I18nSpan text="search.include_nsfw" />
                  </label>
                </div>
              </Show>
            </div>
          </div>
        </Show>
      </form>

      <h4
        class={style.dropdown}
        onClick={() => {
          setSortShown(!sortShown());
        }}
      >
        <I18nSpan text="search.sorting" />
        <span
          classList={{
            [style["dropdown-arrow"]]: !sortShown(),
            [style["dropdown-arrow-highlighted"]]: sortShown(),
          }}
        >
          ▼
        </span>
      </h4>
      <Show when={sortShown()}>
        <div class={style["search-flags"]}>
          <div class={style["sort-flag"]}>
            <label for="sort-relevancy">
              <input
                type="radio"
                id="sort-relevancy"
                name="sorting"
                value="relevancy"
                checked={formData().sort === "relevancy"}
                onClick={() => {
                  setFormData({
                    ...formData(),
                    sort: "relevancy",
                  });
                }}
              />
              <I18nSpan text="search.relevancy" />
            </label>
          </div>
          <div class={style["sort-flag"]}>
            <label for="sort-scenario">
              <input
                type="radio"
                id="sort-scenario"
                name="sorting"
                value="scenario"
                checked={formData().sort === "scenario"}
                onChange={() => {
                  setFormData({
                    ...formData(),
                    sort: "scenario",
                  });
                }}
              />
              <I18nSpan text="search.scenario_ascending" />
            </label>
          </div>
          <div class={style["sort-flag"]}>
            <label for="sort-scenario-desc">
              <input
                type="radio"
                id="sort-scenario-desc"
                name="sorting"
                value="scenariodesc"
                checked={formData().sort === "scenariodesc"}
                onChange={() => {
                  setFormData({
                    ...formData(),
                    sort: "scenariodesc",
                  });
                }}
              />
              <I18nSpan text="search.scenario_descending" />
            </label>
          </div>
          <div class={style["sort-flag"]}>
            <label for="sort-filename">
              <input
                type="radio"
                id="sort-filename"
                name="sorting"
                value="filename"
                checked={formData().sort === "filename"}
                onChange={() => {
                  setFormData({
                    ...formData(),
                    sort: "filename",
                  });
                }}
              />
              <I18nSpan text="search.filename_ascending" />
            </label>
          </div>
          <div class={style["sort-flag"]}>
            <label for="sort-filename-desc">
              <input
                type="radio"
                id="sort-filename-desc"
                name="sorting"
                value="filenamedesc"
                checked={formData().sort === "filenamedesc"}
                onChange={() => {
                  setFormData({
                    ...formData(),
                    sort: "filenamedesc",
                  });
                }}
              />
              <I18nSpan text="search.filename_descending" />
            </label>
          </div>
          <div class={style["sort-flag"]}>
            <label for="sort-last-modified-old">
              <input
                type="radio"
                id="sort-last-modified-old"
                name="sorting"
                value="lastmodifiedold"
                checked={formData().sort === "lastmodifiedold"}
                onChange={() => {
                  setFormData({
                    ...formData(),
                    sort: "lastmodifiedold",
                  });
                }}
              />
              <I18nSpan text="search.last_modified_oldest_first" />
            </label>
          </div>
          <div class={style["sort-flag"]}>
            <label for="sort-last-modified-new">
              <input
                type="radio"
                id="sort-last-modified-new"
                name="sorting"
                value="lastmodifiednew"
                checked={formData().sort === "lastmodifiednew"}
                onChange={() => {
                  setFormData({
                    ...formData(),
                    sort: "lastmodifiednew",
                  });
                }}
              />
              <I18nSpan text="search.last_modified_newest_first" />
            </label>
          </div>
          <div class={style["sort-flag"]}>
            <label for="sort-time-uploaded-old">
              <input
                type="radio"
                name="sorting"
                id="sort-time-uploaded-old"
                value="timeuploadedold"
                checked={formData().sort === "timeuploadedold"}
                onChange={() => {
                  setFormData({
                    ...formData(),
                    sort: "timeuploadedold",
                  });
                }}
              />
              <I18nSpan text="search.time_uploaded_oldest_first" />
            </label>
          </div>
          <div class={style["sort-flag"]}>
            <label for="sort-time-uploaded-new">
              <input
                type="radio"
                name="sorting"
                id="sort-time-uploaded-new"
                value="timeuploadednew"
                checked={formData().sort === "timeuploadednew"}
                onChange={() => {
                  setFormData({
                    ...formData(),
                    sort: "timeuploadednew",
                  });
                }}
              />
              <I18nSpan text="search.time_uploaded_newest_first" />
            </label>
          </div>
        </div>
      </Show>

      <h4 class={style.h4}>
        <I18nSpan text="search.results" />
        <span>
          : {isLoading() ? "" : numSearchResults() ? numSearchResults() : 0}
        </span>
      </h4>
      <Switch>
        <Match when={searchResults()?.length > 0}>
          <div class={style["table-container"]}>
            <table class={style.table}>
              <thead>
                <tr>
                  <th
                    class={style["click-sortable"]}
                    onClick={() => {
                      setFormData({
                        ...formData(),
                        // Ascending first: a name column reads A-Z by default,
                        // unlike the timestamp columns that lead with newest.
                        sort:
                          formData().sort === "scenario"
                            ? "scenariodesc"
                            : formData().sort === "scenariodesc"
                            ? "relevancy"
                            : "scenario",
                      });
                      setSortShown(false);
                      doNavigate(false);
                    }}
                  >
                    <I18nSpan text="search.scenario" />
                    <Switch>
                      <Match when={formData().sort === "scenario"}>▲</Match>
                      <Match when={formData().sort === "scenariodesc"}>▼</Match>
                    </Switch>
                  </th>
                  <th
                    class={style["click-sortable"]}
                    onClick={() => {
                      setFormData({
                        ...formData(),
                        sort:
                          formData().sort === "filename"
                            ? "filenamedesc"
                            : formData().sort === "filenamedesc"
                            ? "relevancy"
                            : "filename",
                      });
                      setSortShown(false);
                      doNavigate(false);
                    }}
                  >
                    <I18nSpan text="common.filename" />
                    <Switch>
                      <Match when={formData().sort === "filename"}>▲</Match>
                      <Match when={formData().sort === "filenamedesc"}>▼</Match>
                    </Switch>
                  </th>
                  <th
                    class={style["click-sortable"]}
                    onClick={() => {
                      setFormData({
                        ...formData(),
                        sort:
                          formData().sort === "lastmodifiednew"
                            ? "lastmodifiedold"
                            : formData().sort === "lastmodifiedold"
                            ? "relevancy"
                            : "lastmodifiednew",
                      });
                      setSortShown(false);
                      doNavigate(false);
                    }}
                  >
                    <I18nSpan text="common.last_modified" />
                    <Switch>
                      <Match when={formData().sort === "lastmodifiedold"}>
                        ▲
                      </Match>
                      <Match when={formData().sort === "lastmodifiednew"}>
                        ▼
                      </Match>
                    </Switch>
                  </th>
                  <th
                    class={style["click-sortable"]}
                    onClick={() => {
                      setFormData({
                        ...formData(),
                        sort:
                          formData().sort === "timeuploadednew"
                            ? "timeuploadedold"
                            : formData().sort === "timeuploadedold"
                            ? "relevancy"
                            : "timeuploadednew",
                      });
                      setSortShown(false);
                      doNavigate(false);
                    }}
                  >
                    <I18nSpan text="search.time_uploaded" />
                    <Switch>
                      <Match when={formData().sort === "timeuploadedold"}>
                        ▲
                      </Match>
                      <Match when={formData().sort === "timeuploadednew"}>
                        ▼
                      </Match>
                    </Switch>
                  </th>
                </tr>
              </thead>
              <tbody>
                <For each={searchResults()}>
                  {(searchResult, id) => (
                    <>
                      <tr>
                        <td class={style["scenario-name"]}>
                          <MinimapHover mapId={searchResult.id}>
                            <A
                              class={style["map-link"]}
                              href={`/map/${searchResult.id}`}
                            >
                              <ColoredTextMenu
                                text={searchResult.scenario_name}
                              />
                            </A>
                          </MinimapHover>
                        </td>
                        <td class={style.filename}>
                          <span class={style["filename-text"]}>
                            {searchResult.filename}
                          </span>
                        </td>
                        <td>
                          <span class={style.monospace}>
                            {unix_time_to_timestamp(searchResult.last_modified)}
                          </span>
                        </td>
                        <td>
                          <span class={style.monospace}>
                            {unix_time_to_timestamp(searchResult.uploaded_time)}
                          </span>
                        </td>
                      </tr>
                    </>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </Match>
        <Match when={isLoading()}>
          <div class={style.loading}>Loading</div>
        </Match>
      </Switch>
    </div>
  );
}
