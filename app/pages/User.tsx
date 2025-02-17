import { For, Show } from "solid-js";
import { A, useNavigate, useParams } from "@solidjs/router";

import { createSignal } from "solid-js";

import style from "./User.module.scss";

import MinimapHover from "../modules/MinimapHover";
import { I18nSpan, i18n_internal } from "../modules/language";
import { useLang, useSession } from "../modules/context";

export default function (props: any) {
  const params = useParams();
  const [lang, _] = useLang();

  const [newUsernamePassword, setNewUsernamePassword] = createSignal("");
  const [newUsername, setNewUsername] = createSignal("");
  const [confirmNewUsername, setConfirmNewUsername] = createSignal("");

  const [session, setSession] = useSession();
  const navigate = useNavigate();

  const [newPassword, setNewPassword] = createSignal("");
  const [confirmNewPassword, setConfirmNewPassword] = createSignal("");

  const changeUsername = async () => {
    try {
      const resp = await fetch(`/api/change-username`, {
        method: "POST",
        credentials: "include",
        cache: "no-cache",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          password: newUsernamePassword(),
          username: newUsername(),
          username_confirm: confirmNewUsername(),
        }),
      });

      if (!resp.ok) {
        const text = await resp.text();

        alert(
          i18n_internal(lang(), "Failed to update your username") + ": " + text
        );
      } else {
        setSession(newUsername());
        navigate(`/user/${newUsername()}`);
        setNewUsername("");
        setConfirmNewUsername("");
        setNewUsernamePassword("");
        alert(i18n_internal(lang(), "Username Updated Successfully"));
      }
    } catch (e) {
      alert(i18n_internal(lang(), "Failed to update your username") + ": " + e);
    }
  };

  const changePassword = async () => {
    try {
      const resp = await fetch(`/api/change-password`, {
        method: "POST",
        credentials: "include",
        cache: "no-cache",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          password: newPassword(),
          password_confirm: confirmNewPassword(),
        }),
      });

      if (!resp.ok) {
        const text = await resp.text();

        alert(
          i18n_internal(lang(), "Failed to update your password") + ": " + text
        );
      } else {
        setNewPassword("");
        setConfirmNewPassword("");
        alert(i18n_internal(lang(), "Password Updated Successfully"));
      }
    } catch (e) {
      alert(i18n_internal(lang(), "Failed to update your password") + ": " + e);
    }
  };

  const logout = () => {
    fetch(`/api/logout`, {
      method: "GET",
      credentials: "include",
      cache: "no-cache",
      headers: {
        "Content-Type": "application/json",
      },
    }).then(
      (resp) => {
        if (resp.ok) {
          setSession(null);
          navigate("/");
        } else {
          alert("failed to log out for some reason");
          setSession(null);
          navigate("/");
        }
      },
      () => {
        alert("failed to log out for some other reason");
      }
    );
  };

  return (
    <>
      <div class={style["vertical-container"]}>
        <h1 class={style.h1}>{params.userName}</h1>

        <Show when={params.userName === session()}>
          <Show when={"RagE" === session()}>
            <A href="/moderation" class={style["a-button"]}>
              <I18nSpan text="Moderation" />
            </A>
          </Show>

          <h2 class={style.h2} style="color: red; background-color: yellow;">
            <I18nSpan text="Account operations are disabled while maintenance is being performed. Expected duration 2 hours." />
          </h2>
          {/* 
          <h3 class={style.h3}>
            <I18nSpan text="Log out" />
          </h3>

          <a class={style["a-button"]} href="/api/uiv2/logout" onClick={logout}>
            <I18nSpan text="Log out" />
          </a>

          <h3 class={style.h3}>
            <I18nSpan text="Change Username" />
          </h3>
          <form
            class={style.form}
            onSubmit={(e) => {
              e.preventDefault();
              changeUsername();
            }}
          >
            <input
              hidden
              type="text"
              autocomplete="username"
              name="username"
              class={style.username}
              value={session()}
              style={{ display: "none" }}
            />
            <label class={style["input-label"]} for="newUsername">
              <I18nSpan text="New Username" />
            </label>
            <input
              type="text"
              id="newUsername"
              autocomplete="off"
              name="new-username"
              class={style.username}
              value={newUsername()}
              onChange={(evt) => setNewUsername(evt.target.value)}
              data-1p-ignore
            />
            <label class={style["input-label"]} for="confirmNewUsername">
              <I18nSpan text="Confirm New Username" />
            </label>
            <input
              id="confirmNewUsername"
              autocomplete="off"
              name="new-username"
              class={style.password}
              value={confirmNewUsername()}
              onChange={(evt) => setConfirmNewUsername(evt.target.value)}
              data-1p-ignore
            />
            <label class={style["input-label"]} for="newUsernamePassword">
              <I18nSpan text="Current Password" />
            </label>
            <input
              type="password"
              id="newUsernamePassword"
              autocomplete="current-password"
              class={style.password}
              value={newUsernamePassword()}
              onChange={(evt) => setNewUsernamePassword(evt.target.value)}
            />
            <button class={style.login} type="submit">
              <I18nSpan text="Change Username" />
            </button>
          </form>

          <h3 class={style.h3}>
            <I18nSpan text="Change Password" />
          </h3>

          <form
            class={style.form}
            onSubmit={(e) => {
              e.preventDefault();
              changePassword();
            }}
          >
            <input
              hidden
              type="text"
              autocomplete="username"
              class={style.username}
              value={session()}
              style={{ display: "none" }}
            />
            <label class={style["input-label"]} for="newPassword">
              <I18nSpan text="New Password" />
            </label>
            <input
              type="password"
              id="newPassword"
              autocomplete="new-password"
              class={style.password}
              value={newPassword()}
              onChange={(evt) => setNewPassword(evt.target.value)}
            />
            <label class={style["input-label"]} for="confirmNewPassword">
              <I18nSpan text="Confirm New Password" />
            </label>
            <input
              type="password"
              id="confirmNewPassword"
              autocomplete="new-password"
              class={style.password}
              value={confirmNewPassword()}
              onChange={(evt) => setConfirmNewPassword(evt.target.value)}
            />
            <button class={style.login} type="submit">
              <I18nSpan text="Change Password" />
            </button>
          </form> */}
        </Show>
      </div>
    </>
  );
}
