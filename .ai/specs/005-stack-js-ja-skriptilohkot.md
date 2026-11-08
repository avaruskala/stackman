# Spec 005, STACK-JS ja CASText-skriptilohkot

## Tarina

Opettajana haluan, että STACK-kysymysten STACK-JS- ja apulohkot (`[[iframe]]`, `[[javascript]]`, `[[jsxgraph]]`, `[[script]]`, `[[style]]`, `[[cors]]`, `[[quid]]`, `[[jsstring]]`, `[[comment]]`, `[[escape]]`, `[[entityescape]]` ja `[[template]]`) jäsennetään ja suoritetaan hiekkalaatikoiduissa `iframe`-kehyksissä `stack_js`- ja `stack_jxg`-rajapintojen kautta ilman, että raakoja `<script>`-tageja suoritetaan pääikkunassa, jotta dynaamiset kuvaajat, syötteiden sidonnat ja vuorovaikutteiset tehtäväelementit toimivat turvallisesti esikatselussa.

## Käsitteet

Sanasto ja kentät on määritelty tiedostossa `docs/domain.md`.

## Tekemiset

| Toiminto | Onnistumisen ehto | Epäonnistumisen seuraus |
|----------|-------------------|-------------------------|
| Poista `[[comment]]` ja suojaa `[[escape]]` | `collect_castext_expressions` ja `render_castext` poistavat `[[comment]]...[[/comment]]`-lohkot ennen CAS-keräystä ja suojaavat `[[escape]]`-sisällön lohkojäsennykseltä. | Jos lohkoa ei ole suljettu, alkuperäinen teksti säilyy ilman kaatumista. |
| Laajenna `[[template]]`, `[[entityescape]]`, `[[jsstring]]`, `[[quid]]` ja `[[cors]]` | `[[template]]` tallettaa ja sijoittaa nimetyt mallilohkot, `[[entityescape]]` muuntaa HTML-erikoismerkit entiteeteiksi, `[[jsstring]]` tuottaa JSON-merkkijonon, `[[quid id="X"/]]` tuottaa tunnisteen `stack-quid-X` ja `[[cors src="Y"/]]` tuottaa polun `/corsscripts/Y`. | Tuntematon `[[template]]` fără oletusta tulostuu tyhjänä merkkijonona. |
| Jäsennä `[[iframe]]`, `[[javascript]]` ja `[[jsxgraph]]` sekä sisäiset `[[script]]`- ja `[[style]]`-lohkot | Lohkot poimivat sisäiset `[[script]]`- ja `[[style]]`-määrittelyt `srcdoc`-dokumentin `<head>`-osioon, erottavat ES-moduulien `import`-lauseet `input-ref-*`-lupausten edelle ja muodostavat `.stack-iframe-holder`-elementin. | Jos `[[script]]` tai `[[style]]` esiintyy `[[iframe]]`-, `[[javascript]]`- tai `[[jsxgraph]]`-lohkon ulkopuolella, renderöijä tulostaa virheen ` ERROR [[script]] blocks must be within iframes. ` tai ` ERROR [[style]] blocks must be within iframes. `. |
| Yhdistä hiekkalaatikoidut kehykset VLE-isäntään | Selaimen `CLIENT_INIT_SCRIPT` luo `<iframe sandbox="allow-scripts">`-elementit `.stack-iframe-holder`-paikkamerkeille ja käsittelee `postMessage`-viestit (`ping`, `register-input-listener`, `track-input-scroll`, `track-validation-state`, `changed-input`, `clear-input`, `register-button-listener`, `toggle-visibility`, `change-content`, `get-content`, `resize-frame`, `query-submit-button`, `enable-submit-button`, `relabel-submit-button`). | Viestit tuntemattomasta `secret`-tunnisteesta tai kysymysalueen `.formulation`/`.outcome` ulkopuolelta hylätään. |

## Säännöt ja paikat

Viittaukset tiedoston `docs/domain.md` sääntöihin:

- Sääntö 15, STACK-JS- ja CASText-apulohkojen jäsennys, irrallisten `[[script]]`- ja `[[style]]`-lohkojen virheilmoitus sekä `stack-iframe-holder`-eristys, sijaitsee funktioissa `stack::render::render_castext` ja `stack::render::collect_castext_expressions`.

## Hyväksymiskriteerit

