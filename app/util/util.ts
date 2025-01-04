import { createResource } from "solid-js";

const useApi = (f: () => string) => {
    return createResource(
      f,
      async (url: string) => {
        let ret = undefined;
        try {
          ret = await (await fetch(url)).json()
        } catch (e) {
          console.log("failed to useApi: ", e);
        }

        return ret;
      }
    );
  };

  const unix_time_to_timestamp = (ut: number) => {
    const dt = new Date(ut * 1000);
    const tzo = -dt.getTimezoneOffset();
    const dif = tzo >= 0 ? "+" : "-";
    const pad = (num: number) => {
      const norm = Math.floor(Math.abs(num));
      return (norm < 10 ? "0" : "") + norm;
    };
  
    return (
      dt.getFullYear() +
      "-" +
      pad(dt.getMonth() + 1) +
      "-" +
      pad(dt.getDate()) +
      " " +
      pad(dt.getHours()) +
      ":" +
      pad(dt.getMinutes()) +
      ":" +
      pad(dt.getSeconds())
    );
  };

export {useApi, unix_time_to_timestamp};
