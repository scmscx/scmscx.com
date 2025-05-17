import { createSignal, createContext, useContext, Signal } from "solid-js";

const LanguageContext = createContext();

export function useLang() {
  return useContext(LanguageContext) as Signal<any>;
}

export function LangProvider(props: any) {
  let preferredLanguage = "en";
  {
    const langs = navigator.languages;
    for (const lang of langs) {
      const langcode = lang.split("-")[0];

      switch (langcode) {
        case "en":
          preferredLanguage = "en";
          break;

        case "ko":
          preferredLanguage = "ko";
          break;

        case "zh":
          preferredLanguage = "zh";
          break;

        case "es":
          preferredLanguage = "es";
          break;

        case "ru":
          preferredLanguage = "ru";
          break;

        case "fr":
          preferredLanguage = "fr";
          break;

        case "de":
          preferredLanguage = "de";
          break;
      }
    }
  }

  const writeLangCookie = (lang: string) => {
    document.cookie = `lang2=${lang};Max-Age=0;path=/map`;
    document.cookie = `lang2=${lang};Max-Age=0;path=/user`;
    document.cookie = `lang2=${lang};Max-Age=0;path=/search`;
    document.cookie = `lang2=${lang};expires=Fri, 31 Dec 9999 23:59:59 GMT;path=/`;
  };

  let lang;
  const cookieLang = readCookie("lang2");
  if (cookieLang === null) {
    writeLangCookie(preferredLanguage);
    lang = preferredLanguage;
  } else {
    lang = cookieLang;
  }

  const [getLang, setLang] = createSignal(lang);

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
