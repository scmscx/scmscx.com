import { ObfuscatedEmail } from "../modules/Email";
import { I18nSpan, I18nText } from "../modules/language";
import style from "./About.module.scss";

export default function () {
  return (
    <>
      <div class={style["vertical-container"]}>
        <h1 class={style.h1}>
          <I18nSpan text="about.title" />
        </h1>

        <div class={style.prose}>
          <h2 class={style.h2}>
            <I18nSpan text="about.faq.heading" />
          </h2>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.what_is_this.q" />
          </h3>
          <p class={style.p}>
            <I18nText text="about.faq.what_is_this.a" />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.why_exists.q" />
          </h3>
          <p class={style.p}>
            <I18nText text="about.faq.why_exists.a" />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.name.q" />
          </h3>
          <p class={style.p}>
            <I18nText text="about.faq.name.a" />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.why_another.q" />
          </h3>
          <p class={style.p}>
            <I18nText text="about.faq.why_another.a1" />
          </p>

          <p class={style.p}>
            <I18nText text="about.faq.why_another.a2" />
          </p>

          <p class={style.p}>
            <I18nText text="about.faq.why_another.a3" />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.how_to_play.q" />
          </h3>
          <p class={style.p}>
            <I18nText text="about.faq.how_to_play.a" />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.how_many.q" />
          </h3>
          <p class={style.p}>
            <I18nText text="about.faq.how_many.a" />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.contribute.q" />
          </h3>
          <p class={style.p}>
            <I18nText text="about.faq.contribute.a" />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.removal.q" />
          </h3>
          <p class={style.p}>
            <I18nText
              text="about.faq.removal.a"
              values={{
                email: () => <ObfuscatedEmail />,
              }}
            />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.tech.q" />
          </h3>
          <p class={style.p}>
            <I18nText text="about.faq.tech.a1" />
          </p>

          <p class={style.p}>
            <I18nText text="about.faq.tech.a2" />
          </p>

          <p class={style.p}>
            <I18nText text="about.faq.tech.a3" />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.linking.q" />
          </h3>
          <p class={style.p}>
            <I18nText text="about.faq.linking.a" />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.api.q" />
          </h3>
          <p class={style.p}>
            <I18nText text="about.faq.api.a" />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.bug.q" />
          </h3>
          <p class={style.p}>
            <I18nText text="about.faq.bug.a" />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.contact.q" />
          </h3>
          <p class={style.p}>
            <I18nText
              text="about.faq.contact.a"
              values={{
                email: () => <ObfuscatedEmail />,
              }}
            />
          </p>

          <h3 class={style.h3}>
            <I18nSpan text="about.faq.donate.q" />
          </h3>
          <p class={style.p}>
            <I18nText text="about.faq.donate.a" />
          </p>

          <h2 class={style.h2}>
            <I18nSpan text="about.credits.heading" />
          </h2>
          <p class={style.p}>
            <I18nText text="about.credits.staredit" />
          </p>

          <p class={style.p}>
            <I18nText text="about.credits.zezula" />
          </p>

          <p class={style.p}>
            <I18nText text="about.credits.chkdraft" />
          </p>

          <p class={style.p}>
            <I18nText text="about.credits.mauz" />
          </p>

          <p class={style.p}>
            <I18nText text="about.credits.wfza1" />
          </p>

          <p class={style.p}>
            <I18nText text="about.credits.zh" />
          </p>
          <p class={style.p}>
            <I18nText text="about.credits.ko" />
          </p>
          <p class={style.p}>
            <I18nText text="about.credits.es" />
          </p>
          <p class={style.p}>
            <I18nText text="about.credits.ru" />
          </p>
          <p class={style.p}>
            <I18nText text="about.credits.fr" />
          </p>
          <p class={style.p}>
            <I18nText text="about.credits.de" />
          </p>
        </div>
      </div>
    </>
  );
}
