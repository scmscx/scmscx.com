import type { Component } from "solid-js";
import { createSignal, Switch, Match, useContext, For } from "solid-js";
import { A, useNavigate } from "@solidjs/router";
import { useLang, useSession } from "./context";
import { I18nSpan, i18n_internal, SUPPORTED_LANGUAGES } from "./language";

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
            <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/"><I18nSpan text="nav.home" /></A>
            <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/search"><I18nSpan text="common.search_2" /></A>
            {/* <A class={style.a} href="/recent"><I18nSpan text="Recent" /></A> */}
            <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/upload"><I18nSpan text="common.upload" /></A>
            <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/about"><I18nSpan text="about.title" /></A>
            {/* <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/upload-replay"><I18nSpan text="Upload Replay" /></A> */}
            <Switch>
                <Match when={session() !== null}>
                    {/* <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/change-username"><I18nSpan text="user.change_username" /></A> */}
                    {/* <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/change-password"><I18nSpan text="user.change_password" /></A> */}
                    {/* <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/api/logout" onClick={logout}><I18nSpan text="user.log_out" /></A> */}
                    <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href={`/user/${session()}`}>{session()}</A>
                </Match>
                <Match when={session() === null}>
                    <A class={style.a} classList={{ [style["hidden"]]: navHidden() }} href="/login"><I18nSpan text="common.log_in_2" /></A>
                </Match>
            </Switch>
            {/* classList={{ [style["hidden"]]: navHidden() }} */}
            <div class={`${style.a} ${style["language-icon-container"]}`}>
              <img class={style["language-icon"]} src={languageIcon} />
              <select class={style.select} value={lang()} onChange={(e) => {
                setLang(e.target.value);
              }}>
                {/* Driven by the table, not a list kept in step by hand: an
                    option the table does not carry writes a cookie the server
                    rejects, and a language the table gains but the list misses
                    is unreachable from the UI. `lang.native` holds each
                    language's own name for itself, so the label under a code is
                    the endonym and `make i18n` guarantees every language has
                    one. */}
                <For each={SUPPORTED_LANGUAGES}>
                  {(code) => <option value={code}>{i18n_internal(code, "lang.native")}</option>}
                </For>
              </select>
            </div>
        </nav >

      {props.children}
    </>
  );
}
