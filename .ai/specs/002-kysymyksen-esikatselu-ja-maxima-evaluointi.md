# Spec 002, Kysymyksen esikatselu ja Maxima CAS -evaluointi

## Tarina

Opettajana haluan nähdä valitun kysymyksen oikeassa reunassa esikatseluna, jossa STACK-kysymyksen Maxima-muuttujat, CASText-kaavat, ehtolauseet, `[[jsxgraph]]`-kuvaajat, syötekentät, mallivastaukset, yleinen palaute ja vastauspuut renderöityvät valitulla satunnaissiemenellä, jotta voin tarkastaa tehtävän eri versiot ja pisteytyksen.

## Käsitteet

Sanasto ja kentät on määritelty tiedostossa `docs/domain.md`.

## Tekemiset

| Toiminto | Onnistumisen ehto | Epäonnistumisen seuraus |
|----------|-------------------|-------------------------|
| Evaluoi STACK-kysymys siemenellä | Valittu kysymys on `Question::Stack` ja `/usr/bin/maxima` suorittaa `questionvariables`- ja CASText-lausekkeet annetulla `seed: u32`-arvolla. Tietolaatikko näyttää tilamerkin `Maxima CAS`. | Jos `/usr/bin/maxima` puuttuu tai lauseke epäonnistuu, palvelin palauttaa staattisen esikatselun `render_stack_question_static`, näyttää tilamerkin `Staattinen esikatselu` ja tallentaa virhesyyn. |
| Vaihda siemen tai arvo uusi versio | Opettaja syöttää uuden luvun, painaa Uusi versio -painiketta tai valitsee arvon `<deployedseed>`-valikosta. | Esikatselu evaluoi kysymyksen uudella siemenellä. |
| Täytä oikeat vastaukset | Opettaja painaa Täytä oikeat vastaukset -painiketta. | STACK-syötekentät täyttyvät `tans`-arvoilla, mallivastauslaatikko avautuu ja monivalinta sekä tosi tai epätosi -kysymys korostavat oikeat vaihtoehdot. |
| Vaihda kielisuodatin | Opettaja valitsee työkalupalkista kielen `Kaikki`, `FI` tai `EN`. | Esikatselu näyttää vain valitun kielen `<span lang="..." class="multilang">`-osiot. |
| Tarkastele muuttujia ja vastauspuita | Opettaja kytkee Muuttujat- tai Vastauspuu-painikkeen päälle. | Näkymä avaa Maxima-koodin, evaluoitujen muuttujien arvotaulukon ja vastauspuun solmut. |

## Säännöt ja paikat

Viittaukset tiedoston `docs/domain.md` sääntöihin:

- Sääntö 3, syötetyypit ja matriisin koko Maximan `matrix`-arvosta, sijaitsee funktiossa `stack::render::evaluate_stack_question_maxima`.
- Sääntö 4, deterministinen `seed: u32` parametrina, sijaitsee funktiossa `stack::render::build_maxima_script` ADR 0001:n mukaisesti.
- Sääntö 5, kommenttien poisto ja puolipisteettömien Maxima-rivien normalisointi sulkusyvyydellä nolla, sijaitsee funktiossa `stack::render::normalize_maxima_statements`.
- Sääntö 6, CASTextin `{@...@}` merkkijonoille ja LaTeX-kaavoille ilman sisäkkäistä matematiikkatilaa, sijaitsee funktiossa `stack::render::render_castext`.
- Sääntö 7, automaattinen staattinen varatie Maximan epäonnistuessa, sijaitsee moduuleissa `stack::render` ja `app::api` ADR 0001:n mukaisesti.
- Sääntö 8, vastauspuun solmun jatkosolmu tai lopetusarvo `-1`, sijaitsee rakenteessa `stack::models::stack::Node`.
- Sääntö 9, monivalinnan `single` ja `fraction`, sijaitsee moduuleissa `stack::models::multichoice` ja `app::components::preview::multichoice`.
- Sääntö 10, tosi tai epätosi -kysymyksen `fraction`, sijaitsee moduuleissa `stack::models::truefalse` ja `app::components::preview::truefalse`.

## Hyväksymiskriteerit

