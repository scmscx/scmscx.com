import { createSignal, createResource, Switch, Match, Show } from "solid-js";
import { For } from "solid-js";

const IngameColorTable = [
  "#000000", // 0x00 NUL
  "#000000", // 0x01 "Mimic"
  "#b8b8e8", // 0x02 Cyan
  "#dcdc3c", // 0x03 Yellow
  "#ffffff", // 0x04 White
  "#847474", // 0x05 Grey
  "#c81818", // 0x06 Red
  "#10fc18", // 0x07 Green
  "#f40404", // 0x08 Red (P1)
  "#000000", // 0x09 Tab
  "#000000", // 0x0A New Line
  "#000000", // 0x0B Invisible
  "#000000", // 0x0C Remove beyond (large font), newline (small font)
  "#000000", // 0x0D Carriage Return
  "#0c48cc", // 0x0E Blue (P2)
  "#2cb494", // 0x0F Teal (P3)
  "#88409c", // 0x10 Purple (P4)
  "#f88c14", // 0x11 Orange (P5)
  "#000000", // 0x12 Right Align
  "#000000", // 0x13 Center Align
  "#000000", // 0x14 Invisible
  "#703014", // 0x15 Brown (P6)
  "#cce0d0", // 0x16 White (P7)
  "#fcfc38", // 0x17 Yellow (P8)
  "#088008", // 0x18 Green (p9)
  "#fcfc7c", // 0x19 Brighter Yellow (P10)
  "#000000", // 0x1A (Seems to do nothing (zero width space) in 1.16.1 and remaster)
  "#ecc4b0", // 0x1B Pinkish (P11)
  "#4068d4", // 0x1C Dark Cyan (P12)
  "#74a47c", // 0x1D Greygreen
  "#9090b8", // 0x1E Bluegrey
  "#00e4fc", // 0x1F Turqoise
];

const MenuColorTable = [
  "#000000", // 0x00 NUL
  "#000000", // 0x01 "Mimic"
  "#a4b4f8", // 0x02 Cyan
  "#4cc428", // 0x03 Green
  "#b4fc74", // 0x04 Light Green
  "#585858", // 0x05 Grey
  "#ffffff", // 0x06 White
  "#fc0000", // 0x07 Red
  "#000000", // 0x08 Black
  "#000000", // 0x09 Tab
  "#000000", // 0x0A New Line
  "#000000", // 0x0B Invisible
  "#000000", // 0x0C Remove Beyond
  "#000000", // 0x0D Carriage Return
  "#000000", // 0x0E Black
  "#000000", // 0x0F Black
  "#000000", // 0x10 Black
  "#000000", // 0x11 Black
  "#000000", // 0x12 Right Align
  "#000000", // 0x13 Center Align
  "#000000", // 0x14 Invisible
  "#000000", // 0x15 Black
  "#000000", // 0x16 Black
  "#000000", // 0x17 Black
  "#000000", // 0x18 Black
  "#000000", // 0x19 Black
  "#000000", // 0x1A (Seems to do nothing (zero width space) in 1.16.1 and remaster)
  "#000000", // 0x1B Black
  "#000000", // 0x1C Black

  "#000000", // 0x1D Black
  "#000000", // 0x1E Black
  "#000000", // 0x1F Black
];

// scenario: "\u001c 나 \u0006 짓전 \u001f 삼국지 \u001b2.3V"

const parse = function (
  str: string | undefined
): { character: string; color: number }[] {
  const output = [];
  let isColorLocked = false;
  let previousColor = 0x02;
  let currentColor = 0x02;

  if (str === undefined || str === null) {
    return [];
  }

  for (const c of Array.from(str)) {
    const charcode = c.charCodeAt(0);

    if (c === "\n") {
      // Newline character doesn't really have a color but needs to be emitted anyway.
      output.push({
        character: c,
        color: currentColor,
      });
    } else if (c === "\r" || charcode === 0x1a) {
      // codes that count as zero width spaces, do nothing.
    } else if (charcode === 0x0b || charcode === 0x14) {
      // Invisible.
      // Don't actually want to remove stuff, it's nice for it to still be selectable in the browser.
      // in SC when the invis character is used, it can not be overidden by subsequent color codes.
      previousColor = currentColor;
      isColorLocked = true;
      currentColor = 0x0b;
    } else if (charcode === 0x0c) {
      // Remove Beyond
      // Don't actually want to remove stuff, it's nice for it to still be selectable in the browser.
      previousColor = currentColor;
      isColorLocked = true;
      currentColor = 0x0b;
    } else if (charcode === 0x01 && !isColorLocked) {
      // Mimic color
      currentColor = previousColor;
    } else if (charcode === 0x12) {
      // Right Align
    } else if (charcode === 0x13) {
      // Center Align
    } else if (!isColorLocked && charcode < 32) {
      // Color from the color table.
      previousColor = currentColor;
      currentColor = charcode;
    } else if (charcode >= 32) {
      output.push({
        character: c,
        color: currentColor,
      });
    }
  }

  return output;
};

const ColoredTextInternal = (props: any) => {
  return (
    <For each={parse(props.text)}>
      {(map, _i) => (
        <span
          style={
            map.color in props.table ? `color:${props.table[map.color]}` : ""
          }
        >
          {map.character}
        </span>
      )}
    </For>
  );
};

const ColoredTextIngame = (props: any) => (
  <ColoredTextInternal text={props.text} table={IngameColorTable} />
);

const ColoredTextMenu = (props: any) => (
  <ColoredTextInternal text={props.text} table={MenuColorTable} />
);

const ColoredTextMenuNoNewlines = (props: any) => (
  <ColoredTextInternal
    text={props.text.replace(/(\n|\r|\u2028|\u2029)/gm, "")}
    table={MenuColorTable}
  />
);

export { ColoredTextMenu, ColoredTextMenuNoNewlines, ColoredTextIngame };
