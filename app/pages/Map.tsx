import { For, Suspense } from "solid-js";
import {
  A,
  BeforeLeaveEventArgs,
  useBeforeLeave,
  useLocation,
  useNavigate,
  useParams,
} from "@solidjs/router";

import { createSignal, createResource, Switch, Match, Show } from "solid-js";

import style from "./Map.module.scss";

import MinimapHover from "../modules/MinimapHover";
import {
  ColoredTextIngame,
  ColoredTextMenu,
  ColoredTextMenuNoNewlines,
} from "../modules/ColoredText";
import { I18nSpan } from "../modules/language";
import { useSession } from "../modules/context";
import MinimapImg from "../modules/MinimapImg";
import { unix_time_to_timestamp, useApi, useFetchImage } from "../util/util";
import {
  map_era_to_tileset_key,
  map_player_owners_to_strings,
  map_player_side_to_strings,
  map_ver_to_string,
  unit_id_to_name,
} from "../util/sc";
import MapImg from "../modules/MapImg";

const replay_frames_to_human_duration = (frames: number) => {
  const s = (frames * 42) / 1000;
  const seconds = s % 60;
  const minutes = s / 60;

  const seconds_part = `${seconds < 10 ? "0" : ""}${seconds.toFixed(2)}`;
  const minutes_part = `${minutes < 10 ? "0" : ""}${Math.trunc(minutes)}`;

  return `${minutes_part}:${seconds_part}`;
};

const Eud = (props: any) => (
  <div class={style["table-container"]}>
    <table class={style.table}>
      <tbody>
        {/* TODO: replace with actual EUD trigger render and what they actually do. */}
        <tr>
          <th>
            <I18nSpan text="map.is_eud_map" />
          </th>
          <td>
            <I18nSpan
              text={`${
                props.map.properties.eups > 0 ||
                props.map.properties.get_death_euds > 0 ||
                props.map.properties.set_death_euds > 0
              }`}
            />
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.eups" />
          </th>
          <td>
            <span>{props.map.properties.eups}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.get_death_euds" />
          </th>
          <td>
            <span>{props.map.properties.get_death_euds}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.set_death_epds" />
          </th>
          <td>
            <span>{props.map.properties.set_death_euds}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.triggerlist_reads" />
          </th>
          <td>
            <span>{props.map.properties.trigger_list_reads}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.triggerlist_writes" />
          </th>
          <td>
            <span>{props.map.properties.trigger_list_writes}</span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
);

const Wavs = (props: any) => (
  <div class={style["list-panel-tall"]}>
    <table class={style.table}>
      <thead>
        <tr>
          <th>
            <I18nSpan text="common.filename" />
          </th>
        </tr>
      </thead>
      <tbody>
        <For each={props.map.wavs}>
          {(wav, id) => (
            <tr>
              <td>
                <span>{wav}</span>
              </td>
            </tr>
          )}
        </For>
      </tbody>
    </table>
  </div>
);

const Meta = (props: any) => (
  <div class={style["table-container"]}>
    <table class={style.table}>
      {/* Downloads, Views, Last Downloaded, Last Viewed */}
      <tbody>
        <tr>
          <th>
            <I18nSpan text="map.mpq_hash" />
          </th>
          <td>
            <span>{props.map.meta.mpq_hash}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.mpq_size" />
          </th>
          <td>
            <span>{props.map.meta.mpq_size}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.chk_hash" />
          </th>
          <td>
            <span>{props.map.meta.chkhash}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.chk_size" />
          </th>
          <td>
            <span>{props.map.meta.chk_size}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.uploaded_by" />
          </th>
          <td>
            <A href={`/user/${props.map.meta.uploaded_by}`}>
              {props.map.meta.uploaded_by}
            </A>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.uploaded_on" />
          </th>
          <td>
            <span class={style.monospace}>
              {unix_time_to_timestamp(props.map.meta.uploaded_time)}
            </span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.last_viewed" />
          </th>
          <td>
            <span class={style.monospace}>
              {unix_time_to_timestamp(props.map.meta.last_viewed)}
            </span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.last_downloaded" />
          </th>
          <td>
            <span class={style.monospace}>
              {unix_time_to_timestamp(props.map.meta.last_downloaded)}
            </span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.views" />
          </th>
          <td>
            <span>{props.map.meta.views}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.downloads" />
          </th>
          <td>
            <span>{props.map.meta.downloads}</span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
);

