#!/usr/bin/env node
import { readFileSync } from "fs";

const NONCE_BYTES = 3;
const BLOCK_BYTES = 16; // required by AES
const COUNTER_BITS = (BLOCK_BYTES - NONCE_BYTES) * 8;

// Where the constants printed below actually live. The key is read back out of
// it rather than kept as a second copy here: re-encrypting without --new-key
// has to use the key the page decrypts with, and a copy that had gone stale
// would emit ciphertext that decrypts to garbage with nothing reporting it.
const EMAIL_TSX = "app/modules/Email.tsx";

// Node has Buffer; the browser side hand-rolls these because it does not.
const toBase64 = (bytes) => Buffer.from(bytes).toString("base64");
const fromBase64 = (text) => new Uint8Array(Buffer.from(text, "base64"));

function keyFromEmailTsx() {
  const src = readFileSync(EMAIL_TSX, "utf8");
  const found = src.match(/const KEY_TEXT = "([^"]+)"/);

  if (found === null) {
    throw new Error(`no KEY_TEXT constant found in ${EMAIL_TSX}`);
  }

  return found[1];
}

async function main() {
  const args = process.argv.slice(2);
  const email = args.find((a) => !a.startsWith("--"));
  const wantNewKey = args.includes("--new-key");

  if (email === undefined) {
    console.error("usage: node tools/obfuscate-email.mjs <email> [--new-key]");
    process.exit(2);
  }

  // Reuse the key currently in Email.tsx unless asked for a fresh one, so that
  // re-encrypting the same address doesn't churn both constants.
  const keyText = wantNewKey
    ? toBase64(new Uint8Array(await crypto.subtle.exportKey(
        "raw",
        await crypto.subtle.generateKey({ name: "AES-CTR", length: 256 }, true, [
          "encrypt",
        ])
      )))
    : keyFromEmailTsx();

  const key = await crypto.subtle.importKey(
    "raw",
    fromBase64(keyText),
    { name: "AES-CTR" },
    false,
    ["encrypt"]
  );

  const nonce = crypto.getRandomValues(new Uint8Array(NONCE_BYTES));
  const counter = new Uint8Array(BLOCK_BYTES);
  counter.set(nonce);

  const cipher = new Uint8Array(
    await crypto.subtle.encrypt(
      { name: "AES-CTR", counter, length: COUNTER_BITS },
      key,
      new TextEncoder().encode(email)
    )
  );

  const output = new Uint8Array(nonce.length + cipher.length);
  output.set(nonce);
  output.set(cipher, nonce.length);

  console.log(`const KEY_TEXT = "${keyText}";`);
  console.log(`const CIPHERTEXT = "${toBase64(output)}";`);
}

main();
