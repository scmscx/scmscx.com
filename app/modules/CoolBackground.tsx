import { createEffect, createResource, createSignal } from "solid-js";

import style from "./CoolBackground.module.scss";

export default function (props: any) {
  createEffect(() => {
    (function () {
      console.log("effect");
      const resizeObserver = new ResizeObserver((entries) => {
        for (const entry of entries) {
          (
            document?.querySelectorAll(`.${style.root}`)[0] as HTMLElement
          )?.style.setProperty(
            "--total-length",
            `${entry.contentBoxSize[0].blockSize + 16}px`
          );
        }
      });

      resizeObserver.observe(document.querySelectorAll(`.${style.content}`)[0]);
    })();
  });

  return (
    <>
      <div id="content" class={style.root}>
        <div class={style.layer1} />
        <div class={style.layer3} />
        <div class={style.layer5} />
        <div class={style.content}>{props.children}</div>
      </div>
    </>

    // <>{props.children}</>
  );
}
