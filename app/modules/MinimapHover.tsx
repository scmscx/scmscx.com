import { Suspense, createSignal } from "solid-js";

import MinimapImg from "./MinimapImg";

import style from "./MinimapHover.module.scss";

export default function (props: any) {
  const [isMouseHovering, setIsMouseHovering] = createSignal(false);
  const [mouseCoords, setMouseCoords] = createSignal([0, 0]);

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
                left: `${mouseCoords()[0]}px`,
                top: `${mouseCoords()[1]}px`,
              }}
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
