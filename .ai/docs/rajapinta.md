# Rajapinta, Stackman

## Yhteystapa

Selain ja palvelin kommunikoivat Leptos `#[server]`-funktioilla polun `/api/*` alla. Lisäksi selain lukee käyttäjän valitsemat paikalliset `.xml`-tiedostot suoraan `FileList`-rajapinnasta ja jäsentää ne funktiolla `stack::render::parse_moodle_xml`.

## Reitit ja palvelinfunktiot tiedostossa `crates/app/src/api.rs`

| Funktio tai reitti | Toiminto | Kutsuja |
|--------------------|----------|---------|
| `GET /` | Palauttaa SSR-renderöidyn HTML-rungon `shell`, MathJax- ja JSXGraph-alustuksen sekä WASM-paketin. | Selain |
| `POST /api/list_sample_xml_files` | Listaa palvelimelta löytyvät esimerkkikysymystiedostot `tentti-Test-top-20261107-1344.xml` ja `material/**/*.xml`. | Käyttöliittymä käynnistyksessä |
| `POST /api/load_sample_xml_file` | Lukee ja jäsentää yhden sallitun esimerkki-XML-tiedoston palvelimelta. | Opettaja |
| `POST /api/evaluate_stack_preview` | Evaluoi `StackQuestion`-kysymyksen annetulla `seed: u32`-arvolla `/usr/bin/maxima`-prosessissa tai palauttaa staattisen esikatselun. | Käyttöliittymä |
| `POST /api/fetch_moodle_questions` | Hakee kysymyskategoriat tai XML-viennin testi-Moodlesta `http://10.134.240.190/` palvelimen kautta, kuten spec 003 määrää. | Opettaja |
| `POST /api/export_moodle_xml` | Tarkistaa ja serialisoi muokatun `Quiz`-rakenteen Moodle XML -merkkijonoksi, kuten spec 004 määrää. | Opettaja |

## Syötteet ja tulosteet

| Toiminto | Syöte | Tuloste |
|----------|-------|---------|
| `list_sample_xml_files` | Ei syötettä | Lista suhteellisia tiedostopolkuja `Vec<String>` |
| `load_sample_xml_file` | Suhteellinen polku `rel_path: String` | Monikko `(tiedostonimi, Quiz)` |
| `evaluate_stack_preview` | `question: StackQuestion` ja `seed: u32` | `EvaluatedStackQuestion`, joka sisältää kentät `seed`, `maxima_ok`, `maxima_error`, `questiontext_html`, `generalfeedback_html`, `questionnote_html`, `variables`, `inputs` ja `prts` |
| `fetch_moodle_questions` | Kategorian tai kurssin tunniste | Monikko `(lähdenimi, Quiz)` ilman Moodlen salasanaa tai istuntotokenia |
| `export_moodle_xml` | Kysymyspankki `Quiz` | Tarkistettu Moodle XML -merkkijono `String` |

## Virhetilanteet

| Tilanne | Paluuarvo ja käyttäytyminen | Perustelu |
|---------|-----------------------------|-----------|
| Rikkinäinen tai tuntematon XML-tiedosto | Jäsentäjä palauttaa `Err(String)`. Käyttöliittymä näyttää virheen `StatusBanners`-komponentissa ja säilyttää aiemmin ladatut kysymykset. | Yksi viallinen tiedosto ei saa tyhjentää koko istuntoa säännön 1 mukaisesti. |
| Kielletty polku kutsussa `load_sample_xml_file` | Palvelin palauttaa virheen `ServerFnError("Invalid sample file path")`. | Tarkistus estää palvelimen tiedostojen luvattoman lukemisen säännön 11 mukaisesti. |
| Maxima-prosessi puuttuu tai CAS-lauseke epäonnistuu | Palvelin palauttaa `Ok(EvaluatedStackQuestion)`, jossa `maxima_ok` on `false` ja `maxima_error` sisältää virhesyyn. | Esikatselu pysyy käytettävissä staattisen näkymän avulla säännön 7 ja ADR 0001:n mukaisesti. |
| Muokatun kysymyksen nimi puuttuu tai `[[input:X]]` on määrittelemätön | Tarkistaja palauttaa virheen ennen XML-serialisointia. | Tarkistus estää virheellisen XML-tiedoston luonnin säännön 14 mukaisesti. |
| Testi-Moodle `http://10.134.240.190/` ei vastaa tai kirjautuminen epäonnistuu | Palvelin palauttaa `Err(ServerFnError)` selkeällä virheviestillä ilman tunnuksia. | Verkko- ja tunnistautumisvirheet käsitellään palvelimella säännön 12 mukaisesti. |

## Suodatus selaimessa

Selain pitää ladatun XML-tiedoston kysymykset muistissa ja suodattaa ne kolmella ehdolla:

1. **Tekstihaku `search_query`.** Haku kohdistuu kysymyksen nimeen, kategoriaan, lähdetiedostoon ja tägeihin.
2. **Kysymystyyppi `QuestionTypeFilter`.** Vaihtoehdot ovat `All`, `Stack`, `Multichoice` ja `TrueFalse`.
3. **Kategoria `category_filter`.** Käyttäjä voi näyttää kaikki kategoriat tai valita yhden siivotun kategorian.
