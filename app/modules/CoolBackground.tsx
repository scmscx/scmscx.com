import { onCleanup, onMount } from "solid-js";

import style from "./CoolBackground.module.scss";

// One entry per layer: the variable it drives, how much of the scroll offset
// it takes, and the tile height its offset wraps against.
//
// The rate is what separates the layers in depth -- smaller drifts less and so
// reads as further away. The tile heights must match the background-size given
// to each layer in CoolBackground.module.scss; wrapping against the wrong one
// would make the layer jump instead of repeating seamlessly.
const LAYERS = [
  { name: "--parallax-1", rate: 0.12, tileHeight: 768 },
  { name: "--parallax-3", rate: 0.28, tileHeight: 1024 },
  { name: "--parallax-5", rate: 0.5, tileHeight: 1408 },
] as const;

export default function (props: any) {
  onMount(() => {
    let queued = false;

    const publish = () => {
      queued = false;
      const y = window.scrollY;
      for (const { name, rate, tileHeight } of LAYERS) {
        // Wrap into a single tile. The background repeats, so a layer offset
        // by n tiles is indistinguishable from one offset by none -- and this
        // is what lets the layer stay a fixed height instead of having to span
        // the whole page.
        const offset = -((y * rate) % tileHeight);
        document.documentElement.style.setProperty(name, `${offset}px`);
      }
    };

    const onScroll = () => {
      if (queued) return;
      queued = true;
      requestAnimationFrame(publish);
    };

    publish();
    window.addEventListener("scroll", onScroll, { passive: true });

    onCleanup(() => {
      window.removeEventListener("scroll", onScroll);
      for (const { name } of LAYERS) {
        document.documentElement.style.removeProperty(name);
      }
    });
  });

  return (
    <div id="content" class={style.root}>
      <div class={style.layer1} />
      <div class={style.layer3} />
      <div class={style.layer5} />
      <div class={style.content}>{props.children}</div>
    </div>
  );
}
