import { For, Match, Show, Switch, createEffect } from "solid-js";
import { useNavigate, useParams } from "@solidjs/router";

import { createSignal } from "solid-js";

import style from "./Upload.module.scss";

import MinimapHover from "../modules/MinimapHover";
import { I18nSpan, i18n_internal } from "../modules/language";
import { useLang, useSession } from "../modules/context";
import { unix_time_to_timestamp } from "../util/util";

import { autofocus } from "@solid-primitives/autofocus";

const pad_array = (a: any[], len: number) => {
  if (a.length < len) {
    return a.concat(Array(len - a.length).fill(undefined));
  }
  return a;
};

const calculateHash = async (algorithm: string, b: ArrayBuffer) => {
  return Array.from(new Uint8Array(await crypto.subtle.digest(algorithm, b)))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
};

const assert = (condition: boolean) => {
  if (!condition) {
    throw "condition failed";
  }
};

enum UploadState {
  Pending,
  Reading,
  Uploading,
  Failed,
  Completed,
}

class Upload {
  playlist: string;
  file: File;
  uploadState: UploadState = UploadState.Pending;
  progress = 0.0;
  tickScheduler: () => void;
  mapId: string | undefined;
  hash: string | undefined;
  error: string | undefined;

  constructor(playlist: string, file: File, tickScheduler: () => void) {
    this.playlist = playlist;
    this.file = file;
    this.tickScheduler = tickScheduler;
  }

  start() {
    assert(this.uploadState == UploadState.Pending);
    this.uploadState = UploadState.Uploading;

    (async () => {
      const arrayBuffer = await this.file.arrayBuffer();

      this.hash = await calculateHash("SHA-256", arrayBuffer);

      const url = `/api/uiv2/upload-map?${new URLSearchParams({
        filename: this.file.name,
        sha256: this.hash,
        last_modified: `${this.file.lastModified}`,
        length: `${this.file.size}`,
        playlist: this.playlist,
      })}`;

      const xhr = new XMLHttpRequest();

      xhr.upload.addEventListener("progress", (e) => {
        this.progress = e.loaded / e.total;
        this.tickScheduler();
      });

      xhr.addEventListener("load", () => {
        if (xhr.status === 200) {
          this.mapId = JSON.parse(xhr.responseText);
          this.uploadState = UploadState.Completed;
        } else {
          this.error = xhr.responseText;
          this.uploadState = UploadState.Failed;
        }

        this.tickScheduler();
      });

      xhr.addEventListener("error", () => {
        console.log("error");
        this.error = "Network Error";
        this.uploadState = UploadState.Failed;
        this.tickScheduler();
      });

      xhr.addEventListener("abort", () => {
        console.log("abort");
        this.error = "Network Error";
        this.uploadState = UploadState.Failed;
        this.tickScheduler();
      });

      xhr.open("POST", url, true);
      xhr.send(arrayBuffer);
    })();
  }

  getProgress() {
    return this.progress;
  }

  getUploadState() {
    return this.uploadState;
  }

  getMapId() {
    return this.mapId;
  }

  getHash() {
    return this.hash;
  }

  getFilename() {
    return this.file.name;
  }

  getLastModified() {
    return this.file.lastModified;
  }

  getFileSize() {
    return this.file.size;
  }

  getError() {
    return this.error;
  }

  reset() {
    this.uploadState = UploadState.Pending;
    this.progress = 0.0;
    this.hash = undefined;
    this.error = undefined;
    this.mapId = undefined;
  }
}

const pending: Upload[] = [];
const inProgress: Upload[] = [];
const failed: Upload[] = [];
const completed: Upload[] = [];