1. **Alkutila:** Valittuna on STACK-kysymys ja `seed = 12345`. **Toiminto:** Funktio `evaluate_stack_question_maxima` ajetaan kahdesti samalla siemenellä. **Lopputulos:** Molemmat ajot palauttavat samat `variables`-, `inputs`- ja `questiontext_html`-arvot sekä tilan `maxima_ok == true`.
2. **Alkutila:** Tiedosto `johdatus_graafiteoriaan.xml` sisältää puolipisteettömiä Maxima-rivejä, merkkijonomuuttujia, `[[jsxgraph]]`-lohkoja ja matriisisyötteitä. **Toiminto:** Kaikki 26 kysymystä evaluoidaan siemenellä `42`. **Lopputulos:** Jokaisessa kysymyksessä `maxima_ok` on `true`, yksikään `[[input:...]]`-paikkamerkki ei jää tekstiin, merkkijonomuuttujat tulostuvat ilman `\(...\)`-käärettä ja matriisisyötteet renderöityvät oikean kokoisina taulukkoina.
3. **Alkutila:** STACK-kysymyksessä on `dropdown`-, `checkbox`- tai `radio`-syöte, jonka `tans` on Maxima-lista. **Toiminto:** Kysymys evaluoidaan. **Lopputulos:** Evaluoija poimii valintavaihtoehdot ja oikeat valinnat Maxima-listasta HTML-elementteihin.
4. **Alkutila:** STACK-kysymyksellä on `<deployedseed>`-arvoja. **Toiminto:** Opettaja valitsee pudotusvalikosta toisen käyttöönotetun version. **Lopputulos:** `seed`-signaali päivittyy ja palvelin evaluoi kysymyksen uudelleen valitulla siemenellä.
5. **Alkutila:** Kysymyksen `{@a@}`-merkintä sijaitsee `\begin{align*} ... {@a@} ... \end{align*}` -ympäristön sisällä. **Toiminto:** Funktio `render_castext` muodostaa HTML-tekstin. **Lopputulos:** Sijoitettu LaTeX-arvo ei saa ympärilleen toista `\(...\)`-käärettä.
6. **Alkutila:** STACK-kysymyksen `questionvariables` sisältää virheellisen lausekkeen tai `/usr/bin/maxima` ei ole käytettävissä. **Toiminto:** Kysymys esikatsellaan. **Lopputulos:** Palvelin palauttaa `render_stack_question_static`-tuloksen tilalla `maxima_ok == false`, ja syötekentät näkyvät paikoillaan ilman kaatumista.
7. **Alkutila:** STACK-kysymyksellä on vastauspuu ja Vastauspuu-painike on päällä. **Toiminto:** Opettaja tarkastelee vastauspuuta. **Lopputulos:** Jokainen solmu näyttää vastaustestin, opiskelijan vastauksen, mallivastauksen sekä molempien haarojen pisteet, palautteen ja jatkosolmun tai tekstin `lopetus` arvolla `-1`.
8. **Alkutila:** Valittuna on `Multichoice`-kysymys. **Toiminto:** Opettaja kytkee Täytä oikeat vastaukset -painikkeen päälle. **Lopputulos:** Vaihtoehdot, joiden `fraction` on suurempi kuin nolla, korostuvat vihreällä ja näyttävät pisteytysosuuden sekä vaihtoehtokohtaisen palautteen.
9. **Alkutila:** Valittuna on `TrueFalse`-kysymys. **Toiminto:** Opettaja valitsee vaihtoehdon tai kytkee Täytä oikeat vastaukset -painikkeen päälle. **Lopputulos:** Oikea vaihtoehto näyttää merkinnän `Oikein` ja väärä vaihtoehto merkinnän `Väärin`.

Kriteerit 1, 2, 3, 5, 6 ja 7 ajetaan myös craten `crates/stack` yksikkötesteinä.

## Ei-tavoitteet

- Sovellus ei pisteytä opiskelijan vapaamuotoista syötettä vastauspuussa Maximalla.
- Tämä spec ei sisällä kysymyksen muokkausta, joka on määritelty specissä 004.

## Tehtävät

1. **Stack-työntekijä tiedostossa `crates/stack/src/render.rs`.** Toteuta `normalize_maxima_statements`, STACK-apufunktiot, CASText-, `[[if]]`-, `[[jsxgraph]]`- ja `[[input]]`-renderöinti sekä staattinen esikatselu sääntöjen 3-8 mukaisesti. Askel on valmis, kun `test_parse_and_eval_all_formats` menee läpi kaikilla 37 XML-tiedostolla.
2. **App-työntekijä tiedostossa `crates/app/src/components/toolbar.rs` ja kansiossa `crates/app/src/components/preview/`.** Toteuta `PreviewToolbar`, `QuestionPreviewPane`, `MoodleInfoBox`, `StackQuestionPreview` alikomponentteineen, `MultichoiceQuestionPreview` ja `TrueFalseQuestionPreview` sääntöjen 9-10 mukaisesti. Askel on valmis, kun `cargo leptos build` ja `cargo test --workspace` menevät läpi.
3. **Frontend-työntekijä tiedostoissa `crates/app/src/lib.rs` ja `style/main.scss`.** Toteuta MathJax 3 -ladonta, JSXGraph-piirto, mallivastausten täyttö syötekenttiin ja monikielisyyssuodattimen tyylit. Askel on valmis, kun kaavat, kuvaajat ja kielivalinnat päivittyvät selaimessa.
