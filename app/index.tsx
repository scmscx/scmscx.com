/* @refresh reload */
import { render } from "solid-js/web";
import { Router, Route, A } from "@solidjs/router";

import { LangProvider, SessionProvider } from "./modules/context";
import Home from "./pages/Home";
import Map from "./pages/Map";
import About from "./pages/About";
import Search from "./pages/Search";

import "./reset.scss";
import "./global.scss";

// Imported here rather than from global.scss so each lands in the bundle once;
// see the note there. Two per script: a display face and a readable one, paired
// up by `--space-font` / `--menu-font` in global.scss and keyed off <html lang>.
// Nothing here is downloaded until a page's text needs it -- they are all
// unicode-range subsetted.
import "./michroma.css"; // display, Latin
import "./unbounded.css"; // display, Cyrillic
import "./m_plus_u.css"; // display, Japanese
import "./nanum_gothic.css"; // display, Korean
import "./chiron_goround_tc.css"; // display, Chinese
import "./noto_sans.css"; // readable, Latin and Cyrillic
import "./noto_sans_jp.css"; // readable, Japanese
import "./noto_sans_kr.css"; // readable, Korean
import "./noto_sans_tc.css"; // readable, Chinese

import Navbar from "./modules/Navbar";
import Login from "./pages/Login";
import CoolBackground from "./modules/CoolBackground";
import User from "./pages/User";
import Upload from "./pages/Upload";
import Moderation from "./pages/Moderation";

import "./assets/favicon.svg";
import "./assets/favicon.ico";
import "./assets/apple-touch-icon-180x180.png";
import "./assets/pwa-64x64.png";
import "./assets/pwa-192x192.png";
import "./assets/pwa-512x512.png";
import "./assets/maskable-icon-512x512.png";

const root = document.getElementById("root");

if (import.meta.env.DEV && !(root instanceof HTMLElement)) {
  throw new Error(
    "Root element not found. Did you forget to add it to your index.html? Or maybe the id attribute got misspelled?"
  );
}

const NotFound = (props: any) => <>NotFound</>;

// The browser's automatic restore cannot work here: every page paints empty and
// fills in from fetches, so at popstate time the document is one viewport tall
// and the saved offset is clamped away. Worse, it would fight any page that
// restores its own position. Pages that care do it themselves once their
// content is back — see pages/Search.tsx.
if ("scrollRestoration" in history) {
  history.scrollRestoration = "manual";
}

render(
  () => (
    <LangProvider>
      <SessionProvider>
        <CoolBackground>
          <Router root={Navbar}>
            <Route path="/about" component={About} />
            <Route path="/login" component={Login} />
            <Route path={["/search", "/search/:query"]} component={Search} />
            <Route path="/map/:mapId" component={Map} />
            <Route path="/user/:userName" component={User} />
            <Route path="/upload" component={Upload} />
            <Route path="/moderation" component={Moderation} />
            <Route path="/" component={Home} />
            <Route path="*paramName" component={NotFound} />
          </Router>
        </CoolBackground>
      </SessionProvider>
    </LangProvider>
  ),
  root!
);