export default function (props: any) {
  const COMPLETED_UPLOADS_PAGE_SIZE = 10;

  const [pendingUploads, setPendingUploads] = createSignal<any[]>([]);
  const [inProgressUploads, setInProgressUploads] = createSignal<any[]>([]);
  const [failedUploads, setFailedUploads] = createSignal<any[]>([]);
  const [completedUploads, setCompletedUploads] = createSignal<any[]>([]);
  const [completedUploadsPage, setCompletedUploadsPage] = createSignal(0);

  const tickScheduler = () => {
    const MAX_CONCURRENCY = 5;

    // move completed or failed items
    for (let i = inProgress.length - 1; i >= 0; i--) {
      const e = inProgress[i];

      if (e.getUploadState() === UploadState.Completed) {
        inProgress.splice(i, 1);
        completed.push(e);
      }

      if (e.getUploadState() === UploadState.Failed) {
        inProgress.splice(i, 1);
        failed.push(e);
      }
    }

    // Start new uploads
    while (inProgress.length < MAX_CONCURRENCY && pending.length > 0) {
      const e = pending.shift()!;
      e.start();
      inProgress.push(e);
    }

    setInProgressUploads(
      inProgress.map((v) => {
        return {
          name: v.getFilename(),
          progress: v.getProgress(),
          size: v.getFileSize(),
          lastModified: v.getLastModified(),
        };
      })
    );

    setPendingUploads(
      pending.map((v) => {
        return {
          name: v.getFilename(),
          progress: v.getProgress(),
          size: v.getFileSize(),
          lastModified: v.getLastModified(),
        };
      })
    );

    setFailedUploads(
      failed.map((v) => {
        return {
          name: v.getFilename(),
          error: v.getError(),
        };
      })
    );

    setCompletedUploads(
      completed.map((v) => {
        return {
          name: v.getFilename(),
          mapId: v.getMapId(),
          hash: v.getHash(),
          size: v.getFileSize(),
          lastModified: v.getLastModified(),
        };
      })
    );
  };

  const submit = (e: Event) => {
    e.preventDefault();
    const form = e.target as HTMLFormElement;
    const input = form.elements.namedItem("maps") as HTMLInputElement;
    const fileList = input.files as FileList;

    const files: Upload[] = [];

    const playlistName = new Date().toISOString();

    for (const file of fileList) {
      if (
        file.name.toLowerCase().endsWith(".scm") ||
        file.name.toLowerCase().endsWith(".scx")
      ) {
        files.push(new Upload(playlistName, file, tickScheduler));
      }
    }

    form.reset();

    pending.push(...files);

    tickScheduler();
  };

  return (
    <>
      <div class={style["vertical-container"]}>
        <h1 class={style.h1}>
          <I18nSpan text="Upload" />
        </h1>

        <p class={style.p}>
          <I18nSpan text="If you want to upload one or more .scm/.scx files, then choose the top file picker." />
        </p>
        <p class={style.p}>
          <I18nSpan text="If you want to upload entire directories and their sub directories, then choose the bottom file picker." />
        </p>
        <p class={style.p}>
          <I18nSpan text="Don't worry about corrupt, broken, unfinished, testing, duplicate, or outdated maps. The website will handle all of this and many of them are important parts of StarCraft map making history. Even uploading the exact same maps multiple times is no concern. So, upload everything you have and let the site do the filtering and processing." />
        </p>
        <p class={style.p}>
          <I18nSpan text="Try uploading your entire StarCraft map directory, it can commonly be found at:" />
          <br />
          <span class={style.mono}>
            &lt;USER_HOME&gt;\My Documents\StarCraft
          </span>
        </p>

        <h2 class={style.h2} style="color: red; background-color: yellow;">
          <I18nSpan text="Uploads are disabled while maintenance is being performed. Expected duration 2 hours." />
        </h2>
        {/* <form class={style.uploader} onSubmit={submit}>
          <label for="file">
            <I18nSpan text=".scm/.scx file upload" />
          </label>
          <input
            autofocus
            type="file"
            name="maps"
            id="file"
            accept=".scm, .scx"
            multiple
          />
          <button class={style.upload}>
            <I18nSpan text="Upload" />
          </button>
        </form>
        <form class={style.uploader} onSubmit={submit}>
          <label for="directory">
            <I18nSpan text="Directory (and sub directories) upload" />
          </label>
          <input
            type="file"
            name="maps"
            id="directory"
            directory
            webkitdirectory
          />

          <button class={style.upload} type="submit">
            <I18nSpan text="Upload" />
          </button>
        </form> */}

        <h3 class={style.h3}>
          <I18nSpan text="In Progress" />
        </h3>
        <div class={style.files}>
          <table class={style.table}>
            <thead>
              <tr>
                <th>
                  <I18nSpan text="Filename" />
                </th>
                <th>
                  <I18nSpan text="Progress" />
                </th>
                <th>
                  <I18nSpan text="Size" />
                </th>
                <th>
                  <I18nSpan text="Last Modified" />
                </th>
              </tr>
            </thead>
            <tbody>
              <For each={inProgressUploads()}>
                {(e: any) => {
                  return (
                    <tr>
                      <td>{e.name}</td>
                      <td>
                        <progress
                          class={style.progress}
                          max="1.1"
                          value={`${e.progress}`}
                        />
                      </td>
                      <td>{e.size}</td>
                      <td>{unix_time_to_timestamp(e.lastModified / 1000)}</td>
                      <td>{e.hash}</td>
                    </tr>
                  );
                }}
              </For>
            </tbody>
          </table>
        </div>

        <h3 class={style.h3}>
          <I18nSpan text="Pending" />{" "}
          <Show when={pendingUploads().length > 0}>
            <span>({pendingUploads().length})</span>
          </Show>
        </h3>
        <div class={style.files}>
          <table class={style.table}>
            <thead>
              <tr>
                <th>
                  <I18nSpan text="Filename" />
                </th>
                <th>
                  <I18nSpan text="Size" />
                </th>
                <th>
                  <I18nSpan text="Last Modified" />
                </th>
              </tr>
            </thead>
            <tbody>
              <For each={pendingUploads().slice(0, 10)}>
                {(e: any) => {
                  return (
                    <tr>
                      <td>{e.name}</td>
                      <td>{e.size}</td>
                      <td>{unix_time_to_timestamp(e.lastModified / 1000)}</td>
                    </tr>
                  );
                }}
              </For>
            </tbody>
          </table>
        </div>
        <h3 class={style.h3}>
          <I18nSpan text="Failed" />{" "}
          <Show when={failedUploads().length > 0}>
            <span>({failedUploads().length})</span>
          </Show>
        </h3>
        <button
          class={style.upload}
          onClick={() => {
            for (;;) {
              const v = failed.pop();
              if (v == undefined) {
                break;
              }
              v.reset();
              pending.push(v);
            }
            tickScheduler();
          }}
        >
          <I18nSpan text="Retry All" />
        </button>
        <div class={style.files}>
          <table class={style.table}>
            <thead>
              <tr>
                <th>
                  <I18nSpan text="Filename" />
                </th>
                <th>
                  <I18nSpan text="Retry" />
                </th>
                <th>
                  <I18nSpan text="Reason" />
                </th>
              </tr>
            </thead>
            <tbody>
              <For each={failedUploads()}>
                {(e: any, index) => {
                  return (
                    <tr>
                      <td>{e.name}</td>
                      <td>
                        <a
                          class={style.retry}
                          onClick={() => {
                            const v = failed.splice(index(), 1)[0];
                            v.reset();
                            pending.push(v);
                            tickScheduler();
                          }}
                        >
                          <I18nSpan text="Retry" />
                        </a>
                      </td>
                      <td>{e.error}</td>
                    </tr>
                  );
                }}
              </For>
            </tbody>
          </table>
        </div>

        <h3 class={style.h3}>
          <I18nSpan text="Completed" />{" "}
          <Show when={completedUploads().length > 0}>
            <span>({completedUploads().length})</span>
          </Show>
        </h3>
        <Show when={completedUploads().length >= 0}>
          <button
            class={
              completedUploadsPage() > 0
                ? style["completed-page-buttons"]
                : style["completed-page-buttons-disabled"]
            }
            onClick={() => {
              if (completedUploadsPage() > 0) {
                setCompletedUploadsPage(completedUploadsPage() - 1);
              }
            }}
          >
            {"<"}
          </button>
          <span class={style["completed-page-display"]}>
            <div style="margin: 0px auto; width: fit-content;">
              {completedUploadsPage() + 1} /{" "}
              {((completedUploads().length +
                (COMPLETED_UPLOADS_PAGE_SIZE - 1)) /
                COMPLETED_UPLOADS_PAGE_SIZE) |
                0}
            </div>
          </span>
          <button
            class={
              completedUploads().length / COMPLETED_UPLOADS_PAGE_SIZE - 1 >
              completedUploadsPage()
                ? style["completed-page-buttons"]
                : style["completed-page-buttons-disabled"]
            }
            onClick={() => {
              if (
                completedUploads().length / COMPLETED_UPLOADS_PAGE_SIZE - 1 >
                completedUploadsPage()
              ) {
                setCompletedUploadsPage(completedUploadsPage() + 1);
              }
            }}
          >
            {">"}
          </button>
        </Show>
        <div class={style.files}>
          <table class={style.table}>
            <thead>
              <tr>
                <th>
                  <I18nSpan text="Filename" />
                </th>
                <th>
                  <I18nSpan text="Link" />
                </th>
                <th>
                  <I18nSpan text="Size" />
                </th>
                <th>
                  <I18nSpan text="Last Modified" />
                </th>
              </tr>
            </thead>
            <tbody>
              <For
                each={pad_array(
                  completedUploads().slice(
                    completedUploadsPage() * COMPLETED_UPLOADS_PAGE_SIZE,
                    (completedUploadsPage() + 1) * COMPLETED_UPLOADS_PAGE_SIZE
                  ),
                  COMPLETED_UPLOADS_PAGE_SIZE
                )}
              >
                {(e: any) => {
                  if (e != undefined) {
                    return (
                      <tr>
                        <td>{e.name}</td>
                        <td>
                          <a href={`/map/${e.mapId}`}>{e.mapId}</a>
                        </td>
                        <td>{e.size}</td>
                        <td>{unix_time_to_timestamp(e.lastModified / 1000)}</td>
                      </tr>
                    );
                  } else {
                    return (
                      <tr>
                        <td>&nbsp;</td>
                      </tr>
                    );
                  }
                }}
              </For>
            </tbody>
          </table>
        </div>
      </div>
    </>
  );
}