const Forces = (props: any) => (
  <For each={props.map.forces}>
    {(force, id) => (
      <div>
        <Show when={force.player_ids.length > 0}>
          <div class={style.force}>
            <ColoredTextMenu text={force.name}> </ColoredTextMenu>
          </div>
          <For each={force.player_ids}>
            {(player_id, id) => (
              <div class={style["force-player"]}>
                <I18nSpan
                  text={map_player_owners_to_strings(
                    props.map.player_owners[player_id],
                  )}
                />{" "}
                (
                <I18nSpan
                  text={map_player_side_to_strings(
                    props.map.player_side[player_id],
                  )}
                />
                )
              </div>
            )}
          </For>
        </Show>
      </div>
    )}
  </For>
);

const KnownFilenames = (props: any) => (
  <div class={style["list-panel"]}>
    <table class={style.table}>
      <thead>
        <tr>
          <th>
            <I18nSpan text="common.filename" />
          </th>
        </tr>
      </thead>
      <tbody>
        <For each={props.filenames}>
          {(filename, id) => (
            <>
              <tr>
                <td>
                  <A
                    class={style["filename-download"]}
                    href={`/api/maps/${props.mpqHash}`}
                    download={filename}
                  >
                    {filename}
                  </A>
                </td>
              </tr>
            </>
          )}
        </For>
      </tbody>
    </table>
  </div>
);

const KnownFiletimes = (props: any) => (
  <div class={style["list-panel"]}>
    <table class={style.table}>
      <thead>
        <tr>
          <th>
            <I18nSpan text="map.last_modified_time" />
          </th>
        </tr>
      </thead>
      <tbody>
        <For each={props.filetimes}>
          {(filetime, id) => (
            <>
              <tr>
                <td>
                  <span class={style.monospace}>
                    {unix_time_to_timestamp(filetime)}
                  </span>
                </td>
              </tr>
            </>
          )}
        </For>
      </tbody>
    </table>
  </div>
);

const KnownFilenames2 = (props: any) => (
  <div class={style["list-panel"]}>
    <table class={style.table}>
      <thead>
        <tr>
          <th>
            <I18nSpan text="common.filename" />
          </th>
          <th>
            <I18nSpan text="map.last_modified_time" />
          </th>
        </tr>
      </thead>
      <tbody>
        <For each={props.filenames2}>
          {(v, id) => (
            <>
              <tr>
                <td>
                  <A
                    class={style["filename-download"]}
                    href={`/api/maps/${props.mpqHash}`}
                    download={v.filename}
                  >
                    {v.filename}
                  </A>
                </td>
                <td>
                  <span class={style.monospace}>
                    {unix_time_to_timestamp(v.modified_time)}
                  </span>
                </td>
              </tr>
            </>
          )}
        </For>
      </tbody>
    </table>
  </div>
);

const Replays = (props: any) => (
  <div class={style["list-panel-tall"]}>
    <table class={style.table}>
      <thead>
        <tr>
          <th>
            <I18nSpan text="map.duration" />
          </th>
          <th>
            <I18nSpan text="map.time_recorded" />
          </th>
          <th>
            <I18nSpan text="map.creator" />
          </th>
        </tr>
      </thead>
      <tbody>
        <For each={props.replays}>
          {(replay, id) => (
            <tr>
              <td>
                <span>{replay_frames_to_human_duration(replay.frames)}</span>
              </td>
              <td>
                <span>
                  <A href={`/replay/${replay.id}`}>
                    {unix_time_to_timestamp(replay.time_saved)}
                  </A>
                </span>
              </td>
              <td>
                <span>{replay.creator}</span>
              </td>
            </tr>
          )}
        </For>
      </tbody>
    </table>
  </div>
);

