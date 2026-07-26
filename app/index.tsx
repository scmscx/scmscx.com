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
      </SessionProvider>
    </LangProvider>
  ),
  root!
);
