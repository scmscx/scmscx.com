import { For } from "solid-js";
import { useNavigate } from "@solidjs/router";

import { createSignal } from "solid-js";

import style from "./Login.module.scss";

import MinimapHover from "../modules/MinimapHover";
import { I18nSpan, i18n_internal } from "../modules/language";
import { useLang, useSession } from "../modules/context";

export default function (props: any) {
  const [username, setUsername] = createSignal("");
  const [password, setPassword] = createSignal("");
  const [session, setSession] = useSession();
  const navigate = useNavigate();

  const [usernameRegister, setUsernameRegister] = createSignal("");
  const [passwordRegister, setPasswordRegister] = createSignal("");
  const [confirmPasswordRegister, setConfirmPasswordRegister] =
    createSignal("");

  const [lang, _] = useLang();

  const logIn = async () => {
    try {
      const resp = await fetch(`/api/login`, {
        method: "POST",
        credentials: "include",
        cache: "no-cache",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          username: username(),
          password: password(),
        }),
      });

      if (!resp.ok) {
        const text = await resp.text();

        alert(i18n_internal(lang(), "Failed to login") + ": " + text);
      } else {
        setSession(username());
        navigate(`/user/${username()}`);
      }
    } catch (e) {
      alert(i18n_internal(lang(), "Failed to login") + ": " + e);
    }
  };

  const register = async () => {
    try {
      const resp = await fetch(`/api/register`, {
        method: "POST",
        credentials: "include",
        cache: "no-cache",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          username: usernameRegister(),
          password: passwordRegister(),
          password_confirm: confirmPasswordRegister(),
        }),
      });

      if (!resp.ok) {
        const text = await resp.text();

        alert(
          i18n_internal(lang(), "Failed to register your account") + ": " + text
        );
      } else {
        setSession(usernameRegister());
        navigate(`/user/${usernameRegister()}`);
      }
    } catch (e) {
      alert(
        i18n_internal(lang(), "Failed to register your account") + ": " + e
      );
    }
  };

  return (
    <>
      <div class={style["vertical-container"]}>
        <h1 class={style.h1}>
          <I18nSpan text="Log in" />
        </h1>

        <form
          class={style.form}
          onSubmit={(e) => {
            e.preventDefault();
            logIn();
          }}
        >
          <label class={style["input-label"]} for="username">
            <I18nSpan text="Username" />
          </label>
          <input
            type="text"
            id="username"
            autocomplete="username"
            class={style.username}
            value={username()}
            onChange={(evt) => setUsername(evt.target.value)}
          />
          <label class={style["input-label"]} for="password">
            <I18nSpan text="Password" />
          </label>
          <input
            type="password"
            id="password"
            autocomplete="current-password"
            class={style.password}
            value={password()}
            onChange={(evt) => setPassword(evt.target.value)}
          />
          <button class={style.login} type="submit">
            <I18nSpan text="Log in-formaction" />
          </button>
        </form>

        <h1 class={style.h1}>
          <I18nSpan text="Register" />
        </h1>
        <form
          class={style.form}
          onSubmit={(e) => {
            e.preventDefault();
            register();
          }}
        >
          <label class={style["input-label"]} for="usernameRegister">
            <I18nSpan text="Username" />
          </label>
          <input
            type="text"
            id="usernameRegister"
            autocomplete="off"
            class={style.username}
            value={usernameRegister()}
            onChange={(evt) => setUsernameRegister(evt.target.value)}
          />
          <label class={style["input-label"]} for="passwordRegister">
            <I18nSpan text="Password" />
          </label>
          <input
            type="password"
            id="passwordRegister"
            autocomplete="new-password"
            class={style.password}
            value={passwordRegister()}
            onChange={(evt) => setPasswordRegister(evt.target.value)}
          />
          <label class={style["input-label"]} for="confirmPasswordRegister">
            <I18nSpan text="Confirm Password" />
          </label>
          <input
            type="password"
            id="confirmPasswordRegister"
            autocomplete="new-password"
            class={style.password}
            value={confirmPasswordRegister()}
            onChange={(evt) => setConfirmPasswordRegister(evt.target.value)}
          />
          <button class={style.login} type="submit">
            <I18nSpan text="Register-formaction" />
          </button>
        </form>
      </div>
    </>
  );
}