const Units = (props: any) => (
  <div class={style["list-panel-tall"]}>
    <table class={style.table}>
      <thead>
        <tr>
          <th>
            <I18nSpan text="map.unit_id" />
          </th>
          <th>
            <I18nSpan text="map.unit" />
          </th>
          <th>
            <I18nSpan text="map.name" />
          </th>
        </tr>
      </thead>
      <tbody>
        <For each={props.units}>
          {(unit, id) => (
            <>
              <tr>
                <td>
                  <span>{unit.unit_id}</span>
                </td>
                <td>
                  <span>
                    <I18nSpan text={unit_id_to_name(unit.unit_id)} />
                  </span>
                </td>
                <td>
                  <ColoredTextIngame text={unit.name} />
                </td>
              </tr>
            </>
          )}
        </For>
      </tbody>
    </table>
  </div>
);

const SimilarMaps = (props: any) => {
  const [similarMaps] = useApi(() => `/api/similar_maps/${props.mapId}`);

  return (
    <Show when={similarMaps()?.v2?.length > 0}>
      <section class={style.card}>
        <h3 class={style.h3}>
          <I18nSpan text="map.similar_maps" />
        </h3>
        <div class={style["similar-maps"]}>
          <For each={similarMaps().v2}>
            {(map, id) => (
              <div class={style["similar-map"]}>
                <A
                  class={style["similar-maps-minimap"]}
                  href={`/map/${map.map_id}`}
                >
                  <MinimapImg
                    mapId={map.map_id}
                    max-width="128"
                    max-height="128"
                  />{" "}
                </A>
                <A href={`/map/${map.map_id}`}>
                  <div class={style["similar-maps-scenario"]}>
                    <ColoredTextMenu text={map.scenario_name} />
                  </div>
                  <div class={style["similar-maps-details"]}>
                    {unix_time_to_timestamp(map.last_modified_time)}
                  </div>
                  <div class={style["similar-maps-details"]}>
                    {map.width}x{map.height}
                  </div>
                  <div class={style["similar-maps-details"]}>
                    <I18nSpan text={map_era_to_tileset_key(map.tileset % 8)} />
                  </div>
                </A>
              </div>
            )}
          </For>
        </div>
      </section>
    </Show>
  );
};

const ScenarioProperties = (props: any) => (
  <div class={style["table-container"]}>
    <table class={style.table}>
      <tbody>
        <tr>
          <th>
            <I18nSpan text="map.version" />
          </th>
          <td>
            <I18nSpan text={`${map_ver_to_string(props.map.properties.ver)}`} />
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.tileset" />
          </th>
          <td>
            <I18nSpan
              text={map_era_to_tileset_key(props.map.properties.tileset % 8)}
            />{" "}
            <span>{`(${props.map.properties.tileset} mod 8 = ${
              props.map.properties.tileset % 8
            })`}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.dimensions" />
          </th>
          <td>
            <span>{`${props.map.properties.width}x${props.map.properties.height}`}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.locations" />
          </th>
          <td>
            <span>{props.map.properties.locations}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="common.units" />
          </th>
          <td>
            <span>{props.map.properties.units}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.doodads" />
          </th>
          <td>
            <span>{props.map.properties.doodads}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.sprites" />
          </th>
          <td>
            <span>{props.map.properties.sprites}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.triggers" />
          </th>
          <td>
            <span>{props.map.properties.triggers}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="map.briefing_triggers" />
          </th>
          <td>
            <span>{props.map.properties.briefing_triggers}</span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
);

