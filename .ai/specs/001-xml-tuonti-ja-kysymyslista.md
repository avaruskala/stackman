# Spec 001, Moodle- ja STACK XML -tuonti sekä kysymyslistan suodatus

## Tarina

Opettajana haluan tuoda yhden tai useamman Moodle XML -tiedoston tai avata valmiin esimerkkitiedoston kansiosta `material/`, nähdä jäsennetyt kysymykset vasemman reunan listassa ja suodattaa listaa hakusanalla, kysymystyypillä ja kategorialla, jotta löydän tarkastettavan tehtävän nopeasti.

## Käsitteet

Sanasto ja kentät on määritelty tiedostossa `docs/domain.md`.

## Tekemiset

| Toiminto | Onnistumisen ehto | Epäonnistumisen seuraus |
|----------|-------------------|-------------------------|
| Tuo XML-tiedostoja selaimesta | Tiedosto on kelvollinen `<quiz>`, yksittäinen `<question>` tai yksinkertaistettu `<question type="stack">`. Sovellus lisää kysymykset listaan ja valitsee ensimmäisen kysymyksen. | Jos XML on rikkinäinen tai tuntematon, sovellus näyttää virheen `StatusBanners`-komponentissa ja säilyttää aiemmin ladatut kysymykset. |
| Lataa esimerkkitiedosto palvelimelta | Polku on `tentti-Test-top-20261107-1344.xml` tai `material/.../*.xml` ilman `..`-osaa. | Jos polussa on `..` tai se osoittaa sallitun kansion ulkopuolelle, palvelin hylkää pyynnön virheellä `Invalid sample file path`. |
| Suodata kysymyslistaa | Opettaja muuttaa hakusanaa, tyyppipilleriä tai kategoria-pudotusvalikkoa. | Lista näyttää vain suodattimia vastaavat kysymysrivit. |
| Tyhjennä lista | Listalla on vähintään yksi kysymys ja opettaja painaa Tyhjennä lista -painiketta. | Kysymyslista ja valinta tyhjenevät. |

## Säännöt ja paikat

Viittaukset tiedoston `docs/domain.md` sääntöihin:

- Sääntö 1, kolme XML-muotoa ja rikkinäisen XML-syötteen hylkäys, sijaitsee funktiossa `stack::render::parse_moodle_xml`.
- Sääntö 2, `category`-alkio päivittää seuraavien kysymysten kategorian, sijaitsee funktiossa `app::models::extract_questions_from_quiz`.
- Sääntö 11, esimerkkilatauksen polkutarkistus, sijaitsee funktiossa `app::api::load_sample_xml_file`.

## Hyväksymiskriteerit

1. **Alkutila:** Kysymyslista on tyhjä. **Toiminto:** Opettaja lataa tiedoston `tentti-Test-top-20261107-1344.xml`. **Lopputulos:** Funktio `parse_moodle_xml` jäsentää 51 `<question>`-alkiota, `category`-alkiot asettavat kategoriat ja listaan syntyy suoritettavat `Stack`-, `Multichoice`- ja `TrueFalse`-rivit, joista ensimmäinen on valittuna.
2. **Alkutila:** Kysymyslista on tyhjä. **Toiminto:** Opettaja lataa tiedoston `material/tentti-Matematiikan opettajat-Discrete Mathematics (TTY)-20160515-1723/johdatus_graafiteoriaan.xml`. **Lopputulos:** Kaikki 26 vuoden 2016 STACK-kysymystä jäsentyvät ilman virheitä.
3. **Alkutila:** Kysymyslista on tyhjä. **Toiminto:** Opettaja lataa yksinkertaistetun tiedoston `material/STACK_johdatus_todennakoisyyslaskentaan/STACK_1_1.xml`. **Lopputulos:** Jäsentäjä muuntaa tiedoston `Quiz`-rakenteeksi, josta syntyy yksi STACK-kysymysrivi.
4. **Alkutila:** Listalla on jo kysymyksiä. **Toiminto:** Opettaja pudottaa rikkinäisen XML-tiedoston `<quiz><question>`. **Lopputulos:** Funktio `parse_moodle_xml` palauttaa virheen, virhepalkki näyttää syyn ja aiemmin ladatut kysymykset säilyvät listalla.
5. **Alkutila:** XML-tiedostossa on `<question type="category"><category><text>$course$/top/Diskreetti matematiikka/Graafit</text></category></question>` ja sen perässä STACK-kysymys. **Toiminto:** Funktio `extract_questions_from_quiz` poimii kysymykset listaan. **Lopputulos:** `category`-alkio ei näy omana kysymysrivinään, ja sitä seuraavan STACK-kysymyksen kategoriaksi tulee `"Diskreetti matematiikka/Graafit"`.
6. **Alkutila:** Tiedosto `tentti-Test-top-20261107-1344.xml` on ladattu. **Toiminto:** Opettaja valitsee tyyppipillerin `STACK`. **Lopputulos:** Listalla näkyvät vain 39 `stack`-tyyppistä kysymystä, ja tyyppipillerit näyttävät kunkin kysymystyypin lukumäärän.
7. **Alkutila:** Palvelin on käynnissä. **Toiminto:** Funktio `load_sample_xml_file` kutsutaan polulla `"../Cargo.toml"` tai `"Cargo.toml"`. **Lopputulos:** Palvelin hylkää pyynnön virheellä `"Invalid sample file path"`.

Kriteerit 1-5 ajetaan myös craten `crates/stack` yksikkötesteinä ilman HTTP-palvelinta.

## Ei-tavoitteet

- Sovellus ei tallenna XML-tiedostoja tietokantaan latauksen yhteydessä.
- Palvelin ei sivuta kysymyslistaa.

## Tehtävät

Yksi askel vastaa yhtä tehtävänantoa:

1. **Stack-työntekijä cratessa `crates/stack`.** Toteuta Moodle 4+ -kentät ja funktio `parse_moodle_xml` säännön 1 ja ADR 0002:n mukaisesti. Askel on valmis, kun `cargo test -p stack` jäsentää kaikki 37 XML-tiedostoa virheittä.
2. **App-työntekijä tiedostoissa `crates/app/src/models.rs` ja `crates/app/src/api.rs`.** Toteuta `QuestionEntry`, `QuestionTypeFilter`, `extract_questions_from_quiz` ja `load_sample_xml_file` sääntöjen 2 ja 11 mukaisesti. Askel on valmis, kun kategoriaotsikot siivoutuvat oikein ja palvelin hylkää kielletyt polut.
3. **App-työntekijä tiedostoissa `crates/app/src/components/header.rs` ja `crates/app/src/components/sidebar.rs`.** Toteuta yläpalkin tiedostonvalitsin, pudotusalue, tyyppipillerit, kategoria-pudotusvalikko ja hakukenttä. Askel on valmis, kun `cargo leptos build` menee läpi ja kriteerit 1-7 täyttyvät.
