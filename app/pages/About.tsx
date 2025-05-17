import { I18nSpan } from "../modules/language";
import style from "./About.module.scss";

export default function (props: any) {
  return (
    <>
      <div class={style["vertical-container"]}>
        <h1 class={style.h1}>
          <I18nSpan text="About" />
        </h1>

        <h2 class={style.h2}>
          <I18nSpan text="Frequently Asked Questions" />
        </h2>

        <h4 class={style.h4}>
          <I18nSpan text="What is this website?" />
        </h4>
        <p class={style.p}>
          This website is a database for{" "}
          <a
            href="https://en.wikipedia.org/wiki/StarCraft_(video_game)"
            rel="external nofollow"
          >
            StarCraft
          </a>
          ,{" "}
          <a
            href="https://en.wikipedia.org/wiki/StarCraft:_Brood_War"
            rel="external nofollow"
          >
            StarCraft: Brood War
          </a>
          , and{" "}
          <a href="https://starcraft.com" rel="external nofollow">
            StarCraft: Remastered
          </a>{" "}
          map files (.scm/.scx). It helps users find the StarCraft maps that
          they are looking for, as well as give them useful information about
          those maps. StarCraft is a video game created by{" "}
          <a href="https://www.blizzard.com" rel="external nofollow">
            Blizzard Entertainment
          </a>
          . This website is not affiliated in any way with Blizzard
          Entertainment.
        </p>

        <h4 class={style.h4}>
          <I18nSpan text="Why does this website exist?" />
        </h4>
        <p class={style.p}>
          In 2020 I started playing StarCraft again. I tried looking for some
          maps that I had previously played but I was unable to find them. So I
          created this website to solve my map searching problem. The site
          launched on 2021-02-28 under the domain name 'bounding.net' and has
          since grown significantly in both scope and scale.
        </p>

        <h4 class={style.h4}>
          <I18nSpan text="What does scmscx.com mean?" />
        </h4>
        <p class={style.p}>
          StarCraft maps come in two file extensions, .scm (for original
          StarCraft) and .scx (for the expansion, Brood War). The domain name is
          the concatenation of the two map file formats: scmscx.
        </p>

        <h4 class={style.h4}>
          <I18nSpan text="Why do we need another map database website?" />
        </h4>
        <p class={style.p}>
          Most popular map websites have limited functionality, typically
          offering little more than basic name search and a list of download
          links. Finding a map still involves a lot of manual effort digging
          through each map file.
        </p>

        <p class={style.p}>
          There's a lot of map info buried inside map files that this website
          will help you search and preview across all maps without downloading a
          single file. Want to search for maps with a particular unit name? Want
          to find maps similar to a specific map? Want to see what the minimap
          looks like? This website helps you find maps by almost any criteria
          very quickly, and it shows you rich map information instantly without
          downloading a file.
        </p>

        <p class={style.p}>
          I'm also focused on making this website a streamlined tool. I want
          everything to load fast and with no ads.
        </p>

        <h4 class={style.h4}>
          <I18nSpan text="How do I play the maps after I have downloaded from here?" />
        </h4>
        <p class={style.p}>
          In order to play a map after you have downloaded it, you need to move
          it from your web browser's download folder (on Windows this is usually
          the 'Download' folder) to your StarCraft downloads folder. Your
          StarCraft downloads folder is usually located at "My
          Documents\StarCraft\Maps\Download" (on Windows). After that, you
          should be able to find it in the "Downloads" folder inside the
          StarCraft lobby creation menu.
        </p>

        <h4 class={style.h4}>
          <I18nSpan text="How many maps does the database have?" />
        </h4>
        <p class={style.p}>
          The amount of maps can be seen over on the statistics page. But this
          answer mostly depends on what the definition of a "map" is. While
          there are no exact byte-for-byte duplicates in the database, there are
          many maps that are very similar and potentially even indistinguishable
          from each other.
        </p>

        <h4 class={style.h4}>
          <I18nSpan text="Can I contribute maps to the database?" />
        </h4>
        <p class={style.p}>
          Yes, please <a href="/register">register</a> an account and then you
          will be able to upload any maps that you have. I would appreciate it
          greatly.
        </p>

        <h4 class={style.h4}>
          <I18nSpan text="How is this website made? What is the technology behind it?" />
        </h4>
        <p class={style.p}>
          The website is primarily written in{" "}
          <a href="https://www.rust-lang.org" rel="external nofollow">
            Rust
          </a>{" "}
          using the{" "}
          <a href="https://github.com/actix/actix-web" rel="external nofollow">
            actix-web
          </a>{" "}
          framework. The produced webapp is running on a VM from Contabo.
          Alongside the webapp is also{" "}
          <a href="https://caddyserver.com/" rel="external nofollow">
            Caddy
          </a>{" "}
          and a{" "}
          <a href="https://www.postgresql.org" rel="external nofollow">
            PostgreSQL
          </a>
          database. The actual mapblobs themselves are stored on Backblaze's B2
          object storage service. To process and parse maps, I use{" "}
          <a
            href="https://github.com/ladislav-zezula/StormLib"
            rel="external nofollow"
          >
            stormlib
          </a>{" "}
          as well as my own chk parser. To guess charsets I use a combination of
          my own algorithm as well as uchardet and compact-enc-det.
        </p>

        <h4 class={style.h4}>
          <I18nSpan text="Can I link directly to map downloads or minimap previews?" />
        </h4>
        <p class={style.p}>
          Please go ahead. However, I make no guarantee that the format of those
          URLs will remain stable.
        </p>

        <h4 class={style.h4}>
          <I18nSpan text="I found a bug. How do I report it?" />
        </h4>
        <p class={style.p}>
          The website is open source, please file a github issue here:{" "}
          <a href="https://github.com/scmscx/scmscx.com/issues">
            https://github.com/scmscx/scmscx.com/issues
          </a>
        </p>

        <h4 class={style.h4}>
          <I18nSpan text="How can I contact you?" />
        </h4>
        <p class={style.p}>
          I can be contacted by email at {atob("c2Ntc2N4QGdtYWlsLmNvbQ==")}
        </p>

        <h4 class={style.h4}>
          <I18nSpan text="Can I make a donation, how much does this site cost to run?" />
        </h4>
        <p class={style.p}>
          I am not currently accepting donations for this site. If I want to
          achieve the goal of this website then I believe that the site has to
          be cheap enough to operate that it will not be a victim of future cost
          cutting, and I also do not want to run ads on the site. To answer the
          other question, the site currently costs ~4 USD per month to operate.
          Also, this website is not a for-profit venture and I don't want the
          website to generate any revenue. I work on this website because I
          enjoy doing it.
        </p>

        <h2 class={style.h2}>
          <I18nSpan text="Credit and Thanks" />
        </h2>
        <p class={style.p}>
          I want to thank the{" "}
          <a href="http://staredit.net" rel="external nofollow">
            staredit.net
          </a>{" "}
          community for a great deal of support and answering my questions with
          regards to parsing maps and many more things. scmscx.com would not be
          possible without their help. Check out their discord at{" "}
          <a href="https://discord.gg/TqShZ66QnV" rel="external nofollow">
            https://discord.gg/TqShZ66QnV
          </a>
        </p>

        <p class={style.p}>
          <a href="http://www.zezula.net" rel="external nofollow">
            Ladislav Zezula
          </a>{" "}
          for creating{" "}
          <a
            href="https://github.com/ladislav-zezula/StormLib"
            rel="external nofollow"
          >
            stormlib
          </a>
          , which is what provides all the MPQ parsing functionality for
          scmscx.com.
        </p>

        <p class={style.p}>
          mauz for a tremendous amount of testing, discussion and ideas, as well
          as letting me use the domain name 'bounding.net'. His original idea
          when registering the domain was to create a list of bounds sorted by
          author and difficulty. He also maintains a list of maps he's made on
          his website{" "}
          <a href="http://rbkz.net/bound" rel="external nofollow">
            http://rbkz.net/bound
          </a>
          .
        </p>

        <p class={style.p}>
          /u/wfza1, for originally linking to bounding.net from scmscx.com in
          the early days, and for eventually giving me the scmscx.com domain.
          Thank you, I greatly appreciate it.
        </p>

        <p class={style.p}>goosegoose, PereC, for the Chinese translations.</p>
        <p class={style.p}>zzt, for the Korean translations.</p>
        <p class={style.p}>ATUQ, for the Spanish translations.</p>
        <p class={style.p}>
          <a href="https://discord.gg/YH9bQuNmsK" rel="external nofollow">
            Ilya Snopchenko
          </a>
          , for the Russian translations.
        </p>
        <p class={style.p}>
          <a href="https://mastodon.zergy.net/@Zergy/" rel="external nofollow">
            Zergy
          </a>
          , for the French translations.
        </p>
        <p class={style.p}>NudeRaider, for the German translations.</p>

        {/* <h2 class={style.h2}>
          <I18nSpan text="Devlog" />
        </h2>

        <h4 class={style.h4}>2025-01-16</h4>
        <ul class={style.ul}>
          <li>
            The website is now open source,{" "}
            <a href="https://github.com/scmscx/scmscx.com">check it out!</a> As
            such, this devlog will no longer be updated.
          </li>
        </ul>

        <h4 class={style.h4}>2024-12-27</h4>
        <ul class={style.ul}>
          <li>Added Russian translations. Thanks Ilya!</li>
          <li>Fixed some missing translations on the search page.</li>
          <li>
            Fixed the Retry All button on the upload page for failed uploads not
            doing anything when clicked sometimes.
          </li>
          <li>Fixed upload accepting unusually capitalized file extensions.</li>
          <li>
            Limited the amount of displayed completed maps on the upload page to
            improve the pages stability when uploading a large number of maps.
          </li>
        </ul>

        <h4 class={style.h4}>2024-12-26</h4>
        <ul class={style.ul}>
          <li>
            Improved the file uploading process by not rendering all maps that
            are in progress.
          </li>
          <li>
            Improved the file uploading process by retrying the server-side
            transactions if they fail, with random backoff. This will help
            prevent spurious failures.
          </li>
        </ul>

        <h4 class={style.h4}>2024-12-06</h4>
        <ul class={style.ul}>
          <li>Fixed some translations,</li>
          <li>Added a featured maps section to the front page.</li>
        </ul>

        <h4 class={style.h4}>2024-09-01</h4>
        <ul class={style.ul}>
          <li>
            Fixed a bug on the upload page where it would not filter out non map
            files before sending them to the server (the server would reject
            them). This saves on bandwidth and upload times, and also helps
            address some out-of-memory situations where people try to do large
            directory uploads and have enormous zip files in the directory which
            the upload chokes on.
          </li>
          <li>
            Fixed the search results page when a map has a very long scenario
            name, the scenario name column is now capped at 200px in width.
          </li>
        </ul>

        <h4 class={style.h4}>2024-08-02</h4>
        <ul class={style.ul}>
          <li>
            Added a way to filter search results by time uploaded and last
            modified time.
          </li>
        </ul>

        <h4 class={style.h4}>2024-06-07</h4>
        <ul class={style.ul}>
          <li>Integrated the corrected Spanish translations from ATUQ.</li>
        </ul>

        <h4 class={style.h4}>2024-05-30</h4>
        <ul class={style.ul}>
          <li>
            The change-language button has been replaced with a dropdown menu
            because there are now too many languages for the button to make
            sense.
          </li>
          <li>
            Added initial spanish translations, will update them when they have
            been checked by a native speaker.
          </li>
        </ul>

        <h4 class={style.h4}>2024-05-29</h4>
        <ul class={style.ul}>
          <li>
            Fixed a bug where the unit names on the map page were being rendered
            with the menu color palette instead of the ingame color palette.
          </li>
          <li>
            Fixed a bug where invalid color codes were being rendered as no
            color, which for links caused them to be blue or purple.
          </li>
        </ul>

        <h4 class={style.h4}>2024-05-28</h4>
        <ul class={style.ul}>
          <li>
            Added the ability to sort search results by clicking on the result
            headers. This was probably what everyone wanted from the beginning.
          </li>
          <li>
            Also fixed the 'recently viewed maps' section on the front page, the
            new map_info api was not updating the view counts of maps.
          </li>
          <li>
            The Similar Maps widget now shows more information about each map,
            such as dimensions, tileset, and last modified time.
          </li>
        </ul>

        <h4 class={style.h4}>2024-05-22</h4>
        <p class={style.p}>
          Added the ability to sort search results by time uploaded and last
          modified time, this was requested by a user. A separate 'sorting'
          section was added to the search page, I'm not sure if this is the best
          way to do it. Maybe clicking on the headers would be better.
        </p>

        <h4 class={style.h4}>2024-05-17</h4>
        <p class={style.p}>
          Launched the new UI, it has been completely rewritten in solidjs. This
          will hopefully make it easier to maintain and develop new features.
          There are several new features launching with the new UI as well, such
          as:
        </p>
        <ul class={style.ul}>
          <li>
            Greatly improved search experience, with infinite scrolling of
            results. Searches are also now cached, so searching again in the
            near future will be very fast.
          </li>
          <li>
            Improved map display, things like the forces are now rendered in a
            way that is familiar to players of the game.
          </li>
          <li>
            Greatly improved Korean translations, the vast majority of the users
            of the site speak Korean so it has been a long time coming for this
            one.
          </li>
          <li>
            For the first time, Chinese translations. The third largest language
            group of users are Chinese speakers, so this should also be a big
            win.
          </li>
          <li>
            Accessability improvements, many pages on the old UI were using
            confusing markup which likely did not work well with screen readers
            and other accessibility tools. The new UI is tremendously more
            standard in its markup.
          </li>
          <li>
            Uploading maps has been greatly improved. This is partly due to
            http3, and partly due to a lot of client frontend work. Uploads now
            really happen in parallel, and when they fail they can be retried. I
            have rewritten the upload page several times, hopefully this is the
            last time.
          </li>
        </ul>
        <p class={style.p}>
          There were also a number of infrastructure changes that just preceded
          the new UI launching. These include:
        </p>
        <ul class={style.ul}>
          <li>
            Containerization of all relevant services. Postgres, the web app,
            the reverse proxy, as well as all the build infrastructure, are now
            in podman containers.
          </li>
          <li>
            Containerizing haproxy as well as certbot proved to be too
            complicated, so I switched to caddy. This also has the benefit of
            adding http3 support, as haproxy still does not have http3 compiled
            in by default.
          </li>
          <li>
            Switching to cloudflare for the domain servers. They have a better
            api than namecheap's domain servers.
          </li>
        </ul>
        <p class={style.p}>
          Some features that existed in the old UI are not yet reimplemented in
          the new UI. They will gradually make their way over. Probably the
          largest thing missing is the fully rendered map previews. They will
          return.
        </p>
        <p class={style.p}>
          I have also decided to start a devlog where I record the changes I
          make and do a little bit of behind the scenes exposition. I'll try to
          retroactively add in some points in the timeline that are interesting.
        </p>

        Write about the saga of the parallax star background

        <h4 class={style.h4}>2024-04-01</h4>
        <p class={style.p}>Give up on trying to get buck2 to work.</p>

        <h4 class={style.h4}>2023-08-12</h4>
        <p class={style.p}>Try to get buck2 to work.</p>

        <h4 class={style.h4}>2022-12-15</h4>
        <p class={style.p}>
          The beginning of the transition over to scmscx.com started.
        </p>

        <h4 class={style.h4}>2022-11-29</h4>
        <p class={style.p}>
          /u/wfza1 reached out to me about giving me the scmscx.com domain.
        </p>

        <h4 class={style.h4}>2021-02-27</h4>
        <p class={style.p}>
          Launched the website under the domain name bounding.net at 11:23pm
          PST. This was version 1.0.
        </p> */}
      </div>
    </>
  );
}
