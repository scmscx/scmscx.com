import { Suspense, createSignal } from "solid-js";

import MinimapImg from "./MinimapImg";

import style from "./MinimapHover.module.scss";

export default function (props: any) {
  const [isMouseHovering, setIsMouseHovering] = createSignal(false);
  const [mouseCoords, setMouseCoords] = createSignal([0, 0]);
  let hoverEl: HTMLDivElement | undefined;

  const isTouchDevice = () => {
    return "ontouchstart" in window || window.navigator.maxTouchPoints > 0;
  };

  return (
    <>
      <span
        onMouseLeave={() => setIsMouseHovering(false)}
        onMouseEnter={() => setIsMouseHovering(!isTouchDevice())}
        onMouseMove={(e) => setMouseCoords([e.clientX, e.clientY])}
      >
        {props.children}
        {isMouseHovering() && (
          <Suspense>
            <div
              class={style.hover}
              style={{
                left: `${Math.max(
                  8,
                  Math.min(
                    mouseCoords()[0] + 8,
                    (typeof window !== "undefined"
                      ? document.documentElement.clientWidth
                      : 0) -
                      (hoverEl?.offsetWidth ?? 256) -
                      8
                  )
                )}px`,
                top: `${Math.max(
                  8,
                  Math.min(
                    mouseCoords()[1] + 8,
                    (typeof window !== "undefined" ? window.innerHeight : 0) -
                      (hoverEl?.offsetHeight ?? 256) -
                      8
                  )
                )}px`,
              }}
              ref={hoverEl}
            >
              <MinimapImg
                mapId={props.mapId}
                max-width="256"
                max-height="256"
              />
            </div>
          </Suspense>
        )}
      </span>
    </>
  );
}