1. **Alkutila:** CASText sisältää `[[comment]]{@bad_cas@}[[/comment]]` ja `[[escape]][[javascript]]][[/escape]]`. **Toiminto:** Kutsutaan `collect_castext_expressions` ja `render_castext`. **Lopputulos:** Kommentin sisäistä `{@bad_cas@}`-lauseketta ei kerätä Maxima-ajoon, ja `[[escape]]`-lohkon sisältö tulostuu sellaisenaan `[[javascript]]]`.
2. **Alkutila:** CASText sisältää `[[template name="board"]]custom();[[/template]]` ja `[[template mode="default" name="board"]]default();[[/template]]` sekä `[[quid id="out"/]]`, `[[cors src="stackjsiframe.min.js"/]]`, `[[jsstring]]he"llo[[/jsstring]]` ja `[[entityescape]]<b>"x"</b>[[/entityescape]]`. **Toiminto:** Kutsutaan `render_castext`. **Lopputulos:** Ensimmäinen `[[template]]` määrittää korvauksen tulostumatta itse, `mode="default"`-kohta tulostaa `custom();`, `[[quid]]` tulostaa `stack-quid-out`, `[[cors]]` tulostaa `/corsscripts/stackjsiframe.min.js`, `[[jsstring]]` tulostaa `"he\"llo"` ja `[[entityescape]]` tulostaa `&lt;b&gt;&quot;x&quot;&lt;/b&gt;`.
3. **Alkutila:** CASText sisältää `[[javascript input-ref-ans1="ans1Ref"]]import {foo} from 'cors://bar.js'; ans1Ref.value = '1';[[/javascript]]`. **Toiminto:** Kutsutaan `render_castext`. **Lopputulos:** Tuloksena on piilotettu `div.stack-iframe-holder`, jonka `data-frame-srcdoc` sisältää `<script type="module">`, `import stack_js from '/corsscripts/stackjsiframe.min.js';`, ylös nostetun `import {foo} from '/corsscripts/bar.js';`-rivin sekä `Promise.all([stack_js.request_access_to_input("ans1",true)]).then(([ans1Ref]) => {` -kääreen.
4. **Alkutila:** CASText sisältää `[[script]]alert(1);[[/script]]` tai `[[style]]body{color:red;}[[/style]]` kehyslohkon ulkopuolella. **Toiminto:** Kutsutaan `render_castext`. **Lopputulos:** Tuloksena on teksti ` ERROR [[script]] blocks must be within iframes. ` ja ` ERROR [[style]] blocks must be within iframes. ` ilman `<script>`- tai `<style>`-elementtiä.
5. **Alkutila:** CASText sisältää `[[jsxgraph width="400px" height="300px" input-ref-ans1="ans1Ref"]]var board = JXG.JSXGraph.initBoard(divid, {});[[/jsxgraph]]`. **Toiminto:** Kutsutaan `render_castext`. **Lopputulos:** Tuloksena on `div.jxgbox.stack-jsxgraph.stack-iframe-holder`, jolla on sekä `data-jxg-code`-attribuutti että `data-frame-srcdoc`-dokumentti, joka lataa `/corsscripts/jsxgraphcore.min.js`-, `/corsscripts/stackjsiframe.min.js`- ja `/corsscripts/stackjsxgraph.min.js`-kirjastot ja sitoo `ans1Ref`-muuttujan.
6. **Alkutila:** Esimerkkitiedostot `moodle-qtype_stack/samplequestions/stack-js/stack-js-issue-specific-example-938.xml`, `moodle-qtype_stack/samplequestions/stackdemo/40-JSX_graph_interactive.xml` ja `moodle-qtype_stack/samplequestions/stacklibrary/Doc-Examples/Specialist-Tools-Docs/STACK-JS/Validation-state-listener.xml` jäsennetään ja renderöidään. **Toiminto:** Kutsutaan `parse_moodle_xml` ja `render_stack_question_static`. **Lopputulos:** Kaikki kolme kysymystä jäsennetään ja renderöidään virheettä niin, että `stack-iframe-holder`-elementit ja syötteiden `data-stack-input-type`-attribuutit ovat mukana HTML-tuloksessa.

Kriteerit 1-6 ajetaan craten `crates/stack` yksikkötesteinä.

## Ei-tavoitteet

- Raakojen `<script>`-HTML-tagien suorittaminen pääikkunassa (`createContextualFragment`- tai `eval`-suoritus). Vain STACK-JS-lohkot tuetaan.
- Palvelinpuolen AJAX-validointi opiskelijan syötteelle kirjoituksen aikana (`dfexplicitvaildate`), sillä esikatselu ei tallenna tenttisuoritusta Moodlen tietokantaan.

## Tehtävät

1. **Stack-työntekijä tiedostossa `crates/stack/src/render.rs`.** Toteuta sääntö 15 (`[[comment]]`, `[[escape]]`, `[[template]]`, `[[entityescape]]`, `[[jsstring]]`, `[[quid]]`, `[[cors]]`, `[[style]]`, `[[script]]`, `[[iframe]]`, `[[javascript]]`, `[[jsxgraph]]` ja syötteiden `data-stack-input-type`-attribuutti) sekä yksikkötestit hyväksymiskriteereille 1-6. Askel on valmis, kun `cargo test -p stack` menee läpi.
2. **App-työntekijä tiedostoissa `crates/app/src/components/preview/stack.rs` ja `crates/app/src/lib.rs`.** Lisää `.formulation`- ja `.outcome`-luokat esikatselukortteihin sekä toteuta `CLIENT_INIT_SCRIPT`-skriptiin STACK-JS VLE -viestinkäsittelijä ja `.stack-iframe-holder`-kehysten alustus. Askel on valmis, kun `cargo leptos build` ja `cargo test --workspace` menevät läpi.
