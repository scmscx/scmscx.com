import { DEFAULT_LANGUAGE, SUPPORTED_LANGUAGES } from "./language";
import {
  createSignal,
  createContext,
  createEffect,
  useContext,
  Signal,
} from "solid-js";

const LanguageContext = createContext();

export function useLang() {
  return useContext(LanguageContext) as Signal<any>;
}

export function LangProvider(props: any) {
  // This mirrors the server's resolver in crates/bwmapserver/src/i18n.rs, and
  // usually does not decide anything: the server resolves first and puts its
  // answer in the lang2 cookie on the HTML response, which the browser applies
  // before running any script. This is what happens when it has not.
  //
  // navigator.languages is ordered most-preferred first, so the FIRST supported
  // entry wins. This used to be a for/switch where the `break` left the switch
  // rather than the loop, so the *last* match won instead -- and since browsers
  // almost always list English as a trailing fallback, nearly every non-English
  // visitor was handed English and then had it stored in the cookie below.
  const matched = Array.from(navigator.languages ?? [])
    .map((tag) => tag.split("-")[0])
    .find((code) => SUPPORTED_LANGUAGES.includes(code));

  // A cookie naming a language we no longer carry counts as absent, so a stale
  // value re-resolves instead of pinning someone to nothing.
  const cookieLang = readCookie("lang2");
  const stored =
    cookieLang !== null && SUPPORTED_LANGUAGES.includes(cookieLang)
      ? cookieLang
      : undefined;

  const writeLangCookie = (lang: string) => {
    document.cookie = `lang2=${lang};Max-Age=0;path=/map`;
    document.cookie = `lang2=${lang};Max-Age=0;path=/user`;
    document.cookie = `lang2=${lang};Max-Age=0;path=/search`;
    document.cookie = `lang2=${lang};expires=Fri, 31 Dec 9999 23:59:59 GMT;path=/`;
  };

  // Only a positive match is stored. Falling back to English is not: someone
  // browsing in a language we do not carry would otherwise be pinned to English
  // by their own cookie, and the day we add their language it would never reach
  // them. Cookie-less means every visit re-resolves. (Choosing English from the
  // navbar still stores it -- that is a choice, not a fallback.)
  if (stored === undefined && matched !== undefined) {
    writeLangCookie(matched);
  }

  const lang = stored ?? matched ?? DEFAULT_LANGUAGE;

  const [getLang, setLang] = createSignal(lang);

  // Keep <html lang> in step with what we are actually rendering. The server
  // already sets it (see uiv2/*.hbs), so this matters for the one case the
  // server cannot cover: switching language from the navbar, which changes the
  // page without reloading it. Getting it wrong is not cosmetic -- it tells
  // screen readers which voice and pronunciation rules to use, which matters
  // most to exactly the readers who are not reading English.
  createEffect(() => {
    document.documentElement.lang = getLang();
  });

  const setLang2 = (str: string) => {
    writeLangCookie(str);
    setLang(str);
  };

  return (
    <LanguageContext.Provider value={[getLang, setLang2]}>
      {props.children}
    </LanguageContext.Provider>
  );
}

const SessionContext = createContext();

export function useSession() {
  return useContext(SessionContext) as Signal<any>;
}

export function SessionProvider(props: any) {
  const cookieUsername = readCookie("username");

  const [getSession, setSession] = createSignal(cookieUsername);

  const setSession2 = (obj: string | null) => {
    // writeSessionCookie(obj.username, obj.token);
    setSession(obj);
  };

  // validate session
  fetch(`/api/uiv2/is_session_valid`, {
    method: "POST",
    credentials: "include",
    cache: "no-cache",
    headers: {
      "Content-Type": "application/json",
    },
  }).then(
    (value: Response) => {
      value.json().then(
        (json) => {
          if (json === true) {
            setSession2(readCookie("username"));
          } else {
            setSession2(null);
          }
        },
        () => {
          console.log("promise rejected 2");
        }
      );
    },
    () => {
      console.log("promise rejected 1");
    }
  );

  return (
    <SessionContext.Provider value={[getSession, setSession2]}>
      {props.children}
    </SessionContext.Provider>
  );
}

function readCookie(name: string): string | null {
  const nameEQ = name + "=";
  const ca = document.cookie.split(";");
  for (let i = 0; i < ca.length; i++) {
    let c = ca[i];
    while (c.charAt(0) == " ") {
      c = c.substring(1, c.length);
    }
    if (c.indexOf(nameEQ) == 0) return c.substring(nameEQ.length, c.length);
  }

  return null;
}

// document.querySelector(".hamburger-icon").addEventListener("click", () => {
//     document.querySelector("nav").classList.toggle("navbar-display");
// });

// document.querySelector("#change-language-button").addEventListener("click", () => {
//     let lang = readCookie("lang");

//     if (lang == null) {
//         document.cookie = "lang=eng;path=/";
//     } else {
//         if (lang == "kor") {
//             document.cookie = "lang=eng;path=/";
//         } else {
//             document.cookie = "lang=kor;path=/";
//         }
//     }