// let playlist_name = (new Date()).toISOString();

// let on_change = async (e) => {
//     e.preventDefault();

//     const fileselector = e.target;

//     if (fileselector.files == undefined || fileselector.files.length == 0) {
//         return;
//     }

//     let total_files_processed = 0;
//     let files = [...fileselector.files].filter(x => x.name.endsWith(".scm") || x.name.endsWith(".scx"));

//     let process_one = async (file, idx) => {
//         const arrayBuffer = await file.arrayBuffer();
//         const sha256digest = await crypto.subtle.digest('SHA-256', arrayBuffer);
//         const sha1digest = await crypto.subtle.digest('SHA-1', arrayBuffer);
//         const sha256array = new Uint8Array(sha256digest);
//         const sha1array = new Uint8Array(sha1digest);
//         const sha256hash = Array.from(sha256array).map((b) => b.toString(16).padStart(2, '0')).join('');
//         const sha1hash = Array.from(sha1array).map((b) => b.toString(16).padStart(2, '0')).join('');

//         const file_size = file.size;

//         var formdata = new FormData();
//         formdata.append(`${file.lastModified}/${file.size}/${sha256hash}/${sha1hash}`, file);

//         const response = await fetch(`/api/upload-maps/${playlist_name}`, {
//             method: 'POST',
//             body: formdata,
//             duplex: 'half'
//         });

