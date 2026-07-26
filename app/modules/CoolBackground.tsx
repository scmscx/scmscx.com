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
  { name: "--parallax-1", cycles: "--parallax-cycles-1", rate: 0.12, tileHeight: 768 },
  { name: "--parallax-3", cycles: "--parallax-cycles-3", rate: 0.28, tileHeight: 1024 },
  { name: "--parallax-5", cycles: "--parallax-cycles-5", rate: 0.5, tileHeight: 1408 },
] as const;

// Where the browser can drive a scroll-timeline animation, the drift belongs on
// the compositor -- see the @supports block in the stylesheet for why. Then the
// only thing left for JS is how many tile-sized iterations span the page, which
// changes when the page does, not when it scrolls.
const SCROLL_DRIVEN =
  typeof CSS !== "undefined" &&
  CSS.supports &&
  CSS.supports("animation-timeline: scroll()");

export default function (props: any) {
  onMount(() => {
    const root = document.documentElement;

    // --- compositor path: recompute iteration counts when the page resizes ---
    const publishCycles = () => {
      const scrollable = root.scrollHeight - window.innerHeight;
      for (const { cycles, rate, tileHeight } of LAYERS) {
        // One iteration drifts one tile, so this is the drift the layer would
        // have accumulated over the whole page, expressed in tiles.
        const n = Math.max(0, (scrollable * rate) / tileHeight);
        root.style.setProperty(cycles, `${n}`);
      }
    };

    // --- fallback path: write the wrapped offset each frame ---
    let queued = false;
    const publishOffsets = () => {
      queued = false;
      const y = window.scrollY;
      for (const { name, rate, tileHeight } of LAYERS) {
        // Wrap into a single tile. The background repeats, so a layer offset
        // by n tiles is indistinguishable from one offset by none -- and this
        // is what lets the layer stay a fixed height instead of having to span
        // the whole page.
        const offset = -((y * rate) % tileHeight);
        root.style.setProperty(name, `${offset}px`);
      }
    };
    const onScroll = () => {
      if (queued) return;
      queued = true;
      requestAnimationFrame(publishOffsets);
    };

    let observer: ResizeObserver | undefined;

    if (SCROLL_DRIVEN) {
      publishCycles();
      // Pages fill in from fetches long after mount, so the scrollable height
      // is not known until the content settles -- watch it rather than sample
      // it once.
      observer = new ResizeObserver(publishCycles);
      observer.observe(document.body);
      window.addEventListener("resize", publishCycles, { passive: true });
    } else {
      publishOffsets();
      window.addEventListener("scroll", onScroll, { passive: true });
    }

    onCleanup(() => {
      observer?.disconnect();
      window.removeEventListener("resize", publishCycles);
      window.removeEventListener("scroll", onScroll);
      for (const { name, cycles } of LAYERS) {
        root.style.removeProperty(name);
        root.style.removeProperty(cycles);
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
