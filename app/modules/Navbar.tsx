import type { Component } from "solid-js";
import { createSignal, Switch, Match, useContext } from "solid-js";
import { A, useNavigate } from "@solidjs/router";
import { useLang, useSession } from "./context";
import { I18nSpan } from "./language";

import style from "./Navbar.module.scss";

import languageIcon from "../assets/language-icon.svg";

export default function (props: any) {
  const [session, setSession] = useSession();
  const [navHidden, setNavHidden] = createSignal(true);
  const [lang, setLang] = useLang();

  return (
    <>
      {/* prettier-ignore */}
      <nav class={style.nav}>
            <a class={`${style.a} ${style["hamburger-icon"]}`} onClick={() => setNavHidden(!(navHidden()))}>☰</a>
            <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/"><I18nSpan text="Home" /></A>
            <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/search"><I18nSpan text="Search" /></A>
            {/* <A class={style.a} href="/recent"><I18nSpan text="Recent" /></A> */}
            <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/upload"><I18nSpan text="Upload" /></A>
            <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/about"><I18nSpan text="About" /></A>
            {/* <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/upload-replay"><I18nSpan text="Upload Replay" /></A> */}
            <Switch>
                <Match when={session() !== null}>
                    {/* <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/change-username"><I18nSpan text="Change Username" /></A> */}
                    {/* <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/change-password"><I18nSpan text="Change Password" /></A> */}
                    {/* <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/api/logout" onClick={logout}><I18nSpan text="Log out" /></A> */}
                    <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href={`/user/${session()}`}>{session()}</A>
                </Match>
                <Match when={session() === null}>
                    <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/login"><I18nSpan text="Log in" /></A>
                </Match>
            </Switch>
            {/* classList={{ [style["hidden"]]: navHidden() }} */}
            <div class={`${style.a} ${style["language-icon-container"]}`}>
              <img class={style["language-icon"]} src={languageIcon} />
              <select class={style.select} value={lang()} onChange={(e) => {
                setLang(e.target.value);
              }}>
                <option value="en">English</option>
                <option value="ko">한국어</option>
                <option value="zh">中文</option>
                <option value="es">Español</option>
                <option value="ru">Русский</option>
                <option value="fr">Français</option>
                <option value="de">Deutsch</option>
              </select>
            </div>
        </nav >

      {props.children}
    </>
  );
}
