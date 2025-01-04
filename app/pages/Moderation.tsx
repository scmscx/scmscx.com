import { For, createEffect } from "solid-js";
import { A, useNavigate } from "@solidjs/router";

import { createSignal } from "solid-js";

import style from "./Moderation.module.scss";

import { useApi } from "../util/util";
import MinimapImg from "../modules/MinimapImg";

export default function (props: any) {
  const [maps] = useApi(() => `/api/get_selection_of_random_maps`);
  const [actualMaps, setActualMaps] = createSignal<any[] | undefined>([]);

  createEffect(() => {
    setActualMaps(maps()?.slice(0, 25));
  });

  return (
    <>
      <h3 class={style.h3}>Maps for Review: {maps()?.length}</h3>

      <button
        class={style["moderate-all-button"]}
        onClick={() => {
          const promises = [];

          for (const map of actualMaps() || []) {
            promises.push(
              fetch(`/api/addtags/${map.map_id}`, {
                method: "POST",
                headers: {
                  "Content-Type": "application/json",
                },
                body: JSON.stringify([
                  { key: "minimap_checked", value: "true" },
                ]),
              })
            );
          }

          Promise.all(promises).then(
            () => {
              alert("all marked");
            },
            (e) => {
              console.log(e);
              alert(`Some error: ${e}`);
            }
          );
        }}
      >
        Moderate All
      </button>
      <div class={style["container"]}>
        <For each={actualMaps()}>
          {(map, index) => {
            return (
              <div class={style.block}>
                <div class={style.spacer}>
                  <A href={`/map/${map.map_id}`} class={style.minimap}>
                    <MinimapImg
                      mapId={map.map_id}
                      max-width="72"
                      max-height="72"
                    />
                  </A>
                </div>
                <button
                  class={style["mark-button"]}
                  onClick={() => {
                    console.log("marked: {}", map.map_id);

                    fetch(`/api/flags/${map.map_id}/nsfw`, {
                      method: "POST",
                      headers: {
                        "Content-Type": "application/json",
                      },
                      body: JSON.stringify(true),
                    }).then(
                      () => alert("marked"),
                      (e) => {
                        console.log(e);
                        alert(`Failed to mark: ${e}`);
                      }
                    );
                  }}
                >
                  NSFW
                </button>
              </div>
            );
          }}
        </For>
      </div>
    </>
  );
}
