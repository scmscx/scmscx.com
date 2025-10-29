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
  map_era_to_tileset,
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
            <I18nSpan text="Is EUD map?" />
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
            <I18nSpan text="EUPs" />
          </th>
          <td>
            <span>{props.map.properties.eups}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Get Death EUDs" />
          </th>
          <td>
            <span>{props.map.properties.get_death_euds}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Set Death EPDs" />
          </th>
          <td>
            <span>{props.map.properties.set_death_euds}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="TriggerList reads" />
          </th>
          <td>
            <span>{props.map.properties.trigger_list_reads}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="TriggerList writes" />
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
  <div class={style["table-container"]}>
    <table class={style.table}>
      <thead>
        <tr>
          <th>
            <I18nSpan text="Filename" />
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
            <I18nSpan text="MPQ Hash" />
          </th>
          <td>
            <span>{props.map.meta.mpq_hash}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="MPQ Size" />
          </th>
          <td>
            <span>{props.map.meta.mpq_size}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="CHK Hash" />
          </th>
          <td>
            <span>{props.map.meta.chkhash}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="CHK Size" />
          </th>
          <td>
            <span>{props.map.meta.chk_size}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Uploaded by" />
          </th>
          <td>
            <A href={`/user/${props.map.meta.uploaded_by}`}>
              {props.map.meta.uploaded_by}
            </A>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Uploaded On" />
          </th>
          <td>
            <span class={style.monospace}>
              {unix_time_to_timestamp(props.map.meta.uploaded_time)}
            </span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Last Viewed" />
          </th>
          <td>
            <span class={style.monospace}>
              {unix_time_to_timestamp(props.map.meta.last_viewed)}
            </span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Last Downloaded" />
          </th>
          <td>
            <span class={style.monospace}>
              {unix_time_to_timestamp(props.map.meta.last_downloaded)}
            </span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Views" />
          </th>
          <td>
            <span>{props.map.meta.views}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Downloads" />
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
                    props.map.player_owners[player_id]
                  )}
                />{" "}
                (
                <I18nSpan
                  text={map_player_side_to_strings(
                    props.map.player_side[player_id]
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
  <div class={style["table-container"]}>
    <table class={style.table}>
      <thead>
        <tr>
          <th>
            <I18nSpan text="Filename" />
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
  <div class={style["table-container"]}>
    <table class={style.table}>
      <thead>
        <tr>
          <th>
            <I18nSpan text="Last Modified Time" />
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
  <div class={style["table-container"]}>
    <table class={style.table}>
      <thead>
        <tr>
          <th>
            <I18nSpan text="Filename" />
          </th>
          <th>
            <I18nSpan text="Last Modified Time" />
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
  <div class={style["table-container"]}>
    <table class={style.table}>
      <thead>
        <tr>
          <th>
            <I18nSpan text="Duration" />
          </th>
          <th>
            <I18nSpan text="Time Recorded" />
          </th>
          <th>
            <I18nSpan text="Creator" />
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
  <div class={style["table-container"]}>
    <table class={style.table}>
      <thead>
        <tr>
          <th>
            <I18nSpan text="Unit" />
          </th>
          <th>
            <I18nSpan text="Name" />
          </th>
        </tr>
      </thead>
      <tbody>
        <For each={props.units}>
          {(unit, id) => (
            <>
              <tr>
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
    <>
      {/* <Show when={similarMaps()}>
        <div class={style["similar-maps"]}>
          <For each={similarMaps().v1}>
            {(map, id) => (
              <>
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
                    <I18nSpan text={map_era_to_tileset(map.tileset % 8)} />
                  </div>
                </A>
              </>
            )}
          </For>
        </div>
      </Show> */}
      <Show when={similarMaps()}>
        <div class={style["similar-maps"]}>
          <For each={similarMaps().v2}>
            {(map, id) => (
              <>
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
                    <I18nSpan text={map_era_to_tileset(map.tileset % 8)} />
                  </div>
                </A>
              </>
            )}
          </For>
        </div>
      </Show>
    </>
  );
};

const ScenarioProperties = (props: any) => (
  <div class={style["table-container"]}>
    <table class={style.table}>
      <tbody>
        <tr>
          <th>
            <I18nSpan text="Version" />
          </th>
          <td>
            <I18nSpan text={`${map_ver_to_string(props.map.properties.ver)}`} />
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Tileset" />
          </th>
          <td>
            <I18nSpan
              text={`${map_era_to_tileset(props.map.properties.tileset % 8)}`}
            />{" "}
            <span>{`(${props.map.properties.tileset} mod 8 = ${
              props.map.properties.tileset % 8
            })`}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Dimensions" />
          </th>
          <td>
            <span>{`${props.map.properties.width}x${props.map.properties.height}`}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Locations" />
          </th>
          <td>
            <span>{props.map.properties.locations}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Units" />
          </th>
          <td>
            <span>{props.map.properties.units}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Doodads" />
          </th>
          <td>
            <span>{props.map.properties.doodads}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Sprites" />
          </th>
          <td>
            <span>{props.map.properties.sprites}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Triggers" />
          </th>
          <td>
            <span>{props.map.properties.triggers}</span>
          </td>
        </tr>
        <tr>
          <th>
            <I18nSpan text="Briefing Triggers" />
          </th>
          <td>
            <span>{props.map.properties.briefing_triggers}</span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
);

const Flags = (props: any) => {
  const [nsfw] = useApi(() => `/api/flags/${props.mapId}/nsfw`);
  const [unfinished] = useApi(() => `/api/flags/${props.mapId}/unfinished`);
  const [outdated] = useApi(() => `/api/flags/${props.mapId}/outdated`);
  const [broken] = useApi(() => `/api/flags/${props.mapId}/broken`);
  const [blackholed] = useApi(() => `/api/flags/${props.mapId}/blackholed`);
  const [spoiler_unit_names] = useApi(
    () => `/api/flags/${props.mapId}/spoiler_unit_names`
  );

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
      <div class={style.flags}>
        <div class={style.flag}>
          <label for="checkbox_nsfw">
            <input
              type="checkbox"
              id="checkbox_nsfw"
              checked={nsfw()}
              onChange={(evt) => {
                mutate(props.mapId, "nsfw", evt.target.checked);
              }}
            />
            <I18nSpan text="NSFW" />
          </label>
        </div>
        <div class={style.flag}>
          <label for="checkbox_unfinished">
            <input
              type="checkbox"
              id="checkbox_unfinished"
              checked={unfinished()}
              onChange={(evt) => {
                mutate(props.mapId, "unfinished", evt.target.checked);
              }}
            />
            <I18nSpan text="Unfinished" />
          </label>
        </div>
        <div class={style.flag}>
          <label for="checkbox_outdated">
            <input
              type="checkbox"
              id="checkbox_outdated"
              checked={outdated()}
              onChange={(evt) => {
                mutate(props.mapId, "outdated", evt.target.checked);
              }}
            />
            <I18nSpan text="Outdated" />
          </label>
        </div>
        <div class={style.flag}>
          <label for="checkbox_broken">
            <input
              type="checkbox"
              id="checkbox_broken"
              checked={broken()}
              onChange={(evt) => {
                mutate(props.mapId, "broken", evt.target.checked);
              }}
            />
            <I18nSpan text="Broken" />
          </label>
        </div>
        <div class={style.flag}>
          <label for="checkbox_blackholed">
            <input
              type="checkbox"
              id="checkbox_blackholed"
              checked={blackholed()}
              onChange={(evt) => {
                mutate(props.mapId, "blackholed", evt.target.checked);
              }}
            />
            <I18nSpan text="Black Holed" />
          </label>
        </div>
        <div class={style.flag}>
          <label for="checkbox_spoiler_unit_names">
            <input
              type="checkbox"
              id="checkbox_spoiler_unit_names"
              checked={spoiler_unit_names()}
              onChange={(evt) => {
                mutate(props.mapId, "spoiler_unit_names", evt.target.checked);
              }}
            />
            <I18nSpan text="Spoiler Unit Names" />
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
      <div class={style["table-container"]}>
        <table class={style.table}>
          <thead>
            <tr>
              <th>
                <I18nSpan text="Key" />
              </th>
              <th>
                <I18nSpan text="Value" />
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
          <I18nSpan text="Admin" />
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
  const [mapImage] = useFetchImage(() => `/api/uiv2/img/${params.mapId}`);

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
                  }?${location.hash.substring(1)}`
                )
              ).json();

              navigate(`/map/${map_id}#${q}`);
            }}
          >
            <I18nSpan text="Next Random Map" />
          </a>
        </Show>

        <Show when={filenames() && filetimes() && replays() && units()}>
          <Show when={map()} keyed>
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
              <I18nSpan text="Download" /> ({(map()?.meta.mpq_size / 1024) | 0}
              KB)
            </a>
            <h3 class={style.h3}>
              <I18nSpan text="Minimap" />
            </h3>
            <div class={style.minimap}>
              <MinimapImg
                mapId={params.mapId}
                max-width={512}
                max-height={512}
              />
            </div>
            <Show when={mapImage()}>
              <h3 class={style.h3}>
                <I18nSpan text="Map Image" />
              </h3>
              <div class={style.mapimg}>
                <MapImg url={mapImage()} />
              </div>
            </Show>
            <h3 class={style.h3}>
              <I18nSpan text="Scenario Properties" />
            </h3>
            <ScenarioProperties map={map()} />
            <h3 class={style.h3}>
              <I18nSpan text="Forces" />
            </h3>
            <Forces map={map()} />
            <h3 class={style.h3}>
              <I18nSpan text="Replays" />
            </h3>
            <Replays replays={replays()} />
            <h3 class={style.h3}>
              <I18nSpan text="Known Filenames" />
            </h3>
            <KnownFilenames
              filenames={filenames()}
              mpqHash={map().meta.mpq_hash}
            />
            <h3 class={style.h3}>
              <I18nSpan text="Known Timestamps" />
            </h3>
            <KnownFiletimes filetimes={filetimes()} />
            <h3 class={style.h3}>
              <span>
                <I18nSpan text="Known Filenames" />
                (beta)
              </span>
            </h3>
            <KnownFilenames2
              filenames2={filenames2()}
              mpqHash={map().meta.mpq_hash}
            />
            <h3 class={style.h3}>
              <I18nSpan text="EUD" />
            </h3>
            <Eud map={map()} />
            <h3 class={style.h3}>
              <I18nSpan text="Units" />
            </h3>
            <Units units={units()} />
            <h3 class={style.h3}>
              <I18nSpan text="Wavs" />
            </h3>
            <Wavs map={map()} />
            <h3 class={style.h3}>
              <I18nSpan text="Similar Maps" />
            </h3>
            <SimilarMaps mapId={params.mapId} />
            <h3 class={style.h3}>
              <I18nSpan text="Flags" />
            </h3>
            <Flags mapId={params.mapId} />
            <h3 class={style.h3}>
              <I18nSpan text="Tags" />
            </h3>
            <Tags mapId={params.mapId} />
            <h3 class={style.h3}>
              <I18nSpan text="Meta" />
            </h3>
            <Meta map={map()} />
            <Admin map={map()} map_id={params.mapId} />
          </Show>
        </Show>
      </div>
    </>
  );
}