//         const results = await response.json();

//         for (const r of results) {

//             total_files_processed++;

//             document.querySelector("#map-progress").innerText = `${total_files_processed} / ${files.length}`;

//             const tr = document.createElement("tr");
//             const td_name = document.createElement("td");
//             td_name.innerText = file.name;
//             tr.appendChild(td_name);

//             const td_hash = document.createElement("td");
//             td_hash.innerText = sha256hash;
//             tr.appendChild(td_hash);

//             if (r.Link) {
//                 const td_link = document.createElement("td");
//                 td_link.innerHTML = `<a href="${r.Link}" target="_blank">Link</a>`;
//                 tr.appendChild(td_link);
//             } else if (r.Err) {
//                 const td_err = document.createElement("td");
//                 td_err.innerText = r.Err;
//                 tr.appendChild(td_err);
//             } else {
//                 console.log("what: ", r, results);
//             }

//             document.querySelector("#tbody").append(tr);
//         }

//         return idx;
//     };

//     let promises = {};

//     let unique_id = 0;
//     let outstanding = 0;
//     for (const file of files) {
//         outstanding++;
//         unique_id++;
//         promises[unique_id] = process_one(file, unique_id);

//         while (outstanding > 15) {
//             let uniq = await Promise.any(Object.values(promises));
//             delete promises[uniq];
//             outstanding--;
//         }

//     }

//     await Promise.all(Object.values(promises));
// };

// document.querySelector("#file").addEventListener("change", on_change);
// document.querySelector("#directory").addEventListener("change", on_change);
