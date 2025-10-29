import { createResource, createSignal, Show } from "solid-js";

import style from "./MapImg.module.scss";

export default function (props: any) {
  const [width, setWidth] = createSignal("");
  const [height, setHeight] = createSignal("");
  const [mousePosition, setMousePosition] = createSignal({ x: 0, y: 0 });
  const [isHovering, setIsHovering] = createSignal(false);
  const [naturalDimensions, setNaturalDimensions] = createSignal({ width: 0, height: 0 });
  const [scalingFactor, setScalingFactor] = createSignal(1);
  const [isTouchDevice, setIsTouchDevice] = createSignal(false);
  const [viewingWindowPosition, setViewingWindowPosition] = createSignal({ top: 20, left: 20 });

  // Detect touch device
  const detectTouchDevice = () => {
    return 'ontouchstart' in window || navigator.maxTouchPoints > 0;
  };

  const onLoad = (evt: any) => {
    const naturalWidth = evt.target.naturalWidth;
    const naturalHeight = evt.target.naturalHeight;
    const maxWidth = props["max-width"] || 512;
    const maxHeight = props["max-height"] || 512;
    const factor = Math.min(
      maxWidth / naturalWidth,
      maxHeight / naturalHeight
    );

    console.log('Image loaded:', { naturalWidth, naturalHeight, maxWidth, maxHeight, factor });
    
    setNaturalDimensions({ width: naturalWidth, height: naturalHeight });
    setScalingFactor(factor);
    setWidth(`${naturalWidth * factor}px`);
    setHeight(`${naturalHeight * factor}px`);
    setIsTouchDevice(detectTouchDevice());
  };

  const onMouseMove = (evt: MouseEvent) => {
    if (isTouchDevice()) return;
    
    const target = evt.currentTarget as HTMLImageElement;
    const rect = target.getBoundingClientRect();
    const x = evt.clientX - rect.left;
    const y = evt.clientY - rect.top;
    setMousePosition({ x, y });
    
    // Calculate optimal viewing window position to avoid mouse intersection
    const mouseScreenX = evt.clientX;
    const mouseScreenY = evt.clientY;
    const windowWidth = 500;
    const windowHeight = 500;
    const margin = 20;
    
    let newTop = margin;
    let newLeft = margin;
    
    // Check if mouse would intersect with top-left positioned window
    const wouldIntersectTopLeft = 
      mouseScreenX >= margin && mouseScreenX <= margin + windowWidth &&
      mouseScreenY >= margin && mouseScreenY <= margin + windowHeight;
    
    if (wouldIntersectTopLeft) {
      // Position on the right side
      newLeft = window.innerWidth - windowWidth - margin;
      // If still intersecting, try bottom
      if (mouseScreenX >= newLeft && mouseScreenX <= newLeft + windowWidth &&
          mouseScreenY >= margin && mouseScreenY <= margin + windowHeight) {
        newTop = window.innerHeight - windowHeight - margin;
        newLeft = margin; // Reset to left
      }
    }
    
    setViewingWindowPosition({ top: newTop, left: newLeft });
    
    const scale = scalingFactor() || 1;
    const natural = naturalDimensions();
    const naturalWidth = naturalDimensions().width || 0;
    const naturalHeight = naturalDimensions().height || 0;
    const scaledWidth = naturalWidth * 0.75;
    const scaledHeight = naturalHeight * 0.75;
    
    const bgX = Math.max(0, Math.min(scaledWidth - 500, (x / scale) * 0.75 - 250));
    const bgY = Math.max(0, Math.min(scaledHeight - 500, (y / scale) * 0.75 - 250));
    console.log('Mouse position:', { x, y, scale, natural, bgX, bgY });
  };

  const onMouseEnter = () => {
    if (isTouchDevice()) return;
    setIsHovering(true);
  };

  const onMouseLeave = () => {
    if (isTouchDevice()) return;
    setIsHovering(false);
  };

  return (
    <div class={style.mapimgContainer}>
      <img
        class={style.mapimg}
        src={props.url}
        onLoad={onLoad}
        onMouseMove={onMouseMove}
        onMouseEnter={onMouseEnter}
        onMouseLeave={onMouseLeave}
        style={{ width: width(), "max-height": height() }}
      />
      <Show when={isHovering() && !isTouchDevice()}>
        <div 
          class={style.zoomOverlay}
          style={{
            "top": `${viewingWindowPosition().top}px`,
            "left": `${viewingWindowPosition().left}px`
          }}
        >
          <div 
            class={style.zoomView}
            style={{
              "background-image": `url(${props.url})`,
              "background-size": `${(naturalDimensions().width || 0) * 0.75}px ${(naturalDimensions().height || 0) * 0.75}px`,
              "background-position": `-${(() => {
                const naturalWidth = naturalDimensions().width || 0;
                const naturalHeight = naturalDimensions().height || 0;
                const scaledWidth = naturalWidth * 0.75;
                const scaledHeight = naturalHeight * 0.75;
                const x = mousePosition().x / (scalingFactor() || 1);
                const y = mousePosition().y / (scalingFactor() || 1);
                return Math.max(0, Math.min(scaledWidth - 500, x * 0.75 - 250));
              })()}px -${(() => {
                const naturalWidth = naturalDimensions().width || 0;
                const naturalHeight = naturalDimensions().height || 0;
                const scaledWidth = naturalWidth * 0.75;
                const scaledHeight = naturalHeight * 0.75;
                const x = mousePosition().x / (scalingFactor() || 1);
                const y = mousePosition().y / (scalingFactor() || 1);
                return Math.max(0, Math.min(scaledHeight - 500, y * 0.75 - 250));
              })()}px`
            }}
          />
        </div>
      </Show>
    </div>
  );
}
