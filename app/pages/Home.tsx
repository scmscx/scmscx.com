import { For, createEffect } from "solid-js";

import { createSignal, createResource, Switch, Match, Show } from "solid-js";

import "../global.scss";
import style from "./Home.module.scss";

import MinimapHover from "../modules/MinimapHover";
import { ColoredTextMenuNoNewlines } from "../modules/ColoredText";
import { I18nSpan } from "../modules/language";
import { useApi } from "../util/util";
import { A } from "@solidjs/router";

const MapBox = (props: any) => {
  const [resource] = useApi(() => props.url);

  return (
    <>
      <div class={style.container}>
        <h4 class={style.h4}>
          <I18nSpan text={props.h4} />
        </h4>
        <ul class={style.ul}>
          <Switch>
            <Match when={resource.loading}>
              <For each={[1, 2, 3, 4, 5]}>
                {(map, i) => (
                  <li class={style.li}>
                    <div class={style.padding}>&nbsp;</div>
                  </li>
                )}
              </For>
            </Match>
            <Match when={resource()}>
              <For each={resource()}>
                {(map, i) => (
                  <li class={style.li}>
                    <MinimapHover mapId={map.map_id}>
                      <A class={style.a} href={`/map/${map.map_id}`}>
                        <ColoredTextMenuNoNewlines text={map.scenario_name} />
                      </A>
                    </MinimapHover>
                  </li>
                )}
              </For>
            </Match>
          </Switch>
        </ul>
      </div>
    </>
  );
};

const ReplayBox = (props: any) => {
  const [resource] = useApi(() => props.url);

  return (
    <>
      <div class={style.container}>
        <h4 class={style.h4}>
          <I18nSpan text={props.h4} />
        </h4>
        <ul class={style.ul}>
          <Switch>
            <Match when={resource.loading}>
              <For each={[1, 2, 3, 4, 5]}>
                {(map, i) => (
                  <li class={style.li}>
                    <div class={style.padding}>&nbsp;</div>
                  </li>
                )}
              </For>
            </Match>
            <Match when={resource()}>
              <For each={resource()}>
                {(map, i) => (
                  <li class={style.li}>
                    <MinimapHover mapId={map.map_id}>
                      <A class={style.a} href={`/replay/${map.replay_id}`}>
                        <ColoredTextMenuNoNewlines text={map.scenario_name} />
                      </A>
                    </MinimapHover>
                  </li>
                )}
              </For>
            </Match>
          </Switch>
        </ul>
      </div>
    </>
  );
};

export default function (props: any) {
  return (
    <>
      {/* prettier-ignore */}
      <>
        <h1 class={style.h1}>
            <I18nSpan text="Welcome to scmscx.com" />
        </h1>
        <h2 class={style.h2}>
            <I18nSpan text="The largest StarCraft: Brood War map database in the universe" />
        </h2>

        <div class={style["vertical-container"]} >

          <MapBox h4="Featured Maps" url={`/api/uiv2/featured_maps`}/>
          <MapBox h4="Recently Viewed Maps" url={`/api/uiv2/last_viewed_maps`} />
          <MapBox h4="Recently Downloaded Maps" url={`/api/uiv2/last_downloaded_maps`} />
          <MapBox h4="Recently Uploaded Maps" url={`/api/uiv2/last_uploaded_maps`} />
          {/* <ReplayBox h4="Recently Uploaded Replays" url={`/api/uiv2/last_uploaded_replays`} /> */}
          {/* <MapBox h4="Most Viewed Maps" url={`/api/uiv2/most_viewed_maps`} />
          <MapBox h4="Most Downloaded Maps" url={`/api/uiv2/most_downloaded_maps`} /> */}
        </div>
      </>
    </>
  );
}