const Flags = (props: { mapId: string; uploadedBy: string }) => {
  const [session] = useSession();
  const [nsfw] = useApi(() => `/api/flags/${props.mapId}/nsfw`);
  const [unfinished] = useApi(() => `/api/flags/${props.mapId}/unfinished`);
  const [outdated] = useApi(() => `/api/flags/${props.mapId}/outdated`);
  const [broken] = useApi(() => `/api/flags/${props.mapId}/broken`);
  const [blackholed] = useApi(() => `/api/flags/${props.mapId}/blackholed`);
  const [spoiler_unit_names] = useApi(
    () => `/api/flags/${props.mapId}/spoiler_unit_names`,
  );

  const canModifyFlags = () => {
    const username = session();
    return username === props.uploadedBy || username === "RagE";
  };

  const mutate = (mapId: string, key: string, value: boolean) => {
    fetch(`/api/flags/${mapId}/${key}`, {
      method: "POST",
      credentials: "include",
      cache: "no-cache",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(value),
    });
  };

  return (
    <Suspense>
      <div
        class={`${style.flags} ${!canModifyFlags() ? style["flags-disabled"] : ""}`}
      >
        <div class={style.flag}>
          <label for="checkbox_nsfw">
            <input
              type="checkbox"
              id="checkbox_nsfw"
              checked={nsfw()}
              disabled={!canModifyFlags()}
              onChange={(evt) => {
                mutate(props.mapId, "nsfw", evt.target.checked);
              }}
            />
            <I18nSpan text="map.nsfw" />
          </label>
        </div>
        <div class={style.flag}>
          <label for="checkbox_unfinished">
            <input
              type="checkbox"
              id="checkbox_unfinished"
              checked={unfinished()}
              disabled={!canModifyFlags()}
              onChange={(evt) => {
                mutate(props.mapId, "unfinished", evt.target.checked);
              }}
            />
            <I18nSpan text="map.unfinished" />
          </label>
        </div>
        <div class={style.flag}>
          <label for="checkbox_outdated">
            <input
              type="checkbox"
              id="checkbox_outdated"
              checked={outdated()}
              disabled={!canModifyFlags()}
              onChange={(evt) => {
                mutate(props.mapId, "outdated", evt.target.checked);
              }}
            />
            <I18nSpan text="map.outdated" />
          </label>
        </div>
        <div class={style.flag}>
          <label for="checkbox_broken">
            <input
              type="checkbox"
              id="checkbox_broken"
              checked={broken()}
              disabled={!canModifyFlags()}
              onChange={(evt) => {
                mutate(props.mapId, "broken", evt.target.checked);
              }}
            />
            <I18nSpan text="map.broken" />
          </label>
        </div>
        <div class={style.flag}>
          <label for="checkbox_blackholed">
            <input
              type="checkbox"
              id="checkbox_blackholed"
              checked={blackholed()}
              disabled={!canModifyFlags()}
              onChange={(evt) => {
                mutate(props.mapId, "blackholed", evt.target.checked);
              }}
            />
            <I18nSpan text="map.black_holed" />
          </label>
        </div>
        <div class={style.flag}>
          <label for="checkbox_spoiler_unit_names">
            <input
              type="checkbox"
              id="checkbox_spoiler_unit_names"
              checked={spoiler_unit_names()}
              disabled={!canModifyFlags()}
              onChange={(evt) => {
                mutate(props.mapId, "spoiler_unit_names", evt.target.checked);
              }}
            />
            <I18nSpan text="map.spoiler_unit_names" />
          </label>
        </div>
      </div>
    </Suspense>
  );
};

const Tags = (props: any) => {
  const [tags] = useApi(() => `/api/tags/${props.mapId}`);

  return (
    <Show when={tags()}>
      <div class={style["list-panel"]}>
        <table class={style.table}>
          <thead>
            <tr>
              <th>
                <I18nSpan text="map.key" />
              </th>
              <th>
                <I18nSpan text="map.value" />
              </th>
            </tr>
          </thead>
          <tbody>
            <For each={tags()}>
              {(kv, id) => (
                <>
                  <tr>
                    <td>
                      <span>{kv.key}</span>
                    </td>
                    <td>
                      <span>{kv.value}</span>
                    </td>
                  </tr>
                </>
              )}
            </For>
          </tbody>
        </table>
      </div>
    </Show>
  );
};

