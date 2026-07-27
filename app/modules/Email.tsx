import { createSignal, For, onCleanup, onMount, Show } from "solid-js";
import { useLang } from "./context";
import { i18n_internal } from "./language";
import style from "./Email.module.scss";

const KEY_TEXT = "5DuwDsPrS4F3xR6Fdfp2L+JD5SAMQ4OTDyfGjr7VmQE=";
const CIPHERTEXT = "nH8R0LpT+yMsfFWXKgZV/ikR8Q==";

// Shown until the decryption lands. In practice a reader never sees it -- the
// reveal fires the moment the text enters the viewport -- but it is also what
// stays on screen in the degraded case below, so it reads as a statement
// rather than as a loading spinner.
const PLACEHOLDER = "email.email_hidden";

const NONCE_BYTES = 3;
const BLOCK_BYTES = 16; // required by AES
const COUNTER_BITS = (BLOCK_BYTES - NONCE_BYTES) * 8;

function fromBase64(text: string): Uint8Array {
  const binary = atob(text);
  const bytes = new Uint8Array(binary.length);

  for (let i = 0; i < binary.length; ++i) {
    bytes[i] = binary.charCodeAt(i);
  }

  return bytes;
}

/** The nonce rides in front of the ciphertext; the counter block is the rest. */
async function decryptEmail(): Promise<string> {
  const key = await window.crypto.subtle.importKey(
    "raw",
    fromBase64(KEY_TEXT),
    { name: "AES-CTR" },
    false,
    ["decrypt"]
  );

  const input = fromBase64(CIPHERTEXT);
  const counter = new Uint8Array(BLOCK_BYTES);
  counter.set(input.slice(0, NONCE_BYTES));

  const plain = await window.crypto.subtle.decrypt(
    { name: "AES-CTR", counter, length: COUNTER_BITS },
    key,
    input.slice(NONCE_BYTES)
  );

  return new TextDecoder().decode(plain);
}

/*
 * Second layer, applied to the decrypted result: the "CSS Display none"
 * technique from §1.5 of the same article. The address is cut into segments
 * with decoy segments interleaved, and only CSS decides which ones are real.
 *
 * A harvester that reads the DOM without applying our stylesheet comes away
 * with a syntactically valid but undeliverable address, which is a better
 * outcome than obvious garbage: there is nothing to notice and retry. Everyone
 * else -- readers, screen readers, and anyone copying the text -- gets the real
 * address, because `display: none` removes the decoys from the accessibility
 * tree and from the selection alike.
 *
 * Every segment is wrapped in a span carrying one of several interchangeable
 * classes, real and decoy alike, so the markup on its own gives nothing away
 * (see Email.module.scss); and both the decoy strings and the class picks vary
 * per reveal, so the harvested address isn't a constant that one rule strips.
 */

const DECOYS = ["-nospam", ".invalid", "-remove", ".example", "-delete"];

// Classes that hide, and classes that don't. Which is which lives only in the
// stylesheet.
const HIDDEN = [style.s1, style.s3, style.s5];
const SHOWN = [style.s2, style.s4, style.s6];

const oneOf = (choices: string[]) =>
  choices[Math.floor(Math.random() * choices.length)];

interface Segment {
  text: string;
  decoy: boolean;
}

function withDecoys(address: string): Segment[] {
  const pick = () => oneOf(DECOYS);

  const at = address.indexOf("@");
  if (at < 0) {
    return [{ text: address, decoy: false }];
  }

  const local = address.slice(0, at);
  const domain = address.slice(at + 1);
  const dot = domain.indexOf(".");

  // One decoy after the local part, one inside the domain, so that what a
  // harvester reads still parses as an address.
  const segments: Segment[] = [
    { text: local, decoy: false },
    { text: pick(), decoy: true },
    { text: "@", decoy: false },
  ];

  if (dot < 0) {
    segments.push({ text: domain, decoy: false });
  } else {
    segments.push(
      { text: domain.slice(0, dot), decoy: false },
      { text: pick(), decoy: true },
      { text: domain.slice(dot), decoy: false }
    );
  }

  return segments;
}

/**
 * The contact address, as a mailto: link once it has been decrypted.
 *
 * Both the visible text and the href stay encrypted until the reveal, so
 * scraping the rendered DOM before scrolling gets nothing either.
 */
export function ObfuscatedEmail() {
  const [lang, _] = useLang();
  const [address, setAddress] = createSignal<string | null>(null);
  const [wantsHref, setWantsHref] = createSignal(false);
  let anchor: HTMLSpanElement | undefined;

  // Last stage: the href is the one place the address has to appear verbatim --
  // a decoyed mailto: would simply be the wrong address -- so it is withheld
  // until someone actually reaches for the link. Hover covers a mouse, focus
  // covers the keyboard (an anchor with no href is not focusable on its own,
  // hence the tabindex below), and pointerdown covers touch, where there is no
  // hover at all; it fires before the click is dispatched, so the anchor is a
  // real link by the time the tap activates it.
  //
  // Derived rather than a second signal kept in step with the first: the href
  // is only ever "the address, once someone has reached for it".
  const href = () => {
    const value = address();
    return wantsHref() && value !== null ? `mailto:${value}` : undefined;
  };
  const revealHref = () => setWantsHref(true);

  const reveal = () => {
    decryptEmail().then(setAddress, (e) => {
      console.log(`could not decrypt the contact address: ${e}`);
    });
  };

  onMount(() => {
    // SubtleCrypto exists only in a secure context. That covers production
    // (https) and local development (localhost), but not the dev server
    // reached over a LAN address -- there the placeholder simply stays put.
    if (!window.crypto?.subtle) {
      console.log(
        "the contact address is only readable in a secure context (https or localhost)"
      );
      return;
    }

    if (anchor === undefined || !("IntersectionObserver" in window)) {
      reveal();
      return;
    }

    const observer = new IntersectionObserver((entries) => {
      if (!entries.some((entry) => entry.isIntersecting)) {
        return;
      }

      observer.disconnect();
      reveal();
    });

    observer.observe(anchor);
    onCleanup(() => observer.disconnect());
  });

  return (
    <Show
      when={address()}
      fallback={<span ref={anchor}>{i18n_internal(lang(), PLACEHOLDER)}</span>}
      keyed
    >
      {(value) => (
        <a
          class={style.link}
          href={href()}
          role="link"
          tabindex="0"
          onMouseEnter={revealHref}
          onFocus={revealHref}
          onPointerDown={revealHref}
        >
          <For each={withDecoys(value)}>
            {(segment) => (
              <span class={oneOf(segment.decoy ? HIDDEN : SHOWN)}>
                {segment.text}
              </span>
            )}
          </For>
        </a>
      )}
    </Show>
  );
}