const Admin = (props: any) => {
  const [session, _] = useSession();

  return (
    <Show when={props.map}>
      <Show when={"RagE" === session()}>
        <h3 class={style.h3}>
          <I18nSpan text="map.admin" />
        </h3>
        <button onClick={() => fetch(`/api/denormalize/${props.map_id}`)}>
          Denormalize
        </button>
        <ul style="color: red;">
          <li>Internal Id: {props.map.internal_id}</li>
        </ul>
      </Show>
    </Show>
  );
};

export default function (prop: any) {
  // const [username, setUsername] = createSignal("");
  // const [password, setPassword] = createSignal("");
  // const [session, setSession] = useSession();
  const params = useParams();
  const location = useLocation();
  const navigate = useNavigate();

  const [map] = useApi(() => `/api/uiv2/map_info/${params.mapId}`);
  const [filenames] = useApi(() => `/api/uiv2/filenames/${params.mapId}`);
  const [filetimes] = useApi(() => `/api/uiv2/timestamps/${params.mapId}`);
  const [replays] = useApi(() => `/api/uiv2/replays/${params.mapId}`);
  const [units] = useApi(() => `/api/uiv2/units/${params.mapId}`);
  const [filenames2] = useApi(() => `/api/uiv2/filenames2/${params.mapId}`);
  const [mapImage] = useFetchImage(() =>
    map()?.meta?.chkhash ? `/api/chk/${map().meta.chkhash}/map_img` : undefined,
  );

  return (
    <>
      <div class={style["vertical-container"]}>
        {/* TODO: */}
        <Show when={location.hash != ""}>
          <a
            class={style["continue-random-button"]}
            onClick={async () => {
              const q = location.hash.substring(1);
              const qq = new URLSearchParams(q);
              const map_id = await (
                await fetch(
                  `/api/uiv2/random/${
                    qq.get("query") ?? ""
                  }?${location.hash.substring(1)}`,
                )
              ).json();

              navigate(`/map/${map_id}#${q}`);
            }}
          >
            <I18nSpan text="map.next_random_map" />
          </a>
        </Show>

        <Show when={filenames() && filetimes() && replays() && units()}>
          <Show when={map()} keyed>
            <header class={style.header}>
              <h1 class={style.h1}>
                <ColoredTextMenu text={map().scenario} />
              </h1>
              <h2 class={style.h2}>
                <ColoredTextMenu text={map().scenario_description} />
              </h2>
              <a
                class={style["download-button"]}
                href={`/api/maps/${map()?.meta.mpq_hash}`}
                download={filenames()[0]}
              >
                <I18nSpan text="map.download" /> (
                {(map()?.meta.mpq_size / 1024) | 0}
                KB)
              </a>
            </header>
            {/* Three fixed columns above the breakpoint, one stack below it.
                A panel's position is decided by which column it is written
                into, not by how tall the map's data happens to make the
                panels before it, so the page is laid out the same way on
                every map. Moving a panel between columns changes where users
                find it -- treat the grouping as the contract it is. */}
            <div class={style.columns}>
              {/* The art, and the other maps that look like it. Fixed-width
                  rail: both images are capped at 512px by their own components,
                  so this column never resizes. */}
              <div class={style.column}>
                <section class={style.card}>
                  <h3 class={style.h3}>
                    <I18nSpan text="map.minimap" />
                  </h3>
                  <div class={style.minimap}>
                    <MinimapImg
                      mapId={params.mapId}
                      max-width={512}
                      max-height={512}
                    />
                  </div>
                </section>
                <Show when={mapImage()}>
                  <section class={style.card}>
                    <h3 class={style.h3}>
                      <I18nSpan text="map.map_image" />
                    </h3>
                    <div class={style.mapimg}>
                      <MapImg url={mapImage()} />
                    </div>
                  </section>
                </Show>
                {/* In the rail rather than full width under the columns. Four
                    tiles across the page left each one too narrow for a
                    timestamp, so the details wrapped; one tile across a 518px
                    column has room to spare. It also gives the rail something
                    to do -- it is two images tall and the lists column is not,
                    so the whitespace was all on this side. */}
                <SimilarMaps mapId={params.mapId} />
              </div>

              {/* What the map is. Every panel here is a fixed or near-fixed
                  number of rows, which is why they share a column: it barely
                  changes height between maps, so it reads next to the art
                  without either one shifting. */}
              <div class={style.column}>
                <section class={style.card}>
                  <h3 class={style.h3}>
                    <I18nSpan text="map.scenario_properties" />
                  </h3>
                  <ScenarioProperties map={map()} />
                </section>
                <section class={style.card}>
                  <h3 class={style.h3}>
                    <I18nSpan text="common.forces" />
                  </h3>
                  <Forces map={map()} />
                </section>
                <section class={style.card}>
                  <h3 class={style.h3}>
                    <I18nSpan text="map.eud" />
                  </h3>
                  <Eud map={map()} />
                </section>
                <section class={style.card}>
                  <h3 class={style.h3}>
                    <I18nSpan text="map.meta" />
                  </h3>
                  <Meta map={map()} />
                </section>
              </div>

              {/* Lists and actions. These are the panels whose length is map
                  data -- a map can have two filenames or nine hundred -- so
                  they are kept together and that variance is confined to one
                  column instead of being spread across the page. The two
                  fixed-size ones lead so the top of the column is stable. */}
              <div class={style.column}>
                <section class={style.card}>
                  <h3 class={style.h3}>
                    <I18nSpan text="map.flags" />
                  </h3>
                  <Flags
                    mapId={params.mapId}
                    uploadedBy={map().meta.uploaded_by}
                  />
                </section>
                <section class={style.card}>
                  <h3 class={style.h3}>
                    <I18nSpan text="map.tags" />
                  </h3>
                  <Tags mapId={params.mapId} />
                </section>
                <Show when={replays()?.length > 0}>
                  <section class={style.card}>
                    <h3 class={style.h3}>
                      <I18nSpan text="map.replays" />
                    </h3>
                    <Replays replays={replays()} />
                  </section>
                </Show>
                <section class={style.card}>
                  <h3 class={style.h3}>
                    <I18nSpan text="map.known_filenames" />
                  </h3>
                  <KnownFilenames2
                    filenames2={filenames2()}
                    mpqHash={map().meta.mpq_hash}
                  />
                </section>
                <section class={style.card}>
                  <h3 class={style.h3}>
                    <I18nSpan text="map.known_filenames" />
                  </h3>
                  <KnownFilenames
                    filenames={filenames()}
                    mpqHash={map().meta.mpq_hash}
                  />
                </section>
                <section class={style.card}>
                  <h3 class={style.h3}>
                    <I18nSpan text="map.known_timestamps" />
                  </h3>
                  <KnownFiletimes filetimes={filetimes()} />
                </section>
                <Show when={map().wavs?.length > 0}>
                  <section class={style.card}>
                    <h3 class={style.h3}>
                      <I18nSpan text="map.wavs" />
                    </h3>
                    <Wavs map={map()} />
                  </section>
                </Show>
                <Show when={units()?.length > 0}>
                  <section class={style.card}>
                    <h3 class={style.h3}>
                      <I18nSpan text="common.units" />
                    </h3>
                    <Units units={units()} />
                  </section>
                </Show>
              </div>
            </div>
            <Admin map={map()} map_id={params.mapId} />
          </Show>
        </Show>
      </div>
    </>
  );
}
